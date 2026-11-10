//! Independent State argument conversion and AreaSection first-attempt dispatch.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,AreaSectionInit,SplitStrategy};
use clap::Parser;
use std::collections::{BTreeMap,HashMap,HashSet};

fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features for the portable State oracle".into());}
 let request:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(request["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".to_string(),"state".to_string()],tokens].concat())?.command{Commands::State(args)=>args,_=>return Err("State arguments required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Explicit metis-core required".into());}
 let (swing,init)=match algo.split{SplitStrategy::AreaSection{area_swing,area_section_init}=>(area_swing,area_section_init),_=>return Err("AreaSection arguments required".into())};
 let graph=&request["graph"];
 let adjacency:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;
 let population:Vec<i64>=serde_json::from_value(graph["population"].clone())?;
 let areas:Vec<f64>=serde_json::from_value(graph["areas"].clone())?;
 let centers:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(graph["edges"].clone())?;
 let geoids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;
 let k=request["districts"].as_u64().ok_or("Districts required")? as usize;
 let mut weights:HashMap<_,_>=if algo.weights.geographic{edges.iter().map(|&(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
 if algo.weights.alpha_county>1e-10&&k>1{
  use bisect_cli::edge_weights::{EdgeWeighter,SubdivisionWeighter};
  weights=SubdivisionWeighter::county_only(&geoids.iter().cloned().enumerate().collect(),geoids.len(),algo.weights.alpha_county).apply(weights);
 }
 let theta=if init==AreaSectionInit::MovingKnife&&!centers.is_empty(){Some(bisect_runner::bisection_runner::split_subgraph_mka_direction(&(0..population.len()).collect::<HashSet<_>>(),&centers,180))}else{None};
 let centroids:HashMap<_,_>=if theta.is_some(){centers.iter().copied().enumerate().collect()}else{HashMap::new()};
 let tolerance=args.balance_tolerance.ok_or("Explicit tolerance required for supplied-graph oracle")?/100.0;
 let seed=algo.metis.seed;
 let count=algo.seeds.seed_count();
 let (plan,left,right,cut)=bisect_runner::bisection_runner::run_geosection_seeded(&adjacency,&population,&weights,k,tolerance,algo.metis.niter as u32,count,None,&centroids,if theta.is_some(){1.0}else{0.0},Some(&areas),swing,None,0.0,theta,seed.unwrap_or(1))?;
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,k,tolerance).is_ok();
 let assignment:BTreeMap<_,_>=geoids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignment,"balanced":balanced,"native_root_cut":cut,"left_districts":left,"right_districts":right,"direction_radians":theta,"effective_base_seed":seed.unwrap_or(1).to_string(),"declared_seed":args.seed.map(|s|s.to_string()),"compositor_seed":seed.map(|s|s.to_string()),"seeds_per_ratio":count,"initializer":if init==AreaSectionInit::MovingKnife{"moving-knife"}else{"ratio-optimal"},"area_swing":swing,"iterations":algo.metis.niter}));
 Ok(())
}
