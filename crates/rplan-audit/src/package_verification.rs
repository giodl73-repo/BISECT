//! Native example/benchmark package verification with host-supplied file access.
use serde::{Deserialize,Serialize};
use crate::{AuditCertificate,AuditResult};

pub const GOLDEN_SCHEMA:&str="u20-public-example-manifest-v1";
pub const BENCHMARK_SCHEMA:&str="benchmark-rplan-package-manifest-v1";
#[derive(Debug,Clone,Deserialize)]
pub struct PackageManifest {pub schema_version:String,pub example_id:Option<String>,pub files:Vec<PackageFile>}
#[derive(Debug,Clone,Deserialize)]
pub struct PackageFile {pub path:String,pub sha256:String,pub role:String}
#[derive(Debug,Serialize)]
pub struct PackageVerification {
    pub example_id:Option<String>,pub checked_files:usize,
    pub plan_path:String,pub context_path:String,pub certificate_path:String,
    pub certificate_id:String,pub certificate_content_hash:String,
    pub plan_hash:String,pub context_hash:Option<String>,pub certificate_result:AuditResult,
}
pub fn parse_package_manifest(content:&str)->Result<Option<PackageManifest>,String>{
    let value:serde_json::Value=serde_json::from_str(content).map_err(|e|format!("invalid manifest JSON: {e}"))?;
    if !matches!(value.get("schema_version").and_then(serde_json::Value::as_str),Some(GOLDEN_SCHEMA|BENCHMARK_SCHEMA)){return Ok(None);}
    serde_json::from_value(value).map(Some).map_err(|e|format!("invalid RPLAN package manifest JSON: {e}"))
}
/// Hash every listed file and verify the first native plan/context/certificate role binding.
/// The filesystem host can stream hashes; the browser host only consults selected bytes.
pub fn verify_package_manifest(
    manifest:&PackageManifest,
    hash:impl Fn(&str)->Result<String,String>,
    read_role:impl Fn(&str,&str)->Result<Vec<u8>,String>,
)->Result<PackageVerification,String>{
    if !matches!(manifest.schema_version.as_str(),GOLDEN_SCHEMA|BENCHMARK_SCHEMA){return Err(format!("unsupported RPLAN package manifest schema_version: {}",manifest.schema_version));}
    for file in &manifest.files {
        let actual=hash(&file.path)?;
        if actual!=file.sha256.to_ascii_lowercase(){return Err(format!("RPLAN package SHA-256 mismatch for {}: manifest={}, current={}",file.path,file.sha256,actual));}
    }
    let required=|role:&str|manifest.files.iter().find(|file|file.role==role).map(|file|file.path.clone()).ok_or_else(||format!("RPLAN package manifest missing required role: {role}"));
    let plan_path=required("plan")?;let context_path=required("context")?;let certificate_path=required("certificate")?;
    let plan_bytes=read_role("plan",&plan_path)?;let context_bytes=read_role("context",&context_path)?;let certificate_bytes=read_role("certificate",&certificate_path)?;
    let plan_text=std::str::from_utf8(&plan_bytes).map_err(|e|format!("read RPLAN package plan: {e}"))?;
    let context_text=std::str::from_utf8(&context_bytes).map_err(|e|format!("read RPLAN package context: {e}"))?;
    let plan=rplan_io::read_rplan_str(plan_text).map_err(|e|format!("parse RPLAN package plan: {e}"))?;
    let context=rplan_io::read_rctx_str(context_text).map_err(|e|format!("parse RPLAN package context: {e}"))?;
    let certificate:AuditCertificate=serde_json::from_slice(&certificate_bytes).map_err(|e|format!("parse RPLAN package certificate: {e}"))?;
    let verified=crate::verify_audit_certificate(&certificate,Some(&plan.plan),Some(&context)).map_err(|e|format!("RPLAN package certificate verification failed: {e}"))?;
    Ok(PackageVerification {example_id:manifest.example_id.clone(),checked_files:manifest.files.len(),plan_path,context_path,certificate_path,certificate_id:verified.certificate_id,certificate_content_hash:verified.content_hash,plan_hash:verified.plan_hash,context_hash:verified.context_hash,certificate_result:verified.result})
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest,Sha256};
    use std::collections::BTreeMap;
    fn fixture()->(PackageManifest,BTreeMap<String,Vec<u8>>){
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark");
        let manifest=parse_package_manifest(&std::fs::read_to_string(root.join("manifest.json")).unwrap()).unwrap().unwrap();
        let files=manifest.files.iter().map(|f|(f.path.clone(),std::fs::read(root.join(&f.path)).unwrap())).collect();(manifest,files)
    }
    fn check(manifest:&PackageManifest,files:&BTreeMap<String,Vec<u8>>)->Result<PackageVerification,String>{verify_package_manifest(manifest,|p|files.get(p).map(|b|format!("{:x}",Sha256::digest(b))).ok_or_else(||"missing selected file".into()),|_,p|files.get(p).cloned().ok_or_else(||"missing selected role file".into()))}
    #[test]
    fn native_public_package_verifies_all_files_and_certificate(){let (manifest,files)=fixture();let out=check(&manifest,&files).unwrap();assert_eq!(out.checked_files,manifest.files.len());assert_eq!(out.plan_path,"plan.rplan");assert_eq!(out.certificate_result,AuditResult::Pass);}
    #[test]
    fn package_hash_tamper_rejected(){let (manifest,mut files)=fixture();files.get_mut("plan.rplan").unwrap().push(b' ');assert!(check(&manifest,&files).unwrap_err().contains("SHA-256 mismatch"));}
    #[test]
    fn rehashed_certificate_tamper_still_rejected(){let (mut manifest,mut files)=fixture();let mut cert:serde_json::Value=serde_json::from_slice(&files["audit-certificate.json"]).unwrap();cert["certificate_id"]="changed".into();let bytes=serde_json::to_vec(&cert).unwrap();manifest.files.iter_mut().find(|f|f.role=="certificate").unwrap().sha256=format!("{:x}",Sha256::digest(&bytes));files.insert("audit-certificate.json".into(),bytes);assert!(check(&manifest,&files).unwrap_err().contains("certificate verification failed"));}
    #[test]
    fn missing_native_role_rejected(){let (mut manifest,files)=fixture();manifest.files.iter_mut().find(|f|f.role=="context").unwrap().role="unbound".into();assert!(check(&manifest,&files).unwrap_err().contains("missing required role: context"));}
}
