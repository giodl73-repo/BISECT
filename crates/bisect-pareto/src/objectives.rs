//! Objective evaluation for redistricting plans.
//!
//! Evaluates three objectives for a given plan:
//! - EC: edge cuts (boundary length) — lower is more compact
//! - D_seats: |D_seats_won - proportional_seats| — lower is more proportional
//! - VRA_deficit: sum of shortfalls below 50% minority VAP — lower is better VRA compliance
//!
//! Per spec §2 and §8.1.

/// Objective values for a single redistricting plan. All three are minimised.
#[derive(Debug, Clone, PartialEq)]
pub struct Objectives {
    /// Edge cuts: number of adjacency edges crossing district boundaries. Lower = more compact.
    pub ec: f64,
    /// |D_seats_won - proportional_seats|. Lower = more proportional.
    /// D_seats = 0.0 if d_votes is None.
    pub d_seats: f64,
    /// Sum of shortfalls below 50% minority VAP across all districts.
    /// VRA_deficit = 0.0 if minority_vap is None.
    pub vra_deficit: f64,
}

/// Count edge cuts for a plan: edges (u, v) where plan[u] != plan[v].
///
/// `adjacency[v]` is the list of neighbours of vertex v (usize indices).
pub fn count_edge_cuts(plan: &[u32], adjacency: &[Vec<usize>]) -> usize {
    let mut cuts = 0usize;
    for (v, neighbours) in adjacency.iter().enumerate() {
        let dv = plan[v];
        for &u in neighbours {
            // Only count each edge once (u > v)
            if u > v && plan[u] != dv {
                cuts += 1;
            }
        }
    }
    cuts
}

/// Evaluate all three objectives for a plan.
///
/// # Parameters
/// - `plan`: district assignment, plan[t] = district ID (1-based, 1..=k)
/// - `adjacency`: adjacency list (usize indices)
/// - `pop`: population per tract
/// - `k`: number of districts
/// - `d_votes`: Democratic 2-party vote share per tract (None = skip D_seats, returns 0.0)
/// - `minority_vap`: minority VAP fraction per tract (None = skip VRA, returns 0.0)
/// - `protected_districts`: unused in Phase 1; spec allows None = all districts checked
pub fn evaluate(
    plan: &[u32],
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    d_votes: Option<&[f64]>,
    minority_vap: Option<&[f64]>,
    _protected_districts: &[u32],
) -> Objectives {
    let n = plan.len();

    // ── EC ────────────────────────────────────────────────────────────────────
    let ec = count_edge_cuts(plan, adjacency) as f64;

    // ── D_seats ───────────────────────────────────────────────────────────────
    let d_seats = if let Some(dv) = d_votes {
        let total_votes: f64 = (0..n).map(|t| pop[t] as f64).sum();
        let total_d: f64 = (0..n).map(|t| dv[t] * pop[t] as f64).sum();

        // Proportional seats
        let proportional_seats = k as f64 * (total_d / total_votes.max(1.0));

        // Compute per-district D_votes and total_votes to determine winners
        let mut d_pop = vec![0.0f64; k + 1];
        let mut t_pop = vec![0.0f64; k + 1];
        for t in 0..n {
            let d = plan[t] as usize;
            if d >= 1 && d <= k {
                d_pop[d] += dv[t] * pop[t] as f64;
                t_pop[d] += pop[t] as f64;
            }
        }
        let d_seats_won: f64 = (1..=k)
            .filter(|&d| t_pop[d] > 0.0 && d_pop[d] / t_pop[d] >= 0.5)
            .count() as f64;

        (d_seats_won - proportional_seats).abs()
    } else {
        0.0
    };

    // ── VRA_deficit ───────────────────────────────────────────────────────────
    let vra_deficit = if let Some(mvap) = minority_vap {
        // Compute minority VAP fraction per district (population-weighted)
        let mut minority_pop = vec![0.0f64; k + 1];
        let mut total_pop = vec![0.0f64; k + 1];
        for t in 0..n {
            let d = plan[t] as usize;
            if d >= 1 && d <= k {
                minority_pop[d] += mvap[t] * pop[t] as f64;
                total_pop[d] += pop[t] as f64;
            }
        }
        (1..=k)
            .map(|d| {
                if total_pop[d] > 0.0 {
                    let frac = minority_pop[d] / total_pop[d];
                    if frac < 0.5 {
                        0.5 - frac
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            })
            .sum::<f64>()
    } else {
        0.0
    };

    Objectives {
        ec,
        d_seats,
        vra_deficit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|i| {
                let mut nb = Vec::new();
                if i > 0 {
                    nb.push(i - 1);
                }
                if i < n - 1 {
                    nb.push(i + 1);
                }
                nb
            })
            .collect()
    }

    #[test]
    fn ec_no_cuts_when_all_same_district() {
        let adj = path_adj(4);
        let plan = vec![1u32; 4];
        assert_eq!(count_edge_cuts(&plan, &adj), 0);
    }

    #[test]
    fn ec_one_cut_path_two_districts() {
        let adj = path_adj(4);
        let plan = vec![1u32, 1, 2, 2];
        // Only edge (1,2) crosses boundary
        assert_eq!(count_edge_cuts(&plan, &adj), 1);
    }

    #[test]
    fn ec_two_cuts_path_four_nodes() {
        let adj = path_adj(4);
        let plan = vec![1u32, 2, 1, 2];
        assert_eq!(count_edge_cuts(&plan, &adj), 3);
    }

    #[test]
    fn evaluate_no_optional_inputs() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let plan = vec![1u32, 1, 2, 2];
        let obj = evaluate(&plan, &adj, &pop, 2, None, None, &[]);
        assert_eq!(obj.ec, 1.0);
        assert_eq!(obj.d_seats, 0.0);
        assert_eq!(obj.vra_deficit, 0.0);
    }

    #[test]
    fn evaluate_d_seats_equal_pop() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let plan = vec![1u32, 1, 2, 2];
        // All D: proportional = 2.0, won = 2, deviation = 0
        let d_votes = vec![1.0f64; 4];
        let obj = evaluate(&plan, &adj, &pop, 2, Some(&d_votes), None, &[]);
        assert_eq!(obj.d_seats, 0.0);
    }

    #[test]
    fn evaluate_vra_deficit() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let plan = vec![1u32, 1, 2, 2];
        // District 1: minority_vap = 0.3 (deficit 0.2)
        // District 2: minority_vap = 0.6 (no deficit)
        let minority_vap = vec![0.3f64, 0.3, 0.6, 0.6];
        let obj = evaluate(&plan, &adj, &pop, 2, None, Some(&minority_vap), &[]);
        assert!(
            (obj.vra_deficit - 0.2).abs() < 1e-9,
            "vra_deficit = {}",
            obj.vra_deficit
        );
    }
}
