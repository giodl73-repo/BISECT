use bisect_ilp::{
    canonical_orientation_rule, canonical_seat_split, certified_split_children_connected,
    certified_split_discovery, certified_split_unit_universe_hash,
    evaluate_certified_split_objective, solve_certified_split_bounded, CertifiedSplitInstance,
    ExactEdge, CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION, CERTIFIED_SPLIT_MODEL_ID,
};
use bisect_runner::bisection_runner::{detect_gpmetis_version, split_subgraph};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;

#[derive(Serialize)]
struct ComparisonRow {
    method: String,
    assignment: Vec<u8>,
    objective: bisect_ilp::CertifiedSplitPrimaryObjective,
    connected: bool,
    matches_certified_assignment: bool,
}

#[derive(Serialize)]
struct ComparisonReport {
    schema_version: String,
    instance_hash: String,
    metis_version: String,
    metis_seed: u64,
    rows: Vec<ComparisonRow>,
    conclusion: String,
    claim_boundary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: certified_vs_metis_path8 <out-dir>".into());
    }
    let out_dir = Path::new(&args[1]);
    std::fs::create_dir_all(out_dir)?;
    let instance = path8_instance()?;
    let exact = solve_certified_split_bounded(&instance)?;
    let certified_assignment = exact
        .proof
        .canonical_assignment
        .clone()
        .ok_or("path8 certified split is infeasible")?;

    let adjacency = path8_adjacency();
    let populations = vec![100_i64; 8];
    let edge_weights = (0..7)
        .map(|left| ((left, left + 1), 1.0))
        .collect::<HashMap<_, _>>();
    let units = (0..8).collect::<HashSet<_>>();
    let seed = 42_u64;
    let (left, _right) = split_subgraph(
        &adjacency,
        &populations,
        1,
        &edge_weights,
        &units,
        1.005,
        10,
        Some(seed),
        None,
        None,
    )?;
    let mut metis_assignment = (0..8)
        .map(|unit| if left.contains(&unit) { 0_u8 } else { 1_u8 })
        .collect::<Vec<_>>();
    if metis_assignment[0] == 1 {
        for label in &mut metis_assignment {
            *label = 1 - *label;
        }
    }
    let suboptimal_assignment = vec![0, 0, 0, 1, 1, 1, 1, 1];
    let rows = [
        ("certified", certified_assignment.clone()),
        ("metis-seed-42", metis_assignment),
        ("deliberate-suboptimal-control", suboptimal_assignment),
    ]
    .into_iter()
    .map(|(method, assignment)| {
        let objective = evaluate_certified_split_objective(&instance, &assignment)?;
        let connected = certified_split_children_connected(&instance, &assignment)?;
        let _discovery = certified_split_discovery(
            &instance,
            method,
            None,
            "same-instance-comparison",
            assignment.clone(),
        )?;
        Ok(ComparisonRow {
            method: method.to_string(),
            matches_certified_assignment: assignment == certified_assignment,
            assignment,
            objective,
            connected,
        })
    })
    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let metis_matches = rows[1].matches_certified_assignment;
    let report = ComparisonReport {
        schema_version: "certified-vs-metis-comparison-v1".to_string(),
        instance_hash: instance.hash()?,
        metis_version: detect_gpmetis_version(),
        metis_seed: seed,
        rows,
        conclusion: if metis_matches {
            "METIS matches the certified optimum on path8; certification adds proof strength, not a better cut on this fixture.".to_string()
        } else {
            "METIS differs from the certified optimum on path8; inspect objective rows before claiming quality differences.".to_string()
        },
        claim_boundary:
            "Single bounded synthetic root split; not national runtime or map-quality superiority."
                .to_string(),
    };
    let report_path = out_dir.join("comparison.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    let manifest = serde_json::json!({
        "schema_version": "certified-vs-metis-comparison-package-v1",
        "package_id": "path8-root-certified-vs-metis",
        "generator_path": "crates/bisect-cli/examples/certified_vs_metis_path8.rs",
        "generator_sha256": sha256_file(Path::new("crates/bisect-cli/examples/certified_vs_metis_path8.rs"))?,
        "files": {
            "comparison.json": sha256_file(&report_path)?
        },
        "verifier_path": "scripts/research/verify_certified_vs_metis.py",
        "claim_boundary": report.claim_boundary
    });
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!("Certified vs METIS path8 comparison: PASS");
    Ok(())
}

fn path8_instance() -> Result<CertifiedSplitInstance, Box<dyn Error>> {
    let unit_ids = (0..8).map(|unit| format!("u{unit:02}")).collect::<Vec<_>>();
    let (k_left, k_right) = canonical_seat_split(4)?;
    Ok(CertifiedSplitInstance {
        schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
        node_path: String::new(),
        parent_certificate_id: None,
        unit_universe_hash: certified_split_unit_universe_hash(&unit_ids)?,
        unit_ids,
        populations: vec![100; 8],
        edges: (0..7)
            .map(|left| ExactEdge {
                left,
                right: left + 1,
                weight: 1,
            })
            .collect(),
        k_parent: 4,
        k_left,
        k_right,
        orientation_rule: canonical_orientation_rule(k_left, k_right),
    })
}

fn path8_adjacency() -> Vec<Vec<usize>> {
    (0..8)
        .map(|unit| {
            let mut neighbors = Vec::new();
            if unit > 0 {
                neighbors.push(unit - 1);
            }
            if unit < 7 {
                neighbors.push(unit + 1);
            }
            neighbors
        })
        .collect()
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
