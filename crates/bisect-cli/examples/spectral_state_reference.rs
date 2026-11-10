//! Actual State argument conversion and spectral dispatch over a supplied graph.
//! Mirrors the native runner's Spectral arm, including k=1 summary generation.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy};
use clap::Parser;
use std::collections::BTreeMap;

fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build the portable oracle with --no-default-features".into());}
 let input:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(input["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 let max_iters=match algo.split{SplitStrategy::Spectral{max_iters}=>max_iters,_=>return Err("Spectral State dispatch required".into())};
 let adjacency:Vec<Vec<usize>>=serde_json::from_value(input["graph"]["adjacency"].clone())?;
 let population:Vec<i64>=serde_json::from_value(input["graph"]["population"].clone())?;
 let ids:Vec<String>=serde_json::from_value(input["graph"]["geoids"].clone())?;
 let k=args.districts.ok_or("Explicit district count required")?;
 let tolerance=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;
 let (plan,summary)=bisect_apportion::spectral::run_spectral_recursive(&adjacency,&population,k,tolerance,max_iters)?;
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[i]+1)).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"summary":summary,"declared_spectral_iters":args.spectral_iters,"resolved_spectral_iters":max_iters,"declared_seed":args.seed.map(|s|s.to_string()),"declared_niter":args.niter}));
 Ok(())
}
