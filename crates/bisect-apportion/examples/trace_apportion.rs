//! Observe actual PFR calls and the upstream root contiguity repair without
//! changing the selected algorithm, targets, seed or acceptance guard.
use bisect_apportion::{ MetisPartitioner, Partitioner, PfrCompositor, SplitError, SubGraph, pfr_tree_depth};
use bisect_apportion::split::MetisEngine;
#[derive(serde::Deserialize)]
struct PreparedGraph { adjacency:Vec<Vec<usize>>, population:Vec<i64>, edges:Vec<(usize,usize,f64)> }
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use sha2::{Digest,Sha256};
#[path="../src/connected_balance.rs"]
mod connected_balance;

struct Trace { inner:MetisPartitioner, experimental:bool, calls:Mutex<Vec<serde_json::Value>> }
impl Trace {
 fn record(&self,g:&SubGraph,targets:&[f32],seed:Option<u64>,a:&[u32]) {
  let mut population=vec![0i64;targets.len()];for (v,&d) in a.iter().enumerate(){population[d as usize]+=i64::from(g.vwgt[v]);}
  let total:i64=population.iter().sum();let deviation=population.iter().zip(targets).map(|(&p,&f)|(p as f64/(total as f64*f as f64)-1.).abs()*100.).fold(0.,f64::max);
  self.calls.lock().unwrap().push(json!({"vertices":g.n_vertices(),"parts":targets.len(),"seed":seed.map(|s|s.to_string()),"target_fractions":targets,"part_population":population,"max_deviation_percent":deviation,"counterexample":if deviation>3.{Some(json!({"global_ids":g.global_ids,"xadj":g.xadj,"adjncy":g.adjncy,"vwgt":g.vwgt,"adjwgt":g.adjwgt}))}else{None}}));
 }
}
impl Partitioner for Trace {
 fn split(&self,g:&SubGraph,k:u32,seed:Option<u64>)->Result<Vec<u32>,SplitError>{let mut a=self.inner.split(g,k,seed)?;if self.experimental{a=connected_balance::refine(&g.to_metis_core_graph().map_err(|e|SplitError::Metis(e.to_string()))?,a,&vec![1;k as usize],(self.inner.balance_tolerance*1000.).max(1.)as u32,self.inner.niter as u32);}self.record(g,&vec![1./k as f32;k as usize],seed,&a);Ok(a)}
 fn split_weighted(&self,g:&SubGraph,f:&[f32],seed:Option<u64>)->Result<Vec<u32>,SplitError>{let mut a=self.inner.split_weighted(g,f,seed)?;if self.experimental{let shares:Vec<_>=f.iter().map(|&v|(v*1000.).round()as u32).collect();a=connected_balance::refine(&g.to_metis_core_graph().map_err(|e|SplitError::Metis(e.to_string()))?,a,&shares,(self.inner.balance_tolerance*1000.).max(1.)as u32,self.inner.niter as u32);}self.record(g,f,seed,&a);Ok(a)}
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let graph_bytes=std::fs::read(args.get(1).ok_or("Prepared graph required")?)?;let graph_sha256=format!("{:x}",Sha256::digest(&graph_bytes));let graph:PreparedGraph=serde_json::from_slice(&graph_bytes)?;
 let k:u32=args.get(2).ok_or("District count required")?.parse()?;let seed:u64=args.get(3).ok_or("Seed required")?.parse()?;
 if k==0{return Err("District count must be positive".into());}
 let depth=pfr_tree_depth(k).max(1);let tolerance=(0.005/(depth+1) as f64).max(0.001);
 let experimental=args.iter().any(|a|a=="--connected-balance");
 let trace=Trace{inner:MetisPartitioner{engine:MetisEngine::RedistMetis,balance_tolerance:tolerance,niter:10},experimental,calls:Mutex::new(Vec::new())};
 let compositor=PfrCompositor::new(trace);let edges:HashMap<_,_>=graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
 let result=compositor.compose(&graph.adjacency,&graph.population,&edges,k,Some(seed))?;
 let mut final_population=vec![0i64;k as usize];for(v,&d)in result.assignment.iter().enumerate(){final_population[d as usize]+=graph.population[v];}
 let total:i64=final_population.iter().sum();let ideal=total as f64/k as f64;
 let max_deviation=final_population.iter().map(|&p|(p as f64/ideal-1.).abs()*100.).fold(0.,f64::max);
 let vertices:Vec<_>=(0..graph.population.len()).collect();let root=SubGraph::build(&vertices,&graph.adjacency,&graph.population,&edges)?.to_metis_core_graph()?;
 use metis_core::{MetisParams,MetisPartitioner as Core,Partitioner as CorePartitioner};
 let root_k=bisect_apportion::split_prescription(k).parts();let root_seed=seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
 let raw=Core::with_params(MetisParams::kway().with_ufactor((tolerance*1000.) as u32).with_niter(10).with_coarsen_to(20).with_seed(root_seed),root_k).split(&root,root_k,Some(root_seed))?;
 let mut raw_population=vec![0i64;root_k as usize];for(v,&d)in raw.assignment().iter().enumerate(){raw_population[d as usize]+=graph.population[v];}
 println!("{}",json!({"seed":seed.to_string(),"districts":k,"graph_sha256":graph_sha256,"experimental_connected_balance":experimental,"split_population_basis":"native positive CSR weights, including zero-population clamp","final_population_basis":"original prepared Census population","node_tolerance_fraction":tolerance,"calls":compositor.strategy.calls.into_inner().unwrap(),"final_population":final_population,"max_deviation_percent":max_deviation,"raw_root_population":raw_population,"raw_root_contiguous":metis_core::check_contiguity(&root,&raw).is_ok()}));Ok(())
}
