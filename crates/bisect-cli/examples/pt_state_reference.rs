//! Actual State parsing/compositor conversion and native tempering dispatch.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SeedCompositor,SplitStrategy};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".to_string(),"state".to_string()],tokens].concat())?.command{Commands::State(args)=>args,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.split,SplitStrategy::Bisect)||!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Standard bisect/metis-core required".into());}
 let (replicas,swap,cold,hot,steps,percentile)=match algo.seeds{SeedCompositor::ParallelTempering{n_replicas,swap_interval,cold_tolerance,hot_tolerance,steps,p}=>(n_replicas,swap_interval,cold_tolerance,hot_tolerance,steps,p),_=>return Err("Parallel tempering required".into())};
 let g=&v["graph"];
 let adjacency:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
 let population:Vec<i64>=serde_json::from_value(g["population"].clone())?;
 let geoids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let weights:HashMap<_,_>=if algo.weights.geographic{edges.into_iter().map(|(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
 if algo.weights.alpha_county>1e-10{return Err("This oracle requires geographic or unweighted overrides".into());}
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
 let seed=algo.metis.seed.unwrap_or(0);
 let plan=bisect_runner::bisection_runner::run_parallel_tempering(&adjacency,&population,&weights,k,algo.metis.niter as u32,seed,replicas,swap,cold,hot,steps,percentile)?;
 let tolerance=args.balance_tolerance.ok_or("Explicit final tolerance required")?/100.0;
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,k,tolerance).is_ok();
 let assignments:BTreeMap<_,_>=geoids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"cold_tolerance":cold,"hot_tolerance":hot,"final_tolerance":tolerance,"steps":steps,"replicas":replicas,"swap_interval":swap,"percentile":percentile,"effective_seed":seed.to_string()}));
 Ok(())
}
