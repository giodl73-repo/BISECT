//! Native alignment and weighters, independent of the WASM input adapter.
use bisect_cli::edge_weights::{EdgeWeighter,EconomicCharacterWeighter,HousingCharacterWeighter};
use bisect_core::{economic_character::{LodesWacRaw,derive_economic_character},housing_character::HousingChar};
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let ids:Vec<String>=serde_json::from_value(v["geoids"].clone())?;
 let index=ids.iter().cloned().enumerate().collect::<HashMap<_,_>>();
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(v["edges"].clone())?;
 let base=edges.into_iter().map(|(u,w,x)|((u,w),x)).collect();let alpha=v["alpha"].as_f64().ok_or("Alpha required")?;
 let (characters,weights)=if v["input"]["data"]["kind"]=="economic"{
  let raw:HashMap<String,LodesWacRaw>=serde_json::from_value(v["input"]["data"]["raw_counts"].clone())?;
  let source=raw.into_iter().map(|(id,x)|(id,derive_economic_character(x))).collect();
  let chars=bisect_cli::lodes::align_lodes_to_adjacency(&source,&index,ids.len());
  let weights=EconomicCharacterWeighter::new(chars.clone(),alpha).apply(base);
  (serde_json::to_value(ids.iter().cloned().zip(chars).collect::<BTreeMap<_,_>>())?,weights)
 }else{
  let source:HashMap<String,HousingChar>=serde_json::from_value(v["input"]["data"]["characters"].clone())?;
  let chars=bisect_cli::housing::align_housing_to_adjacency(&source,&index,ids.len());
  let weights=HousingCharacterWeighter::new(chars.clone(),alpha).apply(base);
  (serde_json::to_value(ids.iter().cloned().zip(chars).collect::<BTreeMap<_,_>>())?,weights)
 };
 let mut edges:Vec<_>=weights.into_iter().map(|((u,w),x)|(u,w,x)).collect();edges.sort_by_key(|&(u,w,_)|(u,w));
 println!("{}",serde_json::json!({"characters":characters,"edges":edges}));Ok(())
}
