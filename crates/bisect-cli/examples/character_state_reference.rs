//! State conversion plus real CLI weighters on explicit aligned observations.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor};
use bisect_cli::edge_weights::{EdgeWeighter,EconomicCharacterWeighter,HousingCharacterWeighter};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".to_string(),"state".to_string()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.split,SplitStrategy::Bisect)||!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Standard bisection/metis-core required".into());}
 let g=&v["graph"];
 let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
 let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;
 let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 // The actual State CLI fixes alpha at .5; explicit override tests the native
 // weighter API used by the browser's additional laboratory control.
 let alpha=v["alpha_override"].as_f64().unwrap_or(algo.weights.econ_alpha);
 if !alpha.is_finite()||!(0.0..=1.0).contains(&alpha){return Err("Invalid alpha".into());}
 let base:HashMap<_,_>=edges.into_iter().map(|(u,w,length)|((u,w),length)).collect();
 let e=if algo.weights.economic_character{
  let raw:Vec<bisect_core::economic_character::LodesWacRaw>=serde_json::from_value(v["raw"].clone())?;
  if raw.len()!=ids.len(){return Err("Complete aligned counts required".into());}
  EconomicCharacterWeighter::new(raw.into_iter().map(bisect_core::economic_character::derive_economic_character).collect(),alpha).apply(base)
 }else if algo.weights.housing_character{
  let chars:Vec<bisect_core::housing_character::HousingChar>=serde_json::from_value(v["characters"].clone())?;
  if chars.len()!=ids.len(){return Err("Complete aligned characters required".into());}
  HousingCharacterWeighter::new(chars,alpha).apply(base)
 }else{return Err("Character override required".into())};
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
 let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;let it=algo.metis.niter as u32;let seed=algo.metis.seed;
 let plan=match algo.seeds{
  SeedCompositor::Single=>bisect_runner::bisection_runner::run_all_splits(&a,&p,&e,k,t,it,seed,None)?,
  SeedCompositor::Multi{seeds}=>bisect_runner::bisection_runner::run_all_splits_multi(&a,&p,&e,k,t,it,seed.unwrap_or(0),seeds,None)?,
  _=>return Err("Single/Multi oracle only".into())
 };
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();
 let mut weighted_edges:Vec<_>=e.iter().map(|(&(u,w),&x)|(u,w,x)).collect();weighted_edges.sort_by_key(|&(u,w,_)|(u,w));
 let weighted_cut:f64=weighted_edges.iter().filter(|&&(u,w,_)|plan[&u]!=plan[&w]).map(|&(_,_,x)|x).sum();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"alpha":alpha,"cli_alpha":algo.weights.econ_alpha,"weighted_edges":weighted_edges,"weighted_cut":weighted_cut}));Ok(())
}
