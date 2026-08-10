use anyhow::{bail, Context, Result};
use bisect_ensemble::block_input::load_block_ensemble_input;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn arguments() -> Result<HashMap<String, String>> {
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

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("cannot hash {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn main() -> Result<()> {
    let values = arguments()?;
    let rctx = PathBuf::from(required(&values, "--rctx")?);
    let assignments = PathBuf::from(required(&values, "--assignments")?);
    let state = required(&values, "--state")?;
    let year: u16 = required(&values, "--year")?.parse()?;
    let districts: u32 = required(&values, "--districts")?.parse()?;
    let tolerance: f64 = required(&values, "--tolerance")?.parse()?;
    let input = load_block_ensemble_input(&rctx, &assignments, &state, year, districts, tolerance)?;
    let directed_edges: usize = input.adjacency.iter().map(Vec::len).sum();
    let report = json!({
        "schema_version": "nrs-block-ensemble-input-audit-v1",
        "status": "pass",
        "state": input.state,
        "year": input.year,
        "units": input.unit_ids.len(),
        "districts": input.districts,
        "population_total": input.populations.iter().sum::<i64>(),
        "max_population_deviation": input.max_population_deviation(),
        "undirected_edges": directed_edges / 2,
        "rctx_sha256": sha256(&rctx)?,
        "assignments_sha256": sha256(&assignments)?,
        "claim_boundary": "Stage 0 input audit only; no ensemble was executed."
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
