//! In-memory label report orchestration over exact native analysis files.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
use bisect_report::label_indexes::{AnalysisIndex,ReportIndex,serialize_analysis_index,serialize_report_index};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub analysis_index_json:String,
    pub registry_json:Option<String>,
    pub files:BTreeMap<String,String>,
    pub file_sha256:BTreeMap<String,String>,
    pub generated_at:String,
    pub format:String,
    pub audit_with_report:bool,
}
fn sha(text:&str)->String {format!("{:x}",Sha256::digest(text.as_bytes()))}
pub(crate) fn run(input:Input)->Result<Value,String>{
    if input.analysis_index_json.len()>1024*1024||input.files.len()>842||input.files.values().map(String::len).sum::<usize>()>25*1024*1024 {
        return Err("Select an analysis index up to 1 MiB and native files totaling at most 25 MiB.".into());
    }
    let index:AnalysisIndex=serde_json::from_str(&input.analysis_index_json).map_err(|e|e.to_string())?;
    serialize_analysis_index(&index)?;
    let registry=crate::label_stage::promote(input.registry_json.as_deref(),&index.label,&index.year,true)?;
    let root=format!("analysis/{}/{}/",index.label,index.year);
    if input.files.get(&(root.clone()+"index.json"))!=Some(&input.analysis_index_json)||input.file_sha256.len()!=input.files.len(){return Err("Supply the exact analysis index and one SHA-256 for each file.".into());}
    let allowed=["plan.rplan","context.rctx","manifest.json","analysis-package.json","summary.json","demographic.json","urban.json","political.json","proportionality.json","partisan.json","partisan-metrics.json","contiguity.json","compactness.json","splits.json"];
    for (path,text) in &input.files {
        if path==".bisect" {if let Some(source)=&input.registry_json{if source!=text{return Err("Selected registry differs from the analysis package registry.".into());}}}
        else if path!=&(root.clone()+"index.json") {
            let suffix=path.strip_prefix(&root).ok_or("Analysis file is outside the selected label/year.")?;
            let parts:Vec<_>=suffix.split('/').collect();
            if parts.len()!=2||!index.states.contains_key(parts[0])||!allowed.contains(&parts[1]) {return Err("Unknown or unsafe state analysis path.".into());}
        }
        if input.file_sha256.get(path)!=Some(&sha(text)){return Err(format!("Analysis source hash mismatch: {path}"));}
    }
    let mut report_index=ReportIndex{label:index.label.clone(),year:index.year.clone(),analysis_index_sha256:sha(&input.analysis_index_json),format:input.format.clone(),created:input.generated_at.clone(),files:Vec::new()};
    serialize_report_index(&report_index)?;
    let mut files=BTreeMap::new();let mut outcomes=Vec::new();let mut bytes=0usize;
    let output_root=format!("reports/{}/{}/",index.label,index.year);
    // Native state names are sorted explicitly because AnalysisIndex retains HashMap compatibility.
    let states:BTreeMap<_,_>=index.states.iter().collect();
    for (state,status) in states {
        if status.status=="skipped" {outcomes.push(json!({"state_name":state,"status":"skipped"}));continue;}
        let source_root=format!("{root}{state}/");
        let required=["manifest.json","summary.json","contiguity.json","compactness.json"];
        let missing:Vec<_>=required.iter().filter(|n|!input.files.contains_key(&(source_root.clone()+*n))).copied().collect();
        if !missing.is_empty(){outcomes.push(json!({"state_name":state,"status":"unavailable","missing":missing}));continue;}
        let mut selected=BTreeMap::new();
        for name in ["manifest.json","plan.rplan","context.rctx","summary.json","contiguity.json","compactness.json","partisan.json","splits.json"] {
            if let Some(text)=input.files.get(&(source_root.clone()+name)) {
                let path=if ["summary.json","contiguity.json","compactness.json","partisan.json","splits.json"].contains(&name){format!("analysis/{name}")}else{name.into()};
                selected.insert(path,text.as_bytes().to_vec());
            }
        }
        let result=(||->Result<BTreeMap<String,String>,String>{
            let manifest=bisect_report::commission_manifest::resolve_report_manifest(selected.get("manifest.json").map(Vec::as_slice),None).map_err(|e|e.to_string())?;
            let policy:Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
            let locations=bisect_analysis::location_registry::PolicyRegistry::new(&policy);
            let name=locations.state_name(&manifest.manifest.state_code).ok_or("Report manifest has an unknown state.")?.to_lowercase().replace(' ',"_");
            if &name!=state||manifest.manifest.year.to_string()!=index.year {return Err("Report manifest state/year does not match its analysis index entry.".into());}
            if let Some(raw)=selected.get("plan.rplan") {
                let plan=crate::plan_audit_text::plan(std::str::from_utf8(raw).map_err(|e|e.to_string())?)?;
                if plan.units.state.as_deref()!=Some(manifest.manifest.state_code.as_str())||plan.units.year.map(|y|y.to_string())!=Some(index.year.clone())||usize::try_from(manifest.manifest.num_districts).ok()!=Some(plan.k) {
                    return Err("Selected report plan state/year/district count does not match the manifest.".into());
                }
            }
            // Reuse the existing operation so report, HTML and audit delivery stay identical.
            let report=crate::toolkit::execute(json!({"operation":"assemble-commission-report","files":selected,"generated_at":input.generated_at,"fallback":null,"delivery":{"label":index.label,"formats":[input.format],"audit_with_report":input.audit_with_report}}))?;
            serde_json::from_value(report["report_files"].clone()).map_err(|e|e.to_string())
        })();
        match result {
            Err(error)=>outcomes.push(json!({"state_name":state,"status":"failed","error":error})),
            Ok(generated)=>{
                for (name,text) in generated {
                    bytes+=text.len();if bytes>25*1024*1024{return Err("Batch reports exceed 25 MiB; use a smaller batch.".into());}
                    let relative=format!("{state}/{name}");report_index.files.push(relative.clone());files.insert(output_root.clone()+&relative,text);
                }
                outcomes.push(json!({"state_name":state,"status":"generated"}));
            }
        }
    }
    let complete=outcomes.iter().all(|s|s["status"]=="generated"||s["status"]=="skipped")&&outcomes.iter().any(|s|s["status"]=="generated");
    let record=json!({"schema_version":"bisect-label-report-record-v1","status":if complete{"generated"}else{"partial"},"states":outcomes});
    let record_text=serde_json::to_string_pretty(&record).map_err(|e|e.to_string())?;
    report_index.files.push("report-batch.json".into());files.insert(output_root.clone()+"report-batch.json",record_text);
    let index_json=serialize_report_index(&report_index)?;let index_path=output_root+"index.json";
    files.insert(index_path.clone(),index_json.clone());
    if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Batch reports exceed 25 MiB.".into());}
    if let Some(text)=&registry{files.insert(".bisect".into(),text.clone());}
    if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Reports and registry exceed 25 MiB.".into());}
    let hashes:BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),sha(t))).collect();
    let mut output=json!({"schema_version":"bisect-label-report-v1","label":index.label,"year":index.year,"status":record["status"],"states":outcomes,"files":files,"file_sha256":hashes,"index_path":index_path,"index_json":index_json,"index_sha256":sha(&index_json),"registry_promoted":registry.is_some(),"scope":"Assembles shared native commission reports from exact hashed analysis files. The native report index binds the raw analysis-index bytes. Missing required sources remain unavailable, errors retain successful state reports, and skipped analysis states stay skipped. Manifest state/year must match the indexed state. This validates file integrity and association, not source authenticity or recomputed analysis. Only supported report sections are selected. Optional supplied registries record native report-command completion even for partial outcomes; individual statuses remain authoritative. No disk registry is written; PDF and repository merge remain unfinished."});
    if let Some(text)=registry{output["registry_json"]=json!(text);output["registry_source_sha256"]=json!(sha(input.registry_json.as_deref().expect("supplied registry")));}
    Ok(output)
}
