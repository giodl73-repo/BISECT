//! Inspect State naming without granting source, path or execution authority.
use serde_json::{json,Value};
use bisect_analysis::location_registry::PolicyRegistry;

pub(crate) fn inspect(arguments:Vec<String>)->Result<Value,String>{
 let review=bisect_runner::state_command::inspect(arguments)?;
 if review["status"]!="parsed"||review["profile_filter"]["allowed"]!=true {
  return Err("Inspect a parsed, profile-eligible native State command first.".into());
 }
 let fields=review["fields"].as_array().ok_or("Missing native State fields")?;
 let value=|id:&str| fields.iter().find(|f|f["id"]==id).and_then(|f|f["values"].get(0)).and_then(Value::as_str);
 let state=value("state").ok_or("Missing native state")?.to_uppercase();
 let adjacency=value("adjacency").is_some();
 let requested=value("state_name");
 let (database,_,kind,policy_hash)=crate::location_policy::database(None,None)?;
 let registry=PolicyRegistry::new(&database);
 let registered_name=registry.state_name(&state);
 let name=bisect_core::state_catalog::resolve_state_name(&state,registered_name.as_deref(),adjacency,requested);
 let requested_year=value("year").ok_or("Missing native year")?;
 let year=if registry.has_location(&state){registry.validate_year(&state,requested_year)}else if adjacency{Ok(requested_year.to_owned())}else{Err(format!("Unknown native location '{state}' requires adjacency"))};
 let chamber=value("chamber").ok_or("Missing native chamber")?;
 let requested_label=value("label");
 let label=year.as_ref().ok().map(|y|requested_label.map(str::to_owned).unwrap_or_else(||bisect_report::default_label(&name,chamber,y)));
 let source_names=year.as_ref().ok().map(|y|{
  let population_name=name.to_lowercase().replace(' ',"_");
  json!({"demographics":format!("{name}_demographics_{y}.csv"),"cvap_presence":format!("{name}_cvap_{y}.csv"),
   "vap_population_candidate":format!("{population_name}_vap_{y}.csv"),"cvap_population_candidate":format!("{population_name}_cvap_{y}.csv"),
   "adjacency_manifest":format!("{name}_adjacency_{y}.adj.bin"),"tract_geoid_manifest_candidate":format!("{name}_adjacency_{y}_geoids.json"),"bg_geoid_manifest_candidate":format!("{name}_bg_adjacency_{y}_geoids.json")})
 });
 Ok(json!({"schema_version":"bisect-native-state-identity-v1","state_code":state,
  "requested_state_name":requested,"effective_state_name":name,
  "override_effective":adjacency&&requested.is_some(),
  "override_ignored":!adjacency&&requested.is_some(),
  "name_source":if adjacency&&requested.is_some(){"explicit-adjacency-name"}else if registered_name.is_some(){"embedded-policy"}else{"lowercase-code"},
  "policy_source_kind":kind,"policy_sha256":policy_hash,
  "requested_year":requested_year,"effective_year":year.as_ref().ok(),"native_year_valid":year.is_ok(),"native_year_error":year.as_ref().err(),
  "chamber":chamber,"requested_label":requested_label,"effective_label":label,"layout":if requested_label.is_some(){"labeled"}else{"state"},
  "native_source_name_intents":source_names,"source_discovery_authorized":false,
  "execution_performed":false,"filesystem_reads_performed":false,"path_authority_granted":false,
  "scope":"Shared native State naming, native location/year preflight and default label generation. Raw names are retained without normalization; explicit labels win. Failed native year/location checks remain observations with no effective label. District allocation, source discovery, portable path authorization, target changes and --state-name execution remain separate."}))
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn inspected_command_uses_native_branch(){
  let get=|args:&[&str]|inspect(args.iter().map(|s|s.to_string()).collect()).unwrap();
  let ignored=get(&["--state","RI","--state-name","Alias"]);
  assert_eq!(ignored["effective_state_name"],"rhode_island");
  assert_eq!(ignored["override_ignored"],true);
  assert_eq!(ignored["effective_label"],"rhode_island_congressional_2020");
  let raw=get(&["--state","ZZ","--adjacency","missing.adj.bin","--districts","2","--state-name","../Alias"]);
  assert_eq!(raw["effective_state_name"],"../Alias");
  assert_eq!(raw["path_authority_granted"],false);
  assert_eq!(raw["override_effective"],true);
 }
 #[test]
 fn labels_and_native_year_validation(){
  let get=|args:&[&str]|inspect(args.iter().map(|s|s.to_string()).collect()).unwrap();
  let raw=get(&["--state","RI","--year","2020","--adjacency","not-opened.adj.bin","--state-name","Research Area"]);
  assert_eq!(raw["effective_state_name"],"Research Area");
  assert_eq!(raw["effective_label"],"research_area_congressional_2020");
  assert_eq!(raw["layout"],"state");
  assert_eq!(raw["native_source_name_intents"]["cvap_presence"],"Research Area_cvap_2020.csv");
  assert_eq!(raw["native_source_name_intents"]["cvap_population_candidate"],"research_area_cvap_2020.csv");
  let explicit=get(&["--state","RI","--adjacency","not-opened.adj.bin","--state-name","Research_Area","--label","chosen_plan"]);
  assert_eq!(explicit["effective_label"],"chosen_plan");
  assert_eq!(explicit["layout"],"labeled");
  let bad=get(&["--state","RI","--year","2024","--adjacency","not-opened.adj.bin"]);
  assert_eq!(bad["native_year_valid"],false);
  assert!(bad["effective_label"].is_null());
 }
 #[test]
 fn parser_errors_grant_nothing(){
  assert!(inspect(vec!["--not-a-native-flag".into()]).is_err());
 }
}
