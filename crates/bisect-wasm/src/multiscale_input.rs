//! Explicit in-memory multiscale graphs; no disk loading or implicit BG defaults.
use crate::engine::{PreparedGraph, Request};
use bisect_runner::bisection_runner::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockGroupGraph {
    pub schema_version: String,
    pub state: String,
    pub year: String,
    pub geoids: Vec<String>,
    pub adjacency: Vec<Vec<usize>>,
    pub population: Vec<i64>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdaptiveOptions {
    pub target_accept: f64,
    pub adapt_interval: usize,
    pub gamma_0: f64,
    pub coarse_tol_factor: f64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiscaleRequest {
    pub request: Request,
    pub fine_level: String,
    pub coarse_level: String,
    pub total_steps: usize,
    pub alpha: f64,
    pub percentile: f64,
    pub block_groups: Option<BlockGroupGraph>,
    pub adaptive: Option<AdaptiveOptions>,
}

fn fraction(v: f64) -> bool { v.is_finite() && !v.is_sign_negative() && v <= 1.0 }

fn hash_units(hash: &mut Sha256, ids: &[String], pop: &[i64], adjacency: &[Vec<usize>]) {
    hash.update((ids.len() as u64).to_le_bytes());
    for (i, id) in ids.iter().enumerate() {
        hash.update((id.len() as u64).to_le_bytes()); hash.update(id.as_bytes());
        hash.update(pop[i].to_le_bytes()); hash.update((adjacency[i].len() as u64).to_le_bytes());
        for &j in &adjacency[i] { hash.update((j as u64).to_le_bytes()); }
    }
}

fn validate_bg(g: &PreparedGraph, b: &BlockGroupGraph) -> Result<(), String> {
    let n = b.geoids.len();
    if b.schema_version != "bisect-block-group-graph-v1" || b.state != g.state || b.year != g.year
        || n == 0 || n > 100_000 || b.adjacency.len() != n || b.population.len() != n {
        return Err("Invalid block-group schema, scope or graph dimensions.".into());
    }
    let parents: HashMap<_, _> = g.geoids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
    let mut ids = HashSet::new();
    let mut parent_population = vec![0i64; g.geoids.len()];
    let mut parent_count = vec![0usize; g.geoids.len()];
    let mut parent_index = vec![0usize; n];
    let mut edge_count = 0usize;
    for (i, id) in b.geoids.iter().enumerate() {
        if id.len() != 12 || !id.bytes().all(|v| v.is_ascii_digit()) || !ids.insert(id)
            || b.population[i] < 0 || b.population[i] > 9_007_199_254_740_991 {
            return Err("Invalid or duplicate block-group GEOID/population.".into());
        }
        let parent = *parents.get(&id[..11]).ok_or("Block group has no selected tract parent.")?;
        parent_index[i] = parent;
        parent_population[parent] = parent_population[parent].checked_add(b.population[i]).ok_or("Block-group population overflow.")?;
        parent_count[parent] += 1;
        let mut neighbors = HashSet::new();
        for &j in &b.adjacency[i] {
            if j >= n || j == i || !neighbors.insert(j) || !b.adjacency[j].contains(&i) {
                return Err("Invalid, duplicate or asymmetric block-group adjacency.".into());
            }
            edge_count += 1;
            if edge_count > 2_000_000 { return Err("Block-group edge limit exceeded.".into()); }
        }
    }
    if parent_count.contains(&0) || parent_population != g.population {
        return Err("Block groups must completely cover tracts and sum to each tract population.".into());
    }
    let mut quotient = HashSet::new();
    for (i, neighbors) in b.adjacency.iter().enumerate() {
        for &j in neighbors { if parent_index[i] != parent_index[j] { quotient.insert((parent_index[i], parent_index[j])); } }
    }
    let tract_arcs: HashSet<_> = g.adjacency.iter().enumerate().flat_map(|(i, neighbors)| neighbors.iter().map(move |&j| (i, j))).collect();
    if quotient != tract_arcs { return Err("Block-group adjacency must reproduce the tract graph when coarsened.".into()); }
    // Every parent must induce a connected fine region for tract seed projection.
    let mut visited = HashSet::new();
    for start in 0..n {
        if visited.contains(&start) { continue; }
        let parent = parent_index[start]; let mut todo = vec![start]; visited.insert(start); let mut count = 0;
        while let Some(i) = todo.pop() { count += 1; for &j in &b.adjacency[i] { if parent_index[j] == parent && visited.insert(j) { todo.push(j); } } }
        if count != parent_count[parent] { return Err("Block groups within a tract must be connected.".into()); }
    }
    let mut seen = vec![false; n];
    let mut todo = vec![0]; seen[0] = true;
    while let Some(i) = todo.pop() { for &j in &b.adjacency[i] { if !seen[j] { seen[j] = true; todo.push(j); } } }
    if seen.contains(&false) { return Err("Block-group graph is disconnected.".into()); }
    Ok(())
}

pub(crate) fn validate(input: &MultiscaleRequest) -> Result<(), String> {
    let r = &input.request; let g = &r.graph; let o = &r.options;
    crate::engine::validate(g, o)?;
    let fips = crate::tract_input::import_scope(&g.state, &g.year, "multiscale graph")?;
    if g.geoids.iter().any(|id| !id.starts_with(&fips) || !id.bytes().all(|b| b.is_ascii_digit())) {
        return Err("Tract graph contains foreign-state or nonnumeric GEOIDs.".into());
    }
    if o.structure != "standard-bisect" || o.search != "single" || o.weights != "geographic"
        || o.metis_objective.is_some() || o.metis_trials.is_some() || r.character.is_some()
        || r.elections.is_some() || r.demographics.is_some() || r.partisan.is_some() {
        return Err("Multiscale initialization requires explicit standard/single geographic inputs with native default refinement.".into());
    }
    if input.total_steps > 100_000 || !fraction(input.alpha) || !fraction(input.percentile)
        || !["tract", "bg"].contains(&input.fine_level.as_str())
        || !["tract", "county"].contains(&input.coarse_level.as_str())
        || (input.fine_level == "tract" && (input.coarse_level != "county" || input.block_groups.is_some())) {
        return Err("Invalid multiscale steps, probability, percentile or resolution pairing.".into());
    }
    if let Some(a) = &input.adaptive {
        if !fraction(a.target_accept) || a.adapt_interval == 0 || a.adapt_interval > 100_000
            || !a.gamma_0.is_finite() || a.gamma_0.is_sign_negative() || a.gamma_0 > 1.
            || !a.coarse_tol_factor.is_finite() || !(1.0..=10.0).contains(&a.coarse_tol_factor)
            || a.coarse_tol_factor * o.balance_tolerance > 100. {
            return Err("Invalid multiscale adaptation parameters.".into());
        }
    }
    if input.fine_level == "bg" {
        validate_bg(g, input.block_groups.as_ref().ok_or("Block-group resolution requires an explicit graph.")?)?;
    }
    let bg = input.block_groups.as_ref();
    let fine_ids = bg.map_or(&g.geoids, |b| &b.geoids);
    // The native algorithm retains every candidate assignment for percentile ranking.
    if (input.total_steps + 1).checked_mul(fine_ids.len()).is_none_or(|n| n > 10_000_000) {
        return Err("Multiscale retained-plan work limit exceeded.".into());
    }
    Ok(())
}

pub fn execute(input: MultiscaleRequest) -> Result<Value, String> {
    validate(&input)?;
    let g = &input.request.graph; let o = &input.request.options;
    let bg = input.block_groups.as_ref();
    let fine_ids = bg.map_or(&g.geoids, |b| &b.geoids);
    let fine_pop = bg.map_or(&g.population, |b| &b.population);
    let fine_adj = bg.map_or(&g.adjacency, |b| &b.adjacency);
    let geoids: HashMap<_, _> = g.geoids.iter().cloned().enumerate().collect();
    let bg_geoids: HashMap<_, _> = bg.map(|b| b.geoids.iter().cloned().enumerate().collect()).unwrap_or_default();
    let bg_tuple = bg.map(|b| (b.adjacency.as_slice(), b.population.as_slice(), &bg_geoids));
    let edges = g.edges.iter().map(|&(i, j, length)| ((i, j), length)).collect();
    let fine_level = if bg.is_some() { MultiscaleFineLevel::BlockGroup } else { MultiscaleFineLevel::Tract };
    let tolerance = o.balance_tolerance / 100.;
    let (plan, diagnostics) = if let Some(a) = &input.adaptive {
        let (plan, result) = run_multiscale_adaptive_portable(&g.adjacency, &g.population, &edges, o.districts, o.iterations, o.seed,
            AdaptiveConfig { total_steps: input.total_steps, target_accept: a.target_accept, initial_alpha: input.alpha,
                adapt_interval: a.adapt_interval, gamma_0: a.gamma_0, pop_tolerance: tolerance, coarse_tol_factor: a.coarse_tol_factor, p: input.percentile },
            Some(&geoids), fine_level, &input.coarse_level, bg_tuple)?;
        (plan, json!({"final_alpha":result.final_alpha,"alpha_trace":result.alpha_trace,
            "fine_acceptance_rate":result.fine_acceptance_rate,"coarse_acceptance_rate":result.coarse_acceptance_rate}))
    } else {
        (run_multiscale_portable(&g.adjacency, &g.population, &edges, o.districts, tolerance, o.iterations,
            o.seed, input.total_steps, input.alpha, input.percentile, Some(&geoids), fine_level, &input.coarse_level, bg_tuple)?, Value::Null)
    };
    let mut assignment = BTreeMap::new(); let mut populations = vec![0i64; o.districts];
    for (i, id) in fine_ids.iter().enumerate() {
        let district = *plan.get(&i).ok_or("Multiscale omitted a fine unit.")?;
        if district == 0 || district > o.districts { return Err("Multiscale produced an invalid district.".into()); }
        assignment.insert(id, district); populations[district - 1] += fine_pop[i];
    }
    let connected: Vec<bool> = (1..=o.districts).map(|district| {
        let units: Vec<_> = (0..fine_ids.len()).filter(|i| plan.get(i) == Some(&district)).collect();
        if units.is_empty() { return false; }
        let mut seen = HashSet::new(); let mut todo = vec![units[0]]; seen.insert(units[0]);
        while let Some(i) = todo.pop() { for &j in &fine_adj[i] { if plan.get(&j) == Some(&district) && seen.insert(j) { todo.push(j); } } }
        seen.len() == units.len()
    }).collect();
    let ideal = fine_pop.iter().sum::<i64>() as f64 / o.districts as f64;
    let max_deviation = populations.iter().map(|p| ((*p as f64 - ideal) / ideal).abs() * 100.).fold(0f64, f64::max);
    let mut hash = Sha256::new(); hash.update(b"BISECT_MULTISCALE_GRAPHS_V1\0");
    hash.update(g.state.as_bytes()); hash.update([0]); hash.update(g.year.as_bytes()); hash.update([0]);
    hash_units(&mut hash, &g.geoids, &g.population, &g.adjacency);
    hash.update((g.edges.len() as u64).to_le_bytes());
    for &(i,j,length) in &g.edges { hash.update((i as u64).to_le_bytes()); hash.update((j as u64).to_le_bytes()); hash.update(length.to_le_bytes()); }
    hash.update([u8::from(bg.is_some())]);
    if let Some(b) = bg { hash_units(&mut hash, &b.geoids, &b.population, &b.adjacency); }
    Ok(json!({"schema_version":"bisect-multiscale-result-v1","state":g.state,"year":g.year,
        "resolution":input.fine_level,"assignments":assignment,"populations":populations,"connected":connected,
        "max_deviation_percent":max_deviation,"within_requested_tolerance":max_deviation <= o.balance_tolerance,
        "evidence":{"method":if input.adaptive.is_some(){"multiscale-adaptive"}else{"multiscale"},
            "rng":"chacha12-u64-portable-wilson-v1","graph_sha256":format!("{:x}",hash.finalize()),
            "coarse_level":input.coarse_level,"total_steps":input.total_steps,"alpha":input.alpha,"percentile":input.percentile,
            "initialization":"weighted-geographic-metis","sampling":"unweighted-wilson-recom",
            "ranking":"fine-unweighted-edge-cut-then-step","coarse_assignment":"last-fine-index-wins-native-policy",
            "diagnostic_policy":"native-counters: fine-attempts-counted-as-accepted; coarse-rebalance-success",
            "seed":serde_json::to_value(o).map_err(|e|e.to_string())?["seed"],"diagnostics":diagnostics,
            "execution":"heuristic","optimality":"unproved"}}))
}
