//! Shared native NSGA-II frontier with separately measured final constraints.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings{pub k:usize,pub population:usize,pub generations:usize,pub tolerance:f64,pub base_seed:String,pub democratic_shares:Option<Vec<f64>>,pub minority_shares:Option<Vec<f64>>}
pub(crate) fn run(text:&str,s:Settings)->Result<Value,String>{run_with_result(text,s).map(|(view,_,_)|view)}
fn run_with_result(text:&str,s:Settings)->Result<(Value,bisect_pareto::ParetoResult,rplan_core::RplanContext),String>{
 if text.len()>8*1024*1024{return Err("Pareto context exceeds 8 MiB.".into());}
 let seed=s.base_seed.parse::<u64>().map_err(|_|"Supply a canonical unsigned 64-bit seed.")?;
 if seed.to_string()!=s.base_seed{return Err("Supply a canonical unsigned 64-bit seed.".into());}
 if !(1..=500).contains(&s.k)||!(2..=1000).contains(&s.population)||s.generations>1000{return Err("Pareto supports 1–500 districts, 2–1,000 plans and 0–1,000 generations.".into());}
 let c=rplan_io::read_rctx_str(text).map_err(|e|e.to_string())?;let n=c.units.unit_ids.len();
 if n>10000||n.checked_mul(s.population).and_then(|w|w.checked_mul(s.generations+1)).is_none_or(|w|w>50000000)||s.population.checked_mul(s.population).and_then(|w|w.checked_mul(s.generations+1)).is_none_or(|w|w>10000000){return Err("Pareto exceeds unit/population/generation work limits.".into());}
 let g=c.graph.as_ref().ok_or("RCTX graph is required.")?;
 if g.edge_semantics!=rplan_core::EdgeSemantics::Undirected||g.adjacency.iter().map(Vec::len).sum::<usize>()>100000{return Err("Pareto requires an undirected graph with at most 100,000 arcs.".into());}
 let p=c.populations.as_ref().ok_or("RCTX populations are required.")?;
 let a:Vec<Vec<usize>>=g.adjacency.iter().map(|es|es.iter().map(|e|e.to as usize).collect()).collect();
 let mut out=bisect_pareto::run_nsga2(&a,p,s.k,s.democratic_shares.as_deref(),s.minority_shares.as_deref(),&[],bisect_pareto::ParetoConfig{n_population:s.population,n_generations:s.generations,base_seed:seed,balance_tolerance:s.tolerance}).map_err(|e|e.to_string())?;
 // Native wall-clock timing is measured outside reproducibility output; WASM has no std clock.
 out.runtime_secs=0.0;
 let mut diagnostics=Vec::new();
 for (i,e) in out.frontier.iter_mut().enumerate(){
  let plan=rplan_core::DistrictPlan{schema_version:rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(),units:c.units.clone(),assignment:e.plan.iter().map(|d|d-1).collect(),k:s.k,display_labels:(1..=s.k).map(|d|d.to_string()).collect(),allow_empty_districts:true};
  let m=rplan_audit::measurements::measure_plan(&plan,Some(&c))?;let ideal=m.total_population.ok_or("Population unavailable.")? as f64/s.k as f64;
  let deviation=m.districts.iter().map(|d|(d.population.as_ref().expect("supplied populations").population as f64-ideal).abs()/ideal).fold(0.0,f64::max);
  let connected=m.districts.iter().all(|d|d.units>0&&d.component_count==Some(1));let balanced=deviation<=s.tolerance;
  e.validity_status=Some(if connected&&balanced{"passes-measured-constraints"}else{"fails-measured-constraints"}.into());
  diagnostics.push(json!({"frontier_index":i,"edge_cut":e.objectives.ec,"partisan_deviation":s.democratic_shares.as_ref().map(|_|e.objectives.d_seats),"minority_deficit":s.minority_shares.as_ref().map(|_|e.objectives.vra_deficit),"generation_found":e.generation_found,"all_nonempty_and_connected":connected,"maximum_population_deviation":deviation,"passes_final_population_tolerance":balanced}));
 }
 let mut bytes=Vec::new();out.write_ndjson(&mut bytes).map_err(|e|e.to_string())?;let ndjson=String::from_utf8(bytes).map_err(|e|e.to_string())?;
 let view=json!({"schema_version":"bisect-pareto-context-output-v1","protocol":out.pareto_version,"base_seed":s.base_seed,"k":s.k,"population":s.population,"generations":s.generations,"context_hash":c.context_hash,"context_bytes_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"ndjson_sha256":format!("{:x}",Sha256::digest(ndjson.as_bytes())),"ndjson":ndjson,"diagnostics":diagnostics,"scope":"Shared native NSGA-II heuristic frontier among generated candidates, not a global Pareto optimum. Final balance/contiguity measured separately; initial generation may fail tolerance. Optional share objectives use total-population weighting; these are proxies, not ballot-count or VAP-weighted measurements or legal VRA certification. Unprovided objectives are unavailable in diagnostics (native NDJSON records zero). Edge weights and protected-district lists are unused. Reproducibility protocol 2 changes version-1 random streams. Runtime in NDJSON is zero (not measured). District labels are one-based. Cancel stops the worker."});
 Ok((view,out,c))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {pub frontier_index:usize,pub label:String,pub generated_at:String}
pub(crate) fn select(text:&str,ndjson:&str,settings:Settings,selection:Selection)->Result<Value,String>{
 if ndjson.len()>8*1024*1024{return Err("Pareto frontier exceeds 8 MiB.".into());}
 for field in [&selection.label,&selection.generated_at]{if field.trim().is_empty()||field.len()>200{return Err("Selection label/timestamp must be nonempty and at most 200 bytes.".into());}}
 let k=settings.k;let tolerance=settings.tolerance;
 let (replay,result,context)=run_with_result(text,settings)?;
 if replay["ndjson"].as_str()!=Some(ndjson){return Err("Frontier does not match exact native replay against supplied context/settings.".into());}
 let built=bisect_pareto::build_selected_frontier_package(&result,selection.frontier_index,&context,k,&selection.label,tolerance*100.0,&selection.generated_at).map_err(|e|e.to_string())?;
 let document_json=&built.files["selected-frontier.rplan"];let context_json=&built.files["selected-frontier.rctx"];
 let certificate:Value=serde_json::from_str(&built.files["audit-certificate.json"]).map_err(|e|e.to_string())?;
 let lineage=serde_json::to_string_pretty(&certificate["algorithm_lineage"]).map_err(|e|e.to_string())?;
 let file_hashes:std::collections::BTreeMap<String,String>=built.files.iter().map(|(name,text)|(name.clone(),format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
 Ok(json!({"schema_version":"bisect-pareto-selected-plan-v1","replay_verified":true,"base_seed":replay["base_seed"],"frontier_index":selection.frontier_index,"frontier_sha256":replay["ndjson_sha256"],"context_hash":context.context_hash,"document_json":document_json,"context_json":context_json,"algorithm_lineage_json":lineage,"diagnostic":replay["diagnostics"][selection.frontier_index],"package":built.package,"files":built.files,"file_hashes":file_hashes,
 "scope":"Selected frontier candidate reproduced by exact native replay using supplied context, settings and optional shares. Native package creation audits population and contiguity against its generated profile; failing plans cannot be packaged. NDJSON does not bind original context/shares/settings, so replay proves reproducibility with supplied inputs, not historical provenance. This generated profile does not cover every legal requirement or certify VRA. No global Pareto optimum or authenticated source claim. Package chamber uses the native congressional default. Four native files retain exact bytes; map geometry is a separate supplied source."}))
}
