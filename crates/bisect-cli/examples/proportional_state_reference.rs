//! Actual State conversion/dispatch, with explicitly supplied election counts.
use bisect_cli::args::{Cli, Commands};
use bisect_cli::runner::{AlgorithmConfig, SplitStrategy};
use clap::Parser;
use std::collections::{BTreeMap, HashMap};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature="c-ffi-engine") { return Err("Build with --no-default-features".into()); }
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
    let tokens: Vec<String> = serde_json::from_value(v["state_args"].clone())?;
    let args = match Cli::try_parse_from([vec!["bisect".to_string(), "state".to_string()], tokens].concat())?.command {
        Commands::State(args) => args, _ => return Err("State required".into())
    };
    let algo = AlgorithmConfig::from_state_args(&args);
    let eta = match algo.split { SplitStrategy::ProportionalSection {eta} => eta, _ => return Err("ProportionalSection required".into()) };
    if !matches!(algo.metis.engine,bisect_apportion::split::MetisEngine::RedistMetis) { return Err("metis-core required".into()); }
    if algo.weights.minority_weighting { return Err("Minority override is outside this oracle".into()); }
    let g=&v["graph"];
    let adjacency: Vec<Vec<usize>>=serde_json::from_value(g["adjacency"].clone())?;
    let population: Vec<i64>=serde_json::from_value(g["population"].clone())?;
    let ids: Vec<String>=serde_json::from_value(g["geoids"].clone())?;
    let edges: Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
    let weights: HashMap<_,_>=if algo.weights.geographic {edges.into_iter().map(|(u,v,w)|((u,v),if algo.weights.alpha_county>1e-10 && ids[u][..5]==ids[v][..5] {w*(1.+algo.weights.alpha_county)} else {w})).collect()} else {HashMap::new()};
    let dem: Vec<f64>=serde_json::from_value(v["dem"].clone())?;
    let two: Vec<f64>=serde_json::from_value(v["two_party"].clone())?;
    let k=v["districts"].as_u64().ok_or("Districts required")? as usize;
    let tolerance=args.balance_tolerance.ok_or("Explicit tolerance required")?/100.;
    let seeds=algo.seeds.seed_count();
    let (plan,left,right,cut,share)=bisect_runner::bisection_runner::run_proportional_section(&adjacency,&population,&dem,&two,&weights,k,tolerance,algo.metis.niter as u32,seeds,eta,None)?;
    let balanced=bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population,k,tolerance).is_ok();
    let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
    println!("{}",serde_json::json!({"assignments":assignments,"balanced":balanced,"eta":eta,"seeds":seeds,"left_seats":left,"right_seats":right,"root_cut":cut,"share":share,"effective_seed":algo.metis.seed}));
    Ok(())
}
