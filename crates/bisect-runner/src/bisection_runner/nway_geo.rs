use super::*;

/// Run n-way partitioning: call gpmetis once with nparts=k.
///
/// Direct n-way is faster than recursive bisection (D.2 research: 3.68s vs 11.33s).
/// D.2 also shows equivalent VRA success rates (47.5% vs 48.3%, p=0.634).
///
/// Target weights: equal partitioning (1/k per district). The last weight is
/// inferred by METIS so the sum is exactly 1.0 (avoids floating-point drift).
///
/// AC-05 invariant: all target weights sum to 1.0.
/// The approach: write n-1 explicit weights of 1/k; METIS infers the last.
/// This guarantees sum = (n-1)*(1/k) + inferred = 1.0 regardless of rounding.
pub fn run_nway_partition(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    ufactor: f64,
    niter: u32,
    seed: Option<u64>,
) -> Result<HashMap<usize, usize>, String> {
    run_nway_partition_tuned(adjacency,vertex_weights,edge_weights,num_districts,ufactor,niter,seed,"cut",1)
}

/// Pure Rust refinement objective and internal trial controls. Trial selection
/// retains metis-core's population-excess then edge-cut ordering.
pub fn run_nway_partition_tuned(
    adjacency:&[Vec<usize>], vertex_weights:&[i64], edge_weights:&HashMap<(usize,usize),f64>,
    num_districts:usize, ufactor:f64, niter:u32, seed:Option<u64>, objective:&str, trials:u32,
) -> Result<HashMap<usize,usize>,String> {
    if !["cut","volume"].contains(&objective) || !(1..=100).contains(&trials) {return Err("Invalid n-way objective or internal trial count.".into());}
    #[cfg(feature="c-ffi-engine")]
    if objective!="cut" || trials!=1 {return Err("Advanced n-way controls require the pure Rust engine.".into());}
    let n = adjacency.len();
    if num_districts == 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let (xadj, adjncy) = adj_to_csr(adjacency);
    let vwgt: Vec<i32> = vertex_weights.iter().map(|&w| (w as i32).max(1)).collect();
    let adjwgt = ew_to_adjwgt(
        adjacency,
        if edge_weights.is_empty() {
            None
        } else {
            Some(edge_weights)
        },
    );

    // Equal target weights: first k-1 partitions get 1/k each; last gets remainder.
    // This guarantees the weights sum exactly to 1.0 in f32 regardless of k.
    let weight_per = 1.0_f32 / num_districts as f32;
    let mut tpwgts: Vec<f32> = vec![weight_per; num_districts];
    let total: f32 = tpwgts[..num_districts - 1].iter().sum();
    tpwgts[num_districts - 1] = 1.0_f32 - total;

    let uf_int = ((ufactor - 1.0) * 1000.0).round() as i32;
    let uf_int = uf_int.clamp(1, 1000);

    // ── k-way partition via the selected METIS backend ───────────────────────
    let part: Vec<i32> = {
        #[cfg(feature = "c-ffi-engine")]
        {
            let mut part = vec![0i32; n];
            let graph = metis::Graph::new(1, num_districts as i32, &xadj, &adjncy)
                .map_err(|e| format!("METIS n-way graph init: {e}"))?
                .set_vwgt(&vwgt)
                .set_tpwgts(&tpwgts);
            let graph = if let Some(ref ew) = adjwgt {
                graph.set_adjwgt(ew)
            } else {
                graph
            };
            let graph = graph
                .set_option(metis::option::UFactor(uf_int))
                .set_option(metis::option::NIter(niter as i32))
                .set_option(metis::option::Contig(true))
                .set_option(metis::option::MinConn(true));
            let graph = if let Some(s) = seed {
                graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
            } else {
                graph
            };
            graph
                .part_kway(&mut part)
                .map_err(|e| format!("METIS n-way partition failed: {e}"))?;
            part
        }
        #[cfg(not(feature = "c-ffi-engine"))]
        {
            use metis_core::{
                CsrGraph as RustNwayCsr, MetisParams as RustNwayParams,
                MetisPartitioner as RustNwayPartitioner, Partitioner as RustNwayTrait,
            };
            let g = RustNwayCsr::new(
                xadj.iter().map(|&x| x as u32).collect(),
                adjncy.iter().map(|&x| x as u32).collect(),
                1,
                vwgt.clone(),
                adjwgt.clone(),
            )
            .map_err(|e| format!("metis-core n-way graph: {e}"))?;
            let uf_u32 = (uf_int as u32).clamp(1, 1000);
            // N-way must use the direct k-way algorithm, matching its native contract.
            let mut params = RustNwayParams::kway()
                .with_objective(if objective=="volume" {metis_core::ObjectiveType::Volume} else {metis_core::ObjectiveType::Cut})
                .with_ncuts(trials)
                .with_ufactor(uf_u32)
                .with_niter(niter as u32)
                .with_contiguity(true)
                .with_min_connectivity(true)
                // Retain enough coarse vertices for every requested district.
                // A fixed target of 20 can collapse a large k-way problem to
                // one indivisible heavy vertex per part before refinement.
                .with_coarsen_to((num_districts as u32).saturating_mul(40).max(20));
            if let Some(seed) = seed {
                params = params.with_seed(seed);
            }
            let k = num_districts as u32;
            let partition = RustNwayPartitioner::with_params(params, k)
                .split(&g, k, seed)
                .map_err(|e| format!("metis-core n-way k={num_districts}: {e}"))?;
            partition.assignment().iter().map(|&p| p as i32).collect()
        }
    };

    // Convert 0-based METIS output to 1-based district IDs
    Ok(part
        .iter()
        .enumerate()
        .map(|(tract, &p)| (tract, p as usize + 1))
        .collect())
}

/// Round the Democratic quota at its geometric-mean threshold, then use the
/// complement for the other child. Both children must receive at least one seat.
/// This is the existing HH-style quota rounding rule, not an outcome guarantee.
pub fn proportional_section_seat_counts(d: f64, seats: usize) -> Result<(usize,usize),String> {
    if !d.is_finite() || !(0.0..=1.0).contains(&d) || seats < 2 {
        return Err("ProportionalSection seat allocation requires a finite share and at least two seats.".into());
    }
    let quota = d.clamp(0.01,0.99)*seats as f64;
    let floor = quota.floor() as usize;
    let threshold = ((floor as f64)*(floor as f64+1.0)).sqrt();
    let rounded = if floor > 0 && quota > threshold {floor+1} else {floor.max(1)};
    let left = rounded.clamp(1,seats-1);
    Ok((left,seats-left))
}

/// ProportionalSection (T.5): root population/vote constraint with quota rounding.
///
/// Uses ncon=2 with vertex weights [population, D_votes]. The tpwgts are set by
/// the T.5 formula: [k_D/k, 1-k_R/(2dk), k_R/k, k_R/(2dk)] where d is the
/// statewide two-party Democratic fraction. k_D is rounded at the geometric
/// mean threshold, k_R is its complement, and both are clamped to positive counts.
///
/// Only this rounded quota ratio is tried (not all ratios). Multiple seeds.
/// Recursive calls use ncon=1 (partisan constraint only at first bisection).
///
/// Returns (assignments, k_D, k_R, best_ec, d_statewide).
pub fn run_proportional_section(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],   // population
    vertex_d_votes: &[f64],   // Democratic vote counts per tract
    vertex_two_party: &[f64], // Democrat + Republican two-party vote totals per tract
    edge_weights: &std::collections::HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds: usize,
    eta: f64, // ubvec[1] for D_votes constraint (1.05–1.20)
    intermediate_dir: Option<&std::path::Path>,
) -> Result<
    (
        std::collections::HashMap<usize, usize>,
        usize,
        usize,
        f64,
        f64,
    ),
    String,
> {
    let n = adjacency.len();
    if n == 0 || vertex_weights.len()!=n || vertex_d_votes.len()!=n || vertex_two_party.len()!=n
        || num_districts==0 || num_districts>n || seeds==0 || niter==0
        || !balance_tolerance.is_finite() || !(0.0..=1.0).contains(&balance_tolerance)
        || !eta.is_finite() || !(1.0..=2.0).contains(&eta)
        || vertex_weights.iter().any(|&p|p<0)
        || vertex_d_votes.iter().zip(vertex_two_party).any(|(&dv,&tv)| !dv.is_finite() || dv.is_sign_negative() || !tv.is_finite() || tv.is_sign_negative() || dv>tv)
        || adjacency.iter().enumerate().any(|(u,ns)|ns.iter().any(|&v|v>=n || v==u || !adjacency[v].contains(&u))) {
        return Err("Invalid ProportionalSection graph, election dimensions, counts or options.".into());
    }
    let total_pop = vertex_weights.iter().try_fold(0i64,|sum,&p|sum.checked_add(p))
        .ok_or("ProportionalSection population total overflows i64.")?;
    let total_d: f64 = vertex_d_votes.iter().sum();
    let total_two_party: f64 = vertex_two_party.iter().sum();
    if total_pop==0 || !total_d.is_finite() || total_d<=0.0 || !total_two_party.is_finite() || total_two_party<=0.0 {
        return Err("ProportionalSection requires positive finite population, Democratic and two-party totals.".into());
    }
    let d = (total_d/total_two_party).clamp(0.01,0.99);
    if num_districts == 1 {return Ok(((0..n).map(|i|(i,1)).collect(),1,0,0.0,d));}
    let (k_d,k_r) = proportional_section_seat_counts(d,num_districts)?;

    eprintln!(
        "[proportional] d={:.3} k_D={} k_R={} eta={}",
        d, k_d, k_r, eta
    );

    // T.5 tpwgts: right (R-bloc) gets minimum D for 50% D concentration
    let d_right_target = (k_r as f64 / (2.0 * d * num_districts as f64)).clamp(0.01, 0.99);
    let d_left_target = 1.0 - d_right_target;
    let pop_left = k_d as f64 / num_districts as f64;
    let pop_right = k_r as f64 / num_districts as f64;

    let tpwgts = vec![
        pop_left as f32,
        d_left_target as f32,
        pop_right as f32,
        d_right_target as f32,
    ];
    let ubvec = vec![1.001f32, eta as f32];

    // Scale D votes to integer weights for METIS (scale to match pop magnitude)
    let pop_scale = total_pop as f64 / total_d.max(1.0);
    let d_weights_i64: Vec<i64> = vertex_d_votes
        .iter()
        .map(|&v| ((v * pop_scale) as i64).max(1))
        .collect();

    // Interleaved vwgt: [pop_0, d_0, pop_1, d_1, ...]
    let vwgt_flat: Vec<i64> = vertex_weights
        .iter()
        .zip(d_weights_i64.iter())
        .flat_map(|(&p, &dv)| [p.max(1), dv])
        .collect();

    // Try multiple seeds on the HH ratio
    let all_tracts: std::collections::HashSet<usize> = (0..n).collect();
    let mut best_ec = f64::INFINITY;
    let mut best_left = std::collections::HashSet::new();
    let mut best_right = std::collections::HashSet::new();

    for seed in 1..=(seeds as u64) {
        match split_subgraph(
            adjacency,
            &vwgt_flat,
            2,
            edge_weights,
            &all_tracts,
            1.0 + balance_tolerance / num_districts as f64,
            niter,
            Some(seed),
            Some(tpwgts.clone()),
            Some(ubvec.clone()),
        ) {
            Ok((l, r)) => {
                let ec = weighted_edge_cut(edge_weights, &l);
                if ec < best_ec {
                    best_ec = ec;
                    best_left = l;
                    best_right = r;
                }
            }
            Err(e) => {
                if seed == 1 {
                    eprintln!(
                        "[proportional] seed 1 error: {}",
                        &e.chars().take(200).collect::<String>()
                    );
                }
            }
        }
    }

    if best_left.is_empty() {
        return Err(format!(
            "proportional-section: all {} seeds failed for {}:{}",
            seeds, k_d, k_r
        ));
    }

    // Actual D fraction achieved
    let mut left_indices: Vec<_> = best_left.iter().copied().collect();
    left_indices.sort_unstable();
    let d_left_actual: f64 = left_indices.iter().map(|&v| vertex_d_votes[v]).sum::<f64>() / total_d;
    eprintln!(
        "[proportional] winner: D_left={:.1}% (target {:.1}%), EC={:.0}km",
        d_left_actual * 100.0,
        d_left_target * 100.0,
        best_ec / 1000.0
    );

    if let Some(dir) = intermediate_dir {
        let _ = std::fs::create_dir_all(dir.join("depth_01"));
        let mut d1 = std::collections::HashMap::new();
        for &v in &best_left {
            d1.insert(v, 1);
        }
        for &v in &best_right {
            d1.insert(v, 2);
        }
        let _ = write_intermediate_round(&dir.join("depth_01"), &d1);
    }

    // Recurse with ncon=1 standard bisection
    let left_asgn = recurse_geosection(
        &best_left,
        adjacency,
        vertex_weights,
        edge_weights,
        k_d,
        balance_tolerance,
        niter,
        seeds.min(50),
        1,
        &crate::geosection_orientation::CentroidMap::new(),
        0.0,
        1,
    )?;
    let right_asgn = recurse_geosection(
        &best_right,
        adjacency,
        vertex_weights,
        edge_weights,
        k_r,
        balance_tolerance,
        niter,
        seeds.min(50),
        k_d + 1,
        &crate::geosection_orientation::CentroidMap::new(),
        0.0,
        1,
    )?;

    let mut assignments = left_asgn;
    assignments.extend(right_asgn);
    if assignments.len() != n {
        return Err(format!(
            "proportional-section incomplete: {}/{}",
            assignments.len(),
            n
        ));
    }
    Ok((assignments, k_d, k_r, best_ec, d))
}

///   - At the top-level ratio scan, uses tpwgts=[pop_frac, 0.5, 1-pop_frac, 0.5]
///     and ubvec=[1.001, 1.10] (tight pop balance, 10% area swing).
///   - Performs Lorenz pre-filtering to skip infeasible ratios.
///   - Recursive calls always use ncon=1 (area constraint only at first level).
///
/// Returns (assignments, natural_ratio_left, natural_ratio_right, natural_ec).
pub fn run_geosection(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    intermediate_dir: Option<&Path>,
    centroids: &crate::geosection_orientation::CentroidMap,
    lambda: f64,
    vertex_areas_m2: Option<&[f64]>,
    area_swing: f64,
    minority_vap: Option<&[f64]>,
    w_vra: f64,
    mka_theta_override: Option<f64>,
) -> Result<(HashMap<usize, usize>, usize, usize, f64), String> {
    run_geosection_seeded(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        seeds_per_ratio,
        intermediate_dir,
        centroids,
        lambda,
        vertex_areas_m2,
        area_swing,
        minority_vap,
        w_vra,
        mka_theta_override,
        1,
    )
}

/// Ratio search using the configured consecutive seed walk at every level.
pub fn run_geosection_seeded(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    intermediate_dir: Option<&Path>,
    // Phase 2: centroid map for directional penalty (empty = no penalty)
    centroids: &crate::geosection_orientation::CentroidMap,
    // Phase 2: directional penalty strength (0.0 = off, use GeoSection without penalty)
    lambda: f64,
    // AreaSection mode: ALAND in m² per vertex. None = standard GeoSection (ncon=1).
    vertex_areas_m2: Option<&[f64]>,
    // AreaSection area imbalance tolerance (ubvec[1]). Default 1.10 = ±10%.
    area_swing: f64,
    // VRASection (T.7): per-vertex minority VAP counts. None = standard GeoSection.
    // When Some, ratio selection score is: normalised - w_vra * alignment * normalised.max(1)
    // where alignment = 2 * |left share of total minority mass - 0.5|.
    minority_vap: Option<&[f64]>,
    // VRASection alignment weight (default 0.40). Only consulted when minority_vap is Some.
    w_vra: f64,
    // MKA warm-start (AreaSection only): when Some(theta), applies a directional edge-weight
    // bias at the top level using theta as the cut angle (radians) computed by
    // split_subgraph_mka_direction(). Ignored when centroids map is non-empty (GeoSection Phase 2).
    mka_theta_override: Option<f64>,
    base_seed: u64,
) -> Result<(HashMap<usize, usize>, usize, usize, f64), String> {
    run_geosection_seeded_tuned(adjacency,vertex_weights,edge_weights,num_districts,balance_tolerance,niter,seeds_per_ratio,intermediate_dir,centroids,lambda,vertex_areas_m2,area_swing,minority_vap,w_vra,mka_theta_override,base_seed,"cut",1)
}

/// Ratio search with explicit METIS refinement controls at each recursive node.
#[allow(clippy::too_many_arguments)]
pub fn run_geosection_seeded_tuned(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    intermediate_dir: Option<&Path>,
    // Phase 2: centroid map for directional penalty (empty = no penalty)
    centroids: &crate::geosection_orientation::CentroidMap,
    // Phase 2: directional penalty strength (0.0 = off, use GeoSection without penalty)
    lambda: f64,
    // AreaSection mode: ALAND in m² per vertex. None = standard GeoSection (ncon=1).
    vertex_areas_m2: Option<&[f64]>,
    // AreaSection area imbalance tolerance (ubvec[1]). Default 1.10 = ±10%.
    area_swing: f64,
    // VRASection (T.7): per-vertex minority VAP counts. None = standard GeoSection.
    // When Some, ratio selection score is: normalised - w_vra * alignment * normalised.max(1)
    // where alignment = 2 * |left share of total minority mass - 0.5|.
    minority_vap: Option<&[f64]>,
    // VRASection alignment weight (default 0.40). Only consulted when minority_vap is Some.
    w_vra: f64,
    // MKA warm-start (AreaSection only): when Some(theta), applies a directional edge-weight
    // bias at the top level using theta as the cut angle (radians) computed by
    // split_subgraph_mka_direction(). Ignored when centroids map is non-empty (GeoSection Phase 2).
    mka_theta_override: Option<f64>,
    base_seed: u64,
    objective: &str,
    trials: u32,
) -> Result<(HashMap<usize, usize>, usize, usize, f64), String> {
    #[cfg(feature="c-ffi-engine")]
    if objective!="cut" || trials!=1 {return Err("Advanced GeoSection METIS controls require the pure Rust engine.".into());}
    let n = adjacency.len();
    if !["cut","volume"].contains(&objective) || !(1..=100).contains(&trials)
        || vertex_weights.len() != n
        || seeds_per_ratio == 0
        || num_districts == 0
        || num_districts > n
        || minority_vap.is_some_and(|mass|mass.len()!=n||mass.iter().any(|v|!v.is_finite()||*v<0.0)||!mass.iter().sum::<f64>().is_finite())
        || (minority_vap.is_some()&&(!w_vra.is_finite()||!(0.0..=1.0).contains(&w_vra)))
        || vertex_areas_m2.is_some_and(|areas| {
            areas.len() != n
                || areas.iter().any(|v| !v.is_finite() || *v < 0.0)
                || areas.iter().sum::<f64>() <= 0.0
        })
    {
        return Err("Invalid GeoSection/AreaSection input dimensions or budget.".into());
    }

    if num_districts == 1 {
        let asgn = (0..n).map(|i| (i, 1)).collect();
        return Ok((asgn, 1, 0, 0.0));
    }
    if num_districts == 2 && vertex_areas_m2.is_none() && seeds_per_ratio == 1 {
        // Only one ratio possible: 1:1
        let asgn = if objective=="cut" && trials==1 { run_all_splits(
            adjacency,
            vertex_weights,
            edge_weights,
            2,
            balance_tolerance,
            niter,
            Some(base_seed),
            intermediate_dir,
        )? } else { run_all_splits_tuned(adjacency,vertex_weights,edge_weights,2,balance_tolerance,niter,Some(base_seed),None,objective,trials)? };
        if objective!="cut" || trials!=1 {
            if let Some(dir)=intermediate_dir {
                let round=dir.join("depth_01");
                let _=std::fs::create_dir_all(&round);
                let _=write_intermediate_round(&round,&asgn);
            }
        }
        let ec: f64 = edge_weights
            .iter()
            .map(
                |(&(u, v), &w)| {
                    if asgn.get(&u) != asgn.get(&v) {
                        w
                    } else {
                        0.0
                    }
                },
            )
            .sum();
        return Ok((asgn, 1, 1, ec));
    }

    // Try all split ratios at the root level
    let node_ufactor = 1.0 + balance_tolerance / num_districts as f64;
    let mut best_ec = f64::INFINITY;
    let mut best_normalised = f64::INFINITY; // isoperimetric-corrected selection criterion
    let mut best_left = 0usize;
    let mut best_right = 0usize;
    let mut best_left_set = HashSet::new();
    let mut best_right_set = HashSet::new();

    let all_tracts: HashSet<usize> = (0..n).collect();
    let max_left = num_districts / 2; // try ratios 1:k-1 through k/2:k/2

    // ── AreaSection mode: build interleaved vwgt and Lorenz feasibility mask ──
    let (ncon, vwgt_flat, lorenz_feasible) = if let Some(areas) = vertex_areas_m2 {
        // Scale areas to hectares (÷10,000 m²) to stay within METIS i32 range.
        // Large rural tracts can be billions of m²; hectares keep them within i32.
        let vertex_areas_ha: Vec<i64> = areas
            .iter()
            .map(|&a| ((a / 10_000.0) as i64).max(1))
            .collect();
        let interleaved: Vec<i64> = vertex_weights
            .iter()
            .zip(&vertex_areas_ha)
            .flat_map(|(&p, &a)| [p, a])
            .collect();

        // Lorenz pre-filtering: skip ratios where area balance is geometrically impossible
        let area_max = 0.5 * area_swing;
        let area_min = 1.0 - area_max;
        let (_, natural_pop, suggested_k) = population_lorenz(vertex_weights, areas, num_districts);
        eprintln!("[areasection] Lorenz: dense-half contains {:.1}% of population -> natural split ~{}:{}",
                  natural_pop * 100.0, num_districts - suggested_k, suggested_k);
        let feasible: Vec<bool> = (0..=max_left)
            .map(|left_k| {
                if left_k == 0 {
                    return false;
                }
                let p = left_k as f64 / num_districts as f64;
                let min_a = lorenz_min_area(vertex_weights, areas, p);
                let max_a = 1.0 - lorenz_min_area(vertex_weights, areas, 1.0 - p);
                min_a <= area_max && max_a >= area_min
            })
            .collect();
        for left_k in 1..=max_left {
            let p = left_k as f64 / num_districts as f64;
            let min_a = lorenz_min_area(vertex_weights, areas, p);
            let max_a = 1.0 - lorenz_min_area(vertex_weights, areas, 1.0 - p);
            eprintln!(
                "[areasection]   ratio {}:{} ({:.1}% pop): Lorenz area range [{:.1}%-{:.1}%] -> {}",
                left_k,
                num_districts - left_k,
                p * 100.0,
                min_a * 100.0,
                max_a * 100.0,
                if feasible[left_k] {
                    "feasible"
                } else {
                    "INFEASIBLE (Lorenz)"
                }
            );
        }
        eprintln!(
            "[areasection] {} ratios x {} seeds (pop+area dual constraint)",
            max_left, seeds_per_ratio
        );
        (2usize, interleaved, feasible)
    } else {
        // ncon=1: plain population weights, all ratios feasible
        let plain: Vec<i64> = vertex_weights.to_vec();
        let feasible = vec![true; max_left + 1];
        eprintln!(
            "[geosection] trying {} ratios x {} seeds for k={}",
            max_left, seeds_per_ratio, num_districts
        );
        (1usize, plain, feasible)
    };

    // ── MKA warm-start: apply directional edge-weight bias using theta* ──────
    // When mka_theta_override is Some(theta) and centroids are provided, bias
    // the edge weights toward cuts perpendicular to the MKA-optimal direction.
    // This pre-bias is applied ONLY at the top level; recursive calls receive
    // the original (unmodified) weights via recurse_geosection.
    //
    // The bias uses apply_directional_penalty with theta as the "minor axis" angle:
    //   - edges running parallel to the cut direction (sin(θ)=1) get penalised
    //   - edges running perpendicular (sin(θ)=0) are unchanged
    // Effect: METIS preferentially cuts across the MKA-optimal direction.
    let biased_edge_weights: Option<HashMap<(usize, usize), f64>> = if let Some(theta) =
        mka_theta_override
    {
        if !centroids.is_empty() {
            eprintln!("[areasection-mka] applying directional bias at theta*={:.4} rad ({:.1} deg) with lambda={:.2}",
                          theta, theta.to_degrees(), lambda);
            Some(crate::geosection_orientation::apply_directional_penalty(
                edge_weights,
                centroids,
                theta,
                lambda,
            ))
        } else {
            None // no centroids → no bias (fallback already warned by caller)
        }
    } else {
        None
    };
    let active_edge_weights: &HashMap<(usize, usize), f64> =
        biased_edge_weights.as_ref().unwrap_or(edge_weights);

    for left_k in 1..=max_left {
        if !lorenz_feasible[left_k] {
            eprintln!(
                "[areasection] skipping ratio {}:{} - Lorenz predicts infeasible area balance",
                left_k,
                num_districts - left_k
            );
            continue;
        }
        let right_k = num_districts - left_k;
        let pop_frac = left_k as f64 / num_districts as f64;

        // Build tpwgts and ubvec based on ncon.
        // Always compute right = 1.0 - left in f32 to guarantee exact sum-to-one.
        let left_w = pop_frac as f32;
        let right_w = 1.0_f32 - left_w;
        let tpwgts: Option<Vec<f32>> = if ncon == 2 {
            // ncon=2: [left_pop, left_area, right_pop, right_area]
            // Area target is always 50/50; sum per constraint = 1.0.
            Some(vec![left_w, 0.5f32, right_w, 0.5f32])
        } else if left_k != right_k {
            Some(vec![left_w, right_w])
        } else {
            None
        };
        let ubvec: Option<Vec<f32>> = if ncon == 2 {
            // Tight population balance (±0.1%), area swing from caller
            Some(vec![1.001f32, area_swing as f32])
        } else {
            None
        };

        let mut ratio_best = f64::INFINITY;
        let mut ratio_best_left = HashSet::new();
        let mut ratio_best_right = HashSet::new();

        for index in 0..seeds_per_ratio {
            let seed = base_seed.wrapping_add(index as u64);
            match split_subgraph_tuned(
                adjacency,
                &vwgt_flat,
                ncon,
                active_edge_weights,
                &all_tracts,
                node_ufactor,
                niter,
                Some(seed),
                tpwgts.clone(),
                ubvec.clone(),
                objective, trials,
            ) {
                Ok((l, r)) => {
                    if ncon == 2 {
                        let pop_total = vertex_weights.iter().sum::<i64>() as f64;
                        let left_pop = l.iter().map(|&v| vertex_weights[v]).sum::<i64>() as f64;
                        let areas = vertex_areas_m2.unwrap();
                        let total_area = areas.iter().sum::<f64>();
                        let left_area = l.iter().map(|&v| areas[v]).sum::<f64>();
                        if l.is_empty()
                            || r.is_empty()
                            || !is_connected_subset(adjacency, &l)
                            || !is_connected_subset(adjacency, &r)
                            || left_pop > pop_total * pop_frac * 1.001 + 1e-6
                            || pop_total - left_pop > pop_total * (1.0 - pop_frac) * 1.001 + 1e-6
                            || left_area > total_area * 0.5 * area_swing + 1e-6
                            || total_area - left_area > total_area * 0.5 * area_swing + 1e-6
                        {
                            continue;
                        }
                    }
                    // EC measured on original (unbiased) edge weights for fair comparison.
                    let ec = weighted_edge_cut(edge_weights, &l);
                    if ec < ratio_best {
                        ratio_best = ec;
                        ratio_best_left = l;
                        ratio_best_right = r;
                    }
                }
                Err(e) => {
                    if seed == 1 && ncon == 2 {
                        eprintln!(
                            "[areasection] seed 1 error (ratio {}:{}): {}",
                            left_k,
                            right_k,
                            &e.chars().take(300).collect::<String>()
                        );
                    }
                    continue;
                }
            }
        }

        // Normalise by sqrt(min(i,k-i)): isoperimetric correction.
        // Raw EC always favours 1:k-1 (tiny boundary) over k/2:k/2 (full bisection).
        // Dividing by sqrt(smaller_half_districts) makes the comparison apples-to-apples.
        let smaller = left_k.min(right_k) as f64;
        let normalised = ratio_best / smaller.sqrt();

        // VRASection (T.7): subtract alignment bonus from the normalised score.
        // A(split) = 2 * |left share of total minority mass - 0.5|.
        // score(ratio) = normalised - w_vra * alignment * normalised.max(1.0)
        // Lower score = preferred. Subtracting means concentrated splits win over dispersed.
        let selection_score = if let Some(mvap) = minority_vap {
            let mvap_total: f64 = mvap.iter().sum();
            let score = if mvap_total > 0.0 {
                // Fixed graph-index order is required for portable scores.
                let mvap_left: f64 = (0..n).filter(|v|ratio_best_left.contains(v)).map(|v| mvap[v]).sum();
                let alignment = (mvap_left / mvap_total - 0.5).abs() * 2.0;
                normalised - w_vra * alignment * normalised.max(1.0)
            } else {
                normalised
            };
            if ncon == 2 {
                // AreaSection doesn't use VRA alignment; just use normalised
                normalised
            } else {
                eprintln!(
                    "[vra-section]   ratio {}:{} normalised={:.1}  score={:.1}",
                    left_k,
                    right_k,
                    normalised / 1000.0,
                    score / 1000.0
                );
                score
            }
        } else {
            normalised
        };

        if ncon == 2 {
            if let Some(areas) = vertex_areas_m2 {
                let area_left: f64 = ratio_best_left.iter().map(|&v| areas[v]).sum();
                let total_area: f64 = areas.iter().sum();
                let area_frac = area_left / total_area;
                eprintln!(
                    "[areasection]   ratio {}:{} best={:.0}km  normalised={:.1}  area_left={:.1}%",
                    left_k,
                    right_k,
                    ratio_best / 1000.0,
                    normalised / 1000.0,
                    area_frac * 100.0
                );
            }
        } else if minority_vap.is_none() {
            eprintln!(
                "[geosection]   ratio {}:{} best={:.0}km  normalised={:.1}",
                left_k,
                right_k,
                ratio_best / 1000.0,
                normalised / 1000.0
            );
        }

        if selection_score < best_normalised {
            best_normalised = selection_score;
            best_ec = ratio_best;
            best_left = left_k;
            best_right = right_k;
            best_left_set = ratio_best_left;
            best_right_set = ratio_best_right;
        }
    }

    let mode_tag = if ncon == 2 {
        "areasection"
    } else if minority_vap.is_some() {
        "vra-section"
    } else {
        "geosection"
    };
    if best_left == 0 || !best_ec.is_finite() {
        return Err(format!(
            "{mode_tag}: no sampled feasible root split ({} seeds per ratio)",
            seeds_per_ratio
        ));
    }
    eprintln!(
        "[{mode_tag}] natural ratio {}:{} at {:.0}km (normalised={:.1})",
        best_left,
        best_right,
        best_ec / 1000.0,
        best_normalised / 1000.0
    );

    // For AreaSection: log the winning split's area fraction and whether it's within ubvec
    if ncon == 2 {
        if let Some(areas) = vertex_areas_m2 {
            let area_left: f64 = best_left_set.iter().map(|&v| areas[v]).sum();
            let total_area: f64 = areas.iter().sum();
            let area_frac = area_left / total_area;
            let area_min = 1.0 - 0.5 * area_swing;
            let area_max = 0.5 * area_swing;
            let in_bounds = area_frac >= area_min && area_frac <= area_max;
            let pop_left: i64 = best_left_set.iter().map(|&v| vertex_weights[v]).sum();
            let total_pop: i64 = vertex_weights.iter().sum();
            let pop_frac = pop_left as f64 / total_pop as f64;
            let pop_target = best_left as f64 / num_districts as f64;
            eprintln!("[areasection] winner: area={:.1}% (target 50% ±{:.0}%, {}) pop={:.1}% (target {:.1}%)",
                      area_frac*100.0, (area_swing-1.0)*100.0,
                      if in_bounds { "OK" } else { "VIOLATED" },
                      pop_frac*100.0, pop_target*100.0);
        }
    }

    // Write depth_01 intermediate
    if let Some(dir) = intermediate_dir {
        let round_dir = dir.join("depth_01");
        let _ = std::fs::create_dir_all(&round_dir);
        let mut d1: HashMap<usize, usize> = HashMap::new();
        for &v in &best_left_set {
            d1.insert(v, 1);
        }
        for &v in &best_right_set {
            d1.insert(v, 2);
        }
        let _ = write_intermediate_round(&round_dir, &d1);
    }

    // Recurse: each half finds its own natural ratio with its own orientation.
    // Recursive calls always use ncon=1 (area constraint only at the first level).
    let left_asgn = recurse_geosection_tuned(
        &best_left_set,
        adjacency,
        vertex_weights,
        edge_weights,
        best_left,
        balance_tolerance,
        niter,
        seeds_per_ratio,
        1,
        centroids,
        lambda,
        base_seed,
        objective, trials,
    )?;
    let right_asgn = recurse_geosection_tuned(
        &best_right_set,
        adjacency,
        vertex_weights,
        edge_weights,
        best_right,
        balance_tolerance,
        niter,
        seeds_per_ratio,
        best_left + 1,
        centroids,
        lambda,
        base_seed,
        objective, trials,
    )?;

    let mut assignments = left_asgn;
    assignments.extend(right_asgn);

    if assignments.len() != n {
        return Err(format!("{mode_tag} incomplete: {}/{n}", assignments.len()));
    }
    Ok((assignments, best_left, best_right, best_ec))
}

/// Fully recursive GeoSection on a geographic subregion.
///
/// At each level:
///   1. Extract local subgraph (local indices)
///   2. Compute local minor axis via PCA of subregion centroids (if available)
///   3. Apply directional penalty λ to edge weights (makes cuts straighter)
///   4. Run isoperimetrically-normalised ratio search on local graph
///   5. Map results back to global indices, offset by district_base
///
/// Each half re-rotates independently — a horizontal first cut produces
/// two halves that each find their OWN narrowest geographic axis.
pub(crate) fn recurse_geosection(
    verts: &HashSet<usize>,
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    district_base: usize,
    centroids: &crate::geosection_orientation::CentroidMap,
    lambda: f64,
    base_seed: u64,
) -> Result<HashMap<usize, usize>, String> {
    recurse_geosection_tuned(verts,adjacency,vertex_weights,edge_weights,k,balance_tolerance,niter,seeds_per_ratio,district_base,centroids,lambda,base_seed,"cut",1)
}

fn recurse_geosection_tuned(
    verts: &HashSet<usize>,
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    district_base: usize,
    centroids: &crate::geosection_orientation::CentroidMap,
    lambda: f64,
    base_seed: u64,
    objective: &str,
    trials: u32,
) -> Result<HashMap<usize, usize>, String> {
    if k == 0 {
        return Ok(HashMap::new());
    }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    // Extract sorted vertex list for deterministic local indexing
    let sorted: Vec<usize> = {
        let mut v: Vec<usize> = verts.iter().copied().collect();
        v.sort_unstable();
        v
    };
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();

    // Build local subgraph components
    let local_adj: Vec<Vec<usize>> = build_subgraph_adjacency(verts, adjacency);
    let local_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let mut local_ew: HashMap<(usize, usize), f64> = edge_weights
        .iter()
        .filter_map(|(&(u, v), &w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m, (k, v)| {
            m.insert(k, v);
            m
        });

    // Phase 2: PCA of THIS subregion's centroids → local minor axis → directional penalty
    if lambda > 1e-10 && !centroids.is_empty() {
        if let Some(angle) = crate::geosection_orientation::compute_minor_axis(verts, centroids) {
            local_ew = crate::geosection_orientation::apply_directional_penalty(
                &local_ew, centroids, angle, lambda,
            );
        }
    }

    // Recursively find natural ratio for THIS subregion
    // Pass empty centroids/lambda=0 here — directional penalty was already
    // applied to local_ew above; run_geosection sees the modified weights.
    // Always ncon=1 for recursive levels (area constraint only at first level).
    let empty_centroids = crate::geosection_orientation::CentroidMap::new();
    let (local_asgn, _, _, _) = run_geosection_seeded_tuned(
        &local_adj,
        &local_vw,
        &local_ew,
        k,
        balance_tolerance,
        niter,
        seeds_per_ratio,
        None,
        &empty_centroids,
        0.0,
        None,
        1.10, // recursive: ncon=1, area_swing unused
        None,
        0.0,  // recursive: no VRA alignment at sub-levels
        None, // recursive: no MKA override at sub-levels
        base_seed,
        objective, trials,
    )?;

    if local_asgn.len() < sorted.len().saturating_sub(1) {
        // Partial assignment — fall back to standard for this subregion
        return recurse_standard(
            verts,
            &local_adj,
            adjacency,
            vertex_weights,
            edge_weights,
            k,
            balance_tolerance,
            niter,
            district_base,
        );
    }

    // Map local indices back to global with district offset
    let result: HashMap<usize, usize> = local_asgn
        .iter()
        .filter_map(|(&local, &dist)| {
            sorted
                .get(local)
                .map(|&global| (global, dist + district_base - 1))
        })
        .collect();
    Ok(result)
}

/// Build adjacency restricted to a subset of vertices (for recursion).
pub(crate) fn build_subgraph_adjacency(
    verts: &HashSet<usize>,
    adj: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    let sorted: Vec<usize> = {
        let mut v: Vec<usize> = verts.iter().copied().collect();
        v.sort_unstable();
        v
    };
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    sorted
        .iter()
        .map(|&g| {
            adj[g]
                .iter()
                .filter_map(|&nb| global_to_local.get(&nb).copied())
                .collect()
        })
        .collect()
}

/// Recurse using standard bisection (UpfloorD k/2 : ceil k/2) within a subgraph.
/// Returns global-index assignments offset by district_base.
pub(crate) fn recurse_standard(
    verts: &HashSet<usize>,
    sub_adj: &[Vec<usize>],
    _global_adj: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    district_base: usize,
) -> Result<HashMap<usize, usize>, String> {
    if k == 0 {
        return Ok(HashMap::new());
    }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    let sorted: Vec<usize> = {
        let mut v: Vec<usize> = verts.iter().copied().collect();
        v.sort_unstable();
        v
    };
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();

    // Extract sub-vertex-weights and sub-edge-weights
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let sub_ew: HashMap<(usize, usize), f64> = edge_weights
        .iter()
        .filter_map(|(&(u, v), &w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m, (k, v)| {
            m.insert(k, v);
            m
        });

    let sub_asgn = run_all_splits(
        sub_adj,
        &sub_vw,
        &sub_ew,
        k,
        balance_tolerance,
        niter,
        Some(42),
        None,
    )?;

    // Map back to global indices with offset
    let result: HashMap<usize, usize> = sub_asgn
        .iter()
        .map(|(&local, &dist)| (sorted[local], dist + district_base - 1))
        .collect();
    Ok(result)
}
