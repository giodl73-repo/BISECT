//! In-memory follow-up stages on the exact accepted configured Run.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::{engine::general_purpose::STANDARD,Engine};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Analysis {
 pub geography_json:Option<String>,pub demographic_csv:Option<String>,pub urban_csv:Option<String>,pub election_csv:Option<String>,
 #[serde(default)]pub election_settings:crate::election_analysis::Settings,
 pub compactness:Option<crate::analysis_package::Geometry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Map {
 pub unit_geometry_json:String,pub dpi:u32,pub map_types:Vec<String>,
 #[serde(default)]pub dissolve_method:bisect_analysis::district_geometry::DissolveMethod,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {pub generated_at:String,pub delivery:Option<bisect_report::report_delivery::ReportDeliverySettings>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Options {pub analysis:Option<Analysis>,pub map:Option<Map>,pub report:Option<Report>}
impl Options {
 pub(crate) fn validate(&self)->Result<(),String>{
  if self.analysis.is_none()&&self.map.is_none()&&self.report.is_none(){return Err("Select a follow-up workflow stage.".into());}
  if let Some(report)=&self.report {
   if self.analysis.is_none(){return Err("Workflow report requires the analysis stage.".into());}
   if report.generated_at.is_empty()||report.generated_at.len()>100||report.generated_at.chars().any(char::is_control){return Err("Record a bounded report timestamp.".into());}
   if let Some(settings)=&report.delivery {settings.validate().map_err(|e|e.to_string())?;}
  }
  let mut bytes=0usize;let mut large_geometry=false;
  if let Some(a)=&self.analysis {
   for s in [&a.geography_json,&a.demographic_csv,&a.urban_csv,&a.election_csv].into_iter().flatten(){bytes=bytes.checked_add(s.len()).ok_or("Workflow size overflow")?;}
   if let Some(g)=&a.compactness {large_geometry|=bisect_analysis::district_geometry::unit_geometry_byte_limit(&g.unit_geometry_json,18*1024*1024)?==64*1024*1024;bytes=bytes.checked_add(g.unit_geometry_json.len()+g.crs_label.len()+g.population_centroids_csv.as_ref().map_or(0,String::len)).ok_or("Workflow size overflow")?;}
   if a.election_settings.bootstrap_seed.parse::<u64>().ok().map(|s|s.to_string()).as_deref()!=Some(a.election_settings.bootstrap_seed.as_str())||!(1..=5000).contains(&a.election_settings.bootstrap_replicates){return Err("Invalid workflow election bootstrap settings.".into());}
  }
  if let Some(m)=&self.map {
   let mut seen=BTreeSet::new();
   if ![72,100,150,200,300].contains(&m.dpi)||m.map_types.is_empty()||m.map_types.len()>4||m.map_types.iter().any(|t|!["districts","political","demographic","compactness"].contains(&t.as_str())||!seen.insert(t)){return Err("Select native DPI and distinct supported workflow maps.".into());}
   let limit=bisect_analysis::district_geometry::unit_geometry_byte_limit(&m.unit_geometry_json,16*1024*1024)?;large_geometry|=limit==64*1024*1024;
   if m.unit_geometry_json.len()>limit{return Err("Workflow map geometry exceeds its 16/64 MiB source profile.".into());}
   bytes=bytes.checked_add(m.unit_geometry_json.len()).ok_or("Workflow size overflow")?;
  }
  if bytes>(if large_geometry{128}else{40})*1024*1024{return Err("Workflow follow-up sources exceed their 40/128 MiB profile.".into());}Ok(())
 }
}
fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn add(files:&mut BTreeMap<String,Vec<u8>>,name:String,bytes:Vec<u8>)->Result<(),String>{
 if let Some(old)=files.get(&name){if old!=&bytes{return Err(format!("Workflow output conflict: {name}"));}return Ok(());}
 if files.len()>=100||files.values().map(Vec::len).sum::<usize>().saturating_add(bytes.len())>64*1024*1024{return Err("Workflow generated files exceed 64 MiB/100 files.".into());}
 files.insert(name,bytes);Ok(())
}
pub(crate) fn run(run:&Value,options:Options)->Result<Value,String>{
 options.validate()?;
 let plan=run["practitioner_export"]["document_json"].as_str().ok_or("Missing accepted plan")?;
 let context=run["practitioner_export"]["context_json"].as_str().ok_or("Missing accepted context")?;
 let tolerance=run["configured_run"]["configuration"]["execution_tolerance_percent"].as_f64().ok_or("Missing accepted tolerance")?;
 let manifest=run["report_manifest"]["manifest_json"].as_str().ok_or("Missing native manifest")?;
 let yaml=run["effective_yaml"].as_str().ok_or("Missing effective YAML")?;
 run_sources(plan,context,tolerance,manifest,Some(yaml),options)
}
/// Supplied accepted exports are parsed and measured by the native boundary.
/// This operation performs no partitioning and does not authenticate their origin.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NamedManifestSource {pub arguments:Vec<String>,pub adjacency_source_name:String,pub adjacency_sha256:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AcceptedInput {pub document_json:String,pub context_json:String,pub options:Options,#[serde(default)]pub native_named_source:Option<NamedManifestSource>}
pub(crate) fn run_accepted(input:AcceptedInput)->Result<Value,String>{
 input.options.validate()?;
 if input.document_json.len()>3*1024*1024||input.context_json.len()>8*1024*1024{return Err("Accepted workflow plan/context exceed 3/8 MiB.".into());}
 let manifest=if let Some(source)=input.native_named_source {crate::plan_audit_text::named_state_manifest(&input.document_json,&input.context_json,source.arguments,&source.adjacency_source_name,&source.adjacency_sha256)?}else{crate::plan_audit_text::report_manifest(&input.document_json,&input.context_json)?};
 let document=rplan_io::read_rplan_str(&input.document_json).map_err(|e|e.to_string())?;
 let producer=&document.provenance.producer;
 let options=producer.get("engine_options").or_else(||producer.get("multiscale_input").and_then(|i|i.get("request")).and_then(|r|r.get("options")));
 let tolerance=options.and_then(|o|o.get("balance_tolerance")).and_then(Value::as_f64).filter(|t|t.is_finite()&&(0.0..=100.0).contains(t)).ok_or("Accepted workflow requires a recorded native engine population tolerance. Use the standalone analysis tools to choose a threshold for other plans.")?;
 let mut workflow=run_sources(&input.document_json,&input.context_json,tolerance,manifest["manifest_json"].as_str().ok_or("Missing native manifest")?,None,input.options)?;
 workflow["scope"]=json!("Native optional analysis, maps and reports on supplied exact RPLAN/RCTX exports with recorded engine tolerance. Ordered units and population sums are checked before stages. No partitioning or filesystem installation occurs. Stage failures retain completed files; absent evidence stays unknown. Hashes establish byte correspondence, not source authenticity, execution history, legal certification or optimality.");
 Ok(json!({"schema_version":"bisect-accepted-state-workflow-v1","partition_execution_performed":false,"workflow":workflow}))
}
fn run_sources(plan:&str,context:&str,tolerance:f64,manifest:&str,yaml:Option<&str>,options:Options)->Result<Value,String>{
 let mut files=BTreeMap::new();let mut stages=Vec::new();let mut analysis=None;
 for (name,text) in [("plan.rplan",plan),("context.rctx",context),("manifest.json",manifest)]{add(&mut files,name.into(),text.as_bytes().to_vec())?;}
 if let Some(yaml)=yaml{add(&mut files,"effective-config.yml".into(),yaml.as_bytes().to_vec())?;}
 if let Some(a)=options.analysis {
  crate::progress::emit(&json!({"schema_version":"bisect-state-workflow-progress-v1","stage":"analysis","message":"Analyzing the accepted native plan and context"}))?;
  let input=crate::analysis_package::Input {document_json:plan.into(),context_json:Some(context.into()),geography_json:a.geography_json,demographic_csv:a.demographic_csv,urban_csv:a.urban_csv,election_csv:a.election_csv,tolerance_percent:tolerance,election_settings:a.election_settings,compactness:a.compactness};
  match crate::analysis_package::run_with_manifest(input,manifest) {
   Err(error)=>stages.push(json!({"name":"analysis","status":"failed","error":error})),
   Ok(package)=>{
    // Commit each stage's files together; a bound failure preserves prior stages.
    let commit=(||{let mut staged=files.clone();for (name,text) in package["files"].as_object().ok_or("Missing analysis files")?{let path=if ["plan.rplan","context.rctx","manifest.json","analysis-package.json"].contains(&name.as_str()){name.clone()}else{format!("analysis/{name}")};add(&mut staged,path,text.as_str().ok_or("Invalid analysis file")?.as_bytes().to_vec())?;}Ok::<_,String>(staged)})();
    match commit {Err(error)=>stages.push(json!({"name":"analysis","status":"failed","error":error})),Ok(staged)=>{files=staged;stages.push(json!({"name":"analysis","status":if package["status"]=="computed"{"completed"}else{"partial"},"native_status":package["status"],"report_ready":package["report_ready"],"steps":package["steps"],"source_sha256":package["source_sha256"]}));analysis=Some(package);}}
   }
  }
 }
 if let Some(m)=options.map {
  for kind in m.map_types {
   let name=format!("map:{kind}");
   let source=if kind=="districts"{None}else{analysis.as_ref().and_then(|a|a["files"][format!("{kind}.json")].as_str()).map(str::to_string)};
   if kind!="districts"&&source.is_none(){stages.push(json!({"name":name,"status":"unavailable","reason":"The requested map analysis was not computed."}));continue;}
   crate::progress::emit(&json!({"schema_version":"bisect-state-workflow-progress-v1","stage":"map","map_type":kind,"message":"Rendering the accepted native plan"}))?;
   let output=crate::toolkit::execute(json!({"operation":"render-native-district-map","input":{"document_json":plan,"unit_geometry_json":m.unit_geometry_json,"dpi":m.dpi,"map_type":kind,"analysis_json":source,"dissolve_method":m.dissolve_method}}));
   match output {
    Err(error)=>stages.push(json!({"name":name,"status":"failed","error":error})),
    Ok(mut result)=>{
     let commit=(||{let mut staged=files.clone();let svg=result["svg_text"].as_str().ok_or("Missing map SVG")?;let png=STANDARD.decode(result["png_base64"].as_str().ok_or("Missing map PNG")?).map_err(|e|e.to_string())?;add(&mut staged,format!("maps/{kind}.svg"),svg.as_bytes().to_vec())?;add(&mut staged,format!("maps/{kind}.png"),png)?;Ok::<_,String>(staged)})();
     match commit{Err(error)=>stages.push(json!({"name":name,"status":"failed","error":error})),Ok(staged)=>{files=staged;result.as_object_mut().unwrap().remove("svg_text");result.as_object_mut().unwrap().remove("png_base64");stages.push(json!({"name":name,"status":"completed","receipt":result}));}}
    }
   }
  }
 }
 if let Some(report)=options.report {
  if analysis.as_ref().is_none_or(|a|a["report_ready"]!=true){stages.push(json!({"name":"report","status":"unavailable","reason":"Required analysis report files were not computed."}));}
  else{
   crate::progress::emit(&json!({"schema_version":"bisect-state-workflow-progress-v1","stage":"report","message":"Assembling the native commission report"}))?;
   let mut allowed:BTreeSet<String>=["plan.rplan","context.rctx","manifest.json","maps/districts.png","maps/district_map.png","maps/political.png","maps/compactness.png"].into_iter().map(str::to_string).collect();
   for name in bisect_report::report_assembly::REQUIRED_ANALYSIS_FILES.iter().chain(bisect_report::report_assembly::OPTIONAL_ANALYSIS_FILES){allowed.insert(format!("analysis/{name}"));}
   let selected: BTreeMap<_,_>=files.iter().filter(|(p,_)|allowed.contains(*p)).map(|(p,b)|(p.clone(),b.clone())).collect();
   match crate::toolkit::execute(json!({"operation":"assemble-commission-report","files":selected,"generated_at":report.generated_at,"delivery":report.delivery})){
    Err(error)=>stages.push(json!({"name":"report","status":"failed","error":error})),
    Ok(result)=>{let commit=(||{let mut staged=files.clone();if let Some(delivery)=result["report_files"].as_object(){for (path,text) in delivery{add(&mut staged,format!("report/{path}"),text.as_str().ok_or("Invalid native report delivery file")?.as_bytes().to_vec())?;}}else{for (field,path) in [("report_json","report/report.json"),("report_html","report/report.html"),("audit_json","report/audit.json")]{if let Some(text)=result[field].as_str(){add(&mut staged,path.into(),text.as_bytes().to_vec())?;}}}Ok::<_,String>(staged)})();match commit{Err(error)=>stages.push(json!({"name":"report","status":"failed","error":error})),Ok(staged)=>{files=staged;stages.push(json!({"name":"report","status":"completed","source_sha256":result["source_sha256"],"manifest_resolution":result["manifest_resolution"]}));}}}
   }
  }
 }
 let status=if stages.iter().all(|s|s["status"]=="completed"){"completed"}else{"partial"};
 let hashes:BTreeMap<_,_>=files.iter().map(|(n,b)|(n.clone(),sha(b))).collect();let encoded:BTreeMap<_,_>=files.iter().map(|(n,b)|(n.clone(),STANDARD.encode(b))).collect();
 Ok(json!({"schema_version":"bisect-state-workflow-v1","status":status,"stages":stages,"files_base64":encoded,"file_sha256":hashes,"accepted_plan_source_sha256":sha(plan.as_bytes()),"accepted_context_source_sha256":sha(context.as_bytes()),"scope":"Selected optional in-memory follow-up stages use the exact accepted configured Run, native analysis kernels, native SVG/PNG renderer and commission report assembly. Each stage records computed, partial, failed or unavailable outcomes; completed files commit together, and follow-up failures retain the accepted Run and earlier files. Optional missing evidence stays unknown. Display simplification affects compactness and may fail topology checks. Reports are measurements and supplied-source records, not certification, authentication or optimality. No filesystem stages, external scripts, source fetching, completion markers or full native State/States orchestration are performed."}))
}
