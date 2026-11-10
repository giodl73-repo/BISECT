use serde_json::{json,Value};
use bisect_report::label_registry::{parse,render_list,render_show};

pub(crate) fn inspect(source:Option<String>,label:Option<String>)->Result<Value,String>{
 if source.as_ref().is_some_and(|s|s.len()>1024*1024)||label.as_ref().is_some_and(|s|s.is_empty()||s.len()>200||s.chars().any(char::is_control)){return Err("Select registry up to 1 MiB and an optional nonempty label up to 200 bytes without control characters.".into());}
 let registry=parse(source.as_deref().unwrap_or(""))?;
 if registry.len()>500||registry.iter().any(|(n,e)|n.len()>200||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Browser inventory supports up to 500 labels, names up to 200 bytes, and 100 recorded years per stage.".into());}
 let labels:Vec<_>=registry.into_iter().collect();let list_text=render_list(&labels,false)?;let list_json=render_list(&labels,true)?;
 let show=label.as_ref().and_then(|l|labels.iter().find(|(n,_)|n==l));let mut show_json=None;let mut show_text=None;let mut show_error=None;
 if let Some(l)=&label{if let Some((_,entry))=show{show_json=Some(render_show(l,entry,true)?);show_text=Some(render_show(l,entry,false)?);}else{show_error=Some(format!("[CONFIG] show: label '{l}' does not exist in registry.\nRun: bisect ls    to see available labels."));}}
 let selected_details=if let Some(text)=&show_json {let native:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;let years:Vec<_>=native["years"].as_object().into_iter().flat_map(|m|m.iter()).map(|(year,flags)|json!({"year":year,"built":flags["built"],"analyzed":flags["analyzed"],"reported":flags["reported"]})).collect();Some(json!({"paths":native["paths"],"years":years}))}else{None};
 let rows:Vec<_>=labels.iter().map(|(name,e)|json!({"label":name,"built":e.built,"analyzed":e.analyzed,"reported":e.reported})).collect();
 Ok(json!({"schema_version":"bisect-label-inventory-v1","status":if show_error.is_some(){"label-not-found"}else{"listed"},"list_native_exit_code":0,"show_native_exit_code":if label.is_none(){Value::Null}else{json!(if show_error.is_some(){1}else{0})},"labels":rows,"selected_label":label,"selected_details":selected_details,"native_list_text":list_text,"native_list_json":list_json,"native_show_text":show_text,"native_show_json":show_json,"show_error":show_error,"scope":"Shared native ls/show parser and renderers. Lists recorded registry stages and displays label paths and per-year stage flags. Imported stage arrays are not checked against files or against built/analyzed/reported subset invariants, just as native read-only ls/show do not enforce them. Paths are informational strings, never opened or fetched. Duplicate years/order are retained in list output; show years are sorted and deduplicated. Empty or absent registry lists no labels. This does not run the engine or verify hash chains, plan contents, authorship or legal compliance. Imported result exports require an explicit Run."}))
}

pub(crate) fn compare(source:Option<String>,a:String,b:String,year:String)->Result<Value,String>{
 if source.as_ref().is_some_and(|s|s.len()>1024*1024)||a.len()>200||b.len()>200||year.len()!=4||!year.bytes().all(|c|c.is_ascii_digit()){return Err("Select a registry up to 1 MiB, labels up to 200 bytes and a four-digit year.".into());}
 bisect_report::label_compare::validate_labels(&a,&b)?;
 let registry=parse(source.as_deref().unwrap_or(""))?;
 if registry.len()>500||registry.iter().any(|(n,e)|n.len()>200||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Browser registry supports up to 500 labels and 100 years per stage.".into());}
 bisect_report::label_compare::check(&registry,&a,&b,&year)?;
 Ok(json!({"schema_version":"bisect-label-comparison-readiness-v1","status":"ready-to-compare","label_a":a,"label_b":b,"year":year,"native_summary":bisect_report::label_compare::summary(&a,&b,&year),"native_text":bisect_report::label_compare::render(&a,&b,&year,false)?,"native_json":bisect_report::label_compare::render(&a,&b,&year,true)?,"registry_sha256":source.as_ref().map(|s|{use sha2::{Digest,Sha256};format!("{:x}",Sha256::digest(s.as_bytes()))}),"scope":"Shared native label-compare prerequisite checks and instructions. Registry stages are unverified claims. No analysis files are opened, no measurements are computed, no registry is changed and no comparison engine executes. Choose the separate measured comparison operation and its original sources explicitly. Paths are informational portable repository paths."}))
}
