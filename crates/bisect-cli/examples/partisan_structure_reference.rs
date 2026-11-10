//! Independent State conversion, native TSV loader/post-weighter and dispatch.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor};
use bisect_cli::edge_weights::{EdgeWeighter,PartisanOverrideWeighter};
use clap::Parser;
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build the portable State oracle with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 if matches!(algo.split,SplitStrategy::Bisect|SplitStrategy::GeoSection|SplitStrategy::AreaSection{..}|SplitStrategy::VraSection{..}|SplitStrategy::ProportionalSection{..}|SplitStrategy::NWay|SplitStrategy::SimulatedAnnealing{..}|SplitStrategy::CompactBisect{..}|SplitStrategy::ApportionRegions)&&!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Portable METIS oracle requires explicit metis-core".into());}
 let graph=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(graph["population"].clone())?;let ids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;let physical:Vec<(usize,usize,f64)>=serde_json::from_value(graph["edges"].clone())?;
 let shares=bisect_cli::partisan_shares::load_partisan_shares(algo.weights.partisan_shares.as_deref().ok_or("Native partisan source required")?,&ids.iter().cloned().enumerate().collect(),ids.len())?;
 let mut e=PartisanOverrideWeighter::new(physical.iter().map(|&(u,v,_)|(u,v)).collect(),shares,algo.weights.dem_threshold,algo.weights.rep_threshold).apply(HashMap::new());
 if let Some(path)=v["coi_path"].as_str(){e=bisect_cli::runner::apply_coi_weights(e,std::path::Path::new(path),&ids.iter().cloned().enumerate().collect())?;}
 let k=args.districts.ok_or("Explicit districts required")?;let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;let it=algo.metis.niter as u32;let seed=algo.metis.seed;
 let mut summary=None;let mut root=None;
 let plan=match algo.split{
  SplitStrategy::Bisect=>match algo.seeds{
   SeedCompositor::Flip{flip_steps,p:q}=>{let (plan,records,rank)=bisect_runner::bisection_runner::run_flip_chain(&a,&p,&e,k,t,flip_steps,seed.unwrap_or(0),q)?;summary=Some(serde_json::json!({"records":records,"rank":rank}));plan},
   SeedCompositor::ForestRecom{steps,p:q}=>bisect_runner::bisection_runner::run_forest_recom(&a,&p,&e,k,t,it,seed.unwrap_or(0),steps,q)?,
   SeedCompositor::MergeSplit{steps,p:q}=>bisect_runner::bisection_runner::run_merge_split(&a,&p,&e,k,t,it,seed.unwrap_or(0),steps,q)?,
   SeedCompositor::ShortBurst{burst_length,n_bursts,p:q}=>{let (plan,seeds,index)=bisect_runner::bisection_runner::run_short_burst(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,q)?;summary=Some(serde_json::json!({"burst_seeds":seeds.iter().map(|s|s.to_string()).collect::<Vec<_>>(),"selected_burst_index":index}));plan},
   SeedCompositor::ShortBurstForest{burst_length,n_bursts,p:q}=>bisect_runner::bisection_runner::run_short_burst_forest(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,q)?,
   SeedCompositor::ShortBurstMergeSplit{burst_length,n_bursts,p:q}=>bisect_runner::bisection_runner::run_short_burst_merge_split(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,q)?,
   SeedCompositor::ParallelTempering{n_replicas,swap_interval,cold_tolerance,hot_tolerance,steps,p:q}=>bisect_runner::bisection_runner::run_parallel_tempering(&a,&p,&e,k,it,seed.unwrap_or(0),n_replicas,swap_interval,cold_tolerance,hot_tolerance,steps,q)?,
   SeedCompositor::SmcPercentile{n_particles,p:q}=>bisect_runner::bisection_runner::run_smc_percentile(&a,&p,k,seed.unwrap_or(0),n_particles,q,args.smc_resample_threshold)?,
   SeedCompositor::VraRecom{steps,p:q,vap_threshold}=>{
    let fractions=if v["demographics_absent"].as_bool()==Some(true){vec![0.0;ids.len()]}else{
      let demo=bisect_cli::demographics::load_demographics(std::path::Path::new(v["demographics_path"].as_str().ok_or("Explicit demographic CSV required")?))?;
      bisect_cli::demographics::align_demographics_to_adjacency(&demo,&ids.iter().cloned().enumerate().collect(),ids.len())
    };
    let (plan,report)=bisect_runner::bisection_runner::run_vra_recom_detailed(&a,&p,&e,k,it,seed.unwrap_or(0),steps,q,vap_threshold,&fractions)?;summary=Some(serde_json::to_value(report)?);plan
   },
   _=>return Err("This oracle does not yet cover the selected standard search".into())
  },
  SplitStrategy::GeoSection=>{let (plan,left,right,cut)=bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,algo.seeds.seed_count(),None,&HashMap::new(),0.,None,0.,None,0.,None,seed.unwrap_or(1))?;root=Some(serde_json::json!({"left":left,"right":right,"cut":cut}));plan},
  SplitStrategy::AreaSection{area_swing,area_section_init}=>{
   let areas:Vec<f64>=serde_json::from_value(graph["areas"].clone())?;let points:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;
   let moving=matches!(area_section_init,bisect_cli::runner::AreaSectionInit::MovingKnife);
   let theta=if moving{Some(bisect_runner::bisection_runner::split_subgraph_mka_direction(&(0..p.len()).collect(),&points,180))}else{None};
   let centroids=if moving{points.into_iter().enumerate().collect()}else{HashMap::new()};
   let (plan,left,right,cut)=bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,algo.seeds.seed_count(),None,&centroids,if moving{1.}else{0.},Some(&areas),area_swing,None,0.,theta,seed.unwrap_or(1))?;
   root=Some(serde_json::json!({"left":left,"right":right,"cut":cut}));plan
  },
  SplitStrategy::VraSection{w_vra}=>{
   let mass=if let Some(path)=v["demographics_path"].as_str(){let demo=bisect_cli::demographics::load_demographics(std::path::Path::new(path))?;Some(bisect_cli::demographics::align_demographics_to_adjacency(&demo,&ids.iter().cloned().enumerate().collect(),ids.len()).into_iter().zip(&p).map(|(f,p)|f*(*p as f64)).collect::<Vec<_>>())}else{None};
   let (plan,left,right,cut)=bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,algo.seeds.seed_count(),None,&HashMap::new(),0.,None,1.10,mass.as_deref(),w_vra,None,seed.unwrap_or(1))?;
   root=Some(serde_json::json!({"left":left,"right":right,"cut":cut}));plan
  },
  SplitStrategy::ProportionalSection{eta}=>{
   let (dem,two)=bisect_cli::partisan_shares::load_dem_vote_counts(std::path::Path::new(v["election_path"].as_str().ok_or("Election CSV required")?),&ids.iter().cloned().enumerate().collect(),ids.len())?;
   let (plan,left,right,cut,share)=bisect_runner::bisection_runner::run_proportional_section(&a,&p,&dem,&two,&e,k,t,it,algo.seeds.seed_count(),eta,None)?;
   root=Some(serde_json::json!({"left":left,"right":right,"cut":cut,"statewide_share":share}));plan
  },
  SplitStrategy::Spectral{max_iters}=>{let (plan,s)=bisect_apportion::spectral::run_spectral_recursive(&a,&p,k,t,max_iters)?;summary=Some(s);plan.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()},
  SplitStrategy::NWay=>bisect_runner::bisection_runner::run_nway_partition(&a,&p,&e,k,1.+algo.metis.ufactor as f64/1000.,it,seed)?,
  SplitStrategy::SimulatedAnnealing{steps_per_tract,t0_factor,t_final}=>bisect_runner::bisection_runner::run_all_splits_sa(&a,&p,&e,k,t,it,seed,None,steps_per_tract,t0_factor,t_final,seed.unwrap_or(0))?,
  SplitStrategy::CompactBisect{epsilon}=>{let areas:Vec<f64>=serde_json::from_value(graph["areas"].clone())?;let exterior:Vec<f64>=serde_json::from_value(graph["exterior_perimeters"].clone())?;bisect_runner::bisection_runner::run_all_splits_compact(&a,&p,&e,&areas,&exterior,k,t,it,None,&bisect_runner::bisection_runner::CompactBisectOpts{seeds_per_level:algo.seeds.seed_count(),epsilon},None)?},
  SplitStrategy::BfsGrowth=>bisect_runner::bisection_runner::run_all_splits_bfs(&a,&p,k,t,None,seed.unwrap_or(0))?,
  SplitStrategy::CentroidalVoronoi{n_iter,metric}=>{let centroids:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;bisect_runner::bisection_runner::run_all_splits_cvd(&a,&p,k,t,None,n_iter,seed.unwrap_or(0),metric,&centroids)?},
  SplitStrategy::MovingKnife{n_orientations,metric}=>{let centroids:Vec<(f64,f64)>=serde_json::from_value(graph["centroids"].clone())?;let metric=if metric=="polsby"{bisect_runner::bisection_runner::MkaMetric::PolsbyPopper}else{bisect_runner::bisection_runner::MkaMetric::Reock};bisect_runner::bisection_runner::run_all_splits_mka(&a,&p,k,t,None,n_orientations,metric,seed.unwrap_or(0),&centroids)?},
  SplitStrategy::CapacityClustering=>{let result=bisect_clustering::capacity_cluster_repaired(&a,&p,bisect_clustering::ClusterConfig{k,tolerance:t})?;if result.status!=bisect_clustering::ClusterStatus::Valid{return Err(format!("[ALGO] capacity-clustering did not produce a valid plan: {:?}",result.status).into());}summary=Some(serde_json::to_value(result.summary)?);result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()},
  SplitStrategy::Regionalization=>{let result=bisect_clustering::regionalize(&a,&p,bisect_clustering::ClusterConfig{k,tolerance:t})?;if result.status!=bisect_clustering::ClusterStatus::Valid{return Err(format!("[ALGO] regionalization did not produce a valid plan: {:?}",result.status).into());}summary=Some(serde_json::json!({"method":"regionalization","summary":result.summary,"merge_log":result.merge_log}));result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()},
  SplitStrategy::FlowConstruction=>{let result=bisect_flow::construct_flow(&a,&p,bisect_flow::FlowConfig::new(k,t))?;if result.status!=bisect_flow::FlowStatus::Valid{return Err(format!("[ALGO] flow construction did not produce a valid plan: {:?}",result.status).into());}summary=Some(serde_json::to_value(result.summary)?);result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()},
  SplitStrategy::ApportionRegions=>{
   use bisect_apportion::{pfr_tree_depth,prime_factor_sequence,MetisPartitioner,PfrCompositor};
   let depth=pfr_tree_depth(k as u32).max(1);let per_level_tolerance=(t/(depth+1) as f64).max(0.001);
   let result=PfrCompositor::new(MetisPartitioner{balance_tolerance:per_level_tolerance,niter:it as i32,engine:algo.metis.engine}).compose(&a,&p,&e,k as u32,seed).map_err(|error|format!("apportion-regions failed: {error}"))?;
   let assignments:HashMap<_,_>=result.assignment.iter().enumerate().map(|(i,&d)|(i,d as usize+1)).collect();
   let balance=bisect_core::Partition::from_assignments(assignments.clone()).population_balance(&p,k);
   if balance>0.03{return Err(format!("apportion-regions balance {:.1}% exceeds 3% research limit",balance*100.).into());}
   summary=Some(serde_json::json!({"method":"apportion-regions","factor_sequence":prime_factor_sequence(k as u32),"tree_depth":depth,"per_level_tolerance":per_level_tolerance,"research_balance_limit_percent":3.0,"total_edge_cut_integer":result.total_edge_cut,"cache_hits":result.cache_hits,"seed":seed.map(|s|if s>9007199254740991{serde_json::json!(s.to_string())}else{serde_json::json!(s)}),"split_prescription":"largest-prime-first; prime > 3 uses floor/ceil binary"}));assignments
  },
  SplitStrategy::Ilp{method,fallback,time_limit_secs,optimality_gap,max_tracts}=>bisect_runner::bisection_runner::run_all_splits_ilp(&a,&p,&e,k,t,method,fallback,time_limit_secs,optimality_gap,max_tracts,None)?,
  _=>return Err("This oracle does not yet cover the selected native structure".into())
 };
 let weighted:f64=physical.iter().filter(|&&(u,v,_)|plan[&u]!=plan[&v]).map(|&(u,v,_)|e.get(&(u,v)).copied().unwrap_or(1.)).sum();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"summary":summary,"root":root,"weighted_boundary":weighted,"dem_threshold":algo.weights.dem_threshold,"rep_threshold":algo.weights.rep_threshold}));Ok(())
}
