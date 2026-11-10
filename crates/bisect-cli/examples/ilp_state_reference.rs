//! Actual State conversion and native ILP dispatch on explicitly supplied graphs.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy};
use clap::Parser;
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Use --no-default-features for the portable native oracle".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("request")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};let algo=AlgorithmConfig::from_state_args(&args);
 let a:Vec<Vec<usize>>=serde_json::from_value(v["graph"]["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(v["graph"]["population"].clone())?;let ids:Vec<String>=serde_json::from_value(v["graph"]["geoids"].clone())?;let k=v["districts"].as_u64().ok_or("districts")? as usize;
 let report_dir=v["report_dir"].as_str().map(std::path::Path::new);
 let plan=match algo.split{SplitStrategy::Ilp{method,fallback,time_limit_secs,optimality_gap,max_tracts}=>bisect_runner::bisection_runner::run_all_splits_ilp(&a,&p,&std::collections::HashMap::new(),k,args.balance_tolerance.unwrap_or(5.)/100.,method,fallback,time_limit_secs,optimality_gap,max_tracts,report_dir)?,_=>return Err("ILP required".into())};
 let assignments:std::collections::BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();println!("{}",serde_json::to_string(&assignments)?);Ok(())
}
