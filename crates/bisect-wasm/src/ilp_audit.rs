//! Verify selected model artifacts and compare ILP audit summaries; no filesystem calls.
use serde::Deserialize;
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};
use base64::{engine::general_purpose::STANDARD,Engine};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub reports:BTreeMap<String,String>,
    #[serde(default)]
    pub reports_base64:Option<BTreeMap<String,String>>,
    pub models_base64:BTreeMap<String,String>,
    pub summary_json:Option<String>,
}
pub(crate) fn audit(input:Input)->Result<Value,String>{
    let reports:BTreeMap<String,Vec<u8>>=if let Some(encoded)=input.reports_base64 {
        if !input.reports.is_empty(){return Err("Choose text reports or byte reports, not both.".into());}
        if encoded.is_empty()||encoded.len()>512{return Err("Select 1–512 solve reports.".into());}
        let mut decoded=BTreeMap::new();let mut total=0usize;
        for (path,value) in encoded {
            if value.len()>((8*1024*1024+2)/3)*4{return Err("Report encoding exceeds 8 MiB.".into());}
            let bytes=STANDARD.decode(value).map_err(|_|"Invalid report artifact base64.")?;total+=bytes.len();if total>8*1024*1024{return Err("Reports exceed 8 MiB.".into());}decoded.insert(path,bytes);
        }decoded
    }else{input.reports.into_iter().map(|(p,t)|(p,t.into_bytes())).collect()};
    if reports.is_empty()||reports.len()>512||input.models_base64.len()>512 {return Err("Select 1–512 solve reports and at most 512 model artifacts.".into());}
    let mut paths=BTreeSet::new();
    for path in reports.keys().chain(input.models_base64.keys()) {
        if path.len()>240||path.is_empty()||!path.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-/".contains(&b))||path.split('/').any(|p|p.is_empty()||[".","..","__proto__","constructor","prototype"].contains(&p))||!paths.insert(path.to_ascii_lowercase()){return Err("Unsafe, duplicate or case-colliding selected ILP artifact path.".into());}
    }
    let report_bytes:usize=reports.values().map(Vec::len).sum();
    if report_bytes>8*1024*1024||input.summary_json.as_ref().is_some_and(|s|s.len()>1024*1024){return Err("Reports exceed 8 MiB or summary exceeds 1 MiB.".into());}
    let mut models=BTreeMap::new();let mut total=report_bytes;
    for(path,encoded)in input.models_base64 {if encoded.len()>((8*1024*1024+2)/3)*4{return Err("Model encoding exceeds 8 MiB.".into());}let bytes=STANDARD.decode(encoded).map_err(|_|"Invalid model artifact base64.")?;if bytes.len()>8*1024*1024{return Err("Model artifact exceeds 8 MiB.".into());}total+=bytes.len();if total>16*1024*1024{return Err("Selected reports and model bytes exceed 16 MiB.".into());}models.insert(path,bytes);}
    let mut summary=bisect_runner::ilp_audit::audit_supplied_reports(&reports,&models);
    let summary_check=input.summary_json.as_ref().map(|s|bisect_runner::ilp_audit::verify_and_retain_portable_supplied_summary(s.as_bytes(),&mut summary));
    let path_binding=summary_check.as_ref().and_then(|r|r.as_ref().ok()).and_then(|binding|binding.clone());
    let failed_summary=summary_check.as_ref().is_some_and(|r|r.is_err());
    let native_exit_code=if summary.failed>0||failed_summary{1}else{0};
    let summary_json=serde_json::to_string_pretty(&summary).map_err(|e|e.to_string())?;
    let report_hashes:BTreeMap<_,_>=reports.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
    let model_hashes:BTreeMap<_,_>=models.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
    let mut result=json!({"schema_version":"bisect-ilp-audit-v1","status":if native_exit_code==0{"verified"}else{"failed"},"native_exit_code":native_exit_code,"summary":summary,"summary_json":summary_json,"supplied_summary_status":match &summary_check{None=>"not-selected",Some(Ok(_))=>"match",Some(Err(_))=>"mismatch"},"supplied_summary_error":summary_check.and_then(Result::err),"report_sha256":report_hashes,"model_sha256":model_hashes,"scope":"Uses the native ILP artifact hash verifier and audit summary records over explicitly selected report/model bytes. Outcome, proof status and fallback flags are reported claims; matching model hashes do not establish solver execution, model correctness, plan validity or an optimization proof. Selected paths are resolved only within the supplied byte map. No local files or referenced paths are opened. Portable selections are bounded; automatic native directory discovery is not performed."});
    if let Some(binding)=path_binding {result.as_object_mut().unwrap().insert("supplied_summary_path_binding".into(),serde_json::to_value(binding).map_err(|e|e.to_string())?);}
    Ok(result)
}
