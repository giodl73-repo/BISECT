//! In-memory location preflight and supplied RCTX diagnostics; no plan generation.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use bisect_analysis::location_registry::{PolicyRegistry,chamber_balance_tolerance};
#[derive(serde::Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct Settings {pub state:String,pub year:String,pub chamber:String,pub resolution:String}
pub(crate) fn inspect(settings:Settings,source_name:Option<&str>,policy_json:Option<&str>,context_name:Option<&str>,context_json:Option<&str>)->Result<Value,String>{
 for value in [&settings.state,&settings.year,&settings.chamber,&settings.resolution]{if value.trim().is_empty()||value.len()>200{return Err("Supply nonempty location, year, chamber and resolution of at most 200 UTF-8 bytes each.".into());}}
 if context_name.is_some()!=context_json.is_some()||context_name.is_some_and(|n|n.is_empty()||n.len()>200)||context_json.is_some_and(|t|t.is_empty()||t.len()>8*1024*1024){return Err("Supply both context name and RCTX of at most 8 MiB, or neither.".into());}
 let (database,name,kind,source_hash)=crate::location_policy::database(source_name,policy_json)?;
 let code=settings.state.to_uppercase();let registry=PolicyRegistry::new(&database);
 let registered=registry.has_location(&code);let year=registry.validate_year(&code,&settings.year);
 let chamber_count=registry.chamber_districts(&code,&settings.chamber,&settings.year);
 let count=chamber_count.or_else(||registry.congressional_districts(&code,&settings.year));
 let entry=database.get(&code);let nesting=if settings.chamber!="congressional"{entry.and_then(|e|e.get("nesting_requirement")).and_then(Value::as_str).filter(|s|*s!="null")}else{None};
 let warning=registry.granularity_warning(&code,&settings.year,&settings.chamber,&settings.resolution);
 let mut metadata=json!({"state":code,"name":registry.state_name(&code),"year":settings.year,"chamber":settings.chamber,"resolution":settings.resolution,
  "registered":registered,"available_years":registry.available_years(&code),"year_valid":year.is_ok(),"year_error":year.err(),
  "districts":count.map(|n|n.to_string()),"district_count_source":if chamber_count.is_some(){"policy-chamber"}else if count.is_some(){"policy-congressional-fallback"}else{"unavailable; native filesystem manifest fallback not supplied"},
  "policy_tolerance_percent":chamber_balance_tolerance(&database,&code,&settings.chamber)*100.,"granularity_warning":warning,
  "nesting_requirement":nesting,"nesting_ratio":nesting.and_then(|_|entry.and_then(|e|e.get("nesting_ratio")).and_then(Value::as_str)),
  "compactness_standard":entry.and_then(|e|e.get("compactness_standard")).and_then(Value::as_str)});
 crate::location_policy::exact_display(&mut metadata);
 let mut checks=vec![json!({"check":"location","status":if registered{"pass"}else{"fail"},"message":if registered{"Location is registered."}else{"Location is not registered."}}),
  json!({"check":"year","status":if metadata["year_valid"]==true{"pass"}else{"fail"},"message":metadata["year_error"].as_str().unwrap_or("Year is permitted by the native registry; an empty year list imposes no restriction.")}),
  json!({"check":"district-count","status":if count.is_some(){"pass"}else{"warn"},"message":count.map(|n|format!("Native policy lookup reports {n} districts.")).unwrap_or_else(||"No policy district count. Native manifest fallback requires a separate supplied catalog; it was not guessed.".into())}),
  json!({"check":"resolution","status":if warning.is_some(){"warn"}else{"info"},"message":warning.unwrap_or_else(||"No native registry granularity warning; this does not prove adequate input resolution.".into())})];
 let mut context_summary=Value::Null;let mut missing=context_json.is_none();
 if let Some(text)=context_json {
  match diagnose_context(text,&code,&settings.year,&settings.resolution,count){
   Ok((summary,mut context_checks))=>{missing=context_checks.iter().any(|c|c["status"]=="missing");context_summary=summary;checks.append(&mut context_checks);},
   Err(error)=>checks.push(json!({"check":"context","status":"fail","message":error})),
  }
 }else{checks.push(json!({"check":"context","status":"missing","message":"Supply an exact RCTX to inspect its graph and populations. No local filesystem was scanned."}));}
 checks.push(json!({"check":"map-geometry","status":"not-checked","message":"TIGER filesystem discovery is unavailable in the browser. Display geometry must be supplied and checked separately against unit identity."}));
 let status=if checks.iter().any(|c|c["status"]=="fail"){"invalid-inputs"}else if missing{"needs-inputs"}else{"checks-passed"};
 Ok(json!({"schema_version":"bisect-location-preflight-v1","status":status,"metadata":metadata,"checks":checks,"context":context_summary,
  "source_name":name,"source_kind":kind,"source_sha256":source_hash,"context_source_name":context_name,"context_source_sha256":context_json.map(|t|format!("sha256:{:x}",Sha256::digest(t.as_bytes()))),
  "scope":"Shared native registry location/year, district-count, tolerance and granularity rules, followed by native RCTX schema/hash checks and explicit dataset diagnostics. Registry year lists may be unrestricted when absent; chamber lookup retains the native congressional fallback. Browser policy overrides are explicit and bounded; environment, home-directory, manifest and adjacency-path discovery do not occur. Map geometry, native plan-label/all-plan diagnosis, manifest provenance verification and pinned tutorial checks are separate capabilities. Policy values are reference data of unverified current legal accuracy and are not applied to a run. These checks do not execute an engine, establish support for every engine option, prove a feasible balanced plan or certify legal compliance."}))
}
fn diagnose_context(text:&str,code:&str,year:&str,resolution:&str,districts:Option<u64>)->Result<(Value,Vec<Value>),String>{
 let raw:Value=serde_json::from_str(text).map_err(|e|format!("Invalid RCTX JSON: {e}"))?;
 let n=raw["units"]["unit_ids"].as_array().map_or(0,Vec::len);
 let arcs=raw["graph"]["adjacency"].as_array().map(|rows|rows.iter().map(|r|r.as_array().map_or(0,Vec::len)).sum::<usize>()).unwrap_or(0);
 if n==0||n>100_000||arcs>2_000_000{return Err("Preflight RCTX supports 1–100,000 units and at most 2,000,000 graph arcs.".into());}
 let context=rplan_io::read_rctx_str(text).map_err(|e|e.to_string())?;
 if context.units.unit_universe_hash!=context.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Context unit-universe hash mismatch.".into());}
 let expected_kind=resolution.replace('_',"-");let actual_kind=serde_json::to_value(context.units.unit_kind).map_err(|e|e.to_string())?;
 let identity=context.units.state.as_deref()==Some(code)&&context.units.year.map(|y|y.to_string()).as_deref()==Some(year)&&actual_kind==expected_kind;
 let mut checks=vec![json!({"check":"context-identity","status":if identity{"pass"}else{"fail"},"message":if identity{"Context location, year and resolution match the request."}else{"Context location, year or unit kind differs from the request, or is absent."}})];
 let mut components=None;
 if let Some(graph)=&context.graph {
  let mut seen=vec![false;n];let mut count=0;
  for root in 0..n{if seen[root]{continue;}count+=1;let mut stack=vec![root];seen[root]=true;while let Some(u)=stack.pop(){for edge in &graph.adjacency[u]{let v=edge.to as usize;if !seen[v]{seen[v]=true;stack.push(v);}}}}
  components=Some(count);
  let impossible=districts.is_some_and(|k|count as u64>k);
  checks.push(json!({"check":"graph","status":if impossible{"fail"}else if count>1{"warn"}else{"pass"},"message":if impossible{format!("{count} graph components exceed the district count; a fully connected plan cannot cover them with that many districts.")}else{format!("Native undirected graph schema passed; {count} connected component(s), including supplied bridge edges.")}}));
 }else{checks.push(json!({"check":"graph","status":"missing","message":"Context has no adjacency graph."}));}
 let mut total=None;
 if let Some(populations)=&context.populations {
  let sum: i128=populations.iter().map(|p|*p as i128).sum();let valid=populations.iter().all(|p|*p>=0)&&sum>0;total=Some(sum.to_string());
  checks.push(json!({"check":"population","status":if valid{"pass"}else{"fail"},"message":if valid{"Nonnegative native counts with positive total population."}else{"Population counts are negative or total population is not positive."}}));
 }else{checks.push(json!({"check":"population","status":"missing","message":"Context has no population vector."}));}
 if let Some(k)=districts {checks.push(json!({"check":"unit-count","status":if k>n as u64{"fail"}else{"info"},"message":format!("{n} units for {k} requested policy districts. Unit count alone does not prove balance feasibility.")}));}
 Ok((json!({"unit_count":n,"graph_arcs":arcs,"components":components,"total_population":total,"unit_kind":actual_kind,"context_hash":context.context_hash,"unit_universe_hash":context.units.unit_universe_hash}),checks))
}
