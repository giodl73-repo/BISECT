//! District proposal for SMC: spanning tree + balanced cut selection.
//! Per spec §2.2.
//!
//! At stage t, we have a PartialPlan with districts 1..t assigned.
//! We grow district t+1 by:
//!   1. Finding the largest connected component C of unassigned tracts
//!   2. Sampling seed tract s uniformly from C
//!   3. Sampling a uniform spanning tree T of C (Wilson's algorithm)
//!   4. Enumerating all balanced cut edges in T
//!   5. Selecting one cut uniformly at random
//!   6. Assigning the component containing s to the new district
//!   7. Returning log(|valid_cuts|) as the importance weight increment
//!
//! Weight derivation (spec §2.2): proposal probability = 1/|valid_cuts|;
//! target = uniform; importance correction = |valid_cuts|; log_increment = log(|valid_cuts|).

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use thiserror::Error;

use redist_ensemble::spanning::{random_spanning_tree, SpanningTree};
use crate::partial_plan::PartialPlan;

#[derive(Debug, Error)]
pub enum ProposeError {
    #[error("no valid balanced cut found in spanning tree (stage {stage}, particle {particle_idx})")]
    NoValidCut { stage: usize, particle_idx: usize },
    #[error("unassigned subgraph is empty at stage {stage}")]
    EmptySubgraph { stage: usize },
}

/// Propose the next district for a partial plan.
///
/// Returns `(updated_partial_plan, log_weight_increment)`.
/// If no balanced cut exists, returns `ProposeError::NoValidCut`.
pub fn propose_district(
    partial: &PartialPlan,
    adj: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    stage: usize,         // 1-based: we are assigning district `stage`
    pop_tolerance: f64,
    rng: &mut SmallRng,
    particle_idx: usize,
) -> Result<(PartialPlan, f64), ProposeError> {
    // Step 1: find largest connected component of unassigned tracts.
    let component = partial.largest_unassigned_component(adj, pop);
    if component.is_empty() {
        return Err(ProposeError::EmptySubgraph { stage });
    }

    // Remaining districts to assign (including this one): k - stage + 1
    let remaining_districts = k - stage + 1;
    // Total unassigned population
    let total_remaining_pop: i64 = component.iter().map(|&t| pop[t]).sum();
    // Target population for this district
    let target_pop = total_remaining_pop as f64 / remaining_districts as f64;
    let pop_tol_abs = pop_tolerance * total_remaining_pop as f64;

    // Step 2: build local index mapping for the component (sorted for determinism)
    let mut sorted_comp = component.clone();
    sorted_comp.sort_unstable();
    let m = sorted_comp.len();

    // global → local index
    let mut g2l = vec![usize::MAX; adj.len()];
    for (local, &global) in sorted_comp.iter().enumerate() {
        g2l[global] = local;
    }

    // Build local adjacency (only within the component)
    let local_adj: Vec<Vec<u32>> = sorted_comp.iter().map(|&g| {
        adj[g].iter()
            .filter(|&&nb| g2l[nb] != usize::MAX)
            .map(|&nb| g2l[nb] as u32)
            .collect()
    }).collect();

    // Single-tract component: assign trivially (only one choice, weight = log(1) = 0)
    if m == 1 {
        let mut new_partial = partial.clone();
        new_partial.assign_district(&sorted_comp, stage as u32);
        return Ok((new_partial, 0.0));
    }

    // Step 3: sample seed tract uniformly from the component
    let seed_local = rng.gen_range(0..m) as u32;

    // Step 4: sample a uniform spanning tree of the component
    let tree = random_spanning_tree(&local_adj, rng);

    // Step 5: enumerate all tree edges and find balanced cuts
    // For each tree edge (a, b), split_on gives two components.
    // A cut is balanced if one component's population is within pop_tol_abs of target_pop.
    let local_pop: Vec<i64> = sorted_comp.iter().map(|&g| pop[g]).collect();
    let valid_cuts: Vec<(u32, u32)> = tree.edges()
        .filter(|&(a, b)| {
            let (ca, _cb) = tree.split_on(a, b);
            let pop_a: i64 = ca.iter().map(|&v| local_pop[v as usize]).sum();
            let pop_b = total_remaining_pop - pop_a;
            // Either component can be the new district — check both
            (pop_a as f64 - target_pop).abs() <= pop_tol_abs
                || (pop_b as f64 - target_pop).abs() <= pop_tol_abs
        })
        .collect();

    if valid_cuts.is_empty() {
        return Err(ProposeError::NoValidCut { stage, particle_idx });
    }

    // Step 6: select one valid cut uniformly at random
    let cut_idx = rng.gen_range(0..valid_cuts.len());
    let (a, b) = valid_cuts[cut_idx];
    let (comp_a, comp_b) = tree.split_on(a, b);

    // The new district is the component whose population is closer to target_pop.
    // If both are valid, use the component containing seed_local.
    let pop_a: i64 = comp_a.iter().map(|&v| local_pop[v as usize]).sum();
    let pop_b = total_remaining_pop - pop_a;
    let a_is_new = if (pop_a as f64 - target_pop).abs() <= pop_tol_abs
                      && (pop_b as f64 - target_pop).abs() <= pop_tol_abs {
        // Both valid: use component containing seed_local
        comp_a.contains(&seed_local)
    } else {
        (pop_a as f64 - target_pop).abs() <= pop_tol_abs
    };

    let new_district_local: Vec<usize> = if a_is_new {
        comp_a.iter().map(|&v| v as usize).collect()
    } else {
        comp_b.iter().map(|&v| v as usize).collect()
    };

    // Convert local → global indices
    let new_district_global: Vec<usize> = new_district_local.iter()
        .map(|&l| sorted_comp[l])
        .collect();

    // Step 7: assign the new district
    let mut new_partial = partial.clone();
    new_partial.assign_district(&new_district_global, stage as u32);

    // Weight increment = log(|valid_cuts|)
    let log_w = (valid_cuts.len() as f64).ln();

    Ok((new_partial, log_w))
}

/// Check if a set of tracts forms a connected subgraph (BFS).
pub fn is_connected(tracts: &[usize], adj: &[Vec<usize>]) -> bool {
    if tracts.is_empty() { return true; }
    let tract_set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(tracts[0]);
    visited.insert(tracts[0]);
    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v] {
            if tract_set.contains(&nb) && !visited.contains(&nb) {
                visited.insert(nb);
                queue.push_back(nb);
            }
        }
    }
    visited.len() == tracts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i - 1); }
            if i < n - 1 { nb.push(i + 1); }
            nb
        }).collect()
    }

    fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<usize>> {
        let n = rows * cols;
        let mut adj = vec![vec![]; n];
        for r in 0..rows {
            for c in 0..cols {
                let v = r * cols + c;
                if c + 1 < cols { adj[v].push(v + 1); adj[v + 1].push(v); }
                if r + 1 < rows { adj[v].push(v + cols); adj[v + cols].push(v); }
            }
        }
        adj
    }

    #[test]
    fn propose_district_4_node_path_k2() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let partial = PartialPlan::empty(4);
        let mut rng = SmallRng::seed_from_u64(42);

        let (new_partial, log_w) = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng, 0)
            .expect("propose must succeed on path graph k=2");

        // One district assigned, one still unassigned
        let assigned: Vec<usize> = new_partial.assignment.iter().enumerate()
            .filter_map(|(i, a)| if *a == Some(1) { Some(i) } else { None })
            .collect();
        assert!(!assigned.is_empty(), "district 1 must have at least one tract");
        assert!(is_connected(&assigned, &adj), "assigned district must be contiguous");
        assert!(log_w >= 0.0, "log_w must be ≥ 0 (at least one valid cut)");
        assert_eq!(new_partial.unassigned_count, 4 - assigned.len());
    }

    #[test]
    fn propose_district_two_stages_k2_then_assign_remaining() {
        // Propose stage 1 of k=2, then assign_remaining for stage 2.
        // This mirrors the actual SMC loop: propose k-1 times, then assign_remaining.
        let adj = grid_adj(4, 4);
        let pop = vec![100i64; 16];
        let mut partial = PartialPlan::empty(16);
        let mut rng = SmallRng::seed_from_u64(7);

        let (p2, log_w) = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng, 0)
            .expect("stage 1 proposal must succeed");
        assert!(log_w >= 0.0);

        partial = p2;
        partial.assign_remaining(2);

        // All 16 tracts assigned to districts 1 or 2
        assert_eq!(partial.unassigned_count, 0);
        let d1: Vec<usize> = (0..16).filter(|&i| partial.assignment[i] == Some(1)).collect();
        let d2: Vec<usize> = (0..16).filter(|&i| partial.assignment[i] == Some(2)).collect();
        assert!(!d1.is_empty() && !d2.is_empty(), "both districts non-empty");
        assert!(is_connected(&d1, &adj), "district 1 contiguous");
        assert!(is_connected(&d2, &adj), "district 2 contiguous");
    }

    #[test]
    fn propose_district_no_valid_cut_error_path() {
        // A star graph with 4 nodes (center + 3 leaves): all spanning tree cuts
        // produce a 1+3 split. With equal pop and tight tolerance, no cut is balanced.
        // adj: 0 connected to 1,2,3; leaves not connected to each other.
        let adj = vec![
            vec![1usize, 2, 3], // center
            vec![0],
            vec![0],
            vec![0],
        ];
        let pop = vec![100i64; 4]; // total=400, target=200, tol=±1 (0.5%→2)
        let partial = PartialPlan::empty(4);
        let mut rng = SmallRng::seed_from_u64(1);

        // With very tight tolerance (0.005 = ±2 pop), cuts of 100+300 are invalid.
        // The star's spanning tree has only 1-leaf cuts — all invalid.
        let result = propose_district(&partial, &adj, &pop, 2, 1, 0.005, &mut rng, 42);
        // Should fail with NoValidCut
        assert!(matches!(result, Err(ProposeError::NoValidCut { .. })),
            "star graph with tight tolerance must produce NoValidCut, got {:?}", result);
    }

    #[test]
    fn propose_district_deterministic() {
        let adj = path_adj(8);
        let pop = vec![100i64; 8];
        let partial = PartialPlan::empty(8);

        let mut rng1 = SmallRng::seed_from_u64(99);
        let mut rng2 = SmallRng::seed_from_u64(99);
        let (p1, w1) = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng1, 0).unwrap();
        let (p2, w2) = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng2, 0).unwrap();

        assert_eq!(p1.assignment, p2.assignment, "same seed → same assignment");
        assert_eq!(w1, w2, "same seed → same log_weight");
    }

    #[test]
    fn propose_district_assigned_is_connected() {
        let adj = grid_adj(4, 4);
        let pop = vec![100i64; 16];
        let partial = PartialPlan::empty(16);
        let mut rng = SmallRng::seed_from_u64(13);

        let (new_partial, _) = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng, 0).unwrap();
        let d1: Vec<usize> = new_partial.assignment.iter().enumerate()
            .filter_map(|(i, a)| if *a == Some(1) { Some(i) } else { None })
            .collect();
        assert!(is_connected(&d1, &adj), "proposed district must be contiguous");
    }

    #[test]
    fn propose_district_weight_is_nonnegative() {
        // log(1) = 0 when only one valid cut; always ≥ 0
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let partial = PartialPlan::empty(4);
        let mut rng = SmallRng::seed_from_u64(5);
        for _ in 0..10 {
            let result = propose_district(&partial, &adj, &pop, 2, 1, 0.1, &mut rng, 0);
            if let Ok((_, lw)) = result {
                assert!(lw >= 0.0, "log_w must be ≥ 0, got {lw}");
            }
        }
    }
}
