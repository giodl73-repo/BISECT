//! Resolve native YAML and execute its real AreaSection dispatcher parameters.
use bisect_cli::algo_config::{AlgoYaml, NativeAlgorithmConfig};
use bisect_cli::runner::{SeedCompositor, SplitStrategy};
use std::collections::HashMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("Request JSON required")?;
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let yaml = AlgoYaml::from_text(v["yaml_text"].as_str().ok_or("YAML required")?)?;
    let algo = yaml.to_algorithm_config()?;
    let SplitStrategy::AreaSection { area_swing, .. } = algo.split else { return Err("AreaSection required".into()); };
    let seeds = match algo.seeds { SeedCompositor::Single => 1, SeedCompositor::Multi { seeds } => seeds, _ => return Err("Single or multi required".into()) };
    let adjacency: Vec<Vec<usize>> = serde_json::from_value(v["graph"]["adjacency"].clone())?;
    let population: Vec<i64> = serde_json::from_value(v["graph"]["population"].clone())?;
    let areas: Vec<f64> = serde_json::from_value(v["graph"]["areas"].clone())?;
    let ids: Vec<String> = serde_json::from_value(v["graph"]["geoids"].clone())?;
    let edges: Vec<(usize, usize, f64)> = serde_json::from_value(v["graph"]["edges"].clone())?;
    let weights: HashMap<(usize, usize), f64> = edges.into_iter().map(|(u,w,length)| ((u,w),length)).collect();
    let (assignments, left, right, cut) = bisect_runner::bisection_runner::run_geosection_seeded(
        &adjacency, &population, &weights, 2, yaml.run_balance_tolerance()?.unwrap_or(0.005),
        algo.metis.niter, seeds, None, &HashMap::new(), 0.0, Some(&areas), area_swing,
        None, 0.0, None, algo.metis.seed.unwrap_or(1),
    )?;
    let assignments: std::collections::BTreeMap<_,_> = ids.into_iter().enumerate().map(|(i,id)| (id,assignments[&i])).collect();
    println!("{}", serde_json::json!({"assignments":assignments,"left":left,"right":right,"cut":cut,"area_swing":area_swing}));
    Ok(())
}
