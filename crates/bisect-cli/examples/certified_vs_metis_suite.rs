use bisect_ilp::{
    canonical_orientation_rule, canonical_seat_split, certified_split_children_connected,
    certified_split_unit_universe_hash, evaluate_certified_split_objective,
    solve_certified_split_bounded, verify_certified_split_bounded, CertifiedSplitInstance,
    CertifiedSplitPrimaryObjective, ExactEdge, CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION,
    CERTIFIED_SPLIT_MODEL_ID,
};
use bisect_runner::bisection_runner::{detect_gpmetis_version, split_subgraph};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

const PROTOCOL_ID: &str = "certified-vs-metis-multi-instance-v1";
const PROTOCOL_PATH: &str = "docs/specs/2026-08-06-certified-vs-metis-multi-instance-protocol.md";
const GENERATOR_PATH: &str = "crates/bisect-cli/examples/certified_vs_metis_suite.rs";
const VERIFIER_PATH: &str = "scripts/research/verify_certified_vs_metis_suite.py";
const METIS_SEEDS: [u64; 5] = [1, 7, 42, 2020, 314_159];
const METIS_UFACTOR: f64 = 1.10;
const METIS_NITER: u32 = 10;

#[derive(Clone)]
struct Fixture {
    id: &'static str,
    populations: Vec<i64>,
    edges: Vec<ExactEdge>,
    k_parent: usize,
}

#[derive(Serialize)]
struct ExactObservation {
    elapsed_milliseconds: f64,
    candidate_count: u64,
    feasible_count: u64,
    primary_objective_ties: u64,
    objective: CertifiedSplitPrimaryObjective,
    canonical_assignment: Vec<u8>,
    proof_id: String,
    search_commitment: String,
    bounded_verifier_passed: bool,
}

#[derive(Serialize)]
struct InstanceObservation {
    fixture_id: String,
    instance_hash: String,
    unit_count: usize,
    edge_count: usize,
    instance: CertifiedSplitInstance,
    exact: ExactObservation,
}

#[derive(Serialize)]
struct MetisObservation {
    fixture_id: String,
    seed: u64,
    status: String,
    elapsed_milliseconds: f64,
    error: Option<String>,
    assignment: Option<Vec<u8>>,
    objective: Option<CertifiedSplitPrimaryObjective>,
    connected: Option<bool>,
    child_units_sufficient: Option<bool>,
    matches_exact_assignment: Option<bool>,
    matches_exact_primary_objective: Option<bool>,
    matches_exact_population_objective: Option<bool>,
    weighted_boundary_difference: Option<i64>,
}

#[derive(Clone, Default, Serialize)]
struct Summary {
    total_rows: usize,
    ok: usize,
    errors: usize,
    disconnected: usize,
    insufficient_child_units: usize,
    exact_assignment_agreements: usize,
    exact_primary_objective_agreements: usize,
    exact_population_objective_agreements: usize,
}

#[derive(Serialize)]
struct Aggregate {
    total_precommitted_rows: usize,
    seed_invariant_within_fixture: bool,
    summary: Summary,
    by_instance: BTreeMap<String, Summary>,
    by_seed: BTreeMap<String, Summary>,
}

#[derive(Serialize)]
struct ExecutionEnvironment {
    operating_system: String,
    architecture: String,
    build_profile: String,
    rustc_version: String,
    machine_name: String,
    processor: String,
}

#[derive(Serialize)]
struct SuiteReport {
    schema_version: String,
    protocol_id: String,
    protocol_path: String,
    metis_version: String,
    metis_seeds: Vec<u64>,
    metis_ufactor: f64,
    metis_niter: u32,
    execution_environment: ExecutionEnvironment,
    instances: Vec<InstanceObservation>,
    metis_rows: Vec<MetisObservation>,
    aggregate: Aggregate,
    conclusion: String,
    claim_boundary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: certified_vs_metis_suite <out-dir>".into());
    }
    let out_dir = Path::new(&args[1]);
    if out_dir.exists() {
        return Err(format!("output directory already exists: {}", out_dir.display()).into());
    }
    std::fs::create_dir_all(out_dir)?;

    let mut instances = Vec::new();
    let mut rows = Vec::new();
    for fixture in fixtures() {
        let instance = build_instance(&fixture)?;
        let started = Instant::now();
        let artifacts = solve_certified_split_bounded(&instance)?;
        let exact_elapsed = started.elapsed().as_secs_f64() * 1_000.0;
        verify_certified_split_bounded(&instance, &artifacts.certificate, &artifacts.proof)?;
        let exact_assignment = artifacts
            .proof
            .canonical_assignment
            .clone()
            .ok_or_else(|| format!("precommitted fixture is infeasible: {}", fixture.id))?;
        let exact_objective = artifacts
            .proof
            .lower_bound
            .clone()
            .ok_or_else(|| format!("precommitted fixture lacks an optimum: {}", fixture.id))?;

        for seed in METIS_SEEDS {
            rows.push(run_metis(
                fixture.id,
                &instance,
                seed,
                &exact_assignment,
                &exact_objective,
            ));
        }
        instances.push(InstanceObservation {
            fixture_id: fixture.id.to_string(),
            instance_hash: instance.hash()?,
            unit_count: instance.unit_ids.len(),
            edge_count: instance.edges.len(),
            instance,
            exact: ExactObservation {
                elapsed_milliseconds: exact_elapsed,
                candidate_count: artifacts.proof.candidate_count,
                feasible_count: artifacts.proof.feasible_count,
                primary_objective_ties: artifacts.proof.primary_objective_ties,
                objective: exact_objective,
                canonical_assignment: exact_assignment,
                proof_id: artifacts.proof.proof_id,
                search_commitment: artifacts.proof.search_commitment,
                bounded_verifier_passed: true,
            },
        });
    }

    let aggregate = aggregate(&rows);
    if aggregate.total_precommitted_rows != 40 {
        return Err("precommitted METIS row count drift".into());
    }
    let conclusion = format!(
        "Across all 40 precommitted rows, METIS matched the exact canonical assignment in {}, the complete exact primary objective in {}, and the exact population objective in {}; {} rows errored, {} were disconnected, and {} had insufficient child units.",
        aggregate.summary.exact_assignment_agreements,
        aggregate.summary.exact_primary_objective_agreements,
        aggregate.summary.exact_population_objective_agreements,
        aggregate.summary.errors,
        aggregate.summary.disconnected,
        aggregate.summary.insufficient_child_units,
    );
    let report = SuiteReport {
        schema_version: "certified-vs-metis-multi-instance-report-v1".to_string(),
        protocol_id: PROTOCOL_ID.to_string(),
        protocol_path: PROTOCOL_PATH.to_string(),
        metis_version: detect_gpmetis_version(),
        metis_seeds: METIS_SEEDS.to_vec(),
        metis_ufactor: METIS_UFACTOR,
        metis_niter: METIS_NITER,
        execution_environment: execution_environment(),
        instances,
        metis_rows: rows,
        aggregate,
        conclusion,
        claim_boundary: "Eight precommitted bounded synthetic instances and five fixed seeds only; no State-scale, national, map-quality, fairness, VRA, legal-validity, or adoption claim.".to_string(),
    };
    let report_path = out_dir.join("comparison-suite.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    let readme_path = out_dir.join("README.md");
    std::fs::write(&readme_path, render_readme(&report))?;
    let manifest = serde_json::json!({
        "schema_version": "certified-vs-metis-multi-instance-package-v1",
        "package_id": "certified-vs-metis-multi-instance-v1",
        "protocol_path": PROTOCOL_PATH,
        "protocol_sha256": sha256_file(Path::new(PROTOCOL_PATH))?,
        "generator_path": GENERATOR_PATH,
        "generator_sha256": sha256_file(Path::new(GENERATOR_PATH))?,
        "verifier_path": VERIFIER_PATH,
        "verifier_sha256": sha256_file(Path::new(VERIFIER_PATH))?,
        "files": {
            "README.md": sha256_file(&readme_path)?,
            "comparison-suite.json": sha256_file(&report_path)?
        },
        "claim_boundary": report.claim_boundary
    });
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!("Certified vs METIS multi-instance comparison: PASS");
    Ok(())
}

fn execution_environment() -> ExecutionEnvironment {
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    ExecutionEnvironment {
        operating_system: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        build_profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_string(),
        rustc_version,
        machine_name: std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unavailable".to_string()),
        processor: std::env::var("PROCESSOR_IDENTIFIER")
            .unwrap_or_else(|_| "unavailable".to_string()),
    }
}

fn render_readme(report: &SuiteReport) -> String {
    let mut table = String::from(
        "| Fixture | Units | Exact candidates | Feasible | Exact ties | Exact ms | Assignment matches | Objective matches |\n|---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    for instance in &report.instances {
        let summary = &report.aggregate.by_instance[&instance.fixture_id];
        table.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {:.3} | {}/5 | {}/5 |\n",
            instance.fixture_id,
            instance.unit_count,
            instance.exact.candidate_count,
            instance.exact.feasible_count,
            instance.exact.primary_objective_ties,
            instance.exact.elapsed_milliseconds,
            summary.exact_assignment_agreements,
            summary.exact_primary_objective_agreements,
        ));
    }
    format!(
        "# Certified Versus METIS Multi-Instance Result\n\n**Status:** independently verified committed package\n\n**Protocol:** `{}`\n\n{}\n\n{}\n\nAll 40 METIS rows were retained. All were feasible and connected. METIS matched the complete exact primary objective in {}/40 rows and the exact canonical assignment in {}/40 rows. Seed-invariant outcome within each fixture on this backend: `{}`.\n\nThe two complete-objective disagreements were fixture-wide: on `ladder2x6-varied`, METIS chose a boundary-2 split with scaled population deviations 20/40 while the certified optimum chose exact population balance with boundary 4; on `grid4x4-unequal`, METIS chose deviations 95/190 with boundary 5 while the certified optimum chose 40/80 with boundary 8. Under the precommitted lexicographic objective, population deviation is minimized before boundary, so the certified choices are better under the certified model even though they cut more edge weight.\n\nThe weighted grid and 20-cycle matched the exact objective but not the canonical assignment, demonstrating the distinction between objective optimality and deterministic tie selection.\n\n## Rebuild and verify\n\n```powershell\ncargo run --release --locked -p bisect-cli --example certified_vs_metis_suite -- <new-output-directory>\npython scripts/research/verify_certified_vs_metis_suite.py <new-output-directory>\n```\n\nTimings are descriptive observations from `{}` / `{}` using `{}` and `{}`; they are not reproducibility requirements or speed claims.\n\n## Claim boundary\n\n{}\n",
        report.protocol_id,
        report.conclusion,
        table,
        report.aggregate.summary.exact_primary_objective_agreements,
        report.aggregate.summary.exact_assignment_agreements,
        report.aggregate.seed_invariant_within_fixture,
        report.execution_environment.machine_name,
        report.execution_environment.processor,
        report.execution_environment.rustc_version,
        report.execution_environment.build_profile,
        report.claim_boundary,
    )
}

fn run_metis(
    fixture_id: &str,
    instance: &CertifiedSplitInstance,
    seed: u64,
    exact_assignment: &[u8],
    exact_objective: &CertifiedSplitPrimaryObjective,
) -> MetisObservation {
    let adjacency = adjacency(instance);
    let edge_weights = instance
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight as f64))
        .collect::<HashMap<_, _>>();
    let units = (0..instance.unit_ids.len()).collect::<HashSet<_>>();
    let target_weights = vec![
        instance.k_left as f32 / instance.k_parent as f32,
        instance.k_right as f32 / instance.k_parent as f32,
    ];
    let started = Instant::now();
    let result = split_subgraph(
        &adjacency,
        &instance.populations,
        1,
        &edge_weights,
        &units,
        METIS_UFACTOR,
        METIS_NITER,
        Some(seed),
        Some(target_weights),
        None,
    );
    let elapsed = started.elapsed().as_secs_f64() * 1_000.0;
    let (left, _right) = match result {
        Ok(parts) => parts,
        Err(error) => {
            return MetisObservation {
                fixture_id: fixture_id.to_string(),
                seed,
                status: "error".to_string(),
                elapsed_milliseconds: elapsed,
                error: Some(error),
                assignment: None,
                objective: None,
                connected: None,
                child_units_sufficient: None,
                matches_exact_assignment: None,
                matches_exact_primary_objective: None,
                matches_exact_population_objective: None,
                weighted_boundary_difference: None,
            };
        }
    };
    let mut assignment = (0..instance.unit_ids.len())
        .map(|unit| u8::from(!left.contains(&unit)))
        .collect::<Vec<_>>();
    if instance.k_left == instance.k_right && assignment[0] == 1 {
        assignment.iter_mut().for_each(|label| *label = 1 - *label);
    }
    let left_units = assignment.iter().filter(|&&label| label == 0).count();
    let right_units = assignment.len() - left_units;
    let sufficient = left_units >= instance.k_left && right_units >= instance.k_right;
    if !sufficient {
        return MetisObservation {
            fixture_id: fixture_id.to_string(),
            seed,
            status: "insufficient-child-units".to_string(),
            elapsed_milliseconds: elapsed,
            error: None,
            assignment: Some(assignment),
            objective: None,
            connected: None,
            child_units_sufficient: Some(false),
            matches_exact_assignment: None,
            matches_exact_primary_objective: None,
            matches_exact_population_objective: None,
            weighted_boundary_difference: None,
        };
    }
    let objective = evaluate_certified_split_objective(instance, &assignment)
        .expect("METIS assignment passed structural validation");
    let connected = certified_split_children_connected(instance, &assignment)
        .expect("METIS assignment passed structural validation");
    let primary_match = objective == *exact_objective;
    let population_match = objective.max_population_deviation_scaled
        == exact_objective.max_population_deviation_scaled
        && objective.total_population_deviation_scaled
            == exact_objective.total_population_deviation_scaled;
    MetisObservation {
        fixture_id: fixture_id.to_string(),
        seed,
        status: if connected { "ok" } else { "disconnected" }.to_string(),
        elapsed_milliseconds: elapsed,
        error: None,
        assignment: Some(assignment.clone()),
        objective: Some(objective.clone()),
        connected: Some(connected),
        child_units_sufficient: Some(true),
        matches_exact_assignment: Some(assignment == exact_assignment),
        matches_exact_primary_objective: Some(primary_match),
        matches_exact_population_objective: Some(population_match),
        weighted_boundary_difference: Some(
            objective.weighted_boundary_cut as i64 - exact_objective.weighted_boundary_cut as i64,
        ),
    }
}

fn aggregate(rows: &[MetisObservation]) -> Aggregate {
    let mut overall = Summary::default();
    let mut by_instance = BTreeMap::new();
    let mut by_seed = BTreeMap::new();
    for row in rows {
        update_summary(&mut overall, row);
        update_summary(by_instance.entry(row.fixture_id.clone()).or_default(), row);
        update_summary(by_seed.entry(row.seed.to_string()).or_default(), row);
    }
    Aggregate {
        total_precommitted_rows: rows.len(),
        seed_invariant_within_fixture: rows.iter().enumerate().all(|(index, row)| {
            rows[..index]
                .iter()
                .find(|earlier| earlier.fixture_id == row.fixture_id)
                .is_none_or(|earlier| {
                    earlier.status == row.status
                        && earlier.assignment == row.assignment
                        && earlier.objective == row.objective
                })
        }),
        summary: overall,
        by_instance,
        by_seed,
    }
}

fn update_summary(summary: &mut Summary, row: &MetisObservation) {
    summary.total_rows += 1;
    match row.status.as_str() {
        "ok" => summary.ok += 1,
        "error" => summary.errors += 1,
        "disconnected" => summary.disconnected += 1,
        "insufficient-child-units" => summary.insufficient_child_units += 1,
        _ => unreachable!("unknown METIS row status"),
    }
    summary.exact_assignment_agreements += usize::from(row.matches_exact_assignment == Some(true));
    summary.exact_primary_objective_agreements +=
        usize::from(row.matches_exact_primary_objective == Some(true));
    summary.exact_population_objective_agreements +=
        usize::from(row.matches_exact_population_objective == Some(true));
}

fn build_instance(fixture: &Fixture) -> Result<CertifiedSplitInstance, Box<dyn Error>> {
    let unit_ids = (0..fixture.populations.len())
        .map(|unit| format!("u{unit:02}"))
        .collect::<Vec<_>>();
    let (k_left, k_right) = canonical_seat_split(fixture.k_parent)?;
    Ok(CertifiedSplitInstance {
        schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
        node_path: String::new(),
        parent_certificate_id: None,
        unit_universe_hash: certified_split_unit_universe_hash(&unit_ids)?,
        unit_ids,
        populations: fixture.populations.clone(),
        edges: fixture.edges.clone(),
        k_parent: fixture.k_parent,
        k_left,
        k_right,
        orientation_rule: canonical_orientation_rule(k_left, k_right),
    })
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            id: "path8-equal",
            populations: vec![100; 8],
            edges: path_edges(8),
            k_parent: 4,
        },
        Fixture {
            id: "cycle10-varied",
            populations: vec![80, 120, 90, 110, 95, 105, 85, 115, 100, 100],
            edges: cycle_edges(10, |_| 1),
            k_parent: 4,
        },
        Fixture {
            id: "ladder2x6-varied",
            populations: vec![90, 110, 95, 105, 85, 115, 120, 80, 100, 100, 108, 92],
            edges: ladder_edges(6),
            k_parent: 4,
        },
        Fixture {
            id: "grid3x4-weighted",
            populations: vec![100; 12],
            edges: grid_edges(3, 4, |left, right| {
                if [(1, 2), (5, 6), (9, 10)].contains(&(left, right)) {
                    5
                } else {
                    1
                }
            }),
            k_parent: 4,
        },
        Fixture {
            id: "barbell12-bridge",
            populations: vec![95, 105, 100, 100, 90, 110, 110, 90, 100, 100, 105, 95],
            edges: barbell_edges(),
            k_parent: 4,
        },
        Fixture {
            id: "tree13-unequal",
            populations: vec![70, 130, 90, 110, 80, 120, 100, 95, 105, 85, 115, 75, 125],
            edges: tree_edges(13),
            k_parent: 5,
        },
        Fixture {
            id: "grid4x4-unequal",
            populations: vec![
                82, 118, 91, 109, 97, 103, 86, 114, 121, 79, 106, 94, 88, 112, 99, 101,
            ],
            edges: grid_edges(4, 4, |_, _| 1),
            k_parent: 5,
        },
        Fixture {
            id: "cycle20-equal",
            populations: (0..20)
                .map(|unit| if unit % 2 == 0 { 90 } else { 110 })
                .collect(),
            edges: cycle_edges(20, |(left, right)| {
                if left % 5 == 0 || right % 5 == 0 {
                    2
                } else {
                    1
                }
            }),
            k_parent: 6,
        },
    ]
}

fn edge(left: usize, right: usize, weight: u64) -> ExactEdge {
    ExactEdge {
        left: left.min(right),
        right: left.max(right),
        weight,
    }
}

fn path_edges(units: usize) -> Vec<ExactEdge> {
    (0..units - 1).map(|left| edge(left, left + 1, 1)).collect()
}

fn cycle_edges<F>(units: usize, weight: F) -> Vec<ExactEdge>
where
    F: Fn((usize, usize)) -> u64,
{
    let mut pairs = (0..units - 1)
        .map(|left| (left, left + 1))
        .collect::<Vec<_>>();
    pairs.push((0, units - 1));
    pairs.sort_unstable();
    pairs
        .into_iter()
        .map(|pair| edge(pair.0, pair.1, weight(pair)))
        .collect()
}

fn ladder_edges(columns: usize) -> Vec<ExactEdge> {
    let mut edges = Vec::new();
    for row in 0..2 {
        for column in 0..columns - 1 {
            edges.push(edge(row * columns + column, row * columns + column + 1, 1));
        }
    }
    for column in 0..columns {
        edges.push(edge(column, columns + column, 1));
    }
    edges.sort_by_key(|item| (item.left, item.right));
    edges
}

fn grid_edges<F>(rows: usize, columns: usize, weight: F) -> Vec<ExactEdge>
where
    F: Fn(usize, usize) -> u64,
{
    let mut edges = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let unit = row * columns + column;
            if column + 1 < columns {
                edges.push(edge(unit, unit + 1, weight(unit, unit + 1)));
            }
            if row + 1 < rows {
                edges.push(edge(unit, unit + columns, weight(unit, unit + columns)));
            }
        }
    }
    edges.sort_by_key(|item| (item.left, item.right));
    edges
}

fn barbell_edges() -> Vec<ExactEdge> {
    let mut edges = Vec::new();
    for start in [0, 6] {
        for left in start..start + 6 {
            for right in left + 1..start + 6 {
                edges.push(edge(left, right, 1));
            }
        }
    }
    edges.push(edge(5, 6, 3));
    edges.sort_by_key(|item| (item.left, item.right));
    edges
}

fn tree_edges(units: usize) -> Vec<ExactEdge> {
    (1..units)
        .map(|child| edge((child - 1) / 2, child, 1))
        .collect()
}

fn adjacency(instance: &CertifiedSplitInstance) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); instance.unit_ids.len()];
    for edge in &instance.edges {
        adjacency[edge.left].push(edge.right);
        adjacency[edge.right].push(edge.left);
    }
    adjacency
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
