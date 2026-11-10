//! Native SMC sampling from an exact, hash-checked context.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub k:usize,
    pub particles:usize,
    pub population_tolerance:f64,
    pub resample_threshold:f64,
    pub base_seed:String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SelectionSettings {
    pub particle_index:usize,
    pub label:String,
    pub chamber:String,
    pub created_at:String,
}

pub(crate) fn select(context_text:&str,ndjson:&str,settings:SelectionSettings)->Result<Value,String>{
    if ndjson.len()>8*1024*1024 {return Err("Ensemble exceeds 8 MiB.".into());}
    for field in [&settings.label,&settings.chamber,&settings.created_at] {
        if field.trim().is_empty()||field.len()>200 {return Err("Selection metadata must be nonempty and at most 200 bytes.".into());}
    }
    let metadata:bisect_smc::output::MetadataRecord=crate::certified_exact::parse(ndjson.lines().last().ok_or("Empty ensemble.")?)?;
    if metadata.record_type!="metadata"||metadata.smc_version!="2.0-chacha12-u64-v1"||metadata.ensemble_output_version!="1.0" {
        return Err("Unsupported SMC ensemble protocol.".into());
    }
    if settings.particle_index>=metadata.n_particles {return Err("Selected particle is out of range.".into());}
    // A payload hash alone cannot bind the native legacy NDJSON to a context.
    // Replay establishes reproducibility against these exact ordered inputs.
    let replay=run(context_text,Settings {k:metadata.k,particles:metadata.n_particles,population_tolerance:metadata.pop_tolerance,
        resample_threshold:metadata.resample_threshold,base_seed:metadata.base_seed.to_string()})?;
    if replay["ndjson"].as_str()!=Some(ndjson) {return Err("Ensemble does not match native replay against the supplied context and recorded settings.".into());}
    let diagnostic=&replay["diagnostics"][settings.particle_index];
    if diagnostic["positive_weight"]!=true||diagnostic["all_nonempty_and_connected"]!=true {
        return Err("Selected particle must have positive weight and nonempty connected districts.".into());
    }
    let particle:Value=crate::certified_exact::parse(ndjson.lines().nth(settings.particle_index).ok_or("Missing particle.")?)?;
    let labels:Vec<u32>=serde_json::from_value(particle["plan"].clone()).map_err(|e|e.to_string())?;
    let context=rplan_io::read_rctx_str(context_text).map_err(|e|e.to_string())?;
    let extra=json!({"protocol":metadata.smc_version,"input_context_hash":context.context_hash,"ensemble_sha256":replay["ndjson_sha256"],
        "base_seed":metadata.base_seed.to_string(),"particle_index":settings.particle_index,"weight":diagnostic["weight"],
        "particles":metadata.n_particles,"population_tolerance":metadata.pop_tolerance,"resample_threshold":metadata.resample_threshold,
        "replay_verified":true});
    let lineage=rplan_audit::AlgorithmLineage::new("bisect-smc",env!("CARGO_PKG_VERSION"),"smc-selected-particle",Vec::new(),extra.clone()).map_err(|e|e.to_string())?;
    let document=rplan_io::RplanDocument {rplan_version:rplan_io::RPLAN_V02.into(),plan:rplan_core::DistrictPlan {
        schema_version:rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(),units:context.units.clone(),assignment:labels.iter().map(|&d|d-1).collect(),
        k:metadata.k,display_labels:(1..=metadata.k).map(|d|d.to_string()).collect(),allow_empty_districts:false},
        metadata:rplan_io::RplanMetadataV02 {label:settings.label,jurisdiction:context.units.state.clone().unwrap_or_else(||"US".into()),chamber:settings.chamber,
            created_at:settings.created_at,description:Some("Selected native SMC particle, reproduced against supplied context. Audit against an explicit profile before relying on legal compliance.".into())},
        provenance:rplan_io::RplanProvenance {producer:std::collections::BTreeMap::from([("crate".into(),json!("bisect-smc")),("method".into(),json!("smc-selected-particle")),("parameters".into(),extra)]),
            source_hashes:context.source_hashes.entries.clone(),conversion_lineage:Vec::new()},geometry:None,extensions:Default::default()};
    let text=rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?;
    rplan_io::read_rplan_str(&text).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-smc-selected-plan-v1","replay_verified":true,"document_json":text,"context_json":context_text,
        "plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,
        "base_seed":metadata.base_seed.to_string(),"ensemble_sha256":replay["ndjson_sha256"],"diagnostic":diagnostic,
        "algorithm_lineage_json":serde_json::to_string_pretty(&lineage).map_err(|e|e.to_string())?,
        "scope":"Selected positive-weight particle with nonempty connected districts. Exact native replay reproduces the ensemble using supplied context/settings and its recorded seed. Legacy NDJSON does not record context identity, so replay cannot establish which context originally produced it. RPLAN district labels are zero-based. Population tolerance is reported separately and may fail. This is reproducibility evidence, not source authentication, calibrated sampling, global optimality or legal certification. No embedded map geometry is generated."}))
}

pub(crate) fn run(text:&str,settings:Settings)->Result<Value,String>{
    if text.len()>8*1024*1024 {return Err("SMC context exceeds 8 MiB.".into());}
    let seed=settings.base_seed.parse::<u64>().map_err(|_|"Seed must be an unsigned 64-bit decimal integer.")?;
    if seed.to_string()!=settings.base_seed {return Err("Seed must use canonical unsigned decimal notation.".into());}
    if !(1..=500).contains(&settings.k)||!(1..=1000).contains(&settings.particles) {return Err("SMC supports 1 through 500 districts and 1 through 1,000 particles.".into());}
    let context=rplan_io::read_rctx_str(text).map_err(|e|e.to_string())?;
    let n=context.units.unit_ids.len();
    if n>10_000 || n.checked_mul(settings.particles).is_none_or(|work|work>1_000_000) {return Err("SMC supports at most 10,000 units and 1,000,000 unit-particle entries.".into());}
    let graph=context.graph.as_ref().ok_or("RCTX graph is required.")?;
    if graph.edge_semantics!=rplan_core::EdgeSemantics::Undirected||graph.adjacency.iter().map(Vec::len).sum::<usize>()>100_000 {return Err("SMC requires an undirected graph with at most 100,000 arcs.".into());}
    let populations=context.populations.as_ref().ok_or("RCTX populations are required.")?;
    let adjacency:Vec<Vec<usize>>=graph.adjacency.iter().map(|edges|edges.iter().map(|e|e.to as usize).collect()).collect();
    let result=bisect_smc::run_smc(&adjacency,populations,settings.k,bisect_smc::SmcConfig {n_particles:settings.particles,resample_threshold:settings.resample_threshold,pop_tolerance:settings.population_tolerance,base_seed:seed}).map_err(|e|e.to_string())?;
    let mut bytes=Vec::new();result.write_ndjson(&mut bytes,&bisect_smc::WriteConfig {resample_threshold:settings.resample_threshold,pop_tolerance:settings.population_tolerance}).map_err(|e|e.to_string())?;
    let ndjson=String::from_utf8(bytes).map_err(|e|e.to_string())?;
    let mut diagnostics=Vec::with_capacity(result.plans.len());
    for (index,labels) in result.plans.iter().enumerate(){
        let plan=rplan_core::DistrictPlan {schema_version:rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(),units:context.units.clone(),assignment:labels.iter().map(|&d|d.saturating_sub(1)).collect(),k:settings.k,
            display_labels:(1..=settings.k).map(|d|d.to_string()).collect(),allow_empty_districts:true};
        let measured=rplan_audit::measurements::measure_plan(&plan,Some(&context))?;
        let total=measured.total_population.ok_or("Population measurement unavailable.")?;
        let ideal=total as f64/settings.k as f64;
        let max_deviation=measured.districts.iter().map(|d|(d.population.as_ref().expect("supplied populations").population as f64-ideal).abs()/ideal).fold(0.0,f64::max);
        diagnostics.push(json!({"particle_index":index,"weight":result.weights[index],"positive_weight":result.weights[index]>0.0,
            "all_nonempty_and_connected":measured.districts.iter().all(|d|d.units>0&&d.component_count==Some(1)),
            "maximum_population_deviation":max_deviation,"passes_final_population_tolerance":max_deviation<=settings.population_tolerance}));
    }
    Ok(json!({"schema_version":"bisect-smc-context-output-v1","base_seed":seed.to_string(),"k":settings.k,"particles":settings.particles,
        "context_hash":context.context_hash,"context_bytes_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),
        "ndjson":ndjson,"ndjson_sha256":format!("{:x}",Sha256::digest(ndjson.as_bytes())),"ess_trace":result.ess_trace,
        "resample_count":result.resample_count,"diagnostics":diagnostics,
        "scope":"Native SMC protocol 2 sample with exact decimal seed and exact NDJSON. Native district labels are one-based. Zero-weight particles have null log_weight in NDJSON and remain recorded. Final population/contiguity diagnostics are measured separately from proposal success: proposal tolerance uses remaining-component population. Finite sample weights and ESS do not establish distribution calibration, global optimality or legal compliance. Graph edge weights do not affect this sampler. Cancel stops the worker."}))
}
