use super::*;

// ── SMC-Percentile (SmcPercentile spec accepted 3.88/4) ───────────────────────

/// Derive the SMC-specific base seed from the run base seed.
///
/// Uses SHA-256("SMCP_RUN_" || base_seed:u64le) → u64le to produce a seed
/// that is independent from all other compositor seeds derived from the same
/// base. This prevents cross-mode seed correlation.
pub(crate) fn derive_smcp_seed(base_seed: u64) -> u64 {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(b"SMCP_RUN_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Run the SMC weighted ensemble and select the plan at the p-th weighted EC quantile.
///
/// This is the only SeedCompositor mode with a calibrated (importance-weighted)
/// stationary distribution — particles represent the uniform distribution over all
/// valid k-district plans, not just a Markov-chain approximation.
///
/// Selection rule (per spec §4.2):
/// - For each particle i, compute EC and record (ec, i, plans[i]).
/// - Sort by (ec ASC, i ASC) for determinism on ties.
/// - Walk the sorted list accumulating weights[orig_idx].
/// - Return the first plan where cumulative_weight >= p.
/// - Special case p=0.0: return first particle with weight > 0.0 (lowest-EC positive-weight plan).
/// - Degenerate case (all weights 0): return the first plan (particle 0).
pub fn run_smc_percentile(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    num_districts: usize,
    base_seed: u64,
    n_particles: usize,
    p: f64,
    resample_threshold: f64,
) -> Result<HashMap<usize, usize>, String> {
    use bisect_smc::{run_smc, SmcConfig};

    let n = adjacency.len();

    if num_districts <= 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let smc_base_seed = derive_smcp_seed(base_seed);

    let config = SmcConfig {
        n_particles,
        resample_threshold,
        pop_tolerance: 0.005,
        base_seed: smc_base_seed,
    };

    let result = run_smc(adjacency, vertex_weights, num_districts, config)
        .map_err(|e| format!("run_smc failed: {e}"))?;

    // For each particle i, compute EC and record (ec, i, plans[i].clone()).
    let mut ranked: Vec<(usize, usize, Vec<u32>)> = result
        .plans
        .iter()
        .enumerate()
        .map(|(i, plan)| {
            let asgn: HashMap<usize, usize> = plan
                .iter()
                .enumerate()
                .map(|(v, &d)| (v, d as usize))
                .collect();
            let ec = count_edge_cuts(&asgn, adjacency);
            (ec, i, plan.clone())
        })
        .collect();

    // Sort by (ec ASC, i ASC) for determinism.
    ranked.sort_by(|(e1, i1, _), (e2, i2, _)| e1.cmp(e2).then(i1.cmp(i2)));

    // Check whether any weight is positive.
    let total_weight: f64 = result.weights.iter().sum();
    let all_zero = total_weight < 1e-300;

    // Walk sorted list accumulating weights by original particle index.
    let mut cumulative = 0.0f64;
    let mut selected: Option<Vec<u32>> = None;

    for (_, orig_idx, plan) in &ranked {
        let w = result.weights[*orig_idx];
        if p == 0.0 {
            // p=0.0: return first positive-weight plan (lowest EC with w > 0).
            if all_zero || w > 0.0 {
                selected = Some(plan.clone());
                break;
            }
        } else {
            cumulative += w;
            if cumulative >= p {
                selected = Some(plan.clone());
                break;
            }
        }
    }

    // Fallback: if nothing selected (e.g. p=1.0 with rounding), take last.
    let chosen = selected.unwrap_or_else(|| {
        ranked
            .last()
            .map(|(_, _, pl)| pl.clone())
            .unwrap_or_else(|| vec![1u32; n])
    });

    // Convert Vec<u32> → HashMap<usize, usize>.
    let assignment: HashMap<usize, usize> = chosen
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    Ok(assignment)
}
