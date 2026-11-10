//! Native manifest prechecks over a selected, portable package inventory.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
use std::path::Path;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub manifest_json:String,pub manifest_path:String,pub files_base64:BTreeMap<String,String>}
fn resolve(root:&str,path:&str,directory:bool)->Result<String,String>{
    if !(directory&&path.is_empty())&&!crate::package_verification::safe(path){return Err(format!("Unsafe manifest reference: {path}"));}
    Ok(if root.is_empty(){path.into()}else if path.is_empty(){root.into()}else{format!("{root}/{path}")})
}
pub(crate) fn verify(input:Input)->Result<Value,String>{
    if input.manifest_json.is_empty()||input.manifest_json.len()>1024*1024||!crate::package_verification::safe(&input.manifest_path){return Err("Select a manifest up to 1 MiB with a portable package-relative path.".into());}
    let mut files=crate::package_verification::selected_bytes(input.files_base64)?;
    if files.get(&input.manifest_path).is_some_and(|b|b.as_slice()!=input.manifest_json.as_bytes()){return Err("Selected manifest bytes differ from manifest_json.".into());}
    let selected_hashes:BTreeMap<_,_>=files.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
    // The native directory walk includes the manifest when its name/directory match.
    files.insert(input.manifest_path.clone(),input.manifest_json.as_bytes().to_vec());
    let root=input.manifest_path.rsplit_once('/').map_or("",|(r,_)|r);
    let read=|path:&str|files.get(path).cloned().ok_or_else(||format!("Manifest file not selected: {path}"));
    let hash=|relative:&str|{let path=resolve(root,relative,false)?;read(&path).map(|b|format!("{:x}",Sha256::digest(b)))};
    let mut ilp=json!({"status":"not-run"});let mut certificate=json!({"status":"not-run"});
    let diagnostic=std::cell::RefCell::new(None);
    let outcome=(||{
        let manifest:bisect_report::manifest_schema::ReadOnlyPlanManifest=serde_json::from_str(&input.manifest_json).map_err(|e|format!("invalid manifest JSON: {e}"))?;
        let value=serde_json::to_value(&manifest).map_err(|e|e.to_string())?;
        let bindings:bisect_runner::ilp_audit::ReadOnlyManifestIlpBindings=serde_json::from_value(value.clone()).map_err(|e|e.to_string())?;
        let checked=bisect_runner::ilp_audit::verify_manifest_ilp_summary(&bindings,hash,|directory,summary|{
            let directory=resolve(root,directory,true)?;let summary_path=resolve(root,summary,false)?;
            let summary_bytes=read(&summary_path)?;if summary_bytes.len()>1024*1024{return Err("Selected audit summary exceeds 1 MiB".into());}
            let selection=bisect_runner::ilp_audit::summary_report_selection(&summary_bytes)?;
            let selected=selection.clone().map(|paths|paths.into_iter().map(|p|resolve(&directory,&p,false)).collect::<Result<std::collections::BTreeSet<_>,_>>()).transpose()?;
            let reports:BTreeMap<_,_>=files.iter().filter(|(p,_)|selected.as_ref().is_none_or(|paths|paths.contains(*p))&&(directory.is_empty()||p.starts_with(&format!("{directory}/")))&&bisect_runner::ilp_audit::is_solve_report_name(Path::new(p))).map(|(p,b)|(p.clone(),b.clone())).collect();
            if reports.is_empty(){return Err(format!("no ILP solve reports found under {directory}"));}
            let total:usize=reports.values().map(Vec::len).sum();if reports.len()>100||total>8*1024*1024{return Err("Selected solve reports exceed 100 files or 8 MiB.".into());}
            let mut audit=bisect_runner::ilp_audit::audit_supplied_reports(&reports,&files);audit.report_selection=selection;*diagnostic.borrow_mut()=Some(audit.clone());
            let bytes=read(&summary_path)?;if bytes.len()>1024*1024{return Err("Selected audit summary exceeds 1 MiB.".into());}
            bisect_runner::ilp_audit::verify_supplied_summary(&bytes,&audit,Path::new(&directory))?;
            if audit.failed>0{return Err(format!("{} ILP solve report(s) failed verification",audit.failed));}
            Ok(audit)
        });
        match checked {Ok(Some(v))=>ilp=json!({"status":"verified","verification":v}),Ok(None)=>ilp=json!({"status":"not-recorded"}),Err(e)=>{ilp=json!({"status":"failed","error":e});return Err(e)}}
        let bindings=serde_json::from_value(value).map_err(|e|e.to_string())?;
        let checked=rplan_audit::manifest_verification::verify_manifest_certificate(&bindings,hash,|relative|read(&resolve(root,relative,false)?));
        match checked {Ok(Some(v))=>certificate=json!({"status":"verified","verification":v}),Ok(None)=>certificate=json!({"status":"not-recorded"}),Err(e)=>{certificate=json!({"status":"failed","error":e});return Err(e)}}
        Ok(())
    })();
    let status=if outcome.is_err(){"failed"}else if ilp["status"]=="not-recorded"&&certificate["status"]=="not-recorded"{"not-recorded"}else{"verified"};
    Ok(json!({"schema_version":"bisect-manifest-prechecks-v1","status":status,"native_exit_code":if outcome.is_err(){1}else{0},"error":outcome.err(),"ilp_summary":ilp,"certificate":certificate,"computed_ilp_summary":diagnostic.into_inner(),"manifest_path":input.manifest_path,"manifest_sha256":format!("{:x}",Sha256::digest(input.manifest_json.as_bytes())),"selected_file_sha256":selected_hashes,"scope":"Native Verify manifest prechecks in native order: manifest-bound ILP summary digest, native recursive report-name selection within the supplied inventory, model hashes and typed summary equality, then manifest certificate bindings/metadata/lineage/source-hash claims. No recorded paths yield not-recorded, not verified checks. Inventory is explicitly selected; completeness against an external directory is not proven. The manifest itself participates in report selection when native filename/directory rules include it. Failure stops subsequent checks. No external path or URL is opened. Solver/audit outcomes are recorded claims; this does not compare assignments, replay the engine, authenticate the author or establish legal validity."}))
}
