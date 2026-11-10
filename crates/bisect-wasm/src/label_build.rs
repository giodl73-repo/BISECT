//! Serial browser label build over explicitly supplied prepared state graphs.
use serde::Deserialize;
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Job {
 #[serde(default)]pub compact_geometry:Option<String>,
 #[serde(default)]pub electoral:Option<crate::electoral_input::Input>,
 pub graph:crate::engine::PreparedGraph,pub districts:usize,pub chamber:String,
 pub demographics:Option<crate::engine::DemographicInput>,
 #[serde(default)]pub coi:Option<crate::coi_input::Input>,
 #[serde(default)]pub vra_absent_file_fallback:bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub label:String,pub yaml_text:String,pub source_name:String,pub profile:String,
 pub years_override:Option<Vec<String>>,pub registry_json:Option<String>,
 pub force:bool,pub created_at:String,pub jobs:Vec<Job>,
 #[serde(default)] pub capture_rounds:bool,
 #[serde(default)] pub write_manifest:bool,
 #[serde(default)] pub retain_replay:bool,
}
type Registry = BTreeMap<String,bisect_report::label_registry::LabelEntry>;
fn prepare(input:&Input,dry_run:bool)->Result<(bisect_runner::config::AlgoYaml,Vec<String>,Registry),String>{
 bisect_report::label_rename::validate_name(&input.label)?;
 if input.label.len()>100||["__proto__","constructor","prototype"].contains(&input.label.as_str())||input.source_name.len()>200||(!dry_run&&input.created_at.is_empty())||input.created_at.len()>100||input.registry_json.as_ref().is_some_and(|s|s.len()>1024*1024)||input.jobs.len()>600 {return Err("Invalid or oversized label build metadata, registry or job count.".into());}
 let validated=crate::config::validate(&input.yaml_text,&input.source_name,&input.profile)?;
 if !dry_run&&validated["browser_engine_available"]!=true{return Err("Label builds require an explicit metis-core engine.".into());}
 let yaml=bisect_runner::config::AlgoYaml::from_text(&input.yaml_text)?;
 let years=input.years_override.clone().unwrap_or_else(||yaml.resolved_years());
 let mut seen_years=BTreeSet::new();
 if years.is_empty()||years.len()>10||years.iter().any(|y|y.len()!=4||!y.bytes().all(|b|b.is_ascii_digit())||!seen_years.insert(y.clone())){return Err("Choose 1–10 distinct four-digit build years.".into());}
 let registry=bisect_report::label_registry::parse(input.registry_json.as_deref().unwrap_or(""))?;
 if registry.len()>500||(!registry.contains_key(&input.label)&&registry.len()==500)||registry.iter().any(|(n,e)|n.len()>200||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Registry exceeds browser label/stage limits.".into());}
 if !dry_run&&registry.get(&input.label).map_or(years.len(),|e|e.built.len()+years.iter().filter(|y|!e.built.contains(y)).count())>100{return Err("Completed registry would exceed 100 built-year records.".into());}
 let mut seen=BTreeSet::new();let mut units=0usize;let mut edges=0usize;
 for job in &input.jobs {
  if job.vra_absent_file_fallback&&job.demographics.is_some(){return Err("An absent-file state job cannot include demographic data.".into());}
  units=units.checked_add(job.graph.geoids.len()).ok_or("Unit count overflow.")?;
  edges=edges.checked_add(job.graph.edges.len()).ok_or("Edge count overflow.")?;
  if !years.contains(&job.graph.year)||job.graph.state.len()!=2||!job.graph.state.bytes().all(|b|b.is_ascii_uppercase())||!seen.insert((job.graph.year.clone(),job.graph.state.clone()))||job.districts==0||job.districts>500||!matches!(job.chamber.as_str(),"congressional"|"house"|"senate") {return Err("Jobs require unique state/year pairs, a selected year, valid chamber and 1–500 districts.".into());}
 }
 if units>500_000||edges>2_000_000{return Err("Selected label build exceeds 500,000 units or 2,000,000 edges.".into());}
 for year in &years {let skipped=registry.get(&input.label).is_some_and(|e|e.built.contains(year))&&!input.force;if !dry_run&&!skipped&&!input.jobs.iter().any(|j|&j.graph.year==year){return Err(format!("No prepared jobs selected for build year {year}."));}if input.jobs.iter().filter(|j|&j.graph.year==year).count()>60{return Err("A build year supports up to 60 state jobs.".into());}}
 Ok((yaml,years,registry))
}

/// Read-only native-order preflight. No configured build or partition is invoked.
pub(crate) fn plan(input:Input,dry_run:bool)->Result<Value,String>{
 let (_yaml,years,mut registry)=prepare(&input,dry_run)?;
 let policy:Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
 let locations=bisect_analysis::location_registry::PolicyRegistry::new(&policy);
 let mut rows=Vec::new();
 for year in years {
  let already=registry.get(&input.label).is_some_and(|e|e.built.contains(&year));
  let skipped=already&&!input.force;
  let invalidate=already&&input.force&&!dry_run;
  let jobs=input.jobs.iter().enumerate().filter(|(_,j)|j.graph.year==year).map(|(ordinal,j)|{
   let name=locations.state_name(&j.graph.state).unwrap_or_else(||j.graph.state.clone()).to_lowercase().replace(' ',"_");
   let root=format!("runs/{}/{year}/{name}",input.label);
   let mut paths=vec![format!("{root}/final_assignments.json"),format!("{root}/plan.rplan"),format!("{root}/context.rctx")];
   if input.capture_rounds{paths.push(format!("{root}/round_capture.json"));}
   if input.write_manifest{paths.push(format!("{root}/manifest.json"));}
   if input.retain_replay{paths.push(format!("{root}/accepted_engine_run.json"));paths.push(format!("{root}/accepted_configured_run.json"));}
   json!({"job_index":ordinal,"state":j.graph.state,"state_name":name,"chamber":j.chamber,"districts":j.districts,"potential_paths":paths})
  }).collect::<Vec<_>>();
  if invalidate{let e=registry.get_mut(&input.label).ok_or("Missing built registry entry")?;e.analyzed.retain(|y|y!=&year);e.reported.retain(|y|y!=&year);}
  let invalidation_registry_json=if invalidate{Some(serde_json::to_string_pretty(&registry).map_err(|e|e.to_string())?)}else{None};
  let remove_paths=if invalidate{vec![format!("analysis/{}/{year}",input.label),format!("reports/{}/{year}",input.label)]}else{Vec::new()};
  rows.push(json!({"year":year,"already_built":already,"status":if skipped{"skipped-already-built"}else if dry_run{"dry-run"}else{"scheduled"},"invalidate":invalidate,"invalidation_registry_json":invalidation_registry_json,"remove_paths":remove_paths,"jobs":jobs,"index_path":format!("runs/{}/{year}/index.json",input.label),"registry_marked_built_before_execution":false}));
  // Virtual completion feeds later year planning, never filesystem authority.
  if !skipped&&!dry_run{let e=registry.entry(input.label.clone()).or_default();if !e.built.contains(&year){e.built.push(year);}}
  let completion=if !skipped&&!dry_run{Some(serde_json::to_string_pretty(&registry).map_err(|e|e.to_string())?)}else{None};
  rows.last_mut().ok_or("Missing planned year")?["completed_registry_json"]=json!(completion);
 }
 Ok(json!({"schema_version":"bisect-label-build-plan-v1","label":input.label,"dry_run":dry_run,"years":rows,"config_sha256":format!("{:x}",Sha256::digest(input.yaml_text.as_bytes())),"registry_source_sha256":input.registry_json.as_ref().map(|s|format!("{:x}",Sha256::digest(s.as_bytes()))),"execution_performed":false,"filesystem_performed":false,"scope":"Read-only native-order config/registry preflight for selected prepared jobs. Force invalidation keeps built declarations and precedes partition execution. Dry-run makes no directories, registry changes or removals. Empty job selection in dry-run is unexpanded native all-state intent, not discovered inputs. Planned completion and paths confer no write authority or successful state outcome."}))
}

pub(crate) fn run(input:Input)->Result<Value,String>{
 let (yaml,years,mut registry)=prepare(&input,false)?;
 let policy:Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
 let locations=bisect_analysis::location_registry::PolicyRegistry::new(&policy);
 let config_path=format!("configs/{}.yml",input.label);
 let mut request_identity=json!({"label":input.label,"yaml_text":input.yaml_text,"source_name":input.source_name,"profile":input.profile,"years":years,"force":input.force,"created_at":input.created_at,"registry_json":input.registry_json,"jobs":input.jobs.iter().map(|j|{let mut row=json!({"graph":j.graph,"districts":j.districts,"chamber":j.chamber,"demographics":j.demographics});if let Some(settings)=&j.electoral {row["electoral"]=serde_json::to_value(settings).unwrap();}if let Some(coi)=&j.coi{row["coi"]=json!(coi);}if let Some(profile)=&j.compact_geometry{row["compact_geometry"]=json!(profile);}if j.vra_absent_file_fallback{row["vra_absent_file_fallback"]=json!(true);}row}).collect::<Vec<_>>()});
 if input.capture_rounds {request_identity["capture_rounds"]=json!(true);}
 if input.write_manifest {request_identity["write_manifest"]=json!(true);}
 if input.retain_replay {request_identity["retain_replay"]=json!(true);}
 let request_hash=format!("{:x}",Sha256::digest(serde_json::to_vec(&request_identity).map_err(|e|e.to_string())?));
 let mut sequence=0usize;
 let mut files=BTreeMap::from([(config_path.clone(),input.yaml_text.clone())]);let mut outcomes=Vec::new();let mut year_rows=Vec::new();let mut invalidated=Vec::new();
 for year in &years {
  let entry=registry.entry(input.label.clone()).or_default();
  if entry.built.contains(year)&&!input.force {year_rows.push(json!({"year":year,"status":"skipped-already-built"}));continue;}
  if entry.built.contains(year)&&input.force {entry.analyzed.retain(|y|y!=year);entry.reported.retain(|y|y!=year);invalidated.push(json!({"year":year,"paths":[format!("analysis/{}/{year}/",input.label),format!("reports/{}/{year}/",input.label)]}));}
  let mut configs=Vec::new();let mut results=Vec::new();
  for job in input.jobs.iter().filter(|j|&j.graph.year==year) {
   let state=job.graph.state.clone();let name=locations.state_name(&state).unwrap_or_else(||state.clone()).to_lowercase().replace(' ',"_");
   if !name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'){return Err("Unsafe resolved state name.".into());}
   configs.push(bisect_runner::build_index::ConfigRow{state_code:state.clone(),state_name:name.clone(),num_districts:job.districts});
   let configured=crate::config::ConfiguredRun{compact_geometry:job.compact_geometry.clone(),vra_absent_file_fallback:job.vra_absent_file_fallback,capture_rounds:input.capture_rounds,yaml_text:input.yaml_text.clone(),source_name:input.source_name.clone(),profile:input.profile.clone(),graph:job.graph.clone(),chamber:job.chamber.clone(),districts:job.districts,demographics:job.demographics.clone(),coi:job.coi.clone(),electoral:job.electoral.clone()};
   let generated=crate::config::build_typed(configured.clone()).and_then(|result|{
    let assignments: BTreeMap<String,u32>=serde_json::from_value(result["result"]["assignments"].clone()).map_err(|e|e.to_string())?;
    let attempt=result["build_report"]["accepted_attempt"].as_u64().ok_or("Missing accepted attempt.")? as u32;
    let configured_replay=if input.retain_replay{Some(json!({"kind":"configured","input":configured,"attempt":attempt}))}else{None};
    let exported=crate::config::export(configured,assignments,input.label.clone(),input.created_at.clone(),Some(attempt))?;
    let manifest=if input.write_manifest {Some(crate::plan_audit_text::report_manifest(exported["document_json"].as_str().ok_or("Missing exported plan")?,exported["context_json"].as_str().ok_or("Missing exported context")?))}else{None};
    Ok((result,exported,manifest,configured_replay))
   });
   let mut state_files=BTreeMap::new();
   match generated {
    Ok((result,exported,manifest,configured_replay))=>{
     let root=format!("runs/{}/{year}/{name}",input.label);
     let rows=job.graph.geoids.iter().enumerate().map(|(index,id)|result["result"]["assignments"][id].as_u64().map(|d|(index,d as usize)).ok_or("Missing generated assignment.".to_string())).collect::<Result<Vec<_>,_>>()?;
     state_files.insert(format!("{root}/final_assignments.json"),bisect_report::assignment_output::serialize_native_assignments(rows).map_err(|e|e.to_string())?);
     state_files.insert(format!("{root}/plan.rplan"),exported["document_json"].as_str().ok_or("Missing plan export.")?.to_string());
     state_files.insert(format!("{root}/context.rctx"),exported["context_json"].as_str().ok_or("Missing context export.")?.to_string());
     let replay_path=if input.retain_replay {let request=&result["engine_request"];if !request.is_object(){return Err("Missing accepted engine request for replay retention".into());}let path=format!("{root}/accepted_engine_run.json");state_files.insert(path.clone(),serde_json::to_string(&json!({"kind":"engine","request":request})).map_err(|e|e.to_string())?);Some(path)}else{None};
     let configured_replay_path=if let Some(recipe)=configured_replay{let path=format!("{root}/accepted_configured_run.json");state_files.insert(path.clone(),serde_json::to_string(&recipe).map_err(|e|e.to_string())?);Some(path)}else{None};
     if let Some(Ok(ref manifest))=manifest {state_files.insert(format!("{root}/manifest.json"),manifest["manifest_json"].as_str().ok_or("Missing native report manifest")?.to_string());}
     let capture_path=if input.capture_rounds {
      let capture=result["result"]["round_capture"].clone();
      if capture["schema_version"]!="bisect-native-round-capture-v1" {return Err("Missing accepted-attempt round capture.".into());}
      let path=format!("{root}/round_capture.json");
      state_files.insert(path.clone(),serde_json::to_string(&capture).map_err(|e|e.to_string())?);
      Some(path)
     } else {None};
     if files.len()+state_files.len()+years.len()+1>2000||files.values().chain(state_files.values()).map(String::len).sum::<usize>()>64*1024*1024{return Err("Generated label build package exceeds 2,000 files or 64 MiB before state checkpoint.".into());}
     outcomes.push(json!({"state":state,"state_name":name,"year":year,"status":"ok","chamber":job.chamber,"districts":job.districts,"metrics":result["result"]["metrics"],"build_report":result["build_report"],"plan_path":format!("{root}/plan.rplan"),"context_path":format!("{root}/context.rctx")}));
     if let Some(path)=replay_path {let row=outcomes.last_mut().ok_or("Missing state outcome")?;row["replay_run"]=json!({"sha256":format!("{:x}",Sha256::digest(state_files[&path].as_bytes())),"path":path,"accepted_attempt":result["build_report"]["accepted_attempt"],"scope":"Exact accepted engine request retained for separate explicit replay. It is not an authenticated record of native execution; replay does not automatically repeat earlier rejected population attempts."});}
     if let Some(path)=configured_replay_path {let row=outcomes.last_mut().ok_or("Missing state outcome")?;row["configured_replay_run"]=json!({"sha256":format!("{:x}",Sha256::digest(state_files[&path].as_bytes())),"path":path,"accepted_attempt":result["build_report"]["accepted_attempt"],"scope":"Original configured inputs and selected native retry attempt retained for explicit resolution, binding and replay. This does not authenticate historical execution or rerun prior rejected trials."});}
     let mut manifest_error=None;
     if let Some(manifest)=manifest {let row=outcomes.last_mut().ok_or("Missing state outcome")?;match manifest {
      Ok(manifest)=>{row["report_manifest"]=json!({"path":format!("{root}/manifest.json"),"sha256":manifest["manifest_sha256"],"plan_source_sha256":manifest["plan_source_sha256"],"context_source_sha256":manifest["context_source_sha256"],"population_balance_evaluated":manifest["population_balance_evaluated"],"scope":manifest["scope"]});},
      Err(error)=>{let error=format!("report manifest: {error}");row["status"]=json!("failed");row["generation_status"]=json!("completed");row["error"]=json!(error);manifest_error=Some(error);}
     }}
     if let Some(path)=capture_path {let row=outcomes.last_mut().ok_or("Missing state outcome.")?;row["round_capture"]=json!({"path":path,"status":result["result"]["round_capture"]["status"],"events":result["result"]["round_capture"]["events"].as_array().map_or(0,Vec::len),"terminal_matches_returned_assignment":result["result"]["round_capture"]["terminal_matches_returned_assignment"],"accepted_attempt":result["build_report"]["accepted_attempt"]});}
     results.push(bisect_runner::build_index::ResultRow{state_code:state,success:manifest_error.is_none(),error:manifest_error});
    },
    Err(failure)=>{let error=failure.native_label_error(&state);outcomes.push(json!({"state":state,"state_name":name,"year":year,"status":"failed","error":error,"failure_details":failure.details()}));results.push(bisect_runner::build_index::ResultRow{state_code:state,success:false,error:Some(error)});}
   }
   sequence+=1;
   crate::progress::emit(&json!({"schema_version":"bisect-label-build-progress-v1","request_sha256":request_hash,"sequence":sequence,"kind":"state-completed","label":input.label,"state":outcomes.last(),"files":state_files}))?;
   files.extend(state_files);
  }
  let index=bisect_runner::build_index::build_build_index(&input.label,year,&config_path,&format!("{:x}",Sha256::digest(input.yaml_text.as_bytes())),&format!("browser label build {}",input.label),&format!("runs/{}/{year}",input.label),&yaml,&configs,&results,input.created_at.clone(),env!("CARGO_PKG_VERSION").into())?;
  files.insert(format!("runs/{}/{year}/index.json",input.label),serde_json::to_string_pretty(&index).map_err(|e|e.to_string())?);
  if !entry.built.contains(year){entry.built.push(year.clone());}
  year_rows.push(json!({"year":year,"status":"built","summary":index.summary}));
  sequence+=1;
  crate::progress::emit(&json!({"schema_version":"bisect-label-build-progress-v1","request_sha256":request_hash,"sequence":sequence,"kind":"year-completed","label":input.label,"year":year_rows.last(),"index_path":format!("runs/{}/{year}/index.json",input.label),"index_json":files[&format!("runs/{}/{year}/index.json",input.label)],"registry_json":serde_json::to_string_pretty(&registry).map_err(|e|e.to_string())?}))?;
 }
 files.insert(".bisect".into(),serde_json::to_string_pretty(&registry).map_err(|e|e.to_string())?);
 if files.values().map(String::len).sum::<usize>()>64*1024*1024{return Err("Generated label build package exceeds 64 MiB.".into());}
 let hashes:Vec<_>=files.iter().map(|(path,text)|json!({"path":path,"sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"bytes":text.len()})).collect();
 Ok(json!({"schema_version":"bisect-label-build-v1","status":if outcomes.iter().any(|r|r["status"]=="failed"){"partial-failure"}else if outcomes.is_empty(){"skipped"}else{"built"},"label":input.label,"years":year_rows,"states":outcomes,"invalidated_downstream":invalidated,"files":files,"output_hashes":hashes,"scope":"Runs selected prepared state/year graphs serially in WASM with native 51-attempt population retries, accepted-seed RPLAN/RCTX exports and shared native build indexes. Per-state failures are retained and a completed year is recorded built even if some states failed, matching native build registry behavior. Force cascades recorded analysis/report stages only for already-built years and records stale downstream paths; no local files are deleted. Download contains newly generated files, not unselected old repository files. Optional native report manifests bind exact generated plan/context bytes and retain research classification. Fetching, parallel workers, analyses, authenticated native manifest/provenance promotion and existing filesystem merge remain unfinished. Host progress callbacks expose completed state outputs and completed year snapshots before the final result. A host may retain received checkpoints when terminating its worker; unfinished years are not newly marked built. Automatic resume is not implemented. This is heuristic execution, not optimality, source-authenticity or legal certification."}))
}


#[cfg(test)]
mod preflight_tests {
 use super::*;
 fn input(force:bool,registry:Option<&str>)->Input {
  serde_json::from_value(json!({"label":"fixture","yaml_text":"name: fixture\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  seed: 42\nyears: [\"2020\", \"2010\"]\n","source_name":"fixture.yml","profile":"open","years_override":null,"registry_json":registry,"force":force,"created_at":"recorded","jobs":[{"graph":{"schema_version":1,"state":"RI","year":"2020","geoids":["deliberately-not-an-engine-ready-graph"],"population":[100],"adjacency":[[]],"edges":[],"areas":[1.0],"exterior_perimeters":[1.0],"centroids":[[0.0,0.0]]},"districts":1,"chamber":"congressional","demographics":null},{"graph":{"schema_version":1,"state":"RI","year":"2010","geoids":["also-not-engine-ready"],"population":[100],"adjacency":[[]],"edges":[],"areas":[1.0],"exterior_perimeters":[1.0],"centroids":[[0.0,0.0]]},"districts":1,"chamber":"congressional","demographics":null}]})).unwrap()
 }
 #[test] fn planning_does_not_partition_or_promote_registry(){let p=plan(input(false,None),false).unwrap();assert_eq!(p["execution_performed"],false);assert_eq!(p["filesystem_performed"],false);assert_eq!(p["years"][0]["jobs"][0]["state_name"],"rhode_island");assert_eq!(p["years"][0]["registry_marked_built_before_execution"],false);assert_eq!(p["years"][1]["year"],"2010");}
 #[test] fn force_invalidation_keeps_built_and_other_labels(){let p=plan(input(true,Some(r#"{"fixture":{"built":["2020"],"analyzed":["2020","2010"],"reported":["2020"]},"other":{"built":["2020"],"analyzed":[],"reported":[]}}"#)),false).unwrap();let r:Value=serde_json::from_str(p["years"][0]["invalidation_registry_json"].as_str().unwrap()).unwrap();assert_eq!(r["fixture"]["built"],json!(["2020"]));assert_eq!(r["fixture"]["analyzed"],json!(["2010"]));assert_eq!(r["fixture"]["reported"],json!([]));assert!(r.get("other").is_some());assert_eq!(p["years"][1]["invalidate"],false);}
 #[test] fn dry_force_never_invalidates_or_requires_jobs(){let mut i=input(true,Some(r#"{"fixture":{"built":["2020"],"analyzed":["2020"],"reported":["2020"]}}"#));i.jobs.clear();i.created_at.clear();let p=plan(i,true).unwrap();for y in p["years"].as_array().unwrap(){assert_eq!(y["status"],"dry-run");assert_eq!(y["remove_paths"],json!([]));assert_eq!(y["invalidation_registry_json"],Value::Null);}}
 #[test] fn skip_precedes_dry_run_and_has_no_removals(){let p=plan(input(false,Some(r#"{"fixture":{"built":["2020"],"analyzed":["2020"],"reported":["2020"]}}"#)),true).unwrap();assert_eq!(p["years"][0]["status"],"skipped-already-built");assert_eq!(p["years"][1]["status"],"dry-run");}
 #[test] fn invalid_config_precedes_invalid_registry(){let mut i=input(true,Some("not JSON"));i.yaml_text="algorithm: [".into();let error=plan(i,false).unwrap_err();assert!(!error.contains("registry"),"{error}");}
}
