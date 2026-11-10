//! Actual filesystem loader, native exact-ID alignment and CLI edge weighter.
//! Invoke only in an isolated fixture directory containing selected test files.
use bisect_cli::edge_weights::{EdgeWeighter,EconomicCharacterWeighter,HousingCharacterWeighter};
use std::collections::{BTreeMap,HashMap,HashSet};
use serde_json::{json,Value};
fn class(v:f64)->&'static str{if v.is_nan(){"nan"}else if v==f64::INFINITY{"positive-infinity"}else if v==f64::NEG_INFINITY{"negative-infinity"}else if v==0.&&v.is_sign_negative(){"negative-zero"}else{"finite"}}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let request:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let r=&request["input"];let s=&r["source"];let target:Vec<String>=serde_json::from_value(r["target"]["geoids"].clone())?;let source_edges:Vec<(usize,usize,f64)>=serde_json::from_value(r["target"]["edges"].clone())?;
 let index:HashMap<_,_>=target.iter().cloned().enumerate().collect();let edges:HashMap<_,_>=source_edges.iter().map(|&(u,v,w)|((u,v),w)).collect();let alpha=r["alpha"].as_f64().ok_or("Alpha required")?;
 let mut ids=Vec::new();let mut records=Vec::new();let mut weighted=edges.clone();let mut error=None;
 let status=if s["kind"]=="economic"{match bisect_cli::lodes::load_lodes_wac_tract("fixture","2020"){
  Err(e)=>{error=Some(e);"error"},Ok(chars)if chars.is_empty()=>"empty",Ok(chars)=>{
   ids.extend(chars.keys().cloned());let aligned=bisect_cli::lodes::align_lodes_to_adjacency(&chars,&index,target.len());
   for(id,c)in target.iter().zip(&aligned){let v=[c.commercial_intensity,c.industrial_fraction,c.jobs_per_resident];records.push(json!({"geoid":id,"supplied":chars.contains_key(id),"values":v,"classes":v.map(class)}));}
   weighted=EconomicCharacterWeighter::new(aligned,alpha).apply(edges);"loaded"
  }
 }}else{match bisect_cli::housing::load_acs_housing_tract("fixture","2020"){
  Err(e)=>{error=Some(e);"error"},Ok(chars)if chars.is_empty()=>"empty",Ok(chars)=>{
   ids.extend(chars.keys().cloned());let aligned=bisect_cli::housing::align_housing_to_adjacency(&chars,&index,target.len());
   for(id,c)in target.iter().zip(&aligned){let v=[c.pct_single_family,c.pct_multifamily,c.pct_owner,c.housing_vintage];records.push(json!({"geoid":id,"supplied":chars.contains_key(id),"values":v,"classes":v.map(class)}));}
   weighted=HousingCharacterWeighter::new(aligned,alpha).apply(edges);"loaded"
  }
 }};
 if status!="loaded"&&r["allow_absent_fallback"]!=true{return Err("Explicit fallback required".into());}
 let selected:HashSet<_>=target.iter().cloned().collect();let supplied:HashSet<_>=ids.iter().cloned().collect();let mut missing:Vec<_>=selected.difference(&supplied).cloned().collect();missing.sort();ids.sort();let ignored:Vec<_>=ids.into_iter().filter(|id|!selected.contains(id)).collect();
 let ordered:BTreeMap<_,_>=weighted.into_iter().collect();let weights=source_edges.iter().map(|&(u,v,_)|{let w=ordered[&(u,v)];json!({"u":u,"v":v,"weight":w,"class":class(w)})}).collect::<Vec<_>>();
 println!("{}",json!({"native_status":status,"native_error":error,"geographic_fallback":status!="loaded","matched_tracts":if status=="loaded"{target.len()-missing.len()}else{0},"missing_tract_ids":missing,"ignored_source_ids":ignored,"records":records,"edges":weights}));Ok(())
}
