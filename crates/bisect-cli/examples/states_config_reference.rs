//! Native States parsing/configuration, frozen pre-extraction equivalence and
//! direct CompactBisect/GeoSection/AreaSection dispatch on identical supplied geometry vectors.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{StateConfig,SplitStrategy};
use bisect_runner::bisection_runner::{run_all_splits_compact,run_geosection_seeded,CompactBisectOpts};
use clap::Parser;
use std::collections::{BTreeMap,HashMap,HashSet};
mod legacy {include!("support/states_config_legacy.rs");}
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Use --no-default-features for the portable native States oracle".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["states_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"states".into()],tokens].concat())?.command{Commands::States(a)=>a,_=>return Err("States required".into())};
 let graph=&v["graph"];let state=graph["state"].as_str().unwrap_or("IA");let k=v["districts"].as_u64().unwrap_or(4) as usize;
 let initial=StateConfig::new_bulk(state.into(),"Supplied native graph".into(),k,args.year.to_string(),args.version.clone(),args.output_dir.clone().into(),2);
 let mut frozen=initial.clone();legacy::legacy_apply(&mut frozen,&args);
 let mut current=initial;current.apply_states_args(&args);
 let same_as_legacy=format!("{current:?}")==format!("{frozen:?}");
 // Only CVD overrides intentionally differ from the frozen bulk implementation.
 // Resolve their expected split through the independent single-State parser.
 let mut expected=frozen.clone();
 if matches!(expected.algo.split,SplitStrategy::CentroidalVoronoi{..}) {
  let state_args=match Cli::try_parse_from(["bisect","state","--state",state,"--partition-mode","centroidal-voronoi","--cvd-iters",&args.cvd_iters.to_string(),"--cvd-metric",&args.cvd_metric])?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
  expected.algo.split=bisect_cli::runner::AlgorithmConfig::from_state_args(&state_args).split;
 }
 if format!("{current:?}")!=format!("{expected:?}"){return Err("States configuration differs beyond the reviewed CVD correction".into());}
 let algo=&current.algo;
 let mut out=serde_json::json!({"same_as_legacy":same_as_legacy,"cvd_correction_applied":!same_as_legacy,"conforms_to_reviewed_correction":true,"algorithm_debug":format!("{algo:?}"),"seeds_per_split":algo.seeds.seed_count(),"compositor_seed":algo.metis.seed.map(|s|s.to_string()),"iterations":algo.metis.niter,"geographic":algo.weights.geographic,"balance_tolerance":current.balance_tolerance,"multiscale_fine":current.multiscale_fine,"multiscale_coarse":current.multiscale_coarse,"smc_resample_threshold":current.smc_resample_threshold});
 if !graph.is_null(){
  if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Portable native States must select metis-core".into());}
  let a:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(graph["population"].clone())?;let ids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(graph["edges"].clone())?;let areas:Vec<f64>=serde_json::from_value(graph["areas"].clone())?;let perimeters:Vec<f64>=serde_json::from_value(graph["exterior_perimeters"].clone())?;
  let weights=if algo.weights.geographic{edges.iter().map(|&(u,v,w)|((u,v),w)).collect()}else{HashMap::new()};
  let t=current.effective_balance_tolerance();let it=algo.metis.niter as u32;let count=algo.seeds.seed_count();
  let plan=match algo.split{
   SplitStrategy::CompactBisect{epsilon}=>run_all_splits_compact(&a,&p,&weights,&areas,&perimeters,k,t,it,None,&CompactBisectOpts{seeds_per_level:count,epsilon},None)?,
   SplitStrategy::AreaSection{area_swing,area_section_init}=>{
    let centroids:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;
    let moving=area_section_init==bisect_cli::runner::AreaSectionInit::MovingKnife;
    let theta=if moving{Some(bisect_runner::bisection_runner::split_subgraph_mka_direction(&(0..a.len()).collect::<HashSet<_>>(),&centroids,180))}else{None};
    let centroid_map=if moving{centroids.iter().enumerate().map(|(i,&c)|(i,c)).collect::<HashMap<_,_>>()}else{HashMap::new()};
    out["area_swing"]=serde_json::json!(area_swing);out["area_init"]=serde_json::json!(if moving{"moving-knife"}else{"ratio-optimal"});
    match run_geosection_seeded(&a,&p,&weights,k,t,it,count,None,&centroid_map,if moving{1.}else{0.},Some(&areas),area_swing,None,0.,theta,algo.metis.seed.unwrap_or(1)){Ok(plan)=>plan.0,Err(error)=>{out["dispatch_error"]=serde_json::json!(error);println!("{out}");return Ok(());}}
   },
   SplitStrategy::CentroidalVoronoi{n_iter,metric}=>{
    let centroids:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;
    out["cvd_iters"]=serde_json::json!(n_iter);out["cvd_metric"]=serde_json::json!(if metric==bisect_runner::bisection_runner::VoronoiMetric::Geographic{"geographic"}else{"graph-distance"});
    bisect_runner::bisection_runner::run_all_splits_cvd(&a,&p,k,t,None,n_iter,algo.metis.seed.unwrap_or(0),metric,&centroids)?
   },
   SplitStrategy::GeoSection=>run_geosection_seeded(&a,&p,&weights,k,t,it,count,None,&HashMap::new(),0.,None,1.1,None,0.,None,algo.metis.seed.unwrap_or(1))?.0,
   _=>return Err("Graph dispatch in this oracle covers CompactBisect, GeoSection, AreaSection and CVD".into())
  };
  let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();out["assignments"]=serde_json::json!(assignments);out["balanced"]=serde_json::json!(balanced);
 }
 println!("{out}");Ok(())
}
