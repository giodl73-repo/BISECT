//! Native kernel API oracle with native partisan TSV/COI loaders; no State acquisition claim.
use bisect_cli::edge_weights::{EdgeWeighter,PartisanOverrideWeighter,GeographicWeighter,SubdivisionWeighter,MinorityOverrideWeighter,EconomicCharacterWeighter,HousingCharacterWeighter};
use bisect_runner::bisection_runner::*;
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Use --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let x=&v["input"];let g=&x["request"]["graph"];let o=&x["request"]["options"];
 let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let physical:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let index:HashMap<usize,String>=ids.iter().cloned().enumerate().collect();
 let mut e=match o["weights"].as_str().ok_or("Weights required")? {
  "unweighted"=>HashMap::new(),
  "geographic"=>GeographicWeighter::from_map(physical.iter().map(|&(u,v,w)|((u,v),w)).collect()).apply(HashMap::new()),
  "county"=>SubdivisionWeighter::county_only(&index,ids.len(),o["alpha_county"].as_f64().ok_or("County alpha required")?).apply(GeographicWeighter::from_map(physical.iter().map(|&(u,v,w)|((u,v),w)).collect()).apply(HashMap::new())),
  "vra-aligned"=>{let data=bisect_cli::demographics::load_demographics(std::path::Path::new(v["demographics_path"].as_str().ok_or("Demographic CSV required")?))?;let fractions=bisect_cli::demographics::align_demographics_to_adjacency(&data,&index,ids.len());MinorityOverrideWeighter::new(physical.iter().map(|&(u,v,_)|(u,v)).collect(),fractions,0.40).apply(HashMap::new())},
  "economic-character"=>{let raw:BTreeMap<String,bisect_core::economic_character::LodesWacRaw>=serde_json::from_value(x["request"]["character"]["data"]["raw_counts"].clone())?;let chars=ids.iter().map(|id|raw.get(id).cloned().map(bisect_core::economic_character::derive_economic_character).ok_or("Missing native economic tract")).collect::<Result<Vec<_>,_>>()?;EconomicCharacterWeighter::new(chars,o["character_alpha"].as_f64().unwrap()).apply(physical.iter().map(|&(u,v,w)|((u,v),w)).collect())},
  "housing-character"=>{let data:BTreeMap<String,bisect_core::housing_character::HousingChar>=serde_json::from_value(x["request"]["character"]["data"]["characters"].clone())?;let chars=ids.iter().map(|id|data.get(id).cloned().ok_or("Missing native housing tract")).collect::<Result<Vec<_>,_>>()?;HousingCharacterWeighter::new(chars,o["character_alpha"].as_f64().unwrap()).apply(physical.iter().map(|&(u,v,w)|((u,v),w)).collect())},
  "partisan"=>{let shares=bisect_cli::partisan_shares::load_partisan_shares(std::path::Path::new(v["shares_path"].as_str().ok_or("Shares required")?),&index,ids.len())?;PartisanOverrideWeighter::new(physical.iter().map(|&(u,v,_)|(u,v)).collect(),shares,o["dem_threshold"].as_f64().unwrap(),o["rep_threshold"].as_f64().unwrap()).apply(HashMap::new())},
  _=>return Err("Unsupported reference weighting".into())
 };
 if let Some(path)=v["coi_path"].as_str(){e=bisect_cli::runner::apply_coi_weights(e,std::path::Path::new(path),&index)?;}
 let b=&x["block_groups"];let ba:Vec<Vec<usize>>=if b.is_null(){vec![]}else{serde_json::from_value(b["adjacency"].clone())?};let bp:Vec<i64>=if b.is_null(){vec![]}else{serde_json::from_value(b["population"].clone())?};let bi:Vec<String>=if b.is_null(){vec![]}else{serde_json::from_value(b["geoids"].clone())?};let bg_index:HashMap<_,_>=bi.iter().cloned().enumerate().collect();
 let bg=if b.is_null(){None}else{Some((ba.as_slice(),bp.as_slice(),&bg_index))};let fine=if bg.is_some(){MultiscaleFineLevel::BlockGroup}else{MultiscaleFineLevel::Tract};
 let k=o["districts"].as_u64().unwrap() as usize;let it=o["iterations"].as_u64().unwrap() as u32;let seed=if o["seed"].is_null(){0}else if let Some(s)=o["seed"].as_str(){s.parse()?}else{o["seed"].as_u64().unwrap()};let steps=x["total_steps"].as_u64().unwrap() as usize;let alpha=x["alpha"].as_f64().unwrap();let q=x["percentile"].as_f64().unwrap();let t=o["balance_tolerance"].as_f64().unwrap()/100.;let coarse=x["coarse_level"].as_str().unwrap();let ad=&x["adaptive"];
 let (plan,diagnostics)=if ad.is_null(){(run_multiscale_portable(&a,&p,&e,k,t,it,seed,steps,alpha,q,Some(&index),fine,coarse,bg)?,serde_json::Value::Null)}else{
  let (plan,d)=run_multiscale_adaptive_portable(&a,&p,&e,k,it,seed,AdaptiveConfig{total_steps:steps,target_accept:ad["target_accept"].as_f64().unwrap(),initial_alpha:alpha,adapt_interval:ad["adapt_interval"].as_u64().unwrap() as usize,gamma_0:ad["gamma_0"].as_f64().unwrap(),pop_tolerance:t,coarse_tol_factor:ad["coarse_tol_factor"].as_f64().unwrap(),p:q},Some(&index),fine,coarse,bg)?;
  (plan,serde_json::json!({"final_alpha":d.final_alpha,"alpha_trace":d.alpha_trace,"fine_acceptance_rate":d.fine_acceptance_rate,"coarse_acceptance_rate":d.coarse_acceptance_rate}))
 };
 let output_ids=if bi.is_empty(){ids}else{bi};let assignments:BTreeMap<_,_>=output_ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();println!("{}",serde_json::json!({"assignments":assignments,"diagnostics":diagnostics}));Ok(())
}
