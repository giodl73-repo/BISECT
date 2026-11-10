//! Actual State conversion and convergence dispatch; optional cap is an API extension.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".to_string(),"state".to_string()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.split,SplitStrategy::Bisect)||!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Standard bisection/metis-core required".into());}
 let threshold=match algo.seeds{SeedCompositor::ConvergenceSweep{threshold}=>threshold as usize,_=>return Err("Convergence required".into())};
 if algo.weights.minority_weighting{return Err("Minority weighting outside oracle".into());}
 let g=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let e:HashMap<_,_>=if algo.weights.geographic{edges.into_iter().map(|(u,w,x)|((u,w),if algo.weights.alpha_county>1e-10&&ids[u][..5]==ids[w][..5]{x*(1.+algo.weights.alpha_county)}else{x})).collect()}else{HashMap::new()};
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;
 let cap=v["cap_override"].as_u64().map(|n|n as usize);
 let (plan,summary)=bisect_runner::bisection_runner::run_all_splits_convergence(&a,&p,&e,k,t,algo.metis.niter as u32,algo.metis.seed.unwrap_or(0),threshold,cap)?;
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"summary":summary,"threshold":threshold,"cli_seed_budget":args.seeds,"native_state_limit":null}));Ok(())
}
