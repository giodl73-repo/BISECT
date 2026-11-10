use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub labels:Vec<String>,
 pub plans:BTreeMap<String,BTreeMap<String,String>>,
}
pub(crate) fn run(input:Input)->Result<Value,String>{
 if !(2..=60).contains(&input.labels.len())||input.plans.len()>60{return Err("Select 2–60 plan labels.".into());}
 let safe=|s:&str| !s.is_empty()&&s.len()<=100&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-');
 if input.labels.iter().any(|s|!safe(s))||input.plans.keys().any(|s|!safe(s)){return Err("Use bounded portable plan labels.".into());}
 let mut total=0usize;let mut hashes=BTreeMap::new();
 for (label,files) in &input.plans{for(name,text)in files{
  if !["summary.json","compactness.json","splits.json","contiguity.json","manifest.json"].contains(&name.as_str())||text.len()>3*1024*1024{return Err("Select only native summary sources of at most 3 MiB each.".into());}
  total=total.saturating_add(text.len());if total>8*1024*1024{return Err("Summary sources exceed 8 MiB.".into());}
  hashes.insert(format!("{label}/{name}"),format!("{:x}",Sha256::digest(text.as_bytes())));
 }}
 let mut rows=Vec::new();let mut availability=Vec::new();
 for label in &input.labels{
  let files=input.plans.get(label);
  let parse=|name:&str| files.and_then(|f|f.get(name)).and_then(|text|crate::national::parse(text).ok());
  let summary=parse("summary.json");let compactness=parse("compactness.json");let splits=parse("splits.json");let contiguity=parse("contiguity.json");
  let manifest=files.and_then(|f|f.get("manifest.json")).and_then(|text|serde_json::from_str::<bisect_report::manifest_schema::ReadOnlyPlanManifest>(text).ok());
  availability.push(json!({"label":label,"summary":summary.is_some(),"compactness":compactness.is_some(),"splits":splits.is_some(),"contiguity":contiguity.is_some(),"manifest":manifest.is_some()}));
  rows.push(bisect_analysis::multi_plan_summary::project(label,summary.as_ref(),compactness.as_ref(),splits.as_ref(),contiguity.as_ref(),manifest.map(|m|m.balance_tolerance_pct)));
 }
 let table=bisect_analysis::multi_plan_summary::render(&rows,false);let csv=bisect_analysis::multi_plan_summary::render(&rows,true);
 Ok(json!({"schema_version":"bisect-multi-plan-summary-v1","table":table,"csv":csv,"availability":availability,"source_sha256":hashes,
  "scope":"Native multi-label supplied-analysis summary in requested label order. Missing or invalid files retain native N/A placeholders; empty compactness rows retain native NaN formatting. Supplied manifests provide balance tolerance. This operation does not recompute analyses, bind them to assignments, authenticate sources or certify plans. Browser folders replace native path discovery; native output-dir's separate manifest lookup must be supplied explicitly."}))
}
