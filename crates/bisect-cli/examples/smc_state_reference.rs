//! Independent State conversion and native SMC percentile first-attempt dispatch.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SeedCompositor,SplitStrategy};
use clap::Parser;
use std::collections::BTreeMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".to_string(),"state".to_string()],tokens].concat())?.command{Commands::State(args)=>args,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.split,SplitStrategy::Bisect){return Err("Standard bisection compositor required".into());}
 let (particles,percentile)=match algo.seeds{SeedCompositor::SmcPercentile{n_particles,p}=>(n_particles,p),_=>return Err("SMC required".into())};
 let g=&v["graph"];
 let adjacency:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
 let population:Vec<i64>=serde_json::from_value(g["population"].clone())?;
 let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
 let seed=algo.metis.seed.unwrap_or(0);
 let plan=bisect_runner::bisection_runner::run_smc_percentile(&adjacency,&population,k,seed,particles,percentile,args.smc_resample_threshold)?;
 let tolerance=args.balance_tolerance.ok_or("Explicit final tolerance required")?/100.0;
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,k,tolerance).is_ok();
 let assignment:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignment,"balanced":balanced,"final_tolerance":tolerance,"sampler_tolerance":0.005,"particles":particles,"percentile":percentile,"resample_threshold":args.smc_resample_threshold,"effective_seed":seed.to_string()}));
 Ok(())
}
