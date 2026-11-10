//! Run shared native plan analyzers together without fabricating missing evidence.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Geometry {
 pub unit_geometry_json:String,pub coordinate_basis:bisect_analysis::district_geometry::CoordinateBasis,pub crs_label:String,
 #[serde(default)] pub include_exact_reock:bool,
 #[serde(default)] pub population_centroids_csv:Option<String>,
 #[serde(default)] pub dissolve_method:bisect_analysis::district_geometry::DissolveMethod,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub document_json:String,pub context_json:Option<String>,pub geography_json:Option<String>,
 pub demographic_csv:Option<String>,pub urban_csv:Option<String>,pub election_csv:Option<String>,
 pub tolerance_percent:f64,pub election_settings:crate::election_analysis::Settings,pub compactness:Option<Geometry>,
}
fn sha(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
fn append(name:&str,result:Option<Result<Value,String>>,files:&mut BTreeMap<String,String>,steps:&mut Vec<Value>)->Result<(),String>{
 match result {
  None=>steps.push(json!({"name":name,"status":"unavailable","reason":"Required optional source was not supplied."})),
  Some(Err(error))=>steps.push(json!({"name":name,"status":"failed","error":error})),
  Some(Ok(out))=>{
   if let Some(map)=out["files"].as_object(){for (n,text) in map {let text=text.as_str().ok_or("Native analyzer returned non-text file.")?;if let Some(old)=files.insert(n.clone(),text.into()){if old!=text{return Err(format!("Native analysis file conflict: {n}"));}}}}
   else if let Some(text)=out["manifest_json"].as_str(){files.insert("manifest.json".into(),text.into());}
   else if let Some(text)=out["report_json"].as_str(){files.insert(format!("{name}.json"),text.into());}
   let mut step=json!({"name":name,"status":"computed","scope":out["scope"]});
   for field in ["coverage","population_balance_evaluated","population_source","plan_source_sha256","context_source_sha256","geography_source_sha256","source_sha256","election_source_sha256"]{if let Some(value)=out.get(field){step[field]=value.clone();}}
   if name=="election" {
    if let (Some(bias),Some(curve))=(out["partisan"]["partisan_bias"]["value"].as_f64(),out["partisan"]["seats_votes"]["bias"].as_f64()) {
     if (bias-curve).abs()>=0.01 {step["warnings"]=json!([{"code":"native-partisan-bias-disagreement","partisan_bias":bias,"seats_votes_bias":curve,"absolute_difference":(bias-curve).abs(),"message":"Native partisan-bias and seats-votes-bias values disagree by at least 0.01. The native consistency check flags possible numerical inconsistency; inspect the metric conventions before interpreting these values."}]);}
    }
   }
   steps.push(step);
  }
 }Ok(())
}
pub(crate) fn run(input:Input)->Result<Value,String>{
 run_selected(input,&ANALYSIS_TYPES.map(String::from))
}
pub(crate) const ANALYSIS_TYPES:[&str;6]=["manifest","contiguity","splits","statistics","compactness","election"];
pub(crate) fn selected_types(types:Option<Vec<String>>)->Result<Vec<String>,String>{
 let Some(types)=types else{return Ok(ANALYSIS_TYPES.map(String::from).to_vec());};
 if types.len()==1&&types[0]=="all"{return Ok(ANALYSIS_TYPES.map(String::from).to_vec());}
 if types.is_empty()||types.len()>ANALYSIS_TYPES.len()||types.iter().any(|t|!ANALYSIS_TYPES.contains(&t.as_str()))||types.iter().collect::<std::collections::HashSet<_>>().len()!=types.len(){return Err("Choose unique implemented analysis types: manifest, contiguity, splits, statistics, compactness, election, or all alone.".into());}
 Ok(ANALYSIS_TYPES.into_iter().filter(|t|types.iter().any(|v|v==t)).map(String::from).collect())
}
pub(crate) fn run_selected(input:Input,types:&[String])->Result<Value,String>{
 run_selected_with_manifest(input,types,None)
}
/// Only the containing native workflow supplies this already-generated manifest.
pub(crate) fn run_with_manifest(input:Input,manifest:&str)->Result<Value,String>{
 run_selected_with_manifest(input,&ANALYSIS_TYPES.map(String::from),Some(manifest))
}
fn run_selected_with_manifest(input:Input,types:&[String],manifest:Option<&str>)->Result<Value,String>{
 let selected=|name:&str|types.iter().any(|t|t==name);
 let plan=crate::plan_audit_text::plan(&input.document_json)?;rplan_audit::measurements::measure_plan(&plan,None)?;
 if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Analysis package plan unit-universe hash mismatch.".into());}
 let mut sources=BTreeMap::from([("plan.rplan",input.document_json.as_str())]);
 for (name,text) in [("context.rctx",input.context_json.as_deref()),("geography.json",input.geography_json.as_deref()),("demographic.csv",input.demographic_csv.as_deref()),("urban.csv",input.urban_csv.as_deref()),("election.csv",input.election_csv.as_deref()),("unit-geometry.json",input.compactness.as_ref().map(|g|g.unit_geometry_json.as_str())),("population-centroids.csv",input.compactness.as_ref().and_then(|g|g.population_centroids_csv.as_deref()))]{if let Some(text)=text{sources.insert(name,text);}}
 let limit=input.compactness.as_ref().map(|g|bisect_analysis::district_geometry::unit_geometry_byte_limit(&g.unit_geometry_json,25*1024*1024)).transpose()?.unwrap_or(25*1024*1024);
 if sources.values().map(|s|s.len()).sum::<usize>()>limit{return Err("Analysis package input sources exceed their 25/64 MiB profile.".into());}
 let source_sha256:BTreeMap<_,_>=sources.iter().map(|(n,s)|(*n,sha(s))).collect();
 let mut files=BTreeMap::from([("plan.rplan".into(),input.document_json.clone())]);let mut steps=Vec::new();
 if let Some(ctx)=&input.context_json{files.insert("context.rctx".into(),ctx.clone());}
 if selected("manifest"){let result=if let Some(text)=manifest{Some(Ok(json!({"manifest_json":text,"scope":"Manifest generated by the containing native State workflow from the same accepted plan/context and selected source."})))}else{input.context_json.as_ref().map(|ctx|crate::plan_audit_text::report_manifest(&input.document_json,ctx))};append("manifest",result,&mut files,&mut steps)?;}
 if selected("contiguity"){append("contiguity",input.context_json.as_ref().map(|ctx|crate::plan_audit_text::contiguity(&input.document_json,ctx)),&mut files,&mut steps)?;}
 if selected("splits"){append("splits",Some(crate::plan_splits::analyze(&input.document_json,input.geography_json.as_deref(),input.context_json.as_deref())),&mut files,&mut steps)?;}
 if selected("statistics"){
 let statistics=if input.demographic_csv.is_some()||input.urban_csv.is_some()||input.election_csv.is_some(){Some(crate::plan_statistics::analyze_with_context(&input.document_json,input.demographic_csv.as_deref(),input.urban_csv.as_deref(),input.election_csv.as_deref(),input.tolerance_percent,input.context_json.as_deref()))}else{match crate::plan_audit_text::context(input.context_json.as_deref()){Ok(Some(ctx)) if ctx.populations.is_some()=>Some(crate::plan_statistics::analyze_with_context(&input.document_json,None,None,None,input.tolerance_percent,input.context_json.as_deref())),Ok(_)=>None,Err(error)=>Some(Err(error))}};
 append("statistics",statistics,&mut files,&mut steps)?;
 }
 if selected("compactness"){append("compactness",input.compactness.map(|g|crate::plan_compactness::analyze(&input.document_json,&g.unit_geometry_json,g.coordinate_basis,&g.crs_label,g.include_exact_reock,g.population_centroids_csv.as_deref(),g.dissolve_method)),&mut files,&mut steps)?;}
 if selected("election"){append("election",input.election_csv.as_ref().map(|csv|crate::election_analysis::analyze(&input.document_json,csv,Some(input.election_settings))),&mut files,&mut steps)?;}
 let failed=steps.iter().any(|s|s["status"]=="failed");let partial=steps.iter().any(|s|s["status"]!="computed");
 let report_ready=["manifest.json","summary.json","contiguity.json","compactness.json"].iter().all(|name|files.contains_key(*name));
 let status=if failed{"partial-failure"}else if partial{"partial"}else{"computed"};
 let record=json!({"schema_version":"bisect-analysis-package-record-v1","status":status,"report_ready":report_ready,"steps":steps,"source_sha256":source_sha256});
 files.insert("analysis-package.json".into(),serde_json::to_string_pretty(&record).map_err(|e|e.to_string())?);
 if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Generated analysis package exceeds 25 MiB.".into());}
 let file_sha256:BTreeMap<_,_>=files.iter().map(|(n,s)|(n.clone(),sha(s))).collect();
 Ok(json!({"schema_version":"bisect-plan-analysis-package-v1","status":status,"report_ready":report_ready,"steps":steps,"files":files,"file_sha256":file_sha256,"plan_source_sha256":sha(&input.document_json),"source_sha256":source_sha256,"scope":"Computes native manifest, contiguity, splits, statistics, compactness and election analysis from the selected exact plan and supplied optional inputs. Without demographic CSV, matching RCTX counts can provide native population summary values while demographic shares remain unknown. Missing optional sources remain unavailable; failed analyzers retain their error and other successful output. Report-ready means required report files exist, not certification, source authentication or every legal check. Each step retains its native scope and source hashes. No label registry stage is promoted and native label wrapper placeholders are not treated as computed analysis. Save the browser project to retain original CSV/geometry sources; the ZIP contains native outputs and exact plan/context, not every raw source."}))
}
