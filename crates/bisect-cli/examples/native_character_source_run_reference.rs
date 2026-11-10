//! Selected actual filesystem source -> native alignment/weighter -> actual
//! State Single/Multi compositor. Does not exercise complete State output IO.
use bisect_cli::{args::{Cli,Commands},runner::{AlgorithmConfig,SeedCompositor},edge_weights::{EdgeWeighter,EconomicCharacterWeighter,HousingCharacterWeighter}};
use clap::Parser;
use serde_json::{json,Value};
use std::collections::{BTreeMap,HashMap};
fn class(v:f64)->&'static str{if v.is_nan(){"nan"}else if v==f64::INFINITY{"positive-infinity"}else if v==f64::NEG_INFINITY{"negative-infinity"}else if v==0.&&v.is_sign_negative(){"negative-zero"}else{"finite"}}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let r:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;let o=&r["options"];let g=&r["graph"];
 if o["structure"]!="standard-bisect"{return Err("Single/Multi standard native reference only".into());}
 let seed=if o["seed"].is_string(){o["seed"].as_str().unwrap().to_string()}else{o["seed"].to_string()};
 let args=vec!["bisect".to_string(),"state".into(),"--state".into(),g["state"].as_str().ok_or("State required")?.into(),"--year".into(),g["year"].as_str().ok_or("Year required")?.into(),"--partition-mode".into(),"edge-weighted".into(),"--weights-override".into(),o["weights"].as_str().ok_or("Weights required")?.into(),"--metis-engine".into(),"metis-core".into(),"--search".into(),o["search"].as_str().ok_or("Search required")?.into(),"--seeds".into(),o["seeds"].to_string(),"--niter".into(),o["iterations"].to_string(),"--balance-tolerance".into(),o["balance_tolerance"].to_string(),"--seed".into(),seed];
 let a=match Cli::try_parse_from(args)?.command{Commands::State(a)=>a,_=>unreachable!()};let config=AlgorithmConfig::from_state_args(&a);
 let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let index=ids.iter().cloned().enumerate().collect::<HashMap<_,_>>();let adjacency:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let pop:Vec<i64>=serde_json::from_value(g["population"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;let base:HashMap<_,_>=edges.iter().map(|&(u,v,w)|((u,v),w)).collect();let alpha=o["character_alpha"].as_f64().ok_or("Alpha required")?;
 let mut fallback=false;
 let weighted=if config.weights.economic_character{match bisect_cli::lodes::load_lodes_wac_tract("fixture","2020"){
  Ok(chars)if !chars.is_empty()=>EconomicCharacterWeighter::new(bisect_cli::lodes::align_lodes_to_adjacency(&chars,&index,ids.len()),alpha).apply(base),_=>{fallback=true;base}
 }}else if config.weights.housing_character{match bisect_cli::housing::load_acs_housing_tract("fixture","2020"){
  Ok(chars)if !chars.is_empty()=>HousingCharacterWeighter::new(bisect_cli::housing::align_housing_to_adjacency(&chars,&index,ids.len()),alpha).apply(base),_=>{fallback=true;base}
 }}else{return Err("Character weights required".into())};
 if fallback&&o["character_absent_file_fallback"]!=true{return Err("Explicit geographic fallback required".into());}
 let k=o["districts"].as_u64().ok_or("Districts required")? as usize;let tolerance=a.balance_tolerance.ok_or("Tolerance required")?/100.;
 let plan=match config.seeds{
  SeedCompositor::Single=>bisect_runner::bisection_runner::run_all_splits(&adjacency,&pop,&weighted,k,tolerance,config.metis.niter as u32,config.metis.seed,None)?,
  SeedCompositor::Multi{seeds}=>bisect_runner::bisection_runner::run_all_splits_multi(&adjacency,&pop,&weighted,k,tolerance,config.metis.niter as u32,config.metis.seed.unwrap_or(0),seeds,None)?,
  _=>return Err("Single/Multi reference required".into())
 };
 let cut=edges.iter().filter(|&&(u,v,_)|plan[&u]!=plan[&v]).map(|&(u,v,_)|weighted[&(u,v)]).fold(0.,|a,b|a+b);
 let assignments:BTreeMap<_,_>=ids.iter().enumerate().map(|(i,id)|(id.clone(),plan[&i])).collect();
 println!("{}",json!({"assignments":assignments,"weighted_boundary":cut,"weighted_boundary_class":class(cut),"geographic_fallback":fallback,"scope":"Selected actual filesystem loader/native alignment/native weighter/State Single-Multi kernel; cut measured in supplied prepared-edge order; no complete State filesystem-output orchestration."}));Ok(())
}
