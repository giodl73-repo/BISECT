//! Explicit in-memory multiscale graphs; no disk loading or implicit BG defaults.
use crate::engine::{PreparedGraph, Request};
use bisect_runner::bisection_runner::*;
use bisect_runner::native_replay_recipe::NativeMultiscaleSampler;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockGroupGraph {
    pub schema_version: String,
    pub state: String,
    pub year: String,
    pub geoids: Vec<String>,
    pub adjacency: Vec<Vec<usize>>,
    pub population: Vec<i64>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdaptiveOptions {
    pub target_accept: f64,
    pub adapt_interval: usize,
    pub gamma_0: f64,
    pub coarse_tol_factor: f64,
}

#[derive(Clone, Deserialize, Serialize)]
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
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_sampler:Option<NativeMultiscaleSampler>,
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
    crate::native_composed::validate_request(r)?;
    crate::native_auxiliary::validate_selection(r.native_auxiliary.as_ref(),r.partisan.is_some(),r.coi.is_some())?;
    if (r.native_auxiliary.is_some()||r.native_weighting.is_some())&&input.native_sampler.is_none(){return Err("Native auxiliary multiscale initialization requires an explicit native sampler profile; selected sources were not ignored.".into());}
    crate::engine::validate_profile(o,r.partisan.is_some(),Some("multiscale"))?;
    crate::engine::validate(g, o)?;
    crate::zone_input::for_engine(g,o,r.zone_membership.as_ref())?;
    if g.effective_unit_kind()!=rplan_core::UnitKind::Tract {return Err("Multiscale coarse/fine adapter requires a tract parent graph; no resolution substitution occurred.".into());}
    let fips = crate::tract_input::import_scope(&g.state, &g.year, "multiscale graph")?;
    if g.geoids.iter().any(|id| !id.starts_with(&fips) || !id.bytes().all(|b| b.is_ascii_digit())) {
        return Err("Tract graph contains foreign-state or nonnumeric GEOIDs.".into());
    }
    if o.structure != "standard-bisect" || o.search != "single" || !["geographic","partisan","unweighted","county","vra-aligned","economic-character","housing-character","zone-membership","native-composed"].contains(&o.weights.as_str())
        || o.metis_objective.is_some() || o.metis_trials.is_some()
        || r.elections.is_some() {
        return Err("Multiscale initialization requires explicit standard/single geographic, partisan, unweighted or county tract inputs with native default refinement.".into());
    }
    match (o.weights.as_str(),r.partisan.as_ref()) {
        ("partisan",Some(input))=>{crate::engine::partisan_shares(g,input)?;},
        ("partisan",None)=>return Err("Multiscale partisan initialization requires complete parent-tract shares.".into()),
        (_,Some(_))=>return Err("Partisan shares have no effect on geographic multiscale initialization.".into()),
        _=>{}
    }
    match (o.weights.as_str(),r.demographics.as_ref()) {
        ("vra-aligned",Some(input))=>{if input.basis!="total-population"{return Err("Multiscale minority initialization requires total-population fractions.".into());}crate::engine::demographic_fractions(g,input)?;},
        ("vra-aligned",None)=>return Err("Multiscale minority initialization requires complete parent-tract fractions.".into()),
        (_,Some(_))=>return Err("Demographics have no effect on this multiscale initialization.".into()),
        _=>{}
    }
    let county_pre=crate::county_augmentation::prepare(g,o,"parent-tract-initialization-only")?;
    let weight_graph=county_pre.as_ref().map(|p|&p.graph).unwrap_or(g);
    let native_character=crate::native_character_weights::for_engine(weight_graph,o,r.native_character_source.as_ref(),r.character.is_some())?;
    if native_character.is_none(){
    match (["economic-character","housing-character"].contains(&o.weights.as_str()),r.character.as_ref()) {
        (true,Some(input))=>{if input.state!=g.state||input.year!=g.year{return Err("Multiscale character scope must match parent tract graph.".into());}let weights=crate::character_input::build_weights(input,&g.geoids,&weight_graph.edges,o.character_alpha.unwrap())?;if format!("{}-character",weights.kind)!=o.weights{return Err("Multiscale character kind disagrees with weights.".into());}},
        (true,None) if o.character_absent_file_fallback==Some(true)=>{},
        (true,None)=>return Err("Multiscale character initialization requires complete parent observations.".into()),
        (false,Some(_))=>return Err("Character input has no effect on this multiscale initialization.".into()),
        _=>{}
    }
    }
    if let Some(settings)=&r.electoral {settings.resolve(o.districts)?;}
    if let Some(coi) = &r.coi { crate::coi_input::validate(coi)?; }
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

pub fn execute(input: MultiscaleRequest) -> Result<Value, String> {execute_dispatch(input,true)}
pub(crate) fn execute_for_build(input:MultiscaleRequest)->Result<Value,String>{execute_dispatch(input,false)}
fn execute_dispatch(input: MultiscaleRequest,require_valid:bool) -> Result<Value, String> {
    let _backend=crate::metis_backend::enter(&input.request.options)?;
    let enabled=input.request.options.capture_rounds==Some(true);
    let capture=if enabled{Some(RoundCapture::start_observed(input.request.graph.geoids.len(),Some(crate::engine::emit_round_progress))?)}else{None};
    let monitor=if !enabled{Some(bisect_runner::bisection_runner::RoundMonitor::start(input.request.graph.geoids.len(),crate::engine::emit_round_monitor)?)}else{None};
    let tract_ids=if enabled{input.request.graph.geoids.clone()}else{vec![]};
    let projected_units=if enabled{input.block_groups.as_ref().map(|b|b.geoids.clone())}else{None};
    let mut output=execute_inner(input,require_valid)?;
    if let Some(monitor)=monitor{monitor.finish()?;}
    if let Some(capture)=capture {
        crate::engine::finish_round_capture(capture,&tract_ids,&mut output,projected_units.as_deref())?;
        output["round_capture"]["unit_universe"]=json!({"unit_kind":"tract","stage":"multiscale-initialization","returned_unit_kind":if projected_units.is_some(){"bg"}else{"tract"},"final_comparison":if projected_units.is_some(){"parent-tract-projection-to-every-returned-block-group"}else{"exact-tract-labels"}});
        output["round_capture"]["scope"]=json!("Native intermediate snapshots from tract-level multiscale initialization. These are not a trace of later fine/coarse proposals or a certified lineage. Complete coverage refers to the tract initializer. For block-group output, final matching requires each parent tract label to match every returned block group; split tracts cannot be collapsed to imply equality. Existing capture bounds and source-hash checks apply.");
    }
    Ok(output)
}
fn execute_inner(input: MultiscaleRequest,require_valid:bool) -> Result<Value, String> {
    validate(&input)?;
    let g = &input.request.graph; let o = &input.request.options;
    let bg = input.block_groups.as_ref();
    let fine_ids = bg.map_or(&g.geoids, |b| &b.geoids);
    let fine_pop = bg.map_or(&g.population, |b| &b.population);
    let fine_adj = bg.map_or(&g.adjacency, |b| &b.adjacency);
    let geoids: HashMap<_, _> = g.geoids.iter().cloned().enumerate().collect();
    let bg_geoids: HashMap<_, _> = bg.map(|b| b.geoids.iter().cloned().enumerate().collect()).unwrap_or_default();
    let bg_tuple = bg.map(|b| (b.adjacency.as_slice(), b.population.as_slice(), &bg_geoids));
    let composed=input.request.native_weighting.as_ref().map(|spec|crate::native_composed::build(spec,input.request.native_auxiliary.as_ref(),g,o)).transpose()?;
    let minority=if o.weights=="vra-aligned"{Some(crate::engine::demographic_fractions(g,input.request.demographics.as_ref().unwrap())?)}else{None};
    let county_pre=crate::county_augmentation::prepare(g,o,"parent-tract-initialization-only")?;
    let weight_graph=county_pre.as_ref().map(|p|&p.graph).unwrap_or(g);
    let native_character=crate::native_character_weights::for_engine(weight_graph,o,input.request.native_character_source.as_ref(),input.request.character.is_some())?;
    let character_absent=native_character.is_none()&&["economic-character","housing-character"].contains(&o.weights.as_str())&&input.request.character.is_none();
    let character=if native_character.is_none()&&!character_absent&&["economic-character","housing-character"].contains(&o.weights.as_str()){Some(crate::character_input::build_weights(input.request.character.as_ref().unwrap(),&g.geoids,&weight_graph.edges,o.character_alpha.unwrap())?)}else{None};
    let zone=crate::zone_input::for_engine(weight_graph,o,input.request.zone_membership.as_ref())?;
    let (edges,partisan_evidence)=if let Some(computed)=&composed{(computed.weights.clone(),None)}else if o.weights=="partisan" {
        let (shares,digest)=crate::engine::partisan_shares(g,input.request.partisan.as_ref().unwrap())?;
        let dem=o.dem_threshold.unwrap();let rep=o.rep_threshold.unwrap();
        let strong=shares.iter().filter(|&&v|v>=dem||v<=rep).count();
        (bisect_core::build_partisan_weights(&g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),&shares,dem,rep),
         Some(json!({"method":"partisan-adaptive-boost","shares_sha256":digest,"dem_threshold":dem,"rep_threshold":rep,"strong_tracts":strong,"tracts":shares.len(),
          "alpha":3.0_f64.max(10.0*(1.0-0.7*strong as f64/shares.len() as f64)),"dispatch_representation":"native-sparse-boost-map",
          "scope":"parent-tract-initialization-only; fine/coarse sampling and ranking remain unweighted; no block-group share observations inferred"})))
    }else if let Some((fractions,_))=&minority{(bisect_core::build_vra_edge_weights(&g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),fractions,0.40),None)}else if let Some(computed)=&native_character{(computed.weights.iter().map(|&(u,v,w)|((u,v),w)).collect(),None)}else if let Some(weights)=&character{(weights.edges.iter().map(|&(u,v,w)|((u,v),w)).collect(),None)}else if let Some(computed)=&zone{(computed.weights.clone(),None)}else if o.weights=="unweighted"{(HashMap::new(),None)}else{(weight_graph.edges.iter().map(|&(i,j,length)|((i,j),if o.weights=="county"&&g.geoids[i][..5]==g.geoids[j][..5]{crate::engine::county_weight(length,o.alpha_county)}else{length})).collect(),None)};
    let (edges,county_evidence)=if let Some(pre)=&county_pre{(edges,Some(pre.evidence.clone()))}else{crate::county_augmentation::apply(g,o,edges,"parent-tract-initialization-only")?};
    // Native State post-weights the tract seed graph, before either sampler.
    // BG IDs in the source are not aggregated into tract weights.
    let (edges, coi_evidence) = if let Some(coi) = &input.request.coi {
        let (adjusted, mut evidence) = crate::coi_input::apply(coi, &g.geoids, edges, o.weights!="unweighted")?;
        evidence["algorithm_scope"] = json!("tract-seed-only; fine/coarse sampling and percentile ranking remain unweighted; block-group IDs are not mapped to parent tracts");
        (adjusted, Some(evidence))
    } else { (edges, None) };
    let (edges,native_coi_evidence)=crate::native_auxiliary::coi(input.request.native_auxiliary.as_ref(),g,edges)?;
    let fine_level = if bg.is_some() { MultiscaleFineLevel::BlockGroup } else { MultiscaleFineLevel::Tract };
    let tolerance = o.effective_balance_tolerance();
    let partition_timer=crate::partition_timing::Attempt::start(&g.state,&g.year,g.geoids.len(),o.districts,o.seed);
    let (plan, diagnostics) = if let Some(a) = &input.adaptive {
        let sampler=match input.native_sampler {
            Some(NativeMultiscaleSampler::Native32)=>run_multiscale_adaptive_native32,
            Some(NativeMultiscaleSampler::Native64)=>run_multiscale_adaptive_native64,
            None=>run_multiscale_adaptive_portable,
        };
        let (plan, result) = sampler(&g.adjacency, &g.population, &edges, o.districts, o.iterations, o.seed.unwrap_or(0),
            AdaptiveConfig { total_steps: input.total_steps, target_accept: a.target_accept, initial_alpha: input.alpha,
                adapt_interval: a.adapt_interval, gamma_0: a.gamma_0, pop_tolerance: tolerance, coarse_tol_factor: a.coarse_tol_factor, p: input.percentile },
            Some(&geoids), fine_level, &input.coarse_level, bg_tuple)?;
        (plan, json!({"final_alpha":result.final_alpha,"alpha_trace":result.alpha_trace,
            "fine_acceptance_rate":result.fine_acceptance_rate,"coarse_acceptance_rate":result.coarse_acceptance_rate}))
    } else {
        let sampler=match input.native_sampler {
            Some(NativeMultiscaleSampler::Native32)=>run_multiscale_native32,
            Some(NativeMultiscaleSampler::Native64)=>run_multiscale_native64,
            None=>run_multiscale_portable,
        };
        (sampler(&g.adjacency, &g.population, &edges, o.districts, tolerance, o.iterations,
            o.seed.unwrap_or(0), input.total_steps, input.alpha, input.percentile, Some(&geoids), fine_level, &input.coarse_level, bg_tuple)?, Value::Null)
    };
    partition_timer.finish()?;
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
    let rng_profile=match input.native_sampler {
        Some(profile)=>serde_json::to_value(profile).map_err(|e|e.to_string())?,
        None=>json!("chacha12-u64-portable-wilson-v1"),
    };
    let mut output = json!({"schema_version":"bisect-multiscale-result-v1","state":g.state,"year":g.year,
        "resolution":input.fine_level,"assignments":assignment,"populations":populations,"connected":connected,
        "max_deviation_percent":max_deviation,"within_requested_tolerance":if input.native_sampler.is_some(){populations.iter().all(|p|(*p as f64-ideal).abs()/ideal<=tolerance)}else{max_deviation <= o.balance_tolerance},
        "evidence":{"method":if input.adaptive.is_some(){"multiscale-adaptive"}else{"multiscale"},
            "rng":rng_profile,"graph_sha256":format!("{:x}",hash.finalize()),
            "coarse_level":input.coarse_level,"total_steps":input.total_steps,"alpha":input.alpha,"percentile":input.percentile,
            "initialization":if composed.is_some(){"native-composed-tract-metis"}else if partisan_evidence.is_some(){"partisan-tract-metis"}else if native_character.is_some(){"native-character-source-tract-metis"}else if character_absent{"absent-character-geographic-metis"}else if character.is_some(){"character-tract-metis"}else if o.weights=="vra-aligned"{"minority-tract-metis"}else if o.weights=="county"{"county-tract-metis"}else if o.weights=="unweighted"{"unweighted-tract-metis"}else if o.weights=="zone-membership"{if zone.is_some(){"zone-membership-tract-metis"}else{"zone-unavailable-geographic-metis"}}else{"weighted-geographic-metis"},"sampling":"unweighted-wilson-recom",
            "ranking":"fine-unweighted-edge-cut-then-step","coarse_assignment":"last-fine-index-wins-native-policy",
            "diagnostic_policy":"native-counters: fine-attempts-counted-as-accepted; coarse-rebalance-success",
            "seed":serde_json::to_value(o).map_err(|e|e.to_string())?["seed"],"diagnostics":diagnostics,
            "execution":"heuristic","optimality":"unproved"}});
    if let Some(computed)=native_character{output["evidence"]["character_initialization"]=crate::native_character_weights::evidence(&computed.report,"parent-tract-initialization-only; no fine-unit character observations inferred");}
    if character_absent{output["evidence"]["character_initialization"]=crate::character_input::absent_evidence(&o.weights,g.geoids.len(),"parent-tract-initialization-only; no fine-unit character observations inferred");}
    if let Some(weights)=&character{output["evidence"]["character_initialization"]=json!({"method":"character-cosine-blend","kind":weights.kind,"alpha":weights.alpha,"character_sha256":weights.character_hash,"formula":weights.formula,"zero_policy":weights.zero_policy,"data_year":input.request.character.as_ref().unwrap().data_year,"tracts":g.geoids.len(),"scope":"parent-tract-initialization-only; no fine-unit character observations inferred"});}
    if let Some((fractions,digest))=minority {let high=fractions.iter().filter(|&&f|f>=0.40).count();output["evidence"]["minority_initialization"]=json!({"method":"native-minority-override-v1","threshold":0.40,"high_tracts":high,"tracts":fractions.len(),"alpha":3.0_f64.max(10.0*(1.0-0.7*high as f64/fractions.len() as f64)),"basis":"total-population","demographics_sha256":digest,"dispatch_representation":"native-sparse-boost-map","scope":"parent-tract-initialization-only; no demographic preservation or block-group observations inferred"});}
    if o.weights=="county"{output["evidence"]["seed_weighting"]=json!({"method":"native-county-additive-v1","alpha":o.alpha_county,"within_county_multiplier":if o.alpha_county>1e-10{1.0+o.alpha_county}else{1.0},"baseline":"physical-geographic-boundary","county_identity":"first-five-geoid-digits","disabled_at_or_below":1e-10,"scope":"parent-tract-initialization-only"});}
    if o.weights=="unweighted"{output["evidence"]["seed_weighting"]=json!({"method":"native-unweighted-empty-map","computed_edges":0,"coi_effect":"none","scope":"parent-tract-initialization-only"});}
    if o.weights=="zone-membership"{output["evidence"]["zone_initialization"]=zone.as_ref().map(|z|z.evidence("parent-tract-initialization-only")).unwrap_or_else(||crate::engine::zone_fallback_evidence("parent-tract-initialization-only"));}
    if let Some(evidence)=partisan_evidence {output["evidence"]["partisan_initialization"]=evidence;}
    if let Some(evidence)=county_evidence{output["evidence"]["county_augmentation"]=evidence;}
    if let Some(evidence) = coi_evidence { output["evidence"]["coi_weighting"] = evidence; }
    if let Some(settings)=&input.request.electoral {output["evidence"]["electoral_settings"]=settings.evidence(o.districts,g.population.iter().sum())?;}
    if let Some(computed)=composed{for(key,value)in computed.metrics{output["evidence"][key]=value;}}
    if let Some(evidence)=native_coi_evidence{output["evidence"]["native_coi_source_evidence"]=evidence;}
    if require_valid{validate_native_final(&input,&output,true)?;}
    Ok(output)
}

pub(crate) fn validate_native_final(input:&MultiscaleRequest,result:&Value,check_population:bool)->Result<(),String>{
    if input.native_sampler.is_none(){return Ok(())}
    let g=&input.request.graph;let o=&input.request.options;
    let ids=input.block_groups.as_ref().map_or(&g.geoids,|b|&b.geoids);
    let population=input.block_groups.as_ref().map_or(&g.population,|b|&b.population);
    if check_population{crate::engine_build::population_check(result,ids,population,o.districts,o.effective_balance_tolerance())??;}
    let connected=result["connected"].as_array().ok_or("Missing native fine connectivity result")?;
    if connected.len()!=o.districts{return Err("Incomplete native fine connectivity result".into());}
    if let Some(d)=connected.iter().position(|v|v!=&Value::Bool(true)){return Err(format!("multiscale final plan is disconnected in district {}",d+1));}
    Ok(())
}
