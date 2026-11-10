use bisect_runner::ilp_audit::{audit_supplied_reports,verify_and_retain_supplied_summary};
use std::collections::BTreeMap;
#[test]
fn verified_scope_is_retained_and_failed_scope_cannot_change_computed_records(){
 let reports=BTreeMap::from([("node.json".into(),include_bytes!("../../../docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/ilp-solve-report.json").to_vec())]);
 let models=BTreeMap::from([("node_root.lp".into(),include_bytes!("../../../docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/node_root.lp").to_vec())]);
 let baseline=audit_supplied_reports(&reports,&models);assert_eq!(baseline.passed,1);
 let mut scoped=serde_json::to_value(&baseline).unwrap();scoped["report_selection"]=serde_json::json!(["node.json"]);
 let mut verified=baseline.clone();verify_and_retain_supplied_summary(&serde_json::to_vec(&scoped).unwrap(),&mut verified,std::path::Path::new("")).unwrap();assert_eq!(verified.report_selection,Some(vec!["node.json".into()]));
 for paths in [serde_json::json!(["../node.json"]),serde_json::json!(["other.json"]),serde_json::json!([])] {
  let mut bad=scoped.clone();bad["report_selection"]=paths;let mut actual=baseline.clone();assert!(verify_and_retain_supplied_summary(&serde_json::to_vec(&bad).unwrap(),&mut actual,std::path::Path::new("")).is_err());assert_eq!(actual,baseline);
 }
 let mut historical=verified.clone();verify_and_retain_supplied_summary(&serde_json::to_vec(&baseline).unwrap(),&mut historical,std::path::Path::new("")).unwrap();assert_eq!(historical.report_selection,None);
}
