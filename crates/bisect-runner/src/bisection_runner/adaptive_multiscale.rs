use super::*;

// ── Adaptive Multi-scale MCMC ────────────────────────────────────────────────

/// Configuration for `run_multiscale_adaptive`.
///
/// Mirrors `AdaptiveMultiScaleConfig` in BISECT-multiscale but is local to
/// bisection_runner to avoid a crate dependency cycle.
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub total_steps: usize,
    pub target_accept: f64,
    pub initial_alpha: f64,
    pub adapt_interval: usize,
    pub gamma_0: f64,
    pub pop_tolerance: f64,
    /// Multiplier for coarse tolerance: coarse_tol = coarse_tol_factor × pop_tolerance.
    /// Default: 3.0 (per U.5 spec; looser than MultiScale's 2× to avoid over-rejection
    /// during early adaptation when alpha may be far from optimal).
    pub coarse_tol_factor: f64,
    pub p: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            total_steps: 2000,
            target_accept: 0.30,
            initial_alpha: 0.30,
            adapt_interval: 50,
            gamma_0: 0.10,
            pop_tolerance: 0.005,
            coarse_tol_factor: 3.0,
            p: 0.0,
        }
    }
}

/// Diagnostic output from `run_multiscale_adaptive`.
#[derive(Debug)]
pub struct AdaptiveResult {
    /// Alpha after the final adaptation round (last entry of alpha_trace, or initial_alpha
    /// if no adaptation rounds occurred).
    pub final_alpha: f64,
    /// Alpha recorded after each adaptation round (length = floor(total_steps / adapt_interval)).
    pub alpha_trace: Vec<f64>,
    /// Fine-step acceptance rate over all steps (RecomChain.step() always accepts; = 1.0).
    pub fine_acceptance_rate: f64,
    /// Coarse-step acceptance rate: fraction of coarse steps whose rebalance succeeded.
    pub coarse_acceptance_rate: f64,
}

/// Run Adaptive Multi-scale MCMC with Robbins-Monro alpha self-tuning (U.5 spec).
///
/// Extends `run_multiscale` by automatically adapting the coarse-move probability
/// alpha toward `config.target_accept` every `config.adapt_interval` steps using
/// the Robbins-Monro update:
///
/// ```text
/// alpha <- clip(alpha + gamma_t * (recent_coarse_accept - target_accept), 0.05, 0.95)
/// gamma_t = gamma_0 / sqrt(t),  t = adaptation round (1-based)
/// ```
///
/// Seeding: identical to `run_multiscale` — SHA-256("MSC_STEP_" || step:u64le || "_" ||
/// 0u32le || "_" || base_seed:u64le) → u64le.  The alpha draw always consumes one
/// RNG value before the step, regardless of whether alpha is 0 or 1.
///
/// Returns `(best_plan, AdaptiveResult)`.
pub fn run_multiscale_adaptive(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    niter: u32,
    base_seed: u64,
    config: AdaptiveConfig,
    geoids: Option<&std::collections::HashMap<usize, String>>,
    // Resolution level for the fine side (Tract = Option B; BlockGroup = Option A or C)
    fine_level: MultiscaleFineLevel,
    // Coarse level string: "county" or "tract"
    coarse_level: &str,
    // BG adjacency graph (required for Options A and C; None for Option B)
    bg_graph: Option<(
        &[Vec<usize>],
        &[i64],
        &std::collections::HashMap<usize, String>,
    )>,
) -> Result<(HashMap<usize, usize>, AdaptiveResult), String> {
    use bisect_ensemble::recom::RecomChain;
    use bisect_multiscale::rebalance::rebalance;
    use rand::Rng;
    use sha2::Digest;

    let geoids = geoids.ok_or_else(|| {
        "[CONFIG] --search multiscale-adaptive requires GEOID data. \
         Ensure the adjacency file has an accompanying _geoids.json file."
            .to_string()
    })?;

    let coarse_tol = config.coarse_tol_factor * config.pop_tolerance;

    // Resolve fine and coarse adjacency structures
    let (fine_adj, fine_pop, fine_geoids, coarse_adj, coarse_pop, fine_to_coarse) =
        build_multiscale_levels(
            adjacency,
            vertex_weights,
            geoids,
            fine_level,
            coarse_level,
            bg_graph,
        )?;

    if coarse_adj.is_empty() {
        return Err(format!(
            "multiscale-adaptive coarse graph ('{coarse_level}') produced no units"
        ));
    }

    // Build initial plan from METIS (always at tract level for seeding)
    let initial_plan = run_all_splits(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        config.pop_tolerance,
        niter,
        Some(base_seed),
        None,
    )?;

    let n_fine = fine_adj.len();
    let n_coarse = coarse_adj.len();

    // Project initial plan to fine level (same logic as run_multiscale)
    let mut assignment_fine: Vec<u32> = match fine_level {
        MultiscaleFineLevel::Tract => (0..n_fine)
            .map(|i| initial_plan.get(&i).copied().unwrap_or(1) as u32)
            .collect(),
        MultiscaleFineLevel::BlockGroup => {
            let n_tracts = adjacency.len();
            let tract_plan: Vec<u32> = (0..n_tracts)
                .map(|i| initial_plan.get(&i).copied().unwrap_or(1) as u32)
                .collect();
            let tract_geoid_lookup: HashMap<&str, usize> =
                geoids.iter().map(|(&idx, g)| (g.as_str(), idx)).collect();
            let mut asgn = vec![1u32; n_fine];
            for (&bg_idx, bg_geoid) in fine_geoids.iter() {
                let tract_prefix = &bg_geoid[..bg_geoid.len().min(11)];
                if let Some(&tract_idx) = tract_geoid_lookup.get(tract_prefix) {
                    if tract_idx < tract_plan.len() {
                        asgn[bg_idx] = tract_plan[tract_idx];
                    }
                }
            }
            asgn
        }
    };

    // Build coarse initial assignment
    let mut assignment_coarse: Vec<u32> = vec![1u32; n_coarse];
    for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
        if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
            assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
        }
    }

    // Build fine adjacency as Vec<Vec<u32>> for RecomChain
    let fine_adj_u32: Vec<Vec<u32>> = fine_adj
        .iter()
        .map(|nb| nb.iter().map(|&x| x as u32).collect())
        .collect();

    // Fine chain
    let mut fine_chain = RecomChain::new(
        fine_adj_u32,
        fine_pop.clone(),
        assignment_fine.clone(),
        num_districts as u32,
        config.pop_tolerance,
    );

    // Coarse chain — coarse_tol_factor × fine tolerance
    let coarse_adj_u32: Vec<Vec<u32>> = coarse_adj
        .iter()
        .map(|nb| nb.iter().map(|&x| x as u32).collect())
        .collect();
    let mut coarse_chain = RecomChain::new(
        coarse_adj_u32,
        coarse_pop,
        assignment_coarse.clone(),
        num_districts as u32,
        coarse_tol,
    );

    // Deterministic per-step seed derivation — identical prefix to run_multiscale
    let step_seed_fn = |step: u64| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"MSC_STEP_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(0u32.to_le_bytes()); // chain_idx = 0
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // Collect (ec, step_idx, assignment) for all visited plans
    let initial_fine_plan: HashMap<usize, usize> = assignment_fine
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();
    let initial_ec = count_edge_cuts(&initial_fine_plan, &fine_adj);
    let mut visited: Vec<(usize, usize, Vec<u32>)> = Vec::with_capacity(config.total_steps + 1);
    visited.push((initial_ec, 0, assignment_fine.clone()));

    // Robbins-Monro state
    let mut alpha = config.initial_alpha;
    let mut alpha_trace: Vec<f64> = Vec::new();
    let mut coarse_accept_window: Vec<bool> = Vec::new();

    // Global counters for AdaptiveResult
    let mut total_fine_steps = 0u64;
    let mut total_coarse_steps = 0u64;
    let mut total_fine_accepted = 0u64;
    let mut total_coarse_accepted = 0u64;

    for step in 1..=config.total_steps {
        let seed = step_seed_fn(step as u64);
        let mut rng = SmallRng::seed_from_u64(seed);

        // Alpha draw always consumes one RNG value (seeding contract)
        let is_coarse = rng.gen::<f64>() < alpha;

        if is_coarse {
            total_coarse_steps += 1;
            // Coarse move: step the coarse-level chain
            coarse_chain.step(&mut rng);

            // Project coarse assignment back to fine level
            for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                    assignment_fine[fine_idx] = coarse_chain.assignment[coarse_idx];
                }
            }

            // Rebalance fine-level plan; reject if rebalancing fails
            let mut asgn_work = assignment_fine.clone();
            let balanced = rebalance(
                &mut asgn_work,
                &fine_adj,
                &fine_pop,
                num_districts as u32,
                config.pop_tolerance,
                200,
            );
            if balanced {
                assignment_fine = asgn_work;
                fine_chain.assignment = assignment_fine.clone();
                // Sync coarse assignment back from rebalanced fine assignment
                for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                    if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                        assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
                    }
                }
                coarse_chain.assignment = assignment_coarse.clone();
                coarse_accept_window.push(true);
                total_coarse_accepted += 1;
            } else {
                // Coarse move rejected — restore from current fine chain
                for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                    if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                        assignment_fine[fine_idx] = fine_chain.assignment[fine_idx];
                        assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
                    }
                }
                coarse_chain.assignment = assignment_coarse.clone();
                coarse_accept_window.push(false);
            }
        } else {
            total_fine_steps += 1;
            total_fine_accepted += 1; // RecomChain::step always accepts
                                      // Fine move: step the fine-level chain
            fine_chain.step(&mut rng);
            assignment_fine = fine_chain.assignment.clone();
            // Sync coarse assignment
            for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                    assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
                }
            }
            coarse_chain.assignment = assignment_coarse.clone();
        }

        // Robbins-Monro alpha adaptation
        if step % config.adapt_interval == 0 {
            let t = (step / config.adapt_interval) as f64; // 1-based round
            let gamma_t = config.gamma_0 / t.sqrt();
            let recent_accept = if coarse_accept_window.is_empty() {
                alpha // no coarse steps this window — leave alpha unchanged
            } else {
                coarse_accept_window.iter().filter(|&&a| a).count() as f64
                    / coarse_accept_window.len() as f64
            };
            alpha = (alpha + gamma_t * (recent_accept - config.target_accept)).clamp(0.05, 0.95);
            alpha_trace.push(alpha);
            coarse_accept_window.clear();
        }

        let current_plan: HashMap<usize, usize> = assignment_fine
            .iter()
            .enumerate()
            .map(|(i, &d)| (i, d as usize))
            .collect();
        let ec = count_edge_cuts(&current_plan, &fine_adj);
        visited.push((ec, step, assignment_fine.clone()));
    }

    // Sort by (EC ASC, step ASC) for determinism; select at rank floor(p * n)
    visited.sort_by(|(e1, s1, _), (e2, s2, _)| e1.cmp(e2).then(s1.cmp(s2)));
    let rank = ((config.p * visited.len() as f64).floor() as usize).min(visited.len() - 1);
    let (_, _, best_asgn) = &visited[rank];
    let result_plan: HashMap<usize, usize> = best_asgn
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    let final_alpha = alpha_trace.last().copied().unwrap_or(config.initial_alpha);
    let adaptive_result = AdaptiveResult {
        final_alpha,
        alpha_trace,
        fine_acceptance_rate: if total_fine_steps > 0 {
            total_fine_accepted as f64 / total_fine_steps as f64
        } else {
            0.0
        },
        coarse_acceptance_rate: if total_coarse_steps > 0 {
            total_coarse_accepted as f64 / total_coarse_steps as f64
        } else {
            0.0
        },
    };

    Ok((result_plan, adaptive_result))
}
