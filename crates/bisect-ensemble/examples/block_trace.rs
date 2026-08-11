use anyhow::{bail, Context, Result};
use bisect_ensemble::block_input::{load_block_ensemble_input, BlockEnsembleInput};
use bisect_ensemble::chain::chain_seed;
use bisect_ensemble::recom::{RecomChain, TreeSampler};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Serialize)]
struct Metric {
    step: u64,
    accepted: bool,
    cut_edges: usize,
    cut_fraction: f64,
    weighted_boundary_cut: f64,
    max_population_deviation: f64,
    runtime_ms: f64,
}

#[derive(Serialize)]
struct Snapshot {
    step: u64,
    assignment: Vec<u32>,
}

#[derive(Serialize)]
struct ChainOutput {
    chain_index: usize,
    seed: u64,
    metrics: Vec<Metric>,
    snapshots: Vec<Snapshot>,
}

#[derive(Serialize)]
struct TraceOutput {
    schema_version: &'static str,
    status: &'static str,
    execution_class: String,
    state: String,
    year: u16,
    units: usize,
    districts: u32,
    sampler: String,
    chains: usize,
    steps_per_chain: u64,
    population_tolerance: f64,
    base_seed: u64,
    snapshot_stride: u64,
    baseline: Metric,
    chain_traces: Vec<ChainOutput>,
    claim_boundary: &'static str,
}

fn parse_args() -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        if !key.starts_with("--") {
            bail!("unexpected positional argument {key}");
        }
        let value = args
            .next()
            .with_context(|| format!("missing value for {key}"))?;
        values.insert(key, value);
    }
    Ok(values)
}

fn required(values: &HashMap<String, String>, key: &str) -> Result<String> {
    values
        .get(key)
        .cloned()
        .with_context(|| format!("missing required argument {key}"))
}

#[allow(clippy::too_many_arguments)]
fn validate_execution(
    execution_class: &str,
    state: &str,
    year: u16,
    districts: u32,
    tolerance: f64,
    steps: u64,
    chains: usize,
    base_seed: u64,
    snapshot_stride: u64,
) -> Result<()> {
    if !matches!(
        execution_class,
        "excluded-engineering-preflight"
            | "governed-stage1"
            | "excluded-expansion-preflight"
            | "governed-stage2"
            | "excluded-expansion-v3-preflight"
            | "governed-stage2-v3"
    ) {
        bail!("unsupported execution class");
    }
    let state = state.to_uppercase();
    let expected_seed = if matches!(
        execution_class,
        "excluded-expansion-v3-preflight" | "governed-stage2-v3"
    ) {
        20260812
    } else {
        20260810
    };
    let common_frozen =
        year == 2020 && tolerance == 0.005 && base_seed == expected_seed && snapshot_stride == 10;
    if execution_class == "governed-stage1"
        && (!common_frozen || state != "RI" || districts != 2 || steps != 2000 || chains != 4)
    {
        bail!("governed-stage1 arguments differ from the frozen RI protocol");
    }
    if matches!(
        execution_class,
        "excluded-expansion-preflight"
            | "governed-stage2"
            | "excluded-expansion-v3-preflight"
            | "governed-stage2-v3"
    ) {
        let expected_districts = match state.as_str() {
            "NH" => 2,
            "NM" => 3,
            "GA" => 14,
            _ => bail!("expansion execution is restricted to NH, NM, and GA"),
        };
        let expected_shape = if matches!(execution_class, "governed-stage2" | "governed-stage2-v3")
        {
            steps == 2000 && chains == 4
        } else {
            steps == 25 && chains == 1
        };
        if !common_frozen || districts != expected_districts || !expected_shape {
            bail!("expansion arguments differ from the frozen execution protocol");
        }
    }
    Ok(())
}

fn weighted_cut(input: &BlockEnsembleInput, assignment: &[u32]) -> f64 {
    let mut total = 0.0;
    for node in 0..input.adjacency.len() {
        for (edge_index, &neighbor) in input.adjacency[node].iter().enumerate() {
            let neighbor = neighbor as usize;
            if node < neighbor && assignment[node] != assignment[neighbor] {
                total += input.edge_weights[node][edge_index];
            }
        }
    }
    total
}

fn metric(
    input: &BlockEnsembleInput,
    assignment: &[u32],
    step: u64,
    accepted: bool,
    cut_edges: usize,
    cut_fraction: f64,
    max_population_deviation: f64,
    runtime_ms: f64,
) -> Metric {
    Metric {
        step,
        accepted,
        cut_edges,
        cut_fraction,
        weighted_boundary_cut: weighted_cut(input, assignment),
        max_population_deviation,
        runtime_ms,
    }
}

fn main() -> Result<()> {
    let values = parse_args()?;
    let state = required(&values, "--state")?;
    let year: u16 = required(&values, "--year")?.parse()?;
    let districts: u32 = required(&values, "--districts")?.parse()?;
    let tolerance: f64 = required(&values, "--tolerance")?.parse()?;
    let steps: u64 = required(&values, "--steps")?.parse()?;
    let chains: usize = required(&values, "--chains")?.parse()?;
    let base_seed: u64 = required(&values, "--base-seed")?.parse()?;
    let snapshot_stride: u64 = required(&values, "--snapshot-stride")?.parse()?;
    let execution_class = values
        .get("--execution-class")
        .cloned()
        .unwrap_or_else(|| "excluded-engineering-preflight".to_string());
    validate_execution(
        &execution_class,
        &state,
        year,
        districts,
        tolerance,
        steps,
        chains,
        base_seed,
        snapshot_stride,
    )?;
    let sampler_name = required(&values, "--sampler")?;
    let sampler = match sampler_name.as_str() {
        "wilson" => TreeSampler::Wilson,
        "kruskal" => TreeSampler::GerryChainKruskal,
        _ => bail!("sampler must be wilson or kruskal"),
    };
    if let Some(contract_only) = values.get("--contract-only") {
        if contract_only != "true" {
            bail!("--contract-only must be true when supplied");
        }
        println!(
            "contract-valid execution_class={execution_class} state={} sampler={sampler_name} base_seed={base_seed}",
            state.to_uppercase()
        );
        return Ok(());
    }
    if steps == 0 || chains == 0 || snapshot_stride == 0 {
        bail!("steps, chains, and snapshot stride must be positive");
    }
    let output = PathBuf::from(required(&values, "--output")?);
    let input = load_block_ensemble_input(
        &PathBuf::from(required(&values, "--rctx")?),
        &PathBuf::from(required(&values, "--assignments")?),
        &state,
        year,
        districts,
        tolerance,
    )?;
    let total_edges = input.adjacency.iter().map(Vec::len).sum::<usize>() / 2;
    let baseline_cut = input
        .adjacency
        .iter()
        .enumerate()
        .map(|(node, neighbors)| {
            neighbors
                .iter()
                .filter(|&&neighbor| {
                    node < neighbor as usize
                        && input.assignment[node] != input.assignment[neighbor as usize]
                })
                .count()
        })
        .sum();
    let baseline = metric(
        &input,
        &input.assignment,
        0,
        true,
        baseline_cut,
        baseline_cut as f64 / total_edges.max(1) as f64,
        input.max_population_deviation(),
        0.0,
    );
    let mut chain_traces = Vec::with_capacity(chains);
    for chain_index in 0..chains {
        let seed = chain_seed(base_seed, chain_index);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut chain = RecomChain::new_with_sampler(
            input.adjacency.clone(),
            input.populations.clone(),
            input.assignment.clone(),
            districts,
            tolerance,
            sampler,
        );
        let mut metrics = Vec::with_capacity(steps as usize);
        let mut snapshots = Vec::new();
        for _ in 0..steps {
            let started = Instant::now();
            let record = chain.step(&mut rng);
            let runtime_ms = started.elapsed().as_secs_f64() * 1000.0;
            metrics.push(metric(
                &input,
                &chain.assignment,
                record.step,
                record.accepted,
                record.cut_edges,
                record.cut_fraction as f64,
                record.pop_deviation as f64,
                runtime_ms,
            ));
            if record.step % snapshot_stride == 0 {
                snapshots.push(Snapshot {
                    step: record.step,
                    assignment: chain.assignment.clone(),
                });
            }
        }
        chain_traces.push(ChainOutput {
            chain_index,
            seed,
            metrics,
            snapshots,
        });
    }
    let claim_boundary = if matches!(
        execution_class.as_str(),
        "governed-stage2" | "governed-stage2-v3"
    ) {
        "Governed NH/NM/GA Stage 2 trace; State-specific diagnostics only, with no national or sampler-equivalence claim."
    } else if matches!(
        execution_class.as_str(),
        "excluded-expansion-preflight" | "excluded-expansion-v3-preflight"
    ) {
        "Excluded Stage 2 engineering preflight; samples are barred from percentiles and convergence claims."
    } else {
        "Excluded Stage 0 engineering preflight; samples are barred from percentiles and convergence claims."
    };
    let trace = TraceOutput {
        schema_version: "nrs-block-ensemble-trace-v1",
        status: "complete",
        execution_class,
        state: input.state,
        year: input.year,
        units: input.unit_ids.len(),
        districts,
        sampler: sampler_name,
        chains,
        steps_per_chain: steps,
        population_tolerance: tolerance,
        base_seed,
        snapshot_stride,
        baseline,
        chain_traces,
        claim_boundary,
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_vec(&trace)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_execution;

    #[test]
    fn frozen_stage2_shapes_pass() {
        for (state, districts) in [("NH", 2), ("NM", 3), ("GA", 14)] {
            validate_execution(
                "governed-stage2",
                state,
                2020,
                districts,
                0.005,
                2000,
                4,
                20260810,
                10,
            )
            .unwrap();
            validate_execution(
                "excluded-expansion-preflight",
                state,
                2020,
                districts,
                0.005,
                25,
                1,
                20260810,
                10,
            )
            .unwrap();
        }
    }

    #[test]
    fn stage2_rejects_state_and_shape_drift() {
        assert!(validate_execution(
            "governed-stage2",
            "RI",
            2020,
            2,
            0.005,
            2000,
            4,
            20260810,
            10,
        )
        .is_err());
        assert!(
            validate_execution("governed-stage2", "NH", 2020, 2, 0.005, 25, 1, 20260810, 10,)
                .is_err()
        );
    }

    #[test]
    fn frozen_v3_shapes_require_fresh_seed() {
        for (state, districts) in [("NH", 2), ("NM", 3), ("GA", 14)] {
            validate_execution(
                "excluded-expansion-v3-preflight",
                state,
                2020,
                districts,
                0.005,
                25,
                1,
                20260812,
                10,
            )
            .unwrap();
            validate_execution(
                "governed-stage2-v3",
                state,
                2020,
                districts,
                0.005,
                2000,
                4,
                20260812,
                10,
            )
            .unwrap();
        }
        assert!(validate_execution(
            "excluded-expansion-v3-preflight",
            "NH",
            2020,
            2,
            0.005,
            25,
            1,
            20260811,
            10,
        )
        .is_err());
    }
}
