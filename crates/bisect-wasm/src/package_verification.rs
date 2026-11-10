//! Selected-byte adapter for native public RPLAN package verification.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
use base64::{engine::general_purpose::STANDARD,Engine};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub manifest_json:String,pub files_base64:BTreeMap<String,String>}
pub(crate) fn safe(path:&str)->bool{!path.is_empty()&&path.len()<=240&&path.bytes().all(|b|b.is_ascii_alphanumeric()||b"._/-+".contains(&b))&&!path.split('/').any(|p|p.is_empty()||[".","..","__proto__","constructor","prototype"].contains(&p))}
pub(crate) fn selected_bytes(selected:BTreeMap<String,String>)->Result<BTreeMap<String,Vec<u8>>,String>{
    if selected.len()>100{return Err("Select at most 100 package files.".into());}
    let mut seen=BTreeSet::new();let mut files=BTreeMap::new();let mut total=0;
    for (path,encoded)in selected{
        if !safe(&path)||!seen.insert(path.to_ascii_lowercase()){return Err("Unsafe or case-colliding selected package path.".into());}
        if encoded.len()>((16*1024*1024+2)/3)*4{return Err("Selected package exceeds 16 MiB.".into());}
        let bytes=STANDARD.decode(encoded).map_err(|_|"Invalid selected package base64.")?;total+=bytes.len();if total>16*1024*1024{return Err("Selected package exceeds 16 MiB.".into());}files.insert(path,bytes);
    }
    Ok(files)
}
pub(crate) fn verify(input:Input)->Result<Value,String>{
    if input.manifest_json.is_empty()||input.manifest_json.len()>1024*1024||input.files_base64.len()>100{return Err("Select a manifest up to 1 MiB and at most 100 package files.".into());}
    let files=selected_bytes(input.files_base64)?;
    let result=(||{
        let manifest=rplan_audit::package_verification::parse_package_manifest(&input.manifest_json)?.ok_or("Select a native public-example or benchmark RPLAN package manifest.")?;
        if manifest.files.len()>100{return Err("Package manifest exceeds 100 files.".into());}
        let mut declared=BTreeSet::new();for entry in &manifest.files{if !safe(&entry.path)||!declared.insert(entry.path.to_ascii_lowercase()){return Err("Unsafe, duplicate or case-colliding manifest package path.".into());}}
        rplan_audit::package_verification::verify_package_manifest(&manifest,
            |path|files.get(path).map(|b|format!("{:x}",Sha256::digest(b))).ok_or_else(||format!("Package file not selected: {path}")),
            |_role,path|files.get(path).cloned().ok_or_else(||format!("Package file not selected: {path}")),
        )
    })();
    let hashes:BTreeMap<_,_>=files.iter().map(|(p,b)|(p.clone(),format!("{:x}",Sha256::digest(b)))).collect();
    let (status,verified,error)=match result{Ok(v)=>("verified",serde_json::to_value(v).map_err(|e|e.to_string())?,None),Err(e)=>("failed",Value::Null,Some(e))};
    Ok(json!({"schema_version":"bisect-rplan-package-verification-v1","status":status,"native_exit_code":if status=="verified"{0}else{1},"verification":verified,"error":error,"manifest_sha256":format!("{:x}",Sha256::digest(input.manifest_json.as_bytes())),"selected_file_sha256":hashes,"scope":"Uses the shared native public-example/benchmark package verifier: checks every manifest-listed byte hash, then verifies the selected first plan/context/certificate role bindings and stored certificate content. Supplied manifest hashes and role labels are claims; matching bytes and certificate bindings do not authenticate their author or independently recompute the recorded audit checks, solver proof or legal validity. Extra selected files are hashed for disclosure but are not manifest-verified. No referenced path is opened outside the selected byte map. General manifest replay and assignment-only verification remain separate workflows."}))
}
