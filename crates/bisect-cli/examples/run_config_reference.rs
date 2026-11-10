//! Compare shared native Run extraction with the original parser and assignment body.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{StateConfig,AlgorithmConfig};
use clap::Parser;
use std::collections::{BTreeMap,HashMap};
mod legacy{include!("support/run_args_legacy.rs");}
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Use --no-default-features for portable native Run reference".into());}
 let input:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request path required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(input["arguments"].clone())?;
 let shared=match Cli::try_parse_from([vec!["bisect".into(),"run".into()],tokens.clone()].concat())?.command{Commands::Run(a)=>a,_=>return Err("Run required".into())};
 let frozen=legacy::RunArgs::try_parse_from([vec!["bisect run".into()],tokens].concat())?;
 if format!("{shared:?}")!=format!("{frozen:?}"){return Err("Run parser changed from the frozen original".into());}
 let expected_output=frozen.output_dir.clone().unwrap_or_else(||format!("outputs/{}",frozen.version));
 let expected_years=if frozen.year=="all"{vec!["2020".to_string(),"2010".to_string(),"2000".to_string()]}else{vec![frozen.year.clone()]};
 let initial=StateConfig::new_bulk("RI".into(),"rhode_island".into(),2,"2020".into(),shared.version.clone(),expected_output.clone().into(),2);
 let mut expected=initial.clone();expected.debug=frozen.debug;expected.reset=frozen.reset;expected.reprocess=frozen.reprocess;expected.algo=AlgorithmConfig::defaults_for_mode(&frozen.partition_mode);
 let mut current=initial;current.apply_run_args(&shared);
 if format!("{current:?}")!=format!("{expected:?}"){return Err("Run configuration changed from original native assignments".into());}
 let mut output=serde_json::json!({"same_as_legacy":true,"algorithm_debug":format!("{:?}",current.algo),"seeds_per_split":current.algo.seeds.seed_count().to_string(),"debug":current.debug,"reset":current.reset,"reprocess":current.reprocess,"years":expected_years,"output_dir":expected_output,"native_debug":format!("{shared:?}")});
 if let Some(graph)=input.get("graph"){
  let adjacency:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;let population:Vec<i64>=serde_json::from_value(graph["population"].clone())?;let ids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;let centroids:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(graph["edges"].clone())?;
  let weights=if current.algo.weights.geographic{edges.into_iter().map(|(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
  let tolerance=current.effective_balance_tolerance();let mut attempts=Vec::new();
  for attempt in 0..=bisect_runner::balance_retry::MAX_BALANCE_RETRIES{
   let seed=bisect_runner::balance_retry::balance_retry_seed(current.algo.metis.seed,attempt);
   let plan=match current.algo.split{
    bisect_cli::runner::SplitStrategy::GeoSection=>bisect_runner::bisection_runner::run_geosection_seeded(&adjacency,&population,&weights,2,tolerance,current.algo.metis.niter as u32,current.algo.seeds.seed_count(),None,&HashMap::new(),0.,None,1.1,None,0.,None,seed.unwrap_or(1))?.0,
    bisect_cli::runner::SplitStrategy::CentroidalVoronoi{n_iter,metric}=>bisect_runner::bisection_runner::run_all_splits_cvd(&adjacency,&population,2,tolerance,None,n_iter,seed.unwrap_or(0),metric,&centroids)?,
    _=>return Err("Independent Run graph dispatch covers GeoSection and CVD".into())
   };
   let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,2,tolerance).is_ok();
   let assignments:BTreeMap<_,_>=ids.iter().cloned().enumerate().map(|(i,id)|(id,plan[&i])).collect();
   attempts.push(serde_json::json!({"attempt":attempt,"seed":seed.map(|s|s.to_string()),"balanced":balanced,"assignments":assignments}));if balanced{break;}
  }
  output["attempts"]=serde_json::json!(attempts);output["balance_tolerance_fraction"]=serde_json::json!(tolerance);
 }
 println!("{output}");Ok(())
}
