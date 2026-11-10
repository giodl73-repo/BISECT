//! Native CLI import dispatch and legacy file construction, without IO or a clock.
use crate::{PlanManifest,RplanFile,RplanMetadata};
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,HashMap};
#[derive(Clone,Deserialize,Serialize)]#[serde(deny_unknown_fields)]
pub struct Settings{pub label:String,pub state:String,pub year:String,pub format:String,pub source_name:String,pub created_at:String,pub as_civic_counter_proposal:bool,pub submitted_by:Option<String>,pub submitted_at:Option<String>}
pub fn parser_kind(format:&str,source_name:&str)->&'static str{
 if format=="gerrychain"{"gerrychain"}else if format=="dra-csv"||std::path::Path::new(source_name).extension().and_then(|e|e.to_str())==Some("csv"){"dra-csv"}else{"geojson"}
}
pub fn parse(format:&str,source_name:&str,text:&str)->anyhow::Result<HashMap<String,usize>>{
 match parser_kind(format,source_name){
  "gerrychain"=>crate::import_gerrychain_to_assignments(text),
  "dra-csv"=>crate::assignment_import::import_assignments_from_dra_csv(text),
  _=>crate::assignment_import::import_assignments_from_geojson_properties(text)
 }
}
pub fn legacy(s:&Settings,assignments:HashMap<String,usize>)->RplanFile{
 let state=s.state.to_uppercase();let k=assignments.values().copied().max().unwrap_or(0);
 RplanFile{rplan_version:"0.1".into(),metadata:RplanMetadata{label:s.label.clone(),state_fips:bisect_core::state_code_to_fips(&state).unwrap_or("00").into(),state_code:state,year:s.year.clone(),chamber:"imported".into(),num_districts:k,population_source:"unknown".into(),balance_tolerance_pct:0.0,created_at:s.created_at.clone(),created_by:"bisect import".into(),notes:Some(format!("imported from {}",s.source_name)),..Default::default()},assignments,geometry:None}
}
pub fn legacy_files(label:&str,plan:&RplanFile)->anyhow::Result<BTreeMap<String,String>>{
 Ok(BTreeMap::from([(format!("{label}.rplan"),serde_json::to_string_pretty(plan)?),("final_assignments.json".into(),serde_json::to_string_pretty(&plan.assignments.iter().collect::<BTreeMap<_,_>>())?)]))
}
pub fn manifest(s:&Settings,districts:usize,created_at:&str,submitted_default:&str)->anyhow::Result<PlanManifest>{
 if s.as_civic_counter_proposal&&s.submitted_by.as_deref().unwrap_or("").trim().is_empty(){anyhow::bail!("[INPUT] --as-civic-counter-proposal requires --submitted-by \"<organization name>\". Without a submitter, the civic-counter-proposal tag is unusable in downstream comparison reports. See docs/superpowers/specs/2026-04-30-state-staff-interop.md.");}
 let manifest=PlanManifest{label:s.label.clone(),state_code:s.state.to_uppercase(),year:s.year.clone(),chamber:"imported".into(),num_districts:districts,created_at:created_at.into(),submission_type:if s.as_civic_counter_proposal{"civic_counter_proposal"}else{"authoritative"}.into(),submitted_by:s.submitted_by.clone(),submitted_at:s.submitted_at.clone().or_else(||s.as_civic_counter_proposal.then(||submitted_default.into())),source_tool:Some(s.format.clone()),..Default::default()};
 Ok(manifest)
}
