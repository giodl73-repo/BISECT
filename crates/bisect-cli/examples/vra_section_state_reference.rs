//! Actual State conversion and original CSV alignment/VRASection dispatch.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 let w=match algo.split{SplitStrategy::VraSection{w_vra}=>w_vra,_=>return Err("VRASection required".into())};
 if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis)||algo.weights.minority_weighting{return Err("Portable geographic/unweighted/county oracle only".into());}
 let g=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
 let mut e:HashMap<_,_>=if algo.weights.geographic{edges.into_iter().map(|(u,v,x)|((u,v),x)).collect()}else{HashMap::new()};
 if algo.weights.alpha_county>1e-10{use bisect_cli::edge_weights::{EdgeWeighter,SubdivisionWeighter};e=SubdivisionWeighter::county_only(&ids.iter().cloned().enumerate().collect(),ids.len(),algo.weights.alpha_county).apply(e);}
 let fractions=if let Some(file)=v["demographic_csv"].as_str(){let demo=bisect_cli::demographics::load_demographics(std::path::Path::new(file))?;Some(bisect_cli::demographics::align_demographics_to_adjacency(&demo,&ids.iter().cloned().enumerate().collect(),ids.len()))}else{None};
 let mass=fractions.as_ref().map(|f|f.iter().zip(&p).map(|(&x,&pop)|x*pop as f64).collect::<Vec<_>>());
 let seed=algo.metis.seed.unwrap_or(1);let seeds=algo.seeds.seed_count();let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;
 let (plan,left,right,cut)=bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,algo.metis.niter as u32,seeds,None,&HashMap::new(),0.,None,1.1,mass.as_deref(),w,None,seed)?;
 let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"fractions":fractions,"weight":w,"effective_base_seed":seed.to_string(),"seeds_per_ratio":seeds,"left_districts":left,"right_districts":right,"root_cut":cut,"source_policy":if mass.is_some(){"native-file-and-alignment"}else{"native-absent-file-plain-geosection"}}));Ok(())
}
