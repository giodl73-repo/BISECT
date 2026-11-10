//! Supplied-state report aggregation using the shared native merge/export code.
use std::collections::{BTreeMap,HashSet};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use serde::de::{DeserializeSeed,Visitor,MapAccess,SeqAccess,Error};
struct Strict<'a>{depth:usize,nodes:&'a mut usize}
impl<'de> DeserializeSeed<'de> for Strict<'_>{type Value=Value;fn deserialize<D:serde::Deserializer<'de>>(self,d:D)->Result<Value,D::Error>{*self.nodes+=1;if self.depth>64||*self.nodes>500_000{return Err(D::Error::custom("Report JSON is too complex"));}d.deserialize_any(self)}}
impl<'de> Visitor<'de> for Strict<'_>{type Value=Value;
 fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("bounded JSON with unique fields")}
 fn visit_bool<E:Error>(self,v:bool)->Result<Value,E>{Ok(v.into())}
 fn visit_unit<E:Error>(self)->Result<Value,E>{Ok(Value::Null)}
 fn visit_i64<E:Error>(self,v:i64)->Result<Value,E>{Ok(v.into())}
 fn visit_u64<E:Error>(self,v:u64)->Result<Value,E>{Ok(v.into())}
 fn visit_f64<E:Error>(self,v:f64)->Result<Value,E>{serde_json::Number::from_f64(v).map(Value::Number).ok_or_else(||E::custom("Nonfinite report number"))}
 fn visit_str<E:Error>(self,v:&str)->Result<Value,E>{Ok(v.into())}
 fn visit_string<E:Error>(self,v:String)->Result<Value,E>{Ok(v.into())}
 fn visit_seq<S:SeqAccess<'de>>(self,mut s:S)->Result<Value,S::Error>{let mut out=Vec::new();while let Some(v)=s.next_element_seed(Strict{depth:self.depth+1,nodes:self.nodes})?{out.push(v);}Ok(Value::Array(out))}
 fn visit_map<M:MapAccess<'de>>(self,mut m:M)->Result<Value,M::Error>{let mut out=serde_json::Map::new();while let Some(k)=m.next_key::<String>()?{if k.len()>200||matches!(k.as_str(),"__proto__"|"constructor"|"prototype")||out.contains_key(&k){return Err(M::Error::custom("Duplicate or unsafe report field"));}let v=m.next_value_seed(Strict{depth:self.depth+1,nodes:self.nodes})?;out.insert(k,v);}Ok(Value::Object(out))}
}
pub(crate) fn parse(text:&str)->Result<Value,String>{let mut d=serde_json::Deserializer::from_str(text);let v=Strict{depth:0,nodes:&mut 0}.deserialize(&mut d).map_err(|e|e.to_string())?;d.end().map_err(|e|e.to_string())?;Ok(v)}
fn display(v:&Value)->Value{match v{
 Value::Number(n) if n.as_u64().is_some_and(|x|x>9_007_199_254_740_991)||n.as_i64().is_some_and(|x|x< -9_007_199_254_740_991)||n.as_f64().is_some_and(|x|x.fract()==0.&&x.abs()>9_007_199_254_740_991.)=>Value::String(n.to_string()),
 Value::Array(a)=>Value::Array(a.iter().map(display).collect()),Value::Object(o)=>Value::Object(o.iter().map(|(k,v)|(k.clone(),display(v))).collect()),_=>v.clone()}}
pub(crate) fn aggregate(sources:&BTreeMap<String,String>)->Result<Value,String>{
 if !(1..=60).contains(&sources.len())||sources.iter().map(|(n,s)|n.len().saturating_add(s.len())).sum::<usize>()>8*1024*1024{return Err("Supply 1–60 state reports totaling at most 8 MiB.".into());}
 let mut reports=Vec::new();let mut states=HashSet::new();let mut analyzer=None;let mut year=None;let mut policy=None;let mut hashes=BTreeMap::new();let mut rows=0usize;
 for(name,text)in sources{
  if name.is_empty()||name.len()>120||!name.ends_with(".json")||!name.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'_'|b'-'|b'.'))||text.len()>3*1024*1024{return Err("Reports require plain JSON basenames and at most 3 MiB each.".into());}
  let v=parse(text)?;let a=v["analyzer"].as_str().ok_or("Reports require an analyzer.")?;
  if !["compactness","demographic","urban","political","proportionality","contiguity","splits","summary"].contains(&a){return Err("Unsupported native analyzer report.".into());}
  let state=v["state"].as_str().filter(|s|!s.is_empty()&&s.len()<=60&&s.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'_'|b'-'|b' '))).ok_or("Reports require an explicit state label.")?.to_owned();
  if !states.insert(state.to_ascii_uppercase()){return Err("Duplicate state report.".into());}
  let y=match &v["year"]{Value::String(s)=>s.clone(),Value::Number(n)=>n.to_string(),_=>return Err("Reports require an explicit year.".into())};
  if y.len()!=4||!y.bytes().all(|b|b.is_ascii_digit()){return Err("Reports require a four-digit year.".into());}
  if analyzer.as_deref().is_some_and(|x|x!=a)||year.as_deref().is_some_and(|x|x!=y){return Err("Reports must share analyzer and year.".into());}
  analyzer=Some(a.to_owned());year=Some(y);
  let fields=["coordinate_basis","crs_label","component_policy","perimeter_policy","reock_method","length_width_method","dissolve_method","exact_reock_method","bootstrap_protocol"];
  let p:Vec<_>=fields.iter().map(|k|v.get(*k).cloned()).collect();if policy.as_ref().is_some_and(|x|x!=&p){return Err("Reports use incompatible coordinate or analysis methods.".into());}policy=Some(p);
  if a!="proportionality"{let districts=v["districts"].as_array().ok_or("District reports require a districts array.")?;if districts.is_empty()||districts.len()>500{return Err("Supply 1–500 district rows per state.".into());}let mut ids=HashSet::new();for d in districts{let id=d["district"].as_u64().filter(|x|*x>0&&*x<=500).ok_or("District rows require IDs 1–500.")?;if !d.is_object()||!ids.insert(id){return Err("Duplicate or invalid district row.".into());}}rows+=districts.len();if rows>25_000{return Err("Too many national district rows.".into());}}
  hashes.insert(name.clone(),format!("{:x}",Sha256::digest(text.as_bytes())));reports.push((state,v));
 }
 let refs:Vec<_>=reports.iter().map(|(s,v)|(s.as_str(),v)).collect();let state_level=analyzer.as_deref()==Some("proportionality");
 use bisect_analysis::national::*;
 let merged=if state_level{merge_state_level_outputs(year.as_deref().unwrap(),&refs)}else{merge_analyzer_outputs(&refs)};
 let csv=if state_level{state_records_to_csv(&merged)}else{districts_to_csv(&merged)};
 let array=if state_level{state_records_to_json_array(&merged)}else{districts_to_json_array(&merged)};
 let json=serde_json::to_string_pretty(&merged).map_err(|e|e.to_string())?;let array_json=serde_json::to_string_pretty(&array).map_err(|e|e.to_string())?;
 Ok(json!({"schema_version":"bisect-national-analysis-v1","report":display(&merged),"report_json":json,"csv":csv,"array_json":array_json,"source_sha256":hashes,"report_sha256":format!("{:x}",Sha256::digest(json.as_bytes())),"provided_states":reports.iter().map(|(s,_)|s).collect::<Vec<_>>(),"year":year,"scope":"Aggregates supplied native reports in lexical filename order. State count describes supplied reports; national coverage and source authenticity are not certified. Native JSON and array exports preserve all row fields and exact counts; display renders large integers as strings. CSV uses columns from the first record; later-only fields remain in JSON. Native merge does not calculate a national weighted metric or legal conclusion."}))
}
