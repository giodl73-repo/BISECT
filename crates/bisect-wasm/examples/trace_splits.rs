//! Native diagnostic for the same pure-Rust splits used by the browser engine.
use bisect_runner::bisection_runner::split_subgraph;
use bisect_wasm::engine::{execute, Options, PreparedGraph, Request};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args=std::env::args().skip(1);
    let input=args.next().ok_or("graph JSON path required")?;
    let districts:usize=args.next().ok_or("district count required")?.parse()?;
    if districts==0 { return Err("district count must be positive".into()); }
    let graph:PreparedGraph=serde_json::from_slice(&std::fs::read(input)?)?;
    let edges:HashMap<_,_>=graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
    let mut queue=VecDeque::from([(String::new(),districts,(0..graph.population.len()).collect::<HashSet<_>>() )]);
    let mut splits=Vec::new();
    while let Some((path,k,vertices))=queue.pop_front() {
        if k==1 { continue; }
        let left_k=k/2;let right_k=k-left_k;
        let target=if left_k==right_k {None}else{
            let w=left_k as f32/k as f32;Some(vec![w,1.0-w])
        };
        let population:i64=vertices.iter().map(|&v|graph.population[v]).sum();
        let ufactor=1.0+0.05/k as f64;
        let (left,right)=split_subgraph(&graph.adjacency,&graph.population,1,&edges,&vertices,ufactor,10,Some(42),target,None)?;
        let left_pop:i64=left.iter().map(|&v|graph.population[v]).sum();
        let right_pop=population-left_pop;
        let left_target=population as f64*left_k as f64/k as f64;
        let right_target=population as f64*right_k as f64/k as f64;
        let deviation=((left_pop as f64-left_target).abs()/left_target)
            .max((right_pop as f64-right_target).abs()/right_target)*100.0;
        splits.push(json!({"path":path,"districts":k,"vertices":vertices.len(),"population":population,
            "left_population":left_pop,"right_population":right_pop,"max_deviation_percent":deviation,
            "requested_node_tolerance_percent":(ufactor-1.0)*100.0}));
        queue.push_back((format!("{path}0"),left_k,left));queue.push_back((format!("{path}1"),right_k,right));
    }
    let result=execute(Request {demographics:None,graph,options:Options {metis_objective:None,metis_trials:None,w_vra:None,vra_threshold:None,structure:"standard-bisect".into(),weights:"geographic".into(),
        search:"single".into(),districts,seed:42,seeds:1,steps:20,percentile:0.0,alpha_county:0.0,
        balance_tolerance:5.0,area_swing:1.1,iterations:10,sa_steps_per_tract:None,sa_t0_factor:None,sa_t_final:None,flow_repair:None,smc_particles:None,smc_resample_threshold:None,cvd_iters:None,cvd_metric:None,mka_orientations:None,mka_metric:None,compact_epsilon:None,area_init:None,burst_length:None,n_bursts:None,pt_replicas:None,pt_swap_interval:None,pt_cold_tol:None,pt_hot_tol:None}})?;
    println!("{}",serde_json::to_string(&json!({"splits":splits,"result":result}))?);
    Ok(())
}
