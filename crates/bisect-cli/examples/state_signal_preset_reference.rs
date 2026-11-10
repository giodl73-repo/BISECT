//! Native State argument conversion, file readers, compositor and partition kernels.
//! This oracle does not call the browser boundary or its preset helper.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor};
use bisect_cli::edge_weights::{ComposedWeighter,GeographicWeighter,MinorityOverrideWeighter,PartisanOverrideWeighter,SubdivisionWeighter};
use clap::Parser;
use std::collections::{HashMap,BTreeMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let tokens:Vec<String>=serde_json::from_value(v["state_args"].clone())?;
 let args=match Cli::try_parse_from([vec!["bisect".into(),"state".into()],tokens].concat())?.command{Commands::State(a)=>a,_=>return Err("State required".into())};
 let algo=AlgorithmConfig::from_state_args(&args);
 let backend_matches=if cfg!(feature="c-ffi-engine"){matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::CFfi)}else{matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis)};
 if !backend_matches{return Err("Select the explicit METIS backend matching this native reference build.".into());}
 let g=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(g["population"].clone())?;let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;let index=ids.iter().cloned().enumerate().collect::<HashMap<_,_>>();
 let k=args.districts.ok_or("Districts required")?;let t=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;
 let mut composer=ComposedWeighter::new();
 if algo.weights.geographic{composer=composer.push(GeographicWeighter::from_map(edges.iter().map(|&(u,v,w)|((u,v),w)).collect()));}
 if algo.weights.minority_weighting{let demo=bisect_cli::demographics::load_demographics(std::path::Path::new(v["demographics_path"].as_str().ok_or("Demographics required")?))?;composer=composer.push(MinorityOverrideWeighter::new(edges.iter().map(|&(u,v,_)|(u,v)).collect(),bisect_cli::demographics::align_demographics_to_adjacency(&demo,&index,ids.len()),0.4));}
 if let Some(path)=&algo.weights.partisan_shares{let shares=bisect_cli::partisan_shares::load_partisan_shares(path,&index,ids.len())?;composer=composer.push(PartisanOverrideWeighter::new(edges.iter().map(|&(u,v,_)|(u,v)).collect(),shares,algo.weights.dem_threshold,algo.weights.rep_threshold));}
 if algo.weights.alpha_county>1e-10{composer=composer.push(SubdivisionWeighter::county_only(&index,ids.len(),algo.weights.alpha_county));}
 let weights=composer.apply();let it=algo.metis.niter as u32;
 let plan=if k==1{(0..ids.len()).map(|i|(i,1)).collect()}else{match (&algo.split,&algo.seeds){
  (SplitStrategy::NWay,_)=>bisect_runner::bisection_runner::run_nway_partition(&a,&p,&weights,k,1.+algo.metis.ufactor as f64/1000.,it,algo.metis.seed)?,
  (SplitStrategy::Bisect,SeedCompositor::Single)=>bisect_runner::bisection_runner::run_all_splits(&a,&p,&weights,k,t,it,algo.metis.seed,None)?,
  (SplitStrategy::Bisect,SeedCompositor::Multi{seeds})=>bisect_runner::bisection_runner::run_all_splits_multi(&a,&p,&weights,k,t,it,algo.metis.seed.unwrap_or(0),*seeds,None)?,
  (SplitStrategy::ApportionRegions,_)=>{
   let depth=bisect_apportion::pfr_tree_depth(k as u32).max(1);
   let partitioner=bisect_apportion::MetisPartitioner{balance_tolerance:(t/(depth+1) as f64).max(0.001),niter:it as i32,engine:algo.metis.engine};
   bisect_apportion::PfrCompositor::new(partitioner).compose(&a,&p,&weights,k as u32,algo.metis.seed)?.assignment.into_iter().enumerate().map(|(i,d)|(i,d as usize+1)).collect()
  },
  (SplitStrategy::GeoSection|SplitStrategy::AreaSection{..},_)=>{
   let areas:Vec<f64>=serde_json::from_value(g["areas"].clone())?;
   let (plan,_,_,_)=bisect_runner::bisection_runner::run_geosection_seeded(&a,&p,&weights,k,t,it,algo.seeds.seed_count(),None,&HashMap::new(),0.,if matches!(algo.split,SplitStrategy::AreaSection{..}){Some(&areas)}else{None},1.1,None,0.4,None,algo.metis.seed.unwrap_or(0))?;plan
  },
  _=>return Err("Oracle covers Bisect Single/Multi, NWay, prime-factor and Geo/AreaSection".into())
 }};
 if matches!(algo.split,SplitStrategy::ApportionRegions){
  let deviation=bisect_core::Partition::from_assignments(plan.clone()).population_balance(&p,k);
  if deviation>0.03{return Err(format!("apportion-regions balance {:.1}% exceeds 3% research limit",deviation*100.).into());}
 }
 let boundary=edges.iter().filter(|&&(u,v,_)|plan[&u]!=plan[&v]).map(|&(u,v,_)|weights.get(&(u,v)).copied().unwrap_or(1.)).sum::<f64>();
 let native_vra=if matches!(algo.split,SplitStrategy::NWay)&&algo.weights.minority_weighting&&k>1{
  let demo=bisect_cli::demographics::load_demographics(std::path::Path::new(v["demographics_path"].as_str().ok_or("Demographics required")?))?;let fractions=bisect_cli::demographics::align_demographics_to_adjacency(&demo,&index,ids.len());let mass=fractions.iter().zip(&p).map(|(fraction,&population)|fraction*population as f64).collect::<Vec<_>>();let zeros=vec![0.0;ids.len()];let a=bisect_analysis::analyze_mm_districts(&plan,&p,&mass,&zeros,&zeros,0.50);
  let output=bisect_cli::output::VraAnalysis{mm_count:a.mm_count,mm_districts:a.mm_districts,districts:a.districts.into_iter().map(|d|bisect_cli::output::VraDistrict{district:d.district,pct_minority:d.pct_minority,pct_black:d.pct_black,pct_hispanic:d.pct_hispanic,is_mm:d.is_mm}).collect()};Some(serde_json::to_string_pretty(&output)?)
 }else{None};
 let native_assignment_json=bisect_report::assignment_output::serialize_native_assignments(plan.iter().map(|(&u,&d)|(u,d)))?;
 let assignments=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect::<BTreeMap<_,_>>();
 println!("{}",serde_json::json!({"assignments":assignments,"native_vra_json":native_vra,"native_assignment_json":native_assignment_json,"weighted_boundary":boundary,"effective":{"structure":match &algo.split{SplitStrategy::NWay=>"nway",SplitStrategy::ApportionRegions=>"prime-factor",SplitStrategy::GeoSection=>"ratio-optimal",SplitStrategy::AreaSection{..}=>"ratio-optimal-area",_=>"standard-bisect"},"geographic":algo.weights.geographic,"minority":algo.weights.minority_weighting,"partisan":algo.weights.partisan_shares.is_some(),"alpha_county":algo.weights.alpha_county,"iterations":it,"ufactor":algo.metis.ufactor,"seed":algo.metis.seed.map(|s|s.to_string()),"candidates":match algo.seeds{SeedCompositor::Single=>1,SeedCompositor::Multi{seeds}=>seeds,_=>0}}}));Ok(())
}
