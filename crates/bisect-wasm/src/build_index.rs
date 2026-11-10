use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub label:String,pub year:String,pub yaml_text:String,pub config_path:String,
 pub command:String,pub output_dir:String,pub created:String,
 pub configs:Vec<bisect_runner::build_index::ConfigRow>,
 pub results:Vec<bisect_runner::build_index::ResultRow>,
}
pub(crate) fn assemble(input:Input)->Result<Value,String>{
 bisect_report::label_rename::validate_name(&input.label)?;
 if input.label.len()>100||input.year.is_empty()||input.year.len()>100||input.yaml_text.len()>1024*1024||input.config_path.len()>240||input.output_dir.len()>240||input.command.len()>2000||input.created.is_empty()||input.created.len()>100||input.configs.len()>60||input.results.len()>60||input.configs.iter().any(|c|c.state_code.len()>100||c.state_name.len()>200||["__proto__","constructor","prototype"].contains(&c.state_name.as_str())||c.num_districts>500)||input.results.iter().any(|r|r.state_code.len()>100||r.error.as_ref().is_some_and(|e|e.len()>2000)){return Err("Build index inputs exceed browser limits or use reserved state names.".into());}
 let yaml=bisect_runner::config::AlgoYaml::from_text(&input.yaml_text)?;
 yaml.resolve()?;
 let sha=format!("{:x}",Sha256::digest(input.yaml_text.as_bytes()));
 let index=bisect_runner::build_index::build_build_index(&input.label,&input.year,&input.config_path,&sha,&input.command,&input.output_dir,&yaml,&input.configs,&input.results,input.created,env!("CARGO_PKG_VERSION").into())?;
 let text=serde_json::to_string_pretty(&index).map_err(|e|e.to_string())?;
 let rows:Vec<_>=index.states.iter().map(|(name,e)|json!({"state_name":name,"status":e.status,"districts":e.districts,"error":e.error})).collect();
 Ok(json!({"schema_version":"bisect-label-build-index-v1","status":"assembled","native_index_json":text,"index_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"config_sha256":sha,"summary":index.summary,"states":rows,"scope":"Shared native label-build index assembly from supplied runner records. Records are declarations; this operation does not run states, fetch inputs or authenticate results. Native last-result-per-state and last-config-per-name rules are retained; summary counts all result records, including duplicates or unscheduled states. Missing scheduled states are listed separately and do not count as failed results in the native summary. Paths, command, creation time and declared engine are inert recorded metadata. Use engine-generated records in the forthcoming multi-year build workflow; standalone imported records are not certification."}))
}
