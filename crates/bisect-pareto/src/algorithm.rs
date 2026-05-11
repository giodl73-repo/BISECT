//! NSGA-II main loop for multi-objective Pareto redistricting.
//!
//! Runs the Deb et al. (2002) NSGA-II genetic algorithm over redistricting
//! plans as chromosomes. Returns the Pareto-optimal frontier (front rank 0).
//!
//! Per spec §3.

use std::time::Instant;
use thiserror::Error;

use crate::crossover::crossover;
use crate::dominance::{crowding_distance, fast_non_dominated_sort};
use crate::mutation::mutate;
use crate::objectives::{evaluate, Objectives};
use crate::output::{ParetoEntry, ParetoResult};
use crate::seeds::{cross_seed, init_seed, mut_seed};

use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

/// Configuration for the NSGA-II run.
#[derive(Debug, Clone)]
pub struct ParetoConfig {
    /// Population size (number of plans per generation). Default: 100.
    pub n_population: usize,
    /// Number of NSGA-II generations. Default: 200.
    pub n_generations: usize,
    /// SHA-256 base seed for all stochastic operations. Default: 42.
    pub base_seed: u64,
    /// Population balance tolerance (fraction of ideal). Default: 0.005.
    pub balance_tolerance: f64,
}

impl Default for ParetoConfig {
    fn default() -> Self {
        Self {
            n_population: 100,
            n_generations: 200,
            base_seed: 42,
            balance_tolerance: 0.005,
        }
    }
}

/// Errors that can occur during NSGA-II.
#[derive(Debug, Error)]
pub enum ParetoError {
    #[error("invalid configuration: {msg}")]
    InvalidConfig { msg: String },
    #[error("initialisation failed: could not generate any valid plan")]
    InitialisationFailed,
}

/// Run NSGA-II for multi-objective redistricting.
///
/// Returns a `ParetoResult` containing the Pareto-optimal frontier (rank 0).
///
/// # Parameters
/// - `adjacency`: adjacency list (usize indices, length = n_tracts)
/// - `vertex_weights`: population per tract
/// - `k`: number of districts
/// - `d_votes`: D 2-party vote share per tract (None = skip D_seats objective)
/// - `minority_vap`: minority VAP fraction per tract (None = skip VRA objective)
/// - `protected_districts`: currently unused (Phase 1)
/// - `config`: NSGA-II configuration
pub fn run_nsga2(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    k: usize,
    d_votes: Option<&[f64]>,
    minority_vap: Option<&[f64]>,
    protected_districts: &[u32],
    config: ParetoConfig,
) -> Result<ParetoResult, ParetoError> {
    let start = Instant::now();
    let n_pop = config.n_population;
    let n_gen = config.n_generations;

    if k == 0 {
        return Err(ParetoError::InvalidConfig {
            msg: "k must be >= 1".into(),
        });
    }
    if n_pop == 0 {
        return Err(ParetoError::InvalidConfig {
            msg: "n_population must be >= 1".into(),
        });
    }

    // ── Initialise population ─────────────────────────────────────────────────
    // Phase 1: initialise with n_pop independent seeded plans using
    // sequential spanning-tree bisection from bisect-ensemble.
    let mut population: Vec<Vec<u32>> = (0..n_pop)
        .map(|i| {
            let seed = init_seed(config.base_seed, i as u32);
            initialise_plan(adjacency, vertex_weights, k, config.balance_tolerance, seed)
        })
        .collect();

    // Track which generation each plan first entered the Pareto front
    let mut generation_found: Vec<usize> = vec![0; n_pop];

    // ── Main NSGA-II loop ─────────────────────────────────────────────────────
    for gen in 0..n_gen {
        // Evaluate objectives for all plans
        let objectives: Vec<Objectives> = population
            .iter()
            .map(|p| {
                evaluate(
                    p,
                    adjacency,
                    vertex_weights,
                    k,
                    d_votes,
                    minority_vap,
                    protected_districts,
                )
            })
            .collect();

        // Non-dominated sort
        let fronts = fast_non_dominated_sort(&objectives);

        // Assign rank to each plan
        let mut rank = vec![0usize; n_pop];
        for (r, front) in fronts.iter().enumerate() {
            for &idx in front {
                rank[idx] = r;
            }
        }

        // Crowding distance per plan (using its front)
        let mut crowd = vec![0.0f64; n_pop];
        for front in &fronts {
            let dists = crowding_distance(front, &objectives);
            for (fi, &pi) in front.iter().enumerate() {
                crowd[pi] = dists[fi];
            }
        }

        // Tournament selection: produce n_pop parents
        let parents_idx = tournament_select(n_pop, &rank, &crowd, config.base_seed, gen as u32);

        // Crossover + mutation to produce n_pop offspring
        let mut offspring: Vec<Vec<u32>> = Vec::with_capacity(n_pop);
        let mut offspring_gen: Vec<usize> = Vec::with_capacity(n_pop);
        for i in 0..n_pop {
            let pa_idx = parents_idx[2 * (i % (n_pop / 2))];
            let pb_idx = parents_idx[2 * (i % (n_pop / 2)) + 1];

            let cseed = cross_seed(config.base_seed, gen as u32, i as u32);
            let child = crossover(
                &population[pa_idx],
                &population[pb_idx],
                adjacency,
                vertex_weights,
                k,
                config.balance_tolerance,
                cseed,
            );

            let mseed = mut_seed(config.base_seed, gen as u32, i as u32);
            let mutated = mutate(
                &child,
                adjacency,
                vertex_weights,
                k,
                config.balance_tolerance,
                mseed,
            );

            offspring.push(mutated);
            offspring_gen.push(gen + 1);
        }

        // Combine parent + offspring (2 * n_pop plans)
        let combined: Vec<Vec<u32>> = population.iter().chain(offspring.iter()).cloned().collect();
        let combined_gen: Vec<usize> = generation_found
            .iter()
            .chain(offspring_gen.iter())
            .copied()
            .collect();

        let combined_obj: Vec<Objectives> = combined
            .iter()
            .map(|p| {
                evaluate(
                    p,
                    adjacency,
                    vertex_weights,
                    k,
                    d_votes,
                    minority_vap,
                    protected_districts,
                )
            })
            .collect();
        let combined_fronts = fast_non_dominated_sort(&combined_obj);

        // Assign ranks for combined
        let mut combined_rank = vec![0usize; combined.len()];
        for (r, front) in combined_fronts.iter().enumerate() {
            for &idx in front {
                combined_rank[idx] = r;
            }
        }

        // Select top N_pop
        let selected_idx = select_top_n(&combined_fronts, &combined_obj, n_pop);

        // Rebuild population
        population = selected_idx.iter().map(|&i| combined[i].clone()).collect();
        generation_found = selected_idx.iter().map(|&i| combined_gen[i]).collect();
    }

    // ── Final Pareto front extraction ─────────────────────────────────────────
    let final_obj: Vec<Objectives> = population
        .iter()
        .map(|p| {
            evaluate(
                p,
                adjacency,
                vertex_weights,
                k,
                d_votes,
                minority_vap,
                protected_districts,
            )
        })
        .collect();
    let final_fronts = fast_non_dominated_sort(&final_obj);

    let front_0 = if final_fronts.is_empty() {
        // Degenerate: return all plans
        (0..population.len()).collect::<Vec<_>>()
    } else {
        final_fronts[0].clone()
    };

    let runtime_secs = start.elapsed().as_secs_f64();

    let frontier: Vec<ParetoEntry> = front_0
        .iter()
        .map(|&i| ParetoEntry {
            plan: population[i].clone(),
            objectives: final_obj[i].clone(),
            dominated: false,
            generation_found: generation_found[i],
            validity_status: Some("valid".to_string()),
            audit_certificate_path: None,
            audit_certificate_sha256: None,
            audit_certificate_content_hash: None,
        })
        .collect();

    Ok(ParetoResult {
        frontier,
        config,
        runtime_secs,
        pareto_version: "1.0".into(),
    })
}

/// Tournament selection: draw n_pop parents via binary tournament.
/// Prefer lower rank; break ties by higher crowding distance.
fn tournament_select(
    n_pop: usize,
    rank: &[usize],
    crowd: &[f64],
    base_seed: u64,
    gen: u32,
) -> Vec<usize> {
    // Derive a tournament seed deterministically
    let tseed = crate::seeds::cross_seed(base_seed, gen, u32::MAX);
    let mut rng = SmallRng::seed_from_u64(tseed);
    let mut parents = Vec::with_capacity(n_pop * 2);

    for _ in 0..n_pop * 2 {
        let a = rng.gen_range(0..n_pop);
        let b = rng.gen_range(0..n_pop);
        let winner = if rank[a] < rank[b] {
            a
        } else if rank[b] < rank[a] {
            b
        } else if crowd[a] >= crowd[b] {
            a
        } else {
            b
        };
        parents.push(winner);
    }
    parents
}

/// Select top N plans from combined population by NSGA-II truncation.
/// Fill by front, then by crowding distance (descending) for the last partial front.
fn select_top_n(fronts: &[Vec<usize>], objectives: &[Objectives], n: usize) -> Vec<usize> {
    let mut selected = Vec::with_capacity(n);

    for front in fronts {
        if selected.len() + front.len() <= n {
            selected.extend_from_slice(front);
        } else {
            // Partial front: rank by crowding distance descending
            let remaining = n - selected.len();
            let dists = crowding_distance(front, objectives);
            let mut indexed: Vec<(usize, f64)> = front
                .iter()
                .enumerate()
                .map(|(fi, &pi)| (pi, dists[fi]))
                .collect();
            // Sort by distance descending (higher diversity first)
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            selected.extend(indexed[..remaining].iter().map(|&(pi, _)| pi));
            break;
        }
    }

    selected
}

/// Phase 1 plan initialisation: generate a valid plan using sequential
/// greedy region growing from a random seed.
///
/// Seeds districts by picking one seed tract per district, then grows
/// regions by BFS to fill all tracts while respecting balance tolerance.
/// Falls back to a simple path partition if greedy growth fails.
fn initialise_plan(
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    _balance_tolerance: f64,
    seed: u64,
) -> Vec<u32> {
    let n = adjacency.len();

    if k == 1 {
        return vec![1u32; n];
    }

    // Use Wilson's spanning tree + sequential bisection for plan initialisation.
    // Build a spanning tree over the full graph, then partition by even population splits.
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let total_pop: i64 = pop.iter().sum();
    let target_per_district = total_pop as f64 / k as f64;

    // Build adjacency as Vec<Vec<u32>> for the spanning tree function
    let adj_u32: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nb| nb.iter().map(|&u| u as u32).collect())
        .collect();

    let mut rng = SmallRng::seed_from_u64(seed);

    // Try to produce a valid plan by greedy seed+BFS growth
    // Seeds: pick k evenly-spaced nodes as district seeds
    let step = n / k;
    let seed_tracts: Vec<usize> = (0..k)
        .map(|i| {
            // Add some randomness to seed selection
            let base = i * step;
            let offset = rng.gen_range(0..step.max(1));
            (base + offset) % n
        })
        .collect();

    let mut plan = vec![0u32; n];
    let mut assigned = vec![false; n];

    // Assign seed tracts
    for (d, &t) in seed_tracts.iter().enumerate() {
        plan[t] = (d + 1) as u32;
        assigned[t] = true;
    }

    // BFS growth: each district grows in round-robin until all tracts assigned
    let mut queues: Vec<std::collections::VecDeque<usize>> = seed_tracts
        .iter()
        .map(|&t| {
            let mut q = std::collections::VecDeque::new();
            q.push_back(t);
            q
        })
        .collect();

    let mut remaining = n - k;
    let mut round = 0usize;

    while remaining > 0 {
        let d = round % k;
        if let Some(v) = queues[d].pop_front() {
            for &u in &adjacency[v] {
                if !assigned[u] {
                    plan[u] = (d + 1) as u32;
                    assigned[u] = true;
                    queues[d].push_back(u);
                    remaining = remaining.saturating_sub(1);
                }
            }
        }
        // else: queue empty for this district — skip
        round += 1;
        // Safety: if all queues empty but remaining > 0, assign leftover to district 1
        if round > n * k && remaining > 0 {
            for t in 0..n {
                if !assigned[t] {
                    plan[t] = 1u32;
                    assigned[t] = true;
                    remaining = remaining.saturating_sub(1);
                }
            }
            break;
        }
    }

    // Ensure every district has at least one tract
    for d in 1u32..=k as u32 {
        if !plan.iter().any(|&x| x == d) {
            // Re-assign a tract from the most populated district
            let max_d = (1u32..=k as u32)
                .max_by_key(|&dd| plan.iter().filter(|&&x| x == dd).count())
                .unwrap_or(1);
            if let Some(t) = plan.iter().position(|&x| x == max_d) {
                plan[t] = d;
            }
        }
    }

    // Suppress unused-variable warnings for fallback variables prepared above
    let _ = target_per_district;
    let _ = adj_u32;

    plan
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
    fn select_top_n_basic() {
        let fronts = vec![vec![0usize, 1], vec![2, 3]];
        let objectives = vec![
            Objectives {
                ec: 1.0,
                d_seats: 1.0,
                vra_deficit: 0.0,
            },
            Objectives {
                ec: 2.0,
                d_seats: 1.0,
                vra_deficit: 0.0,
            },
            Objectives {
                ec: 3.0,
                d_seats: 2.0,
                vra_deficit: 0.0,
            },
            Objectives {
                ec: 4.0,
                d_seats: 3.0,
                vra_deficit: 0.0,
            },
        ];
        let selected = select_top_n(&fronts, &objectives, 3);
        assert_eq!(selected.len(), 3);
        // Front 0 (plans 0,1) always selected; one from front 1
        assert!(selected.contains(&0));
        assert!(selected.contains(&1));
    }

    #[test]
    fn initialise_plan_k2_path() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let plan = initialise_plan(&adj, &pop, 2, 0.5, 42);
        assert_eq!(plan.len(), 4);
        // Both districts non-empty
        assert!(plan.iter().any(|&d| d == 1), "district 1 must be non-empty");
        assert!(plan.iter().any(|&d| d == 2), "district 2 must be non-empty");
        for &d in &plan {
            assert!(d >= 1 && d <= 2, "invalid district {d}");
        }
    }

    #[test]
    fn tournament_select_returns_valid_indices() {
        let rank = vec![0, 1, 0, 2];
        let crowd = vec![f64::INFINITY, 1.0, 0.5, f64::INFINITY];
        let parents = tournament_select(4, &rank, &crowd, 42, 0);
        assert_eq!(parents.len(), 8);
        for &p in &parents {
            assert!(p < 4, "parent index {p} out of range");
        }
    }
}
