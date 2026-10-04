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
/// Uses the native weighted particle sampler. A finite sample and its numerical
/// diagnostics do not establish distribution calibration or global optimality.
///
/// Selection rule (per spec §4.2):
/// - For each particle i, compute EC and record (ec, i, plans[i]).
/// - Sort by (ec ASC, i ASC) for determinism on ties.
/// - Walk the sorted list accumulating weights[orig_idx].
/// - Return the first plan where cumulative_weight >= p.
/// - Special case p=0.0: return first particle with weight > 0.0 (lowest-EC positive-weight plan).
/// - Zero-weight particles are never selected; no positive weight is an error.
pub fn run_smc_percentile(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    num_districts: usize,
    base_seed: u64,
    n_particles: usize,
    p: f64,
    resample_threshold: f64,
) -> Result<HashMap<usize, usize>, String> {
    run_smc_percentile_with_evidence(adjacency, vertex_weights, num_districts, base_seed, n_particles, p, resample_threshold).map(|result| result.0)
}

/// Same native SMC selection, with the ranked weight ledger and diagnostics.
pub fn run_smc_percentile_with_evidence(
    adjacency: &[Vec<usize>], vertex_weights: &[i64], num_districts: usize,
    base_seed: u64, n_particles: usize, p: f64, resample_threshold: f64,
) -> Result<(HashMap<usize, usize>, serde_json::Value), String> {
    use bisect_smc::{run_smc, SmcConfig};
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err("Invalid SMC percentile.".into());
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
    if !total_weight.is_finite() || total_weight < 1e-300 || result.weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return Err("SMC has no finite positive particle weight.".into());
    }

    // Walk sorted list accumulating weights by original particle index.
    let mut cumulative = 0.0f64;
    let mut selected: Option<Vec<u32>> = None;
    let mut selected_index = None;

    for (_, orig_idx, plan) in &ranked {
        let w = result.weights[*orig_idx];
        if w <= 0.0 { continue; }
        if p == 0.0 {
            // p=0.0: return first positive-weight plan (lowest EC with w > 0).
            if w > 0.0 {
                selected = Some(plan.clone());
                selected_index = Some(*orig_idx);
                break;
            }
        } else {
            cumulative += w;
            if cumulative >= p {
                selected = Some(plan.clone());
                selected_index = Some(*orig_idx);
                break;
            }
        }
    }

    // Fallback: if nothing selected (e.g. p=1.0 with rounding), take last.
    let chosen = selected.unwrap_or_else(|| {
        let (_, index, plan) = ranked.iter().rev().find(|(_, index, _)| result.weights[*index] > 0.0).expect("positive particle weight checked");
        selected_index = Some(*index);
        plan.clone()
    });

    // Convert Vec<u32> → HashMap<usize, usize>.
    let assignment: HashMap<usize, usize> = chosen
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    let selected_index = selected_index.expect("selected positive particle");
    let evidence = serde_json::json!({
        "method":"smc-percentile", "rng":"chacha12-u64-v1", "objective":"unweighted-edge-cut",
        "base_seed":base_seed.to_string(), "sampler_seed":smc_base_seed.to_string(),
        "particles":n_particles, "percentile":p, "resample_threshold":resample_threshold,
        "sampler_tolerance":0.005, "tolerance_basis":"remaining-component-total", "math":"libm-exp-log-v1",
        "selected_particle":selected_index, "selected_edge_cut":count_edge_cuts(&assignment, adjacency),
        "resample_count":result.resample_count, "resample_rounds":result.resample_rounds,
        "ess_trace":result.ess_trace,
        "ranked_particles":ranked.iter().map(|(cut,index,_)|serde_json::json!({"particle":index,"edge_cut":cut,"weight":result.weights[*index]})).collect::<Vec<_>>()
    });
    Ok((assignment,evidence))
}
