//! Independent State clap/config conversion and supplied-graph split dispatch.
//! Geometry vectors stand in for explicitly supplied native TIGER measurements.
use bisect_cli::args::{Cli, Commands};
use bisect_cli::runner::{AlgorithmConfig, SplitStrategy};
use bisect_runner::bisection_runner::{run_all_splits_compact,run_geosection_seeded,CompactBisectOpts};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Explicit metis-core required".into());}
 let g=&v["graph"];
 let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
 let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;
 let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let areas:Vec<f64>=serde_json::from_value(g["areas"].clone())?;
 let perimeters:Vec<f64>=serde_json::from_value(g["exterior_perimeters"].clone())?;
 let weights=if algo.weights.geographic{edges.iter().map(|&(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
 let k=args.districts.ok_or("Explicit districts required")?;let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;let count=algo.seeds.seed_count();let it=algo.metis.niter as u32;
 let (plan,structure)=match algo.split{
  SplitStrategy::CompactBisect{epsilon}=>(run_all_splits_compact(&a,&p,&weights,&areas,&perimeters,k,t,it,None,&CompactBisectOpts{seeds_per_level:count,epsilon},None)?,"compact-polsby"),
  SplitStrategy::GeoSection=>{let (plan,_,_,_)=run_geosection_seeded(&a,&p,&weights,k,t,it,count,None,&HashMap::new(),0.,None,1.1,None,0.,None,algo.metis.seed.unwrap_or(1))?;(plan,"ratio-optimal")},
  _=>return Err("CompactBisect or GeoSection required".into())
 };
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"structure":structure,"seeds_per_split":count,"compositor_seed":algo.metis.seed.map(|s|s.to_string()),"declared_seed":args.seed.map(|s|s.to_string()),"iterations":it,"epsilon":if structure=="compact-polsby"{Some(0.05)}else{None}}));Ok(())
}
