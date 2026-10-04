use crate::error::PartitionError;
use crate::graph::{CsrGraph, Partition};

/// Label-propagation balance refinement.
///
/// Each iteration: every boundary vertex considers moving to the adjacent
/// part with the highest weight deficit (most under target).  A move is
/// accepted only when it strictly reduces the maximum per-part imbalance and
/// keeps both the source and destination within `target ± epsilon`.
///
/// Mirrors METIS `BalanceAndRefineLP` from `kmetis.c`.
#[cfg(test)]
pub(crate) fn lp_balance(g: &CsrGraph, partition: &mut Partition, ufactor: u32, max_iter: u32) {
    lp_balance_with_contiguity(g, partition, ufactor, max_iter, false);
}

pub(crate) fn lp_balance_with_contiguity(
    g: &CsrGraph,
    partition: &mut Partition,
    ufactor: u32,
    max_iter: u32,
    contig: bool,
) {
    if max_iter == 0 {
        return;
    }
    let n = g.n();
    let k = partition.k as usize;
    if k <= 1 {
        return;
    }

    let total_wgt: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
    let targets = balance_targets(total_wgt, k, partition.tpwgts.as_deref());
    let epsilons: Vec<i64> = targets
        .iter()
        .map(|&target| (target.abs() * ufactor as i64 + 999) / 1000)
        .collect();

    let mut pwgts = vec![0i64; k];
    for v in 0..n {
        pwgts[partition.assignment[v] as usize] += g.vwgt[v] as i64;
    }

    for _iter in 0..max_iter {
        let mut moved = 0usize;

        for v in 0..n {
            let from = partition.assignment[v] as usize;
            let vwgt = g.vwgt[v] as i64;

            // Only consider boundary vertices (at least one neighbour in a different part)
            let is_boundary = (g.xadj[v] as usize..g.xadj[v + 1] as usize)
                .any(|j| partition.assignment[g.adjncy[j] as usize] as usize != from);
            if !is_boundary {
                continue;
            }

            // Source part must be overloaded before we consider moving anyone out.
            if pwgts[from] <= targets[from] + epsilons[from] {
                continue;
            }

            // Find the most under-target adjacent part (minimum pwgts wins)
            let best_to = (g.xadj[v] as usize..g.xadj[v + 1] as usize)
                .map(|j| partition.assignment[g.adjncy[j] as usize] as usize)
                .filter(|&to| to != from)
                .min_by_key(|&to| pwgts[to]);

            if let Some(to) = best_to {
                // Accept the move only when it keeps both sides in balance
                let new_from = pwgts[from] - vwgt;
                let new_to = pwgts[to] + vwgt;
                if new_from >= targets[from] - epsilons[from]
                    && new_to <= targets[to] + epsilons[to]
                {
                    if contig
                        && (partition
                            .assignment
                            .iter()
                            .filter(|&&p| p as usize == from)
                            .count()
                            <= 1
                            || crate::refine::fm::would_disconnect(
                                g,
                                &partition.assignment,
                                v,
                                from,
                            ))
                    {
                        continue;
                    }
                    partition.assignment[v] = to as u32;
                    pwgts[from] = new_from;
                    pwgts[to] = new_to;
                    moved += 1;
                }
            }
        }

        if moved == 0 {
            break;
        }
    }
}

fn balance_targets(total_wgt: i64, k: usize, tpwgts: Option<&[f32]>) -> Vec<i64> {
    match tpwgts {
        Some(weights) if weights.len() == k => weights
            .iter()
            .map(|&weight| (total_wgt as f64 * weight as f64).round() as i64)
            .collect(),
        _ => vec![total_wgt / k as i64; k],
    }
}

/// Multiconstraint pre-balance. Use all interleaved weights and require each
/// move to reduce normalized violation without worsening another constraint.
pub(crate) fn lp_balance_multiconstraint(
    g: &CsrGraph,
    p: &mut Partition,
    ufactor: u32,
    ubvec: Option<&[f32]>,
    max_iter: u32,
    contig: bool,
) {
    let nc = g.ncon as usize;
    let k = p.k as usize;
    if k <= 1 {
        return;
    }
    let totals: Vec<i64> = (0..nc)
        .map(|c| (0..g.n()).map(|v| i64::from(g.vwgt[v * nc + c])).sum())
        .collect();
    let targets: Vec<Vec<f64>> = (0..k)
        .map(|part| {
            (0..nc)
                .map(|c| {
                    totals[c] as f64
                        * if c == 0 {
                            p.tpwgts
                                .as_ref()
                                .map_or(1.0 / k as f64, |tw| tw[part] as f64)
                        } else {
                            1.0 / k as f64
                        }
                })
                .collect()
        })
        .collect();
    let limits: Vec<Vec<f64>> = targets
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, t)| {
                    t * ubvec.map_or(1.0 + ufactor as f64 / 1000.0, |values| values[c] as f64)
                })
                .collect()
        })
        .collect();
    let mut weights = vec![vec![0i64; nc]; k];
    let mut sizes = vec![0usize; k];
    for v in 0..g.n() {
        let part = p.assignment[v] as usize;
        sizes[part] += 1;
        for c in 0..nc {
            weights[part][c] += i64::from(g.vwgt[v * nc + c]);
        }
    }
    for _ in 0..max_iter {
        let mut moved = false;
        for v in 0..g.n() {
            let from = p.assignment[v] as usize;
            if sizes[from] <= 1 {
                continue;
            }
            let adjacent: std::collections::BTreeSet<usize> = (g.xadj[v] as usize
                ..g.xadj[v + 1] as usize)
                .map(|j| p.assignment[g.adjncy[j] as usize] as usize)
                .filter(|&to| to != from)
                .collect();
            let mut best: Option<(f64, usize)> = None;
            for to in adjacent {
                let mut before = 0.0;
                let mut after = 0.0;
                let mut worsens = false;
                for c in 0..nc {
                    let value = i64::from(g.vwgt[v * nc + c]);
                    let violation = |weight: i64, part: usize| {
                        (weight as f64 - limits[part][c]).max(0.0) / targets[part][c].max(1.0)
                    };
                    let previous =
                        violation(weights[from][c], from) + violation(weights[to][c], to);
                    let next = violation(weights[from][c] - value, from)
                        + violation(weights[to][c] + value, to);
                    if next > previous + 1e-12 {
                        worsens = true;
                        break;
                    }
                    before += previous;
                    after += next;
                }
                if !worsens
                    && after + 1e-12 < before
                    && best.is_none_or(|current| (after, to) < current)
                {
                    best = Some((after, to));
                }
            }
            if let Some((_, to)) = best {
                if contig && crate::refine::fm::would_disconnect(g, &p.assignment, v, from) {
                    continue;
                }
                p.assignment[v] = to as u32;
                sizes[from] -= 1;
                sizes[to] += 1;
                for c in 0..nc {
                    let value = i64::from(g.vwgt[v * nc + c]);
                    weights[from][c] -= value;
                    weights[to][c] += value;
                }
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
}

/// Deterministically repair equal-weight balance after k-way refinement.
///
/// This is intentionally conservative: it only moves boundary vertices from
/// parts above the METIS `ufactor` limit into adjacent parts that can accept the
/// vertex without exceeding their own limit. Among legal moves it chooses the
/// smallest edge-cut penalty across all overweight parts, with deterministic
/// tie-breakers on source part and vertex id.
pub fn rebalance_to_ufactor(
    g: &CsrGraph,
    partition: &mut Partition,
    ufactor: u32,
) -> Result<(), PartitionError> {
    g.validate()?;
    partition.validate_for_graph(g)?;

    let k = partition.k as usize;
    if k <= 1 || g.n() == 0 {
        return Ok(());
    }

    let total_wgt: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
    let avg = (total_wgt + k as i64 - 1) / k as i64;
    let epsilon = (avg * ufactor as i64 + 999) / 1000;
    let max_wgt = avg + epsilon;

    let mut pwgts = vec![0i64; k];
    for v in 0..g.n() {
        pwgts[partition.assignment[v] as usize] += g.vwgt[v] as i64;
    }

    let mut seen_parts = vec![0u32; k];
    let mut adjacent_parts = Vec::new();
    let mut visit_mark = 1u32;

    for _ in 0..g.n().saturating_mul(k) {
        let Some((from, v, to)) = best_rebalance_move(
            g,
            partition,
            &pwgts,
            max_wgt,
            &mut seen_parts,
            &mut adjacent_parts,
            &mut visit_mark,
        ) else {
            break;
        };

        let v_wgt = g.vwgt[v] as i64;
        partition.assignment[v] = to as u32;
        pwgts[from] -= v_wgt;
        pwgts[to] += v_wgt;
    }

    Ok(())
}

fn best_rebalance_move(
    g: &CsrGraph,
    partition: &Partition,
    pwgts: &[i64],
    max_wgt: i64,
    seen_parts: &mut [u32],
    adjacent_parts: &mut Vec<usize>,
    visit_mark: &mut u32,
) -> Option<(usize, usize, usize)> {
    let k = partition.k as usize;
    let mut best: Option<(i64, usize, usize, usize)> = None;

    for from in 0..k {
        if pwgts[from] <= max_wgt {
            continue;
        }

        for v in 0..g.n() {
            if partition.assignment[v] as usize != from {
                continue;
            }

            let v_wgt = g.vwgt[v] as i64;
            adjacent_parts.clear();

            for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
                let to = partition.assignment[g.adjncy[j] as usize] as usize;
                if to == from || seen_parts[to] == *visit_mark {
                    continue;
                }
                seen_parts[to] = *visit_mark;
                adjacent_parts.push(to);
            }
            *visit_mark = visit_mark.wrapping_add(1);
            if *visit_mark == 0 {
                seen_parts.fill(0);
                *visit_mark = 1;
            }

            for &to in adjacent_parts.iter() {
                if pwgts[to] + v_wgt > max_wgt {
                    continue;
                }
                let delta = move_cut_delta(g, &partition.assignment, v, to as u32);
                let candidate = (delta, from, v, to);
                if best.is_none_or(|current| candidate < current) {
                    best = Some(candidate);
                }
            }
        }
    }

    best.map(|(_, from, v, to)| (from, v, to))
}

fn move_cut_delta(g: &CsrGraph, assignment: &[u32], v: usize, to_part: u32) -> i64 {
    let from_part = assignment[v];
    let mut delta = 0i64;

    for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
        let u_part = assignment[g.adjncy[j] as usize];
        let edge_wgt = g.adjwgt.as_ref().map_or(1i64, |weights| weights[j] as i64);
        if u_part == from_part {
            delta += edge_wgt;
        } else if u_part == to_part {
            delta -= edge_wgt;
        }
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dual_prebalance_uses_interleaved_weights() {
        let mut g = path_graph(6);
        g.ncon = 2;
        g.vwgt = vec![100; 12];
        let mut p = Partition {
            assignment: vec![0, 0, 0, 0, 1, 1],
            k: 2,
            tpwgts: None,
        };
        lp_balance_multiconstraint(&g, &mut p, 5, Some(&[1.005, 1.1]), 10, true);
        assert_eq!(p.assignment, vec![0, 0, 0, 1, 1, 1]);
    }
    #[test]
    fn dual_prebalance_cannot_repair_population_by_violating_area() {
        let mut g = path_graph(4);
        g.ncon = 2;
        g.vwgt = vec![100, 100, 100, 100, 100, 1000, 100, 1000];
        let mut p = Partition {
            assignment: vec![0, 0, 0, 1],
            k: 2,
            tpwgts: None,
        };
        let before = p.assignment.clone();
        lp_balance_multiconstraint(&g, &mut p, 1, Some(&[1.001, 1.2]), 10, true);
        assert_eq!(p.assignment, before);
    }

    #[test]
    fn contiguous_prebalance_keeps_articulation_vertex() {
        // Part zero is 0--1--2, with its only boundary at articulation 1.
        // Moving vertex 1 balances population exactly but strands 0 and 2.
        let graph = CsrGraph::new(
            vec![0, 1, 4, 5, 7, 9, 10],
            vec![1, 0, 2, 3, 1, 1, 4, 3, 5, 4],
            1,
            vec![4, 2, 4, 2, 2, 2],
            None,
        )
        .unwrap();
        let mut connected = Partition {
            assignment: vec![0, 0, 0, 1, 1, 1],
            k: 2,
            tpwgts: None,
        };
        let mut unrestricted = connected.clone();
        lp_balance_with_contiguity(&graph, &mut connected, 0, 10, true);
        assert_eq!(connected.assignment, vec![0, 0, 0, 1, 1, 1]);
        lp_balance_with_contiguity(&graph, &mut unrestricted, 0, 10, false);
        assert_eq!(unrestricted.assignment[1], 1);
    }

    fn path_graph(n: usize) -> CsrGraph {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 {
                adjncy.push((i - 1) as u32);
            }
            if i < n - 1 {
                adjncy.push((i + 1) as u32);
            }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph {
            xadj,
            adjncy,
            ncon: 1,
            vwgt: vec![1i32; n],
            adjwgt: None,
        }
    }

    fn make_grid_graph(rows: usize, cols: usize) -> CsrGraph {
        let n = rows * cols;
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let mut nbrs = Vec::new();
                if r > 0 {
                    nbrs.push((r - 1) * cols + c);
                }
                if r < rows - 1 {
                    nbrs.push((r + 1) * cols + c);
                }
                if c > 0 {
                    nbrs.push(r * cols + (c - 1));
                }
                if c < cols - 1 {
                    nbrs.push(r * cols + (c + 1));
                }
                for &u in &nbrs {
                    adjncy.push(u as u32);
                }
                xadj.push(adjncy.len() as u32);
            }
        }
        CsrGraph {
            xadj,
            adjncy,
            ncon: 1,
            vwgt: vec![1i32; n],
            adjwgt: None,
        }
    }

    fn pwgtss(g: &CsrGraph, p: &Partition) -> Vec<i64> {
        let k = p.k as usize;
        let mut wgts = vec![0i64; k];
        for v in 0..g.n() {
            wgts[p.assignment[v] as usize] += g.vwgt[v] as i64;
        }
        wgts
    }

    fn max_imbalance(wgts: &[i64], target: i64) -> i64 {
        wgts.iter().map(|&p| (p - target).abs()).max().unwrap_or(0)
    }

    /// LP with 0 iterations must be a no-op.
    #[test]
    fn lp_zero_iter_is_noop() {
        let g = path_graph(10);
        let mut p = Partition {
            assignment: vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1],
            k: 2,
            tpwgts: None,
        };
        let before = p.assignment.clone();
        lp_balance(&g, &mut p, 5, 0);
        assert_eq!(p.assignment, before);
    }

    /// Already-balanced partition should not be disturbed.
    #[test]
    fn lp_balanced_partition_unchanged() {
        let g = path_graph(10);
        // Perfect 5-5 split
        let mut p = Partition {
            assignment: vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1],
            k: 2,
            tpwgts: None,
        };
        let before = p.assignment.clone();
        lp_balance(&g, &mut p, 5, 10);
        // Should not have moved anyone (already balanced)
        assert_eq!(
            p.assignment, before,
            "perfectly balanced partition must not be changed by LP"
        );
    }

    #[test]
    fn lp_respects_ufactor_tolerance() {
        let g = path_graph(10);
        let assignment = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1];

        let mut strict = Partition {
            assignment: assignment.clone(),
            k: 2,
            tpwgts: None,
        };
        lp_balance(&g, &mut strict, 0, 10);

        let mut loose = Partition {
            assignment,
            k: 2,
            tpwgts: None,
        };
        lp_balance(&g, &mut loose, 200, 10);

        assert_ne!(
            strict.assignment, loose.assignment,
            "strict ufactor should rebalance where loose ufactor accepts the starting split"
        );
        assert_eq!(pwgtss(&g, &strict), vec![5, 5]);
        assert_eq!(pwgtss(&g, &loose), vec![6, 4]);
    }

    #[test]
    fn lp_respects_target_partition_weights() {
        let g = path_graph(10);
        let assignment = vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1];

        let mut weighted = Partition {
            assignment: assignment.clone(),
            k: 2,
            tpwgts: Some(vec![0.7, 0.3]),
        };
        lp_balance(&g, &mut weighted, 5, 10);

        let mut equal = Partition {
            assignment,
            k: 2,
            tpwgts: None,
        };
        lp_balance(&g, &mut equal, 0, 10);

        assert_eq!(
            pwgtss(&g, &weighted),
            vec![7, 3],
            "weighted targets should preserve the requested 70/30 split"
        );
        assert_eq!(
            pwgtss(&g, &equal),
            vec![5, 5],
            "without tpwgts LP should rebalance toward equal parts"
        );
    }

    /// Heavily imbalanced 4-part partition on a grid should improve balance.
    #[test]
    fn lp_improves_imbalance_on_grid() {
        let g = make_grid_graph(4, 4); // 16 vertices
                                       // Intentionally unbalanced: put 10 vertices in part 0, 2 each in parts 1-3
                                       // Layout: first 10 in part 0, next 2 in 1, next 2 in 2, last 2 in 3
        let assignment: Vec<u32> = (0..16)
            .map(|v| {
                if v < 10 {
                    0
                } else if v < 12 {
                    1
                } else if v < 14 {
                    2
                } else {
                    3
                }
            })
            .collect();
        let mut p = Partition {
            assignment,
            k: 4,
            tpwgts: None,
        };
        let total: i64 = 16;
        let target = total / 4; // = 4
        let before_pops = pwgtss(&g, &p);
        let before_imb = max_imbalance(&before_pops, target);
        lp_balance(&g, &mut p, 5, 20);
        let after_pops = pwgtss(&g, &p);
        let after_imb = max_imbalance(&after_pops, target);
        assert!(
            after_imb <= before_imb,
            "LP must not worsen imbalance: before={before_imb} after={after_imb}"
        );
    }

    /// Assignment validity: all assignments must remain in 0..k after LP.
    #[test]
    fn lp_assignment_stays_in_range() {
        let g = make_grid_graph(4, 4);
        let k = 4u32;
        let mut p = Partition {
            assignment: (0..16).map(|v| (v % k as usize) as u32).collect(),
            k,
            tpwgts: None,
        };
        lp_balance(&g, &mut p, 5, 10);
        assert!(
            p.assignment.iter().all(|&a| a < k),
            "LP produced out-of-range assignment"
        );
    }

    /// Part populations must sum to total vertex weight after LP.
    #[test]
    fn lp_total_wgt_conserved() {
        let g = make_grid_graph(4, 4);
        let total_before: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
        let mut p = Partition {
            assignment: (0..16).map(|v| (v % 4) as u32).collect(),
            k: 4,
            tpwgts: None,
        };
        lp_balance(&g, &mut p, 5, 10);
        let total_after: i64 = g.vwgt.iter().map(|&w| w as i64).sum();
        assert_eq!(total_before, total_after);
        // Also check per-part wgts sum to total
        let wgts = pwgtss(&g, &p);
        assert_eq!(wgts.iter().sum::<i64>(), total_before);
    }

    /// k=1 trivial partition — LP should be a no-op.
    #[test]
    fn lp_k1_noop() {
        let g = path_graph(5);
        let mut p = Partition {
            assignment: vec![0; 5],
            k: 1,
            tpwgts: None,
        };
        let before = p.assignment.clone();
        lp_balance(&g, &mut p, 5, 10);
        assert_eq!(p.assignment, before);
    }

    #[test]
    fn rebalance_only_moves_to_adjacent_parts() {
        let g = path_graph(3);
        let mut p = Partition::new(vec![0, 0, 1], 3).unwrap();

        rebalance_to_ufactor(&g, &mut p, 0).unwrap();

        assert_eq!(
            p.assignment(),
            &[0, 0, 1],
            "rebalance must not move vertices into non-adjacent empty parts"
        );
    }

    #[test]
    fn rebalance_rejects_partition_with_wrong_length() {
        let g = path_graph(3);
        let mut p = Partition::new(vec![0, 1], 2).unwrap();

        assert!(matches!(
            rebalance_to_ufactor(&g, &mut p, 0),
            Err(PartitionError::InvalidPartition(_))
        ));
    }

    #[test]
    fn rebalance_prefers_global_best_move_across_overweight_parts() {
        let g = CsrGraph {
            xadj: vec![0, 3, 5, 8, 10, 12, 14, 15, 16, 18],
            adjncy: vec![
                1, 2, 6, // 0
                0, 2, // 1
                0, 1, 8, // 2
                4, 8, // 3
                3, 5, // 4
                4, 7, // 5
                0, // 6
                5, // 7
                2, 3, // 8
            ],
            ncon: 1,
            vwgt: vec![1i32; 9],
            adjwgt: None,
        };
        let partition = Partition::new(vec![0, 0, 0, 1, 1, 1, 0, 1, 2], 3).unwrap();
        let pwgts = vec![4, 4, 1];
        let mut seen_parts = vec![0u32; 3];
        let mut adjacent_parts = Vec::new();
        let mut visit_mark = 1u32;

        let candidate = best_rebalance_move(
            &g,
            &partition,
            &pwgts,
            3,
            &mut seen_parts,
            &mut adjacent_parts,
            &mut visit_mark,
        )
        .unwrap();
        assert_eq!(candidate, (1, 3, 2));
    }
}
