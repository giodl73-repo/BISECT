//! Native State conversion/VRA-ReCom dispatch with explicitly supplied fractions.
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
 if !matches!(algo.split,SplitStrategy::Bisect)||!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Standard bisection/metis-core required".into());}
 let (steps,percentile,threshold)=match algo.seeds{SeedCompositor::VraRecom{steps,p,vap_threshold}=>(steps,p,vap_threshold),_=>return Err("VRA ReCom required".into())};
 let g=&v["graph"];
 let adjacency:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
 let population:Vec<i64>=serde_json::from_value(g["population"].clone())?;
 let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let weights:HashMap<_,_>=if algo.weights.geographic{edges.into_iter().map(|(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
 if algo.weights.alpha_county>1e-10||algo.weights.minority_weighting{return Err("This oracle requires geographic or unweighted overrides".into());}
 let fractions:Vec<f64>=serde_json::from_value(v["fractions"].clone())?;
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
 let (plan,report)=bisect_runner::bisection_runner::run_vra_recom_detailed(&adjacency,&population,&weights,k,algo.metis.niter as u32,algo.metis.seed.unwrap_or(0),steps,percentile,threshold,&fractions)?;
 let final_tolerance=args.balance_tolerance.ok_or("Explicit final tolerance required")?/100.0;
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,k,final_tolerance).is_ok();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"final_tolerance":final_tolerance,"proposal_tolerance":0.005,"report":report}));
 Ok(())
}
