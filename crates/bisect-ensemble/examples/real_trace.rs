use anyhow::{bail, Context, Result};
use bisect_data::deserialize_adjacency;
use bisect_ensemble::chain::chain_seed;
use bisect_ensemble::recom::{RecomChain, TreeSampler};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    state: String,
    adjacency: PathBuf,
    assignments: PathBuf,
    elections: PathBuf,
    steps: u64,
    chains: usize,
    tolerance: f64,
    base_seed: u64,
    snapshot_stride: u64,
    output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ElectionInput {
    election_input_version: String,
    state: String,
    year: u16,
    geoid_count: usize,
    unmatched_geoids: BTreeMap<String, usize>,
    democratic_2016: Vec<f64>,
    republican_2016: Vec<f64>,
    democratic_2020: Vec<f64>,
    republican_2020: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct MetricRecord {
    step: u64,
    accepted: bool,
    cut_edges: usize,
    cut_fraction: f64,
    pop_deviation: f64,
    democratic_seats_2016: usize,
    democratic_seats_2020: usize,
}

#[derive(Debug, Serialize)]
struct Snapshot {
    step: u64,
    assignment: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct ChainTrace {
    chain_idx: usize,
    seed: u64,
    metrics: Vec<MetricRecord>,
    snapshots: Vec<Snapshot>,
}

#[derive(Debug, Serialize)]
struct TraceOutput {
    trace_version: &'static str,
    implementation: &'static str,
    implementation_version: &'static str,
    tree_sampler: &'static str,
    state: String,
    steps_per_chain: u64,
    chains: usize,
    population_tolerance: f64,
    base_seed: u64,
    snapshot_stride: u64,
    adjacency_vertices: usize,
    adjacency_edges: usize,
    election_input_version: String,
    unmatched_geoids: BTreeMap<String, usize>,
    baseline: MetricRecord,
    chain_traces: Vec<ChainTrace>,
}

fn parse_args() -> Result<Args> {
    let mut values = HashMap::new();
    let mut it = env::args().skip(1);
    while let Some(key) = it.next() {
        if !key.starts_with("--") {
            bail!("unexpected positional argument: {key}");
        }
        let value = it
            .next()
            .with_context(|| format!("missing value for {key}"))?;
        values.insert(key, value);
    }
    let required = |name: &str| -> Result<String> {
        values
            .get(name)
            .cloned()
            .with_context(|| format!("missing required argument {name}"))
    };
    Ok(Args {
        state: required("--state")?,
        adjacency: PathBuf::from(required("--adjacency")?),
        assignments: PathBuf::from(required("--assignments")?),
        elections: PathBuf::from(required("--elections")?),
        steps: required("--steps")?.parse()?,
        chains: required("--chains")?.parse()?,
        tolerance: required("--tolerance")?.parse()?,
        base_seed: required("--base-seed")?.parse()?,
        snapshot_stride: required("--snapshot-stride")?.parse()?,
        output: PathBuf::from(required("--output")?),
    })
}

fn load_assignments(path: &PathBuf, n: usize) -> Result<Vec<u32>> {
    let raw: BTreeMap<String, u32> =
        serde_json::from_slice(&fs::read(path).with_context(|| path.display().to_string())?)?;
    let mut assignments = vec![0; n];
    for (key, district) in raw {
        let index: usize = key.parse()?;
        if index >= n {
            bail!("assignment index {index} exceeds graph size {n}");
        }
        assignments[index] = district;
    }
    if assignments.iter().any(|&district| district == 0) {
        bail!("assignments must cover every graph vertex with 1-based district labels");
    }
    Ok(assignments)
}

fn democratic_seats(assignment: &[u32], dem: &[f64], rep: &[f64], k: usize) -> usize {
    let mut district_dem = vec![0.0; k];
    let mut district_rep = vec![0.0; k];
    for (index, &district) in assignment.iter().enumerate() {
        let d = district.saturating_sub(1) as usize;
        district_dem[d] += dem[index];
        district_rep[d] += rep[index];
    }
    district_dem
        .iter()
        .zip(district_rep.iter())
        .filter(|(d, r)| d > r)
        .count()
}

fn metric(
    step: u64,
    accepted: bool,
    cut_edges: usize,
    cut_fraction: f64,
    pop_deviation: f64,
    assignment: &[u32],
    elections: &ElectionInput,
    k: usize,
) -> MetricRecord {
    MetricRecord {
        step,
        accepted,
        cut_edges,
        cut_fraction,
        pop_deviation,
        democratic_seats_2016: democratic_seats(
            assignment,
            &elections.democratic_2016,
            &elections.republican_2016,
            k,
        ),
        democratic_seats_2020: democratic_seats(
            assignment,
            &elections.democratic_2020,
            &elections.republican_2020,
            k,
        ),
    }
}

fn baseline_metric(
    adjacency: &[Vec<u32>],
    population: &[i64],
    assignment: &[u32],
    elections: &ElectionInput,
    k: usize,
) -> MetricRecord {
    let mut cut_edges = 0usize;
    for (node, neighbors) in adjacency.iter().enumerate() {
        for &neighbor in neighbors {
            if assignment[node] != assignment[neighbor as usize] {
                cut_edges += 1;
            }
        }
    }
    cut_edges /= 2;
    let total_edges = adjacency.iter().map(Vec::len).sum::<usize>() / 2;
    let ideal = population.iter().sum::<i64>() as f64 / k as f64;
    let mut district_pop = vec![0_i64; k];
    for (index, &district) in assignment.iter().enumerate() {
        district_pop[district.saturating_sub(1) as usize] += population[index];
    }
    let max_deviation = district_pop
        .iter()
        .map(|&value| (value as f64 - ideal).abs() / ideal)
        .fold(0.0, f64::max);
    metric(
        0,
        true,
        cut_edges,
        cut_edges as f64 / total_edges.max(1) as f64,
        max_deviation,
        assignment,
        elections,
        k,
    )
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let graph = deserialize_adjacency(&fs::read(&args.adjacency)?)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let adjacency: Vec<Vec<u32>> = graph
        .adjacency
        .iter()
        .map(|neighbors| neighbors.iter().map(|&v| v as u32).collect())
        .collect();
    let population = graph.vertex_weights;
    let initial = load_assignments(&args.assignments, adjacency.len())?;
    let k = initial.iter().copied().max().unwrap_or(0) as usize;
    if k < 2 {
        bail!("real ensemble trace requires at least two districts");
    }
    let elections: ElectionInput = serde_json::from_slice(&fs::read(&args.elections)?)?;
    if elections.state != args.state || elections.year != 2020 {
        bail!("election input state/year does not match trace request");
    }
    if elections.geoid_count != adjacency.len()
        || elections.democratic_2016.len() != adjacency.len()
        || elections.republican_2016.len() != adjacency.len()
        || elections.democratic_2020.len() != adjacency.len()
        || elections.republican_2020.len() != adjacency.len()
    {
        bail!("election arrays must match graph vertex count");
    }

    let baseline = baseline_metric(&adjacency, &population, &initial, &elections, k);
    let mut chain_traces = Vec::with_capacity(args.chains);
    for chain_idx in 0..args.chains {
        let seed = chain_seed(args.base_seed, chain_idx);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut chain = RecomChain::new_with_sampler(
            adjacency.clone(),
            population.clone(),
            initial.clone(),
            k as u32,
            args.tolerance,
            TreeSampler::GerryChainKruskal,
        );
        let mut metrics = Vec::with_capacity(args.steps as usize);
        let mut snapshots = Vec::new();
        for _ in 0..args.steps {
            let record = chain.step(&mut rng);
            metrics.push(metric(
                record.step,
                record.accepted,
                record.cut_edges,
                record.cut_fraction as f64,
                record.pop_deviation as f64,
                &chain.assignment,
                &elections,
                k,
            ));
            if record.step % args.snapshot_stride == 0 {
                snapshots.push(Snapshot {
                    step: record.step,
                    assignment: chain.assignment.clone(),
                });
            }
        }
        chain_traces.push(ChainTrace {
            chain_idx,
            seed,
            metrics,
            snapshots,
        });
    }

    let output = TraceOutput {
        trace_version: "g-real-rust-recom-trace v1",
        implementation: "bisect-ensemble",
        implementation_version: env!("CARGO_PKG_VERSION"),
        tree_sampler: "gerrychain-compatible-random-weight-kruskal",
        state: args.state,
        steps_per_chain: args.steps,
        chains: args.chains,
        population_tolerance: args.tolerance,
        base_seed: args.base_seed,
        snapshot_stride: args.snapshot_stride,
        adjacency_vertices: adjacency.len(),
        adjacency_edges: adjacency.iter().map(Vec::len).sum::<usize>() / 2,
        election_input_version: elections.election_input_version,
        unmatched_geoids: elections.unmatched_geoids,
        baseline,
        chain_traces,
    };
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.output, serde_json::to_vec(&output)?)?;
    Ok(())
}
