use super::*;

/// Resolution level for the fine side of multi-scale MCMC.
///
/// - `Tract`      — Option B: fine=tract, coarse=county (default; no extra data).
/// - `BlockGroup` — Option A (fine=BG, coarse=tract) or C (fine=BG, coarse=county).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiscaleFineLevel {
    Tract,
    BlockGroup,
}

impl MultiscaleFineLevel {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "tract" => Ok(Self::Tract),
            "bg" | "block_group" | "block-group" => Ok(Self::BlockGroup),
            other => Err(format!(
                "unknown multiscale fine level '{other}'. Valid: tract, bg"
            )),
        }
    }
}

/// Multi-scale MCMC — supports Options A, B, and C.
///
/// **Option B** (default, `fine_level=Tract`, `coarse="county"`):
///   fine=tract, coarse=county. County adjacency is built on-the-fly from tract GEOIDs
///   (GEOID prefix[:5] = county FIPS) — no extra data files needed.
///
/// **Option A** (`fine_level=BlockGroup`, `coarse="tract"`):
///   fine=block-group, coarse=tract. Caller must supply `bg_graph` with BG adjacency.
///   `derive_partition(bg_geoids, tract_geoids)` maps each BG to its parent tract.
///   The fine chain operates on BG graph; coarse chain on the tract graph (= `adjacency`).
///
/// **Option C** (`fine_level=BlockGroup`, `coarse="county"`):
///   fine=block-group, coarse=county. Caller must supply `bg_graph`.
///   County adjacency is derived from BG GEOIDs via prefix[:5].
///
/// `adjacency` / `vertex_weights` / `edge_weights` / `geoids`:
///   For Options B: the tract-level graph.
///   For Options A/C: the tract-level graph is used only for the initial METIS seed;
///   the fine chain runs on `bg_graph`.
///
/// When `geoids` is `None` the function returns a `[CONFIG]` error.
/// When `fine_level=BlockGroup` and `bg_graph=None`, returns a descriptive error.
///
/// CLI: `--search multiscale --multiscale-steps 2000 --multiscale-alpha 0.3`
pub fn run_multiscale(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    total_steps: usize,
    alpha: f64,
    p: f64,
    // GEOID map for building county coarsening — None falls back to CONFIG error
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
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::recom::RecomChain;
    use bisect_multiscale::rebalance::rebalance;
    use rand::Rng;
    use sha2::Digest;

    let geoids = geoids.ok_or_else(|| {
        "[CONFIG] --search multiscale requires GEOID data. \
         Ensure the adjacency file has an accompanying _geoids.json file."
            .to_string()
    })?;

    // Resolve the fine and coarse adjacency structures based on option
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
            "multiscale coarse graph ('{coarse_level}') produced no units"
        ));
    }

    // Build initial plan from METIS (always at the tract level for seeding)
    let initial_plan = run_all_splits(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        Some(base_seed),
        None,
    )?;

    let n_fine = fine_adj.len();
    let n_coarse = coarse_adj.len();

    // For Options A/C: project the tract-level initial plan down to BG level
    // by assigning each BG the district of its parent tract.
    // For Option B: fine IS tract, so initial_plan maps directly.
    let mut assignment_fine: Vec<u32> = match fine_level {
        MultiscaleFineLevel::Tract => {
            // Option B: fine = tract (same as initial_plan)
            (0..n_fine)
                .map(|i| initial_plan.get(&i).copied().unwrap_or(1) as u32)
                .collect()
        }
        MultiscaleFineLevel::BlockGroup => {
            // Options A/C: fine = BG; map each BG to its parent tract's district
            let n_tracts = adjacency.len();
            let tract_plan: Vec<u32> = (0..n_tracts)
                .map(|i| initial_plan.get(&i).copied().unwrap_or(1) as u32)
                .collect();
            // bg_to_tract_partition: bg_idx -> tract_idx (built during build_multiscale_levels)
            // We need to recover this mapping from fine_geoids (BG) to tract geoids.
            // Re-derive from fine_geoids (already validated in build_multiscale_levels).
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

    // Build coarse initial assignment: each coarse unit takes the mode district of its fine units
    let mut assignment_coarse: Vec<u32> = vec![1u32; n_coarse];
    for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
        if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
            assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
        }
    }

    // Build adjacency as Vec<Vec<u32>> for RecomChain
    let fine_adj_u32: Vec<Vec<u32>> = fine_adj
        .iter()
        .map(|nb| nb.iter().map(|&x| x as u32).collect())
        .collect();

    // Fine chain (operates at the fine resolution)
    let mut fine_chain = RecomChain::new(
        fine_adj_u32,
        fine_pop.clone(),
        assignment_fine.clone(),
        num_districts as u32,
        balance_tolerance,
    );

    // Coarse chain — looser tolerance to allow coarse moves to succeed
    let coarse_adj_u32: Vec<Vec<u32>> = coarse_adj
        .iter()
        .map(|nb| nb.iter().map(|&x| x as u32).collect())
        .collect();
    let mut coarse_chain = RecomChain::new(
        coarse_adj_u32,
        coarse_pop,
        assignment_coarse.clone(),
        num_districts as u32,
        balance_tolerance * 3.0,
    );

    // Deterministic per-step seed derivation
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

    // Compute initial EC on the fine graph (BG for A/C, tract for B)
    let initial_fine_plan: HashMap<usize, usize> = assignment_fine
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();
    let initial_ec = count_edge_cuts(&initial_fine_plan, &fine_adj);
    let mut visited: Vec<(usize, usize, Vec<u32>)> = Vec::with_capacity(total_steps + 1);
    visited.push((initial_ec, 0, assignment_fine.clone()));

    for step in 1..=total_steps {
        let seed = step_seed_fn(step as u64);
        let mut rng = SmallRng::seed_from_u64(seed);

        let is_coarse = rng.gen::<f64>() < alpha;

        if is_coarse {
            // Coarse move: step the coarse-level chain
            coarse_chain.step(&mut rng);

            // Project coarse assignment back to fine level
            for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                    assignment_fine[fine_idx] = coarse_chain.assignment[coarse_idx];
                }
            }

            // Rebalance fine-level plan; reject coarse move if rebalancing fails
            let mut asgn_work = assignment_fine.clone();
            let balanced = rebalance(
                &mut asgn_work,
                &fine_adj,
                &fine_pop,
                num_districts as u32,
                balance_tolerance,
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
            } else {
                // Coarse move rejected — restore from current fine chain
                for (fine_idx, &coarse_idx) in fine_to_coarse.iter().enumerate() {
                    if coarse_idx < n_coarse && fine_idx < assignment_fine.len() {
                        assignment_fine[fine_idx] = fine_chain.assignment[fine_idx];
                        assignment_coarse[coarse_idx] = assignment_fine[fine_idx];
                    }
                }
                coarse_chain.assignment = assignment_coarse.clone();
            }
        } else {
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
    let rank = ((p * visited.len() as f64).floor() as usize).min(visited.len() - 1);
    let (_, _, best_asgn) = &visited[rank];
    Ok(best_asgn
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect())
}

/// Build fine and coarse adjacency structures for multi-scale MCMC.
///
/// Returns `(fine_adj, fine_pop, fine_geoids, coarse_adj, coarse_pop, fine_to_coarse)`.
///
/// - Option B (Tract→County): fine_adj = adjacency (tract), coarse built from geoid prefix[:5].
/// - Option A (BG→Tract): fine_adj = bg_graph adjacency, coarse_adj = adjacency (tract),
///   fine_to_coarse via derive_partition(bg_geoids, tract_geoids).
/// - Option C (BG→County): fine_adj = bg_graph adjacency, coarse built from bg_geoids prefix[:5].
pub(crate) fn build_multiscale_levels<'a>(
    tract_adj: &[Vec<usize>],
    tract_pop: &[i64],
    tract_geoids: &'a std::collections::HashMap<usize, String>,
    fine_level: MultiscaleFineLevel,
    coarse_level: &str,
    bg_graph: Option<(
        &[Vec<usize>],
        &[i64],
        &'a std::collections::HashMap<usize, String>,
    )>,
) -> Result<
    (
        Vec<Vec<usize>>,                              // fine_adj
        Vec<i64>,                                     // fine_pop
        &'a std::collections::HashMap<usize, String>, // fine_geoids (borrows bg or tract)
        Vec<Vec<usize>>,                              // coarse_adj
        Vec<i64>,                                     // coarse_pop
        Vec<usize>,                                   // fine_to_coarse
    ),
    String,
> {
    use crate::adjacency_loader::{build_county_coarsening, derive_partition};

    match fine_level {
        MultiscaleFineLevel::Tract => {
            // Option B: fine=tract, coarse=county (only valid coarse for tract-fine)
            let (coarse_adj, coarse_pop, fine_to_coarse) =
                build_county_coarsening(tract_geoids, tract_adj, tract_pop)
                    .map_err(|e| format!("county coarsening failed: {e}"))?;
            Ok((
                tract_adj.to_vec(),
                tract_pop.to_vec(),
                tract_geoids,
                coarse_adj,
                coarse_pop,
                fine_to_coarse,
            ))
        }
        MultiscaleFineLevel::BlockGroup => {
            let (bg_adj, bg_pop, bg_geoids) = bg_graph.ok_or_else(|| {
                "[CONFIG] --multiscale-fine bg requires block-group adjacency data. \
                 Run: bisect fetch --type adjacency --resolution block_group --states <STATE> --year <YEAR>".to_string()
            })?;

            match coarse_level {
                "tract" => {
                    // Option A: fine=BG, coarse=tract
                    // fine_to_coarse: bg_idx -> tract_idx via GEOID prefix[:11]
                    let fine_to_coarse = derive_partition(bg_geoids, tract_geoids)
                        .map_err(|e| format!("BG->tract partition failed: {e}"))?;
                    // Coarse (tract) adjacency and population
                    let coarse_adj = tract_adj.to_vec();
                    let coarse_pop = tract_pop.to_vec();
                    Ok((
                        bg_adj.to_vec(),
                        bg_pop.to_vec(),
                        bg_geoids,
                        coarse_adj,
                        coarse_pop,
                        fine_to_coarse,
                    ))
                }
                "county" | _ => {
                    // Option C: fine=BG, coarse=county
                    // Build county coarsening directly from BG GEOIDs (prefix[:5])
                    let (coarse_adj, coarse_pop, fine_to_coarse) =
                        build_county_coarsening(bg_geoids, bg_adj, bg_pop)
                            .map_err(|e| format!("BG->county coarsening failed: {e}"))?;
                    Ok((
                        bg_adj.to_vec(),
                        bg_pop.to_vec(),
                        bg_geoids,
                        coarse_adj,
                        coarse_pop,
                        fine_to_coarse,
                    ))
                }
            }
        }
    }
}
