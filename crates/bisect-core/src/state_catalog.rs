//! Native bulk-state enumeration without filesystem or process access.
use crate::fetch_plan::Manifest;
/// State CLI naming rule. An explicit name is effective only with adjacency.
/// Native raw overrides are preserved; callers must separately authorize paths.
pub fn resolve_state_name(state_code:&str,registry_name:Option<&str>,adjacency:bool,override_name:Option<&str>)->String {
 if adjacency {
  if let Some(name)=override_name { return name.to_owned(); }
  if registry_name.is_none() { return state_code.to_lowercase(); }
 }
 registry_name.unwrap_or(state_code).to_lowercase().replace(' ',"_")
}
/// Native bulk runner's legacy completion markers, relative to its output root.
/// The historical path uses the lowercased code, not the human state slug.
pub fn legacy_completion_paths(state_code:&str,year:&str)->[String;2]{
 let base=format!("{year}/states/{}/data/final_assignments",state_code.to_lowercase());
 [format!("{base}.json"),format!("{base}.pkl")]
}
pub fn legacy_state_complete(reprocess:bool,json_exists:bool,pkl_exists:bool)->bool{
 !reprocess&&(json_exists||pkl_exists)
}
/// Completion evidence for an unlabeled bulk State target. Presence is not proof
/// of valid contents or a successful engine run.
#[derive(Debug, serde::Serialize)]
pub struct StateCompletion {
 pub complete: bool,
 pub reason: &'static str,
 pub marker_paths: Vec<String>,
 pub observed_paths: Vec<String>,
 pub incomplete_paths: Vec<String>,
}
/// Check the actual writer namespace first, then historical code folders.
/// An interrupted canonical target must never be hidden by a historical marker.
/// Reprocess bypasses all observations; it does not authorize overwriting an
/// incomplete target (the writer's separate incomplete guard still applies).
pub fn state_completion(
 state_code:&str,state_name:&str,year:&str,reprocess:bool,mut exists:impl FnMut(&str)->bool,
)->Result<StateCompletion,String>{
 if state_code.len()!=2||!state_code.bytes().all(|b|b.is_ascii_alphabetic())
  ||year.len()!=4||!year.bytes().all(|b|b.is_ascii_digit())
  ||state_name.is_empty()||state_name.len()>200||!state_name.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b)) {
  return Err("State completion requires a portable state slug, two-letter code and four-digit year".into());
 }
 let mut roots=vec![format!("{year}/states/{state_name}")];
 let historical=format!("{year}/states/{}",state_code.to_lowercase());
 if roots[0]!=historical {roots.push(historical);}
 let marker_paths=roots.iter().flat_map(|r|[format!("{r}/data/final_assignments.json"),format!("{r}/data/final_assignments.pkl")]).collect();
 let mut result=StateCompletion{complete:false,reason:if reprocess{"reprocess"}else{"missing-completion"},marker_paths,observed_paths:Vec::new(),incomplete_paths:Vec::new()};
 if reprocess{return Ok(result);}
 for (i,root) in roots.iter().enumerate(){
  for suffix in ["manifest.tmp","data/.final_assignments.tmp.json","data/.vra_analysis.tmp.json","data/.provenance.tmp.json","data/.district_summary.tmp.csv"] {
   let path=format!("{root}/{suffix}");if exists(&path){result.incomplete_paths.push(path);}
  }
  for path in &result.marker_paths[i*2..i*2+2] {if exists(path){result.observed_paths.push(path.clone());}}
  if !result.incomplete_paths.is_empty(){result.reason=if i==0{"incomplete-state-target"}else{"incomplete-historical-target"};return Ok(result);}
  if !result.observed_paths.is_empty(){result.complete=true;result.reason=if i==0{"completed-state-target"}else{"completed-historical-target"};return Ok(result);}
 }
 Ok(result)
}
pub fn congressional_states(manifest:&Manifest,year:&str)->Result<Vec<(String,String,usize)>,String>{
 if !["2020","2010","2000"].contains(&year){return Err(format!("unsupported year '{year}' — valid years are 2020, 2010, 2000"));}
 let mut states:Vec<_>=manifest.states.iter().filter_map(|(code,state)|{
  let districts=*state.districts.get(year)?;
  if districts==0{return None;}
  Some((code.clone(),state.name.to_lowercase().replace(' ',"_"),districts))
 }).collect();
 states.sort_by(|a,b|a.0.cmp(&b.0));Ok(states)
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn state_name_branch_and_raw_override(){
  assert_eq!(resolve_state_name("RI",Some("Rhode Island"),false,Some("ignored")),"rhode_island");
  assert_eq!(resolve_state_name("RI",Some("Rhode Island"),true,None),"rhode_island");
  assert_eq!(resolve_state_name("ZZ",None,true,None),"zz");
  assert_eq!(resolve_state_name("ZZ",None,false,None),"zz");
  assert_eq!(resolve_state_name("NORTH STATE",None,true,None),"north state");
  assert_eq!(resolve_state_name("NORTH STATE",None,false,None),"north_state");
  for name in ["Research County","MiXeD","../unsafe","", "東京"] {
   assert_eq!(resolve_state_name("ZZ",None,true,Some(name)),name);
  }
 }
 #[test]
 fn completion_precedence_and_reprocess(){
  let canonical="2020/states/rhode_island/data/final_assignments.json";
  let old="2020/states/ri/data/final_assignments.pkl";
  let pending="2020/states/rhode_island/manifest.tmp";
  let check=|paths:Vec<&str>|state_completion("RI","rhode_island","2020",false,|p|paths.contains(&p)).unwrap();
  assert_eq!(check(vec![canonical]).reason,"completed-state-target");
  assert_eq!(check(vec![old]).reason,"completed-historical-target");
  assert!(!check(vec![canonical,old,pending]).complete);
  assert!(check(vec![canonical,"2020/states/ri/manifest.tmp"]).complete);
  for suffix in ["manifest.tmp","data/.final_assignments.tmp.json","data/.vra_analysis.tmp.json","data/.provenance.tmp.json","data/.district_summary.tmp.csv"] {
   let path=format!("2020/states/rhode_island/{suffix}");assert!(!check(vec![canonical,old,&path]).complete);
  }
  assert_eq!(state_completion("RI","rhode_island","2020",true,|_|panic!("reprocess must not observe files")).unwrap().reason,"reprocess");
  for slug in ["../ri","/ri","C:ri","ri\\outside",".",""] {assert!(state_completion("RI",slug,"2020",false,|_|panic!("unsafe path observed")).is_err());}
  let same=state_completion("RI","ri","2020",false,|p|p==old).unwrap();assert_eq!(same.marker_paths.len(),2);assert!(same.complete);
 }
 fn manifest()->Manifest{serde_json::from_str(include_str!("../../../data/manifest.json")).unwrap()}
 #[test]
 fn vintages_and_reapportionment(){
  for (year,ca,tx) in [("2020",52,38),("2010",53,36),("2000",53,32)]{
   let rows=congressional_states(&manifest(),year).unwrap();assert_eq!(rows.len(),50);
   assert!(rows.windows(2).all(|r|r[0].0<r[1].0));
   assert_eq!(rows.iter().find(|r|r.0=="CA").unwrap().2,ca);
   assert_eq!(rows.iter().find(|r|r.0=="TX").unwrap().2,tx);
   assert_eq!(rows.iter().find(|r|r.0=="RI").unwrap().1,"rhode_island");
  }
 }
 #[test]
 fn excludes_zero_and_missing_vintage_without_changing_other_years(){
  let mut data=manifest();data.states.get_mut("RI").unwrap().districts.remove("2020");data.states.get_mut("IA").unwrap().districts.insert("2020".into(),0);
  let rows=congressional_states(&data,"2020").unwrap();assert_eq!(rows.len(),48);assert!(!rows.iter().any(|r|["RI","IA"].contains(&r.0.as_str())));assert_eq!(congressional_states(&data,"2010").unwrap().len(),50);
  assert!(congressional_states(&data,"2030").unwrap_err().contains("unsupported year"));
 }
}
