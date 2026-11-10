//! Actual State argument conversion, native absent-file loaders and geographic
//! weighter/single-split kernel. Does not claim the full State output workflow.
use bisect_cli::{args::{Cli,Commands},runner::AlgorithmConfig,edge_weights::{EdgeWeighter,GeographicWeighter}};
use clap::Parser;
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let state=v["graph"]["state"].as_str().ok_or("State required")?;
 let state_name=match state{"AL"=>"alabama","RI"=>"rhode_island","IA"=>"iowa",_=>return Err("This reference requires AL, RI or IA fixtures".into())};
 let args=match Cli::try_parse_from(["bisect","state","--state",state,"--year","2020","--partition-mode","edge-weighted","--weights-override",v["kind"].as_str().ok_or("Kind required")?,"--metis-engine","metis-core","--search","single","--balance-tolerance","25","--seed","42","--niter","10"])?.command{Commands::State(a)=>a,_=>unreachable!()};
 let algo=AlgorithmConfig::from_state_args(&args);if !algo.weights.geographic{return Err("Native geographic base required".into());}
 // The caller supplies an isolated working directory without data installation.
 let empty=if algo.weights.economic_character{bisect_cli::lodes::load_lodes_wac_tract(state_name,"2020")?.is_empty()}else if algo.weights.housing_character{bisect_cli::housing::load_acs_housing_tract(state_name,"2020")?.is_empty()}else{return Err("Character mode required".into())};if !empty{return Err("Expected absent source; no existing data may be replaced".into());}
 let g=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let map:HashMap<_,_>=edges.into_iter().map(|(u,w,x)|((u,w),x)).collect();let e=GeographicWeighter::from_map(map.clone()).apply(map);
 let assignments=bisect_runner::bisection_runner::run_all_splits(&a,&p,&e,2,0.25,10,Some(42),None)?;
 let result:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,assignments[&i])).collect();println!("{}",serde_json::json!({"assignments":result,"native_absent_loader_empty":true,"native_geographic_base":true}));Ok(())
}
