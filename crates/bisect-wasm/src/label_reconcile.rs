//! Reconcile registry completion records against a current upstream index snapshot.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
use bisect_report::label_registry::LabelEntry;
#[derive(Deserialize)]
#[serde(rename_all="lowercase")]
pub(crate) enum Stage {Analyzed,Reported}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub stage:Stage,
    pub source_registry_json:String,
    pub source_registry_sha256:String,
    pub current_registry_json:String,
    pub proposed_registry_json:String,
    pub upstream_index_json:String,
    pub generated_index_json:String,
}
fn sha(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
fn registry(text:&str)->Result<BTreeMap<String,LabelEntry>,String>{
    if text.len()>1024*1024{return Err("Registry snapshot exceeds 1 MiB.".into());}
    let map=bisect_report::label_registry::parse(text)?;
    if map.len()>500||map.iter().any(|(name,e)|name.len()>200||["__proto__","constructor","prototype"].contains(&name.as_str())||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Registry exceeds browser label/year limits or contains unsafe names.".into());}Ok(map)
}
pub(crate) fn reconcile(input:Input)->Result<Value,String>{
    use bisect_report::label_indexes::{AnalysisIndex,ReportIndex,serialize_analysis_index,serialize_report_index};
    if input.upstream_index_json.len()>1024*1024||input.generated_index_json.len()>1024*1024{return Err("Selected indexes exceed 1 MiB each.".into());}
    if sha(&input.source_registry_json)!=input.source_registry_sha256{return Err("Original registry snapshot hash mismatch.".into());}
    let reported=matches!(input.stage,Stage::Reported);
    let (label,year,expected_upstream)=if reported {
        let index:ReportIndex=serde_json::from_str(&input.generated_index_json).map_err(|e|e.to_string())?;serialize_report_index(&index)?;
        let upstream:AnalysisIndex=serde_json::from_str(&input.upstream_index_json).map_err(|e|e.to_string())?;serialize_analysis_index(&upstream)?;
        if index.label!=upstream.label||index.year!=upstream.year{return Err("Report and current analysis index identities differ.".into());}
        (index.label,index.year,index.analysis_index_sha256)
    }else{
        let index:AnalysisIndex=serde_json::from_str(&input.generated_index_json).map_err(|e|e.to_string())?;serialize_analysis_index(&index)?;
        let upstream:bisect_runner::build_index::BuildIndex=serde_json::from_str(&input.upstream_index_json).map_err(|e|e.to_string())?;
        if index.label!=upstream.label||index.year!=upstream.year{return Err("Analysis and current build index identities differ.".into());}
        (index.label,index.year,index.run_index_sha256)
    };
    let upstream_sha=sha(&input.upstream_index_json);
    if expected_upstream!=upstream_sha{return Err("Current upstream index changed since this stage ran. Recompute the stage against the current index.".into());}
    let source=registry(&input.source_registry_json)?;let current=registry(&input.current_registry_json)?;let proposed=registry(&input.proposed_registry_json)?;
    let merged=bisect_report::label_registry::reconcile_stage(&source,&current,&proposed,&label,&year,reported)?;
    let registry_json=serde_json::to_string_pretty(&merged).map_err(|e|e.to_string())?;registry(&registry_json)?;
    let upstream_path=if reported{format!("analysis/{label}/{year}/index.json")}else{format!("runs/{label}/{year}/index.json")};
    let generated_path=if reported{format!("reports/{label}/{year}/index.json")}else{format!("analysis/{label}/{year}/index.json")};
    Ok(json!({"schema_version":"bisect-label-registry-reconciliation-v1","status":"reconciled","label":label,"year":year,"stage":if reported{"reported"}else{"analyzed"},"registry_json":registry_json,"registry_sha256":sha(&registry_json),"source_registry_sha256":input.source_registry_sha256,"current_registry_sha256":sha(&input.current_registry_json),"proposed_registry_sha256":sha(&input.proposed_registry_json),"upstream_index_path":upstream_path,"upstream_index_sha256":upstream_sha,"generated_index_path":generated_path,"generated_index_sha256":sha(&input.generated_index_json),"files":{".bisect":registry_json},"scope":"Validates the original registry snapshot digest, verifies the proposal changes only the selected native completion stage, and checks that the current upstream index bytes match the generated stage index. Reapplies that stage onto the supplied current registry, preserving concurrent labels, other years, order and duplicates. Stage flags retain native command-completion semantics, including partial outcomes. This is snapshot reconciliation, not authentication or an atomic disk merge. No files are written. Before applying later, recheck current registry and upstream index digests; artifact conflicts, full repository merge and large archive handling remain unfinished."}))
}
