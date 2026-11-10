//! In-memory delivery of the native certified_model_package compact compiler.
use std::collections::BTreeMap;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
pub(crate) fn compile(instance_text:&str,discovery_text:&str)->Result<Value,String>{inner(instance_text,discovery_text).map_err(|e|e.to_string())}
fn inner(instance_text:&str,discovery_text:&str)->Result<Value,Box<dyn std::error::Error>>{
 if instance_text.len()>3*1024*1024||discovery_text.len()>3*1024*1024{return Err("Proof model sources exceed 3 MiB each".into());}
 let instance:bisect_ilp::CertifiedSplitInstance=serde_json::from_str(instance_text)?;
 let discovery:bisect_ilp::CertifiedSplitDiscovery=serde_json::from_str(discovery_text)?;
 instance.validate()?;
 if instance.unit_ids.len()>100000||instance.edges.len()>200000{return Err("Proof model input exceeds 100000 units or 200000 edges".into());}
 let bits=usize::BITS as usize-(instance.unit_ids.len()-1).leading_zeros() as usize;
 // Conservative budget for five compact OPB models, including depth terms.
 let estimate=5u128*(instance.unit_ids.len() as u128*4096+instance.edges.len() as u128*(1024+256*bits as u128));
 if estimate>128*1024*1024{return Err("Compact proof model estimated working set exceeds 128 MiB".into());}
 let mut files:BTreeMap<String,String>=BTreeMap::new();let mut artifacts=BTreeMap::new();
 let mut add=|stem:String,artifact:bisect_ilp::CertifiedOpbArtifact|->Result<(),Box<dyn std::error::Error>>{
  let request_text=serde_json::to_string_pretty(&artifact.request)?;
  let opb_name=format!("{stem}.opb");let request_name=format!("{stem}.request.json");
  artifacts.insert(stem,json!({"path":opb_name,"bytes":artifact.opb.len(),"sha256":sha(artifact.opb.as_bytes()),"request_path":request_name,"request_bytes":request_text.len(),"request_sha256":sha(request_text.as_bytes()),"variable_count":artifact.request.variable_count,"constraint_count":artifact.request.constraint_count,"status":artifact.request.status}));
  files.insert(opb_name,artifact.opb);files.insert(request_name,request_text);
  if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Compact proof model package exceeds 25 MiB".into());}Ok(())
 };
 for (stem,stage) in [("01-population",bisect_ilp::CertifiedDecisionStage::PopulationLowerBound),("02-boundary",bisect_ilp::CertifiedDecisionStage::BoundaryLowerBound),("03-canonical",bisect_ilp::CertifiedDecisionStage::CanonicalTieBreak)]{
  add(stem.into(),bisect_ilp::compile_certified_split_compact_proof_request(&instance,&discovery,stage)?)?;
 }
 if discovery.objective.primary.max_population_deviation_scaled>0 {
  let total=instance.populations.iter().map(|&v|v as i128).sum::<i128>();let numerator=instance.k_right as i128*total;let divisor=instance.k_parent as i128;
  let lower=i64::try_from(numerator.div_euclid(divisor))?;let upper=i64::try_from((numerator+divisor-1).div_euclid(divisor))?;
  for (suffix,population) in [("low",lower),("high",upper)]{add(format!("02-{suffix}-boundary-right-{population}"),bisect_ilp::compile_certified_split_compact_boundary_branch(&instance,&discovery,population)?)?;}
 }
 let compiler_sha256=sha(include_bytes!("../../bisect-ilp/src/proof_backend.rs"));
 let manifest=json!({"schema_version":"certified-split-model-package-v1","package_id":"browser-compact-proof-models","status":"proof-required-unclassified","instance_hash":discovery.instance_hash,"discovery_id":discovery.discovery_id,"compiler_path":"crates/bisect-ilp/src/proof_backend.rs","compiler_sha256":compiler_sha256,"artifacts":artifacts,"claim_boundary":"Hash-bound native compact proof models for the supplied unproved incumbent; no generated proof, checked UNSAT certificate or legal certification is claimed. Small native decision classifications do not replace a checked external proof."});
 files.insert("manifest.json".into(),serde_json::to_string_pretty(&manifest)?);
 let hashes=files.iter().map(|(name,text)|(name.clone(),sha(text.as_bytes()))).collect::<BTreeMap<_,_>>();
 Ok(json!({"schema_version":"bisect-certified-proof-models-v1","status":"models-compiled","proof_status":"not-generated","instance_hash":instance.hash()?,"discovery_id":discovery.discovery_id,"unit_count":instance.unit_ids.len(),"compiler_sha256":compiler_sha256,"instance_source_sha256":sha(instance_text.as_bytes()),"discovery_source_sha256":sha(discovery_text.as_bytes()),"artifacts":artifacts,"files":files,"file_sha256":hashes,"scope":"Native compact OPB/request files for population, weighted boundary and canonical tie-break decisions, plus the native low/high population branches when required. Compilation validates incumbent identity, exact objective and connectivity. Recorded solver commands are inert text. No proof generator or external proof checker ran; this package is not an optimality certificate."}))
}
