//! Native TUI Home's in-memory summary projection and stable filtering/sorting.
use serde::{Serialize,Deserialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug,Clone,PartialEq,Default,Serialize,Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum SortColumn {#[default] Label,Splits,Pp,Deviation}
#[derive(Debug,Clone,PartialEq,Default,Serialize,Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum SortDir {#[default] Asc,Desc}
fn decimal<S:serde::Serializer>(v:&u64,s:S)->Result<S::Ok,S::Error>{s.serialize_str(&v.to_string())}
fn optional_decimal<S:serde::Serializer>(v:&Option<u64>,s:S)->Result<S::Ok,S::Error>{match v{Some(v)=>s.serialize_some(&v.to_string()),None=>s.serialize_none()}}
#[derive(Debug,Clone,PartialEq,Serialize)]
pub struct PlanRow {
 pub label:String,pub state_code:String,pub state_name:String,pub chamber:String,pub year:String,
 #[serde(serialize_with="decimal")] pub num_districts:u64,
 pub mean_pp:Option<f64>,pub max_deviation_pct:Option<f64>,
 #[serde(serialize_with="optional_decimal")] pub county_splits:Option<u64>,
 pub all_contiguous:Option<bool>,pub plan_dir:String,
}
pub trait PlanOrdering {
 fn label(&self)->&str;fn state_code(&self)->&str;fn chamber(&self)->&str;
 fn mean_pp(&self)->Option<f64>;fn max_deviation_pct(&self)->Option<f64>;fn county_splits(&self)->Option<u64>;
}
impl PlanOrdering for PlanRow {
 fn label(&self)->&str{&self.label}fn state_code(&self)->&str{&self.state_code}fn chamber(&self)->&str{&self.chamber}
 fn mean_pp(&self)->Option<f64>{self.mean_pp}fn max_deviation_pct(&self)->Option<f64>{self.max_deviation_pct}fn county_splits(&self)->Option<u64>{self.county_splits}
}
pub fn visible_plans<'a,T:PlanOrdering>(plans:&'a[T],filter:&str,column:&SortColumn,direction:&SortDir)->Vec<&'a T>{
 let f=filter.to_lowercase();let mut visible:Vec<_>=plans.iter().filter(|p|f.is_empty()||p.label().to_lowercase().contains(&f)||p.state_code().to_lowercase().contains(&f)||p.chamber().to_lowercase().contains(&f)).collect();
 use SortColumn::*;use SortDir::*;
 match (column,direction){
  (Label,Asc)=>visible.sort_by(|a,b|a.label().cmp(b.label())),(Label,Desc)=>visible.sort_by(|a,b|b.label().cmp(a.label())),
  (Splits,Asc)=>visible.sort_by_key(|p|p.county_splits().unwrap_or(u64::MAX)),(Splits,Desc)=>visible.sort_by_key(|p|std::cmp::Reverse(p.county_splits().unwrap_or(0))),
  (Pp,Asc)=>visible.sort_by(|a,b|a.mean_pp().partial_cmp(&b.mean_pp()).unwrap_or(std::cmp::Ordering::Equal)),(Pp,Desc)=>visible.sort_by(|a,b|b.mean_pp().partial_cmp(&a.mean_pp()).unwrap_or(std::cmp::Ordering::Equal)),
  (Deviation,Asc)=>visible.sort_by(|a,b|a.max_deviation_pct().partial_cmp(&b.max_deviation_pct()).unwrap_or(std::cmp::Ordering::Equal)),(Deviation,Desc)=>visible.sort_by(|a,b|b.max_deviation_pct().partial_cmp(&a.max_deviation_pct()).unwrap_or(std::cmp::Ordering::Equal)),
 }visible
}
/// Invalid/missing manifest skips a plan. Invalid/missing analysis means unknown,
/// matching native Home; declarations are not recomputed or authenticated.
pub fn read_plan<F:FnOnce(&str)->String>(directory:&str,folder_name:&str,manifest:&str,analysis:&BTreeMap<String,String>,state_name:F)->Option<PlanRow>{
 let manifest:Value=serde_json::from_str(manifest).ok()?;
 let string=|field:&str|manifest.get(field).and_then(Value::as_str).unwrap_or("").to_string();
 let read=|file:&str,field:&str|analysis.get(file).and_then(|text|serde_json::from_str::<Value>(text).ok()).and_then(|v|v.get(field).cloned());
 let state_code=string("state_code");let state_name=state_name(&state_code);
 Some(PlanRow{label:manifest.get("label").and_then(Value::as_str).unwrap_or(folder_name).to_string(),state_code,state_name,chamber:string("chamber"),year:string("year"),num_districts:manifest.get("num_districts").and_then(Value::as_u64).unwrap_or(0),mean_pp:read("compactness.json","mean_polsby_popper").and_then(|v|v.as_f64()),max_deviation_pct:read("summary.json","max_deviation_pct").and_then(|v|v.as_f64()),county_splits:read("splits.json","split_count").and_then(|v|v.as_u64()),all_contiguous:read("contiguity.json","all_contiguous").and_then(|v|v.as_bool()),plan_dir:directory.into()})
}
#[cfg(test)] mod tests {
 use super::*;
 #[test]fn native_projection_and_missing_data(){let row=read_plan("plans/a","a",r#"{"state_code":"RI","num_districts":18446744073709551615}"#,&BTreeMap::from([("compactness.json".into(),"broken".into()),("splits.json".into(),r#"{"split_count":5}"#.into())]),|s|s.into()).unwrap();assert_eq!(row.label,"a");assert_eq!(row.mean_pp,None);assert_eq!(row.county_splits,Some(5));assert_eq!(serde_json::to_value(&row).unwrap()["num_districts"],"18446744073709551615");assert!(read_plan("p","p","bad",&BTreeMap::new(),|s|s.into()).is_none());}
 #[test]fn stable_native_order_and_filter(){let mut a=read_plan("a","a","{}",&BTreeMap::new(),|s|s.into()).unwrap();a.label="Alpha".into();a.state_code="RI".into();let mut b=a.clone();b.label="Beta".into();b.mean_pp=Some(0.3);b.county_splits=Some(2);let mut c=b.clone();c.label="Gamma".into();c.county_splits=Some(1);let plans=vec![a,b,c];let labels=|v:Vec<&PlanRow>|v.iter().map(|r|r.label.clone()).collect::<Vec<_>>();assert_eq!(labels(visible_plans(&plans,"ri",&SortColumn::Splits,&SortDir::Asc)),vec!["Gamma","Beta","Alpha"]);assert_eq!(labels(visible_plans(&plans,"",&SortColumn::Pp,&SortDir::Asc)),vec!["Alpha","Beta","Gamma"]);assert_eq!(visible_plans(&plans,"absent",&SortColumn::Label,&SortDir::Asc).len(),0);}
}
