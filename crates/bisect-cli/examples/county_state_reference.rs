//! Independent State conversion, native county composer and selected kernels.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor};
use bisect_cli::edge_weights::{EdgeWeighter,SubdivisionWeighter};
use clap::Parser;
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis)||!algo.weights.geographic||algo.weights.minority_weighting{return Err("Portable county objective required".into());}
 let g=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;let base:HashMap<_,_>=edges.into_iter().map(|(u,w,x)|((u,w),x)).collect();let e=SubdivisionWeighter::county_only(&ids.iter().cloned().enumerate().collect(),ids.len(),algo.weights.alpha_county).apply(base);
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;let t=args.balance_tolerance.ok_or("Tolerance required")?/100.;let it=algo.metis.niter as u32;let seed=algo.metis.seed;
 let plan=match algo.split{
  SplitStrategy::Bisect=>match algo.seeds{SeedCompositor::Single=>bisect_runner::bisection_runner::run_all_splits(&a,&p,&e,k,t,it,seed,None)?,SeedCompositor::Multi{seeds}=>bisect_runner::bisection_runner::run_all_splits_multi(&a,&p,&e,k,t,it,seed.unwrap_or(0),seeds,None)?,_=>return Err("Single/Multi required".into())},
  SplitStrategy::GeoSection=>bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,algo.seeds.seed_count(),None,&HashMap::new(),0.,None,1.1,None,0.,None,seed.unwrap_or(1))?.0,
  _=>return Err("Standard/GeoSection required".into())
 };
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();let mut weighted_edges:Vec<_>=e.iter().map(|(&(u,w),&x)|(u,w,x)).collect();weighted_edges.sort_by_key(|&(u,w,_)|(u,w));let cut:f64=weighted_edges.iter().filter(|&&(u,w,_)|plan[&u]!=plan[&w]).map(|&(_,_,x)|x).sum();let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();println!("{}",serde_json::json!({"assignments":assignments,"weighted_edges":weighted_edges,"weighted_cut":cut,"balanced":balanced,"alpha":algo.weights.alpha_county}));Ok(())
}
