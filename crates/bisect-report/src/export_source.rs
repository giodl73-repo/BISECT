//! Native export source precedence and legacy assignment reconstruction, without IO.
use crate::manifest_schema::PlanManifestWithCounts;
use crate::rplan::{PortableExportRplan,RplanFile,RplanFileWithIds,RplanMetadataWithCount};
use serde::de::DeserializeOwned;

pub fn source_path(label:&str,exists:impl Fn(&str)->bool)->Option<String>{
 [format!("{label}.rplan"),"final_assignments.json".into(),"data/final_assignments.json".into()].into_iter().find(|path|exists(path))
}
pub fn load(label:&str,rplan:Option<&str>,assignments:Option<&str>,manifest:Option<&str>)->Result<RplanFile,String>{
 load_with(label,rplan,assignments,manifest)
}

/// Read the native 64-bit legacy export domain on every host.
pub fn load_portable(label:&str,rplan:Option<&str>,assignments:Option<&str>,manifest:Option<&str>)->Result<PortableExportRplan,String>{
 load_with(label,rplan,assignments,manifest)
}

fn load_with<D>(label:&str,rplan:Option<&str>,assignments:Option<&str>,manifest:Option<&str>)->Result<RplanFileWithIds<D>,String>
where D:DeserializeOwned+Default+From<u8>+TryFrom<usize>+TryFrom<u64>{
 if let Some(text)=rplan{
  #[cfg(any(feature="filesystem",feature="interchange"))]
  {
  let raw:serde_json::Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
  if raw["rplan_version"]=="0.2" {
   let document=rplan_io::read_rplan_str(text).map_err(|e|e.to_string())?;
   let plan=&document.plan;
   let districts=D::try_from(plan.k).map_err(|_|"District count exceeds the export value range")?;
   let assignments=plan.units.unit_ids.iter().cloned().zip(&plan.assignment).map(|(id,d)| {
    D::try_from(u64::from(*d)+1).map(|value|(id,value)).map_err(|_|"District ID exceeds the export value range")
   }).collect::<Result<_,_>>()?;
   return Ok(RplanFileWithIds{rplan_version:"0.1".into(),metadata:RplanMetadataWithCount{
    label:document.metadata.label.clone(),state_code:plan.units.state.clone().unwrap_or_else(||document.metadata.jurisdiction.clone()),
    year:plan.units.year.map(|y|y.to_string()).unwrap_or_default(),chamber:document.metadata.chamber.clone(),num_districts:districts,
    created_at:document.metadata.created_at.clone(),..Default::default()},
    assignments,geometry:document.geometry.clone()});
  }
  }
  return serde_json::from_str(text).map_err(|e|e.to_string());
 }
 let assignments=serde_json::from_str(assignments.ok_or("No .rplan or final_assignments.json supplied")?).map_err(|e|e.to_string())?;
 let metadata=if let Some(text)=manifest{
  let manifest:PlanManifestWithCounts<D>=serde_json::from_str(text).map_err(|e|e.to_string())?;
  RplanMetadataWithCount{label:manifest.label,state_fips:"00".into(),state_code:manifest.state_code,year:manifest.year,chamber:manifest.chamber,num_districts:manifest.num_districts,population_source:manifest.population_source,balance_tolerance_pct:manifest.balance_tolerance_pct,created_at:manifest.created_at,created_by:format!("bisect {}",manifest.binary_version),..Default::default()}
 }else{RplanMetadataWithCount{label:label.to_string(),..Default::default()}};
 Ok(RplanFileWithIds{rplan_version:"0.1".into(),metadata,assignments,geometry:None})
}

#[cfg(test)]mod tests{
 use super::*;
 use crate::RplanMetadata;
 #[test]
 #[cfg(any(feature="filesystem",feature="interchange"))]
 fn modern_export_preserves_ordered_assignments(){
  let text=include_str!("../../rplan-audit/fixtures/grid3x3-valid.rplan");
  let document=rplan_io::read_rplan_str(text).unwrap();let legacy=load("fixture",Some(text),None,None).unwrap();
  assert_eq!(legacy.metadata.label,document.metadata.label);assert_eq!(legacy.metadata.num_districts,document.plan.k);
  for (id,d) in document.plan.units.unit_ids.iter().zip(&document.plan.assignment){assert_eq!(legacy.assignments[id],*d as usize+1);}
 }
 #[test]fn legacy_export_retains_metadata_and_assignments(){
  let original=RplanFile{rplan_version:"0.1".into(),metadata:RplanMetadata{label:"original".into(),state_fips:"44".into(),population_source:"explicit".into(),..Default::default()},assignments:std::collections::HashMap::from([("44001000100".into(),2)]),geometry:None};
  let text=serde_json::to_string(&original).unwrap();let loaded=load("other",Some(&text),None,None).unwrap();assert_eq!(serde_json::to_value(loaded).unwrap(),serde_json::to_value(original).unwrap());
 }
}
