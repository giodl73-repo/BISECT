//! Native registry/runner doctor metadata, with optional actual CLI rendering.
use serde_json::{json,Value};
fn main()->anyhow::Result<()> {
 let args:Vec<_>=std::env::args().skip(1).collect();if args.is_empty(){anyhow::bail!("Expected cases JSON, optionally policy path and doctor mode.");}
 std::env::remove_var("BISECT_LOCATION_POLICY");std::env::remove_var("BISECT_STATE_POLICY");
 if let Some(path)=args.get(1){std::env::set_var("BISECT_LOCATION_POLICY",path);}
 let policy=bisect_cli::policy::load_policy();let registry=bisect_cli::registry::LocationRegistry::from_str(&policy.to_string())?;
 let cases:Vec<Value>=serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;
 if args.get(2).is_some_and(|s|s=="doctor"){
  let c=&cases[0];bisect_cli::doctor::run_doctor(&bisect_cli::args::DoctorArgs{state:c["state"].as_str().unwrap().into(),year:c["year"].as_str().unwrap().into(),chamber:c["chamber"].as_str().unwrap().into(),resolution:c["resolution"].as_str().unwrap().into(),output_base:"target/doctor-empty".into(),label:None,version:"v1".into(),all:false,verify_manifest:None,check_tutorial_data:false,tutorial:"vermont-2020".into()});return Ok(());
 }
 let mut results=Vec::new();for c in cases {
  let code=c["state"].as_str().unwrap().to_uppercase();let year=c["year"].as_str().unwrap();let chamber=c["chamber"].as_str().unwrap();let resolution=c["resolution"].as_str().unwrap();
  let valid=registry.validate_year(&code,year);let chamber_count=registry.chamber_districts(&code,chamber,year);let count=chamber_count.or_else(||registry.congressional_districts(&code,year));
  let entry=policy.get(&code);let nesting=if chamber!="congressional"{entry.and_then(|e|e.get("nesting_requirement")).and_then(Value::as_str).filter(|s|*s!="null")}else{None};
  results.push(json!({"state":code,"name":registry.state_name(&code),"year":year,"chamber":chamber,"resolution":resolution,"registered":registry.has_location(&code),"available_years":registry.available_years(&code),"year_valid":valid.is_ok(),"year_error":valid.err(),"districts":count.map(|n|n.to_string()),"policy_tolerance_percent":bisect_cli::runner::chamber_balance_tolerance(&code,chamber)*100.,"granularity_warning":registry.granularity_warning(&code,year,chamber,resolution),"nesting_requirement":nesting,"nesting_ratio":nesting.and_then(|_|entry.and_then(|e|e.get("nesting_ratio")).and_then(Value::as_str)),"compactness_standard":entry.and_then(|e|e.get("compactness_standard")).and_then(Value::as_str)}));
 }
 println!("{}",serde_json::to_string(&results)?);Ok(())
}
