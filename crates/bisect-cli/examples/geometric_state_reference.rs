//! Independent State parsing and CVD/MKA/SA/n-way first-attempt dispatch.
//! Uses the CLI conversion, not the WASM request adapter. No filesystem retries.
use bisect_cli::args::{Cli, Commands};
use bisect_cli::runner::{AlgorithmConfig, SplitStrategy};
use bisect_runner::bisection_runner::{run_all_splits_cvd, run_all_splits_mka, run_all_splits_sa, run_nway_partition, MkaMetric, VoronoiMetric};
use clap::Parser;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "c-ffi-engine") { return Err("Build with --no-default-features".into()); }
    let input: serde_json::Value = serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
    let tokens: Vec<String> = serde_json::from_value(input["state_args"].clone())?;
    let args = match Cli::try_parse_from([vec!["bisect".into(), "state".into()], tokens].concat())?.command {
        Commands::State(a) => a, _ => return Err("State required".into()),
    };
    let algo = AlgorithmConfig::from_state_args(&args);
    let g = &input["graph"];
    let adjacency: Vec<Vec<usize>> = serde_json::from_value(g["adjacency"].clone())?;
    let population: Vec<i64> = serde_json::from_value(g["population"].clone())?;
    let ids: Vec<String> = serde_json::from_value(g["geoids"].clone())?;
    let centers: Vec<(f64,f64)> = serde_json::from_value(g["centroids"].clone())?;
    let k = args.districts.ok_or("Districts required")?;
    let tolerance = args.balance_tolerance.ok_or("Tolerance required")? / 100.;
    let seed = algo.metis.seed.unwrap_or(0);
    let (plan, resolved) = match algo.split {
        SplitStrategy::CentroidalVoronoi { n_iter, metric } => {
            let name = if metric == VoronoiMetric::Geographic { "geographic" } else { "graph-distance" };
            (run_all_splits_cvd(&adjacency, &population, k, tolerance, None, n_iter, seed, metric, &centers)?,
             serde_json::json!({"structure":"centroidal-voronoi","count":n_iter,"metric":name}))
        },
        SplitStrategy::MovingKnife { n_orientations, metric } => {
            let effective = if metric == "polsby" { MkaMetric::PolsbyPopper } else { MkaMetric::Reock };
            (run_all_splits_mka(&adjacency, &population, k, tolerance, None, n_orientations, effective, seed, &centers)?,
             serde_json::json!({"structure":"moving-knife","count":n_orientations,"metric":metric}))
        },
        SplitStrategy::SimulatedAnnealing { steps_per_tract, t0_factor, t_final } => {
            let edges: Vec<(usize,usize,f64)> = serde_json::from_value(g["edges"].clone())?;
            let weights = if algo.weights.geographic { edges.into_iter().map(|(u,v,w)|((u,v),w)).collect() } else { std::collections::HashMap::new() };
            (run_all_splits_sa(&adjacency,&population,&weights,k,tolerance,algo.metis.niter,algo.metis.seed,None,steps_per_tract,t0_factor,t_final,seed)?,
             serde_json::json!({"structure":"simulated-annealing","steps_per_tract":steps_per_tract,"t0_factor":t0_factor,"t_final":t_final,"niter":algo.metis.niter}))
        },
        SplitStrategy::NWay => {
            let edges: Vec<(usize,usize,f64)> = serde_json::from_value(g["edges"].clone())?;
            let weights = if algo.weights.geographic { edges.into_iter().map(|(u,v,w)|((u,v),w)).collect() } else { std::collections::HashMap::new() };
            let plan = if k == 1 { (0..population.len()).map(|i|(i,1)).collect() } else {
                run_nway_partition(&adjacency,&population,&weights,k,1.0+algo.metis.ufactor as f64/1000.0,algo.metis.niter,algo.metis.seed)?
            };
            (plan,serde_json::json!({"structure":"nway","ufactor":algo.metis.ufactor,"niter":algo.metis.niter,"seed":algo.metis.seed.map(|s|s.to_string())}))
        },
        _ => return Err("CVD, MKA, SA or n-way required".into()),
    };
    let balanced = bisect_core::Partition::from_assignments(plan.clone()).assert_balanced(&population, k, tolerance).is_ok();
    let assignments: BTreeMap<_,_> = ids.into_iter().enumerate().map(|(i,id)| (id,plan[&i])).collect();
    println!("{}", serde_json::json!({"assignments":assignments,"balanced":balanced,"resolved":resolved,"effective_seed":seed.to_string()}));
    Ok(())
}
