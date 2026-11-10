//! Native YAML conversion plus native dispatch, preserving Option<u64>.
use bisect_cli::algo_config::{AlgoYaml,NativeAlgorithmConfig};
use bisect_cli::runner::{SeedCompositor,SplitStrategy};
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let path=std::env::args().nth(1).ok_or("Request required")?;
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
 let yaml=AlgoYaml::from_text(v["yaml_text"].as_str().ok_or("YAML required")?)?;
 let algo=yaml.to_algorithm_config()?;
 let k=v["options"]["districts"].as_u64().ok_or("Districts required")? as usize;
 let metis_used=k!=1&&!matches!(algo.split,SplitStrategy::MovingKnife{..}|SplitStrategy::Spectral{..}|SplitStrategy::Regionalization|SplitStrategy::FlowConstruction);
 if metis_used&&cfg!(feature="c-ffi-engine"){return Err("Build this portable parity reference with --no-default-features; C METIS cannot serve as a metis-core oracle".into());}
 if metis_used&&!matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis){return Err("Portable parity reference requires an explicit metis-core declaration".into());}
 let a:Vec<Vec<usize>>=serde_json::from_value(v["graph"]["adjacency"].clone())?;
 let p:Vec<i64>=serde_json::from_value(v["graph"]["population"].clone())?;
 let edges:Vec<(usize,usize,f64)>=serde_json::from_value(v["graph"]["edges"].clone())?;
 let physical:HashMap<_,_>=edges.iter().map(|&(u,w,l)|((u,w),l)).collect();
 let ids:Vec<String>=serde_json::from_value(v["graph"]["geoids"].clone())?;
 // Follow native State geographic base and county composer dispatch.
 let mut e=if algo.weights.geographic{physical.clone()}else{HashMap::new()};
 let fractions:Option<Vec<f64>>=if v["demographics"].is_object(){
  Some(ids.iter().map(|id|v["demographics"]["minority_fractions"][id].as_f64().ok_or("Missing explicit demographic fraction")).collect::<Result<Vec<_>,_>>()?)
 }else{None};
 if algo.weights.minority_weighting&&k>1{
  use bisect_cli::edge_weights::{EdgeWeighter,MinorityOverrideWeighter};
  e=MinorityOverrideWeighter::new(edges.iter().map(|&(u,w,_)|(u,w)).collect(),fractions.as_ref().ok_or("Demographics required for minority weights")?.clone(),0.40).apply(e);
 }
 if algo.weights.alpha_county>1e-10&&k>1{
  use bisect_cli::edge_weights::{EdgeWeighter,SubdivisionWeighter};
  let index=ids.iter().cloned().enumerate().collect::<HashMap<_,_>>();
  e=SubdivisionWeighter::county_only(&index,ids.len(),algo.weights.alpha_county).apply(e);
 }
 // Explicit shares model the native State partisan override after geographic input.
 if v["partisan"].is_object()&&k>1{
  use bisect_cli::edge_weights::{EdgeWeighter,PartisanOverrideWeighter};
  let shares=ids.iter().map(|id|v["partisan"]["dem_shares"][id].as_f64().ok_or("Missing explicit partisan share")).collect::<Result<Vec<_>,_>>()?;
  e=PartisanOverrideWeighter::new(edges.iter().map(|&(u,w,_)|(u,w)).collect(),shares,v["options"]["dem_threshold"].as_f64().ok_or("Democratic threshold required")?,v["options"]["rep_threshold"].as_f64().ok_or("Republican threshold required")?).apply(e);
 }
 let t=yaml.run_balance_tolerance()?.unwrap_or(0.005);
 let seed=if let Some(attempt)=v["attempt"].as_u64() {
  if attempt>bisect_runner::balance_retry::MAX_BALANCE_RETRIES as u64 {return Err("Attempt exceeds native retry policy".into());}
  bisect_runner::balance_retry::balance_retry_seed(algo.metis.seed,attempt as u32)
 } else {algo.metis.seed};let it=u32::try_from(v["options"]["iterations"].as_u64().unwrap_or(algo.metis.niter as u64))?;
 let plan=match algo.split {
  SplitStrategy::Bisect=>match algo.seeds {
   SeedCompositor::Single=>bisect_runner::bisection_runner::run_all_splits(&a,&p,&e,k,t,it,seed,None)?,
   SeedCompositor::Multi{seeds}=>bisect_runner::bisection_runner::run_all_splits_multi(&a,&p,&e,k,t,it,seed.unwrap_or(0),seeds,None)?,
   SeedCompositor::ConvergenceSweep {threshold}=>bisect_runner::bisection_runner::run_all_splits_convergence(&a,&p,&e,k,t,it,seed.unwrap_or(0),threshold as usize,None)?.0,
   SeedCompositor::ShortBurst {burst_length,n_bursts,p:percentile}=>bisect_runner::bisection_runner::run_short_burst(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,percentile)?.0,
   SeedCompositor::ShortBurstForest {burst_length,n_bursts,p:percentile}=>bisect_runner::bisection_runner::run_short_burst_forest(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,percentile)?,
   SeedCompositor::ShortBurstMergeSplit {burst_length,n_bursts,p:percentile}=>bisect_runner::bisection_runner::run_short_burst_merge_split(&a,&p,&e,k,t,it,seed.unwrap_or(0),burst_length,n_bursts,percentile)?,
   _=>return Err("Unsupported reference search".into()),
  },
  SplitStrategy::NWay=>bisect_runner::bisection_runner::run_nway_partition(&a,&p,&e,k,1.0+t,it,seed)?,
  SplitStrategy::ApportionRegions=>{
   use bisect_apportion::{pfr_tree_depth,MetisPartitioner,PfrCompositor};
   let depth=pfr_tree_depth(k as u32).max(1);
   let compositor=PfrCompositor::new(MetisPartitioner{balance_tolerance:(t/(depth+1) as f64).max(0.001),niter:it as i32,engine:algo.metis.engine});
   let result=compositor.compose(&a,&p,&e,k as u32,seed)?;
   let assignments:HashMap<usize,usize>=result.assignment.into_iter().enumerate().map(|(i,d)|(i,d as usize+1)).collect();
   if bisect_core::Partition::from_assignments(assignments.clone()).population_balance(&p,k)>0.03 {return Err("apportion-regions exceeds native 3% research limit".into());}
   assignments
  },
  SplitStrategy::GeoSection|SplitStrategy::AreaSection{..}=>{
   let areas:Vec<f64>=serde_json::from_value(v["graph"]["areas"].clone())?;
   let swing=match algo.split {SplitStrategy::AreaSection{area_swing,..}=>Some(area_swing),_=>None};
   let seeds=algo.seeds.seed_count();
   bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,seeds,None,&HashMap::new(),0.0,swing.map(|_|areas.as_slice()),swing.unwrap_or(1.1),None,0.0,None,seed.unwrap_or(1))?.0
  },
  SplitStrategy::VraSection{w_vra}=>{
   let minority=fractions.as_ref().map(|values|values.iter().zip(&p).map(|(&fraction,&pop)|fraction*pop as f64).collect::<Vec<_>>());
   bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&e,k,t,it,algo.seeds.seed_count(),None,&HashMap::new(),0.0,None,1.10,minority.as_deref(),w_vra,None,seed.unwrap_or(1))?.0
  },
  SplitStrategy::MovingKnife{n_orientations,metric}=>{
   let centroids:Vec<(f64,f64)>=serde_json::from_value(v["graph"]["centroids"].clone())?;
   if centroids.is_empty(){return Err("Native moving-knife requires centroid data".into());}
   let scoring=if metric=="polsby"{bisect_runner::bisection_runner::MkaMetric::PolsbyPopper}else{bisect_runner::bisection_runner::MkaMetric::Reock};
   bisect_runner::bisection_runner::run_all_splits_mka(&a,&p,k,t,None,n_orientations,scoring,seed.unwrap_or(0),&centroids)?
  },
  SplitStrategy::CompactBisect{epsilon}=>{
   let areas:Vec<f64>=serde_json::from_value(v["graph"]["areas"].clone())?;
   let exterior:Vec<f64>=serde_json::from_value(v["graph"]["exterior_perimeters"].clone())?;
   let opts=bisect_runner::bisection_runner::CompactBisectOpts{seeds_per_level:algo.seeds.seed_count(),epsilon};
   bisect_runner::bisection_runner::run_all_splits_compact(&a,&p,&e,&areas,&exterior,k,t,it,None,&opts,None)?
  },
  SplitStrategy::FlowConstruction=>{
   let result=bisect_flow::construct_flow(&a,&p,bisect_flow::FlowConfig::new(k,t))?;
   if result.status!=bisect_flow::FlowStatus::Valid {
    return Err(format!("[ALGO] flow construction did not produce a valid plan: {:?}",result.status).into());
   }
   result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()
  },
  SplitStrategy::Spectral{max_iters}=>{
   bisect_apportion::spectral::run_spectral_recursive(&a,&p,k,t,max_iters)?.0.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()
  },
  SplitStrategy::Regionalization=>{
   let result=bisect_clustering::regionalize(&a,&p,bisect_clustering::ClusterConfig{k,tolerance:t})?;
   if result.status!=bisect_clustering::ClusterStatus::Valid{
    return Err(format!("[ALGO] regionalization did not produce a valid plan: {:?}",result.status).into());
   }
   result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()
  },
  SplitStrategy::CapacityClustering=>{
   let result=bisect_clustering::capacity_cluster_repaired(&a,&p,bisect_clustering::ClusterConfig{k,tolerance:t})?;
   if result.status!=bisect_clustering::ClusterStatus::Valid{return Err(format!("[ALGO] capacity-clustering did not produce a valid plan: {:?}",result.status).into());}
   result.assignment.into_iter().enumerate().map(|(i,d)|(i,d+1)).collect()
  },
  _=>return Err("Unsupported reference structure".into()),
 };
 let balanced=bisect_core::partition::Partition::from_assignments(plan.clone()).assert_balanced(&p,k,t).is_ok();
 let weighted_cut:f64=edges.iter().filter(|&&(u,w,_)|plan[&u]!=plan[&w]).map(|&(u,w,_)|e.get(&(u,w)).copied().unwrap_or(1.0)).sum();
 let physical_cut:f64=edges.iter().filter(|&&(u,w,_)|plan[&u]!=plan[&w]).map(|&(_,_,length)|length).sum();
 let assignment:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignment,"declared_seed":seed.map(|s|s.to_string()),"balanced":balanced,"weighted_cut":weighted_cut,"physical_cut":physical_cut}));Ok(())
}
