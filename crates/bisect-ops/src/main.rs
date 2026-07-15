use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const SCREEN_TIMEOUT: Duration = Duration::from_secs(180);
const GENERATED_AT: &str = "2026-07-12T00:00:00Z";
const BUILDER_SNAPSHOT: &str = "builder-source.rs";

#[derive(Parser)]
#[command(about = "Rust-native deterministic operational recursive-tree builder")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    Build {
        #[arg(long)]
        bisect: PathBuf,
        #[arg(long)]
        context: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        districts: usize,
        #[arg(long, default_value_t = 1)]
        root_seed: u64,
        #[arg(long, default_value_t = 2)]
        child_seed_0: u64,
        #[arg(long, default_value_t = 3)]
        child_seed_1: u64,
        #[arg(long, default_value_t = 16)]
        max_seed: u64,
    },
    Verify {
        package: PathBuf,
    },
    Batch {
        #[arg(long)]
        bisect: PathBuf,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        retry_failed: bool,
        #[arg(long, default_value_t = 16)]
        max_seed: u64,
    },
    AuditPython {
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        base: Option<String>,
    },
    AnalyzeTree {
        #[arg(long)]
        state: String,
        #[arg(long)]
        package: PathBuf,
        #[arg(long)]
        rctx_report: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    VerifyTreeReport {
        manifest: PathBuf,
    },
    VerifyNationalRctx {
        #[arg(long, default_value = "docs/experiments/nationwide-2020")]
        out_dir: PathBuf,
    },
    RctxBatch {
        #[arg(long, default_value_t = 2)]
        workers: usize,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value = "python")]
        adapter_runtime: PathBuf,
        #[arg(long, default_value = "scripts/research/build_state_block_rctx.py")]
        adapter: PathBuf,
    },
}

#[derive(Clone)]
struct Candidate {
    discovery: Value,
    seed: u64,
    dir: PathBuf,
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn write_json(path: &Path, value: &Value, pretty: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };
    if pretty {
        bytes.push(b'\n');
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn sha256(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn custody_source(path: &Path) -> PathBuf {
    if path.is_file() {
        path.to_path_buf()
    } else {
        Path::new("archive/legacy-python").join(path)
    }
}

fn canonical_hash(value: &Value) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}

fn field_u64(value: &Value, path: &[&str]) -> Result<u64> {
    let mut current = value;
    for key in path {
        current = current
            .get(*key)
            .with_context(|| format!("missing {}", path.join(".")))?;
    }
    current
        .as_u64()
        .with_context(|| format!("{} is not u64", path.join(".")))
}

fn objective(discovery: &Value) -> Result<&Value> {
    discovery
        .pointer("/objective/primary")
        .context("missing objective.primary")
}

fn discovery_seed(discovery: &Value) -> Result<u64> {
    discovery["method"]
        .as_str()
        .context("missing discovery method")?
        .split(';')
        .find_map(|part| part.trim().strip_prefix("seed=")?.parse().ok())
        .context("discovery method has no seed")
}

fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    let key = |c: &Candidate, name| {
        field_u64(&c.discovery, &["objective", "primary", name]).unwrap_or(u64::MAX)
    };
    key(a, "max_population_deviation_scaled")
        .cmp(&key(b, "max_population_deviation_scaled"))
        .then_with(|| {
            key(a, "total_population_deviation_scaled")
                .cmp(&key(b, "total_population_deviation_scaled"))
        })
        .then_with(|| key(a, "weighted_boundary_cut").cmp(&key(b, "weighted_boundary_cut")))
        .then_with(|| a.seed.cmp(&b.seed))
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn prune(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.file_name().and_then(|v| v.to_str()) != Some("certified-discovery.json") {
            remove_path(&path)?;
        }
    }
    Ok(())
}

fn run_discovery(
    bisect: &Path,
    context: &Path,
    districts: usize,
    out: &Path,
    seed: u64,
    refinement: &str,
    timeout: Option<Duration>,
) -> Result<Option<Value>> {
    let mut child = Command::new(bisect)
        .args(["exact", "--context"])
        .arg(context)
        .args([
            "--districts",
            &districts.to_string(),
            "--method",
            "certified-discovery",
            "--out-dir",
        ])
        .arg(out)
        .args([
            "--generated-at",
            GENERATED_AT,
            "--discovery-seed",
            &seed.to_string(),
            "--discovery-refinement",
            refinement,
        ])
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("launch seed {seed}"))?;
    if let Some(limit) = timeout {
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    bail!("seed {seed} failed with {status}");
                }
                break;
            }
            if start.elapsed() >= limit {
                child.kill()?;
                let _ = child.wait();
                return Ok(None);
            }
            thread::sleep(Duration::from_millis(100));
        }
    } else if !child.wait()?.success() {
        bail!("seed {seed} discovery failed");
    }
    Ok(Some(read_json(&out.join("certified-discovery.json"))?))
}

fn floor_discovery(
    bisect: &Path,
    context: &Path,
    districts: usize,
    out: &Path,
    preferred: u64,
    floor: u64,
    max_seed: u64,
) -> Result<(Value, u64, Value)> {
    let completed_path = out.join("certified-discovery.json");
    if completed_path.is_file() {
        let completed = read_json(&completed_path)?;
        if field_u64(
            &completed,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            let report = if out.join("seed-screening.json").is_file() {
                read_json(&out.join("seed-screening.json"))?
            } else {
                json!([{"seed": discovery_seed(&completed)?, "status":"selected-node-reused", "objective": objective(&completed)?}])
            };
            println!(
                "{}: reused completed node at arithmetic floor {floor}",
                out.file_name().unwrap().to_string_lossy()
            );
            return Ok((completed.clone(), discovery_seed(&completed)?, report));
        }
    }
    let mut seeds = vec![preferred];
    seeds.extend((1..=max_seed).filter(|seed| *seed != preferred));
    let mut screened = Vec::new();
    let mut report = Vec::new();
    for seed in seeds {
        let screen = out.with_file_name(format!(
            "{}-screen-seed-{seed:02}",
            out.file_name().unwrap().to_string_lossy()
        ));
        let timeout_path = screen.with_extension("timeout.json");
        if timeout_path.is_file() {
            report
                .push(json!({"seed":seed,"status":"timeout","timeout_seconds":180,"reused":true}));
            continue;
        }
        let path = screen.join("certified-discovery.json");
        let reused = path.is_file();
        let discovery = if reused {
            let value = read_json(&path)?;
            prune(&screen)?;
            Some(value)
        } else {
            remove_path(&screen)?;
            match run_discovery(
                bisect,
                context,
                districts,
                &screen,
                seed,
                "metis",
                Some(SCREEN_TIMEOUT),
            ) {
                Ok(value) => value,
                Err(error) => {
                    remove_path(&screen)?;
                    report.push(json!({"seed":seed,"status":"failed","error":error.to_string()}));
                    continue;
                }
            }
        };
        let Some(discovery) = discovery else {
            remove_path(&screen)?;
            write_json(
                &timeout_path,
                &json!({"status":"timeout","timeout_seconds":180}),
                true,
            )?;
            report.push(json!({"seed":seed,"status":"timeout","timeout_seconds":180}));
            continue;
        };
        prune(&screen)?;
        report.push(json!({"seed":seed,"status":"completed","reused":reused,"objective":objective(&discovery)?}));
        screened.push(Candidate {
            discovery,
            seed,
            dir: screen,
        });
    }
    screened.sort_by(rank);
    let mut candidates = Vec::new();
    let mut selected = None;
    for screen in &screened {
        if field_u64(
            &screen.discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            selected = Some(screen.clone());
            break;
        }
        let dir = out.with_file_name(format!(
            "{}-seed-{:02}",
            out.file_name().unwrap().to_string_lossy(),
            screen.seed
        ));
        let path = dir.join("certified-discovery.json");
        let reused = path.is_file();
        let discovery = if reused {
            read_json(&path)?
        } else {
            remove_path(&dir)?;
            run_discovery(
                bisect,
                context,
                districts,
                &dir,
                screen.seed,
                "population",
                None,
            )?
            .context("unbounded discovery returned timeout")?
        };
        prune(&dir)?;
        if let Some(row) = report
            .iter_mut()
            .find(|row| row["seed"].as_u64() == Some(screen.seed))
        {
            row["refined_objective"] = objective(&discovery)?.clone();
            row["refined_reused"] = json!(reused);
        }
        let candidate = Candidate {
            discovery,
            seed: screen.seed,
            dir,
        };
        if field_u64(
            &candidate.discovery,
            &["objective", "primary", "max_population_deviation_scaled"],
        )? == floor
        {
            selected = Some(candidate.clone());
            candidates.push(candidate);
            break;
        }
        candidates.push(candidate);
    }
    let report_value = Value::Array(report);
    let Some(selected) = selected else {
        fs::create_dir_all(out)?;
        write_json(&out.join("seed-screening.json"), &report_value, true)?;
        let best = candidates
            .iter()
            .min_by(|a, b| rank(a, b))
            .context("no completed refinement")?;
        write_json(
            &out.join("unresolved-floor.json"),
            &json!({"population_floor":floor,"status":"unresolved-local-search-frontier","best_seed":best.seed,"best_objective":objective(&best.discovery)?,"claim_boundary":"No screened deterministic seed reached the arithmetic population floor; this is not a proof of infeasibility."}),
            true,
        )?;
        bail!(
            "no seed reached arithmetic population floor {floor}; best seed {} reached {}",
            best.seed,
            field_u64(
                &best.discovery,
                &["objective", "primary", "max_population_deviation_scaled"]
            )?
        );
    };
    remove_path(out)?;
    fs::rename(&selected.dir, out)?;
    write_json(&out.join("seed-screening.json"), &report_value, true)?;
    for candidate in candidates.iter().chain(screened.iter()) {
        if candidate.dir != selected.dir && candidate.dir.exists() {
            remove_path(&candidate.dir)?;
        }
    }
    for name in [
        "audit-certificate.json",
        "certified-discovery-manifest.json",
        "discovery.rctx",
        "discovery.rplan",
    ] {
        remove_path(&out.join(name))?;
    }
    Ok((selected.discovery, selected.seed, report_value))
}

fn ratio_floor(population: i64, seats: usize, right: usize) -> u64 {
    let rem = (right as i64 * population).rem_euclid(seats as i64);
    rem.min(seats as i64 - rem) as u64
}

fn subset_context(context: &Value, selected: &[usize], source_id: String) -> Result<Value> {
    let chosen: BTreeSet<_> = selected.iter().copied().collect();
    let remap: std::collections::BTreeMap<_, _> = selected
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let ids = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?;
    let pops = context["populations"].as_array().context("populations")?;
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    let mut units = context["units"].clone();
    units["unit_ids"] = Value::Array(selected.iter().map(|i| ids[*i].clone()).collect());
    units["source_id"] = json!(source_id);
    let mut projected = Vec::new();
    for old in selected {
        projected.push(Value::Array(
            adjacency[*old]
                .as_array()
                .context("edge list")?
                .iter()
                .filter_map(|edge| {
                    let to = edge["to"].as_u64()? as usize;
                    chosen.contains(&to).then(|| {
                        let mut e = edge.clone();
                        e["to"] = json!(remap[&to]);
                        e
                    })
                })
                .collect(),
        ));
    }
    let mut projection = json!({"units":units,"graph":{"edge_semantics":"undirected","adjacency":projected},"populations":selected.iter().map(|i|pops[*i].clone()).collect::<Vec<_>>(),"source_hashes":context["source_hashes"]});
    projection["units"]["unit_universe_hash"] = json!(canonical_hash(&projection["units"])?);
    let hash = canonical_hash(&projection)?;
    let mut result = Map::new();
    result.insert("rctx_version".into(), json!("0.1"));
    result.insert("context_hash".into(), json!(hash));
    for (key, value) in projection.as_object().unwrap() {
        result.insert(key.clone(), value.clone());
    }
    Ok(Value::Object(result))
}

struct BuildState<'a> {
    bisect: &'a Path,
    out: &'a Path,
    original: &'a Value,
    child_seeds: [u64; 2],
    max_seed: u64,
    assignment: Vec<i64>,
    nodes: Vec<Value>,
    leaves: Vec<Value>,
}

struct Visit {
    context: Value,
    context_path: PathBuf,
    global: Vec<usize>,
    seats: usize,
    path: String,
    offset: usize,
    seed: u64,
}

impl BuildState<'_> {
    fn visit(&mut self, visit: Visit) -> Result<()> {
        let Visit {
            context,
            context_path,
            global,
            seats,
            path,
            offset,
            seed,
        } = visit;
        let populations = context["populations"]
            .as_array()
            .context("node populations")?;
        if seats == 1 {
            for &unit in &global {
                self.assignment[unit] = offset as i64;
            }
            let population: i64 = global
                .iter()
                .map(|&unit| self.original["populations"][unit].as_i64().unwrap_or(0))
                .sum();
            self.leaves.push(json!({"path":path,"district":offset,"unit_count":global.len(),"population":population}));
            return Ok(());
        }
        let name = if path.is_empty() {
            "root".into()
        } else {
            format!("node-{path}")
        };
        let node_dir = self.out.join(name);
        let left = seats / 2;
        let right = seats - left;
        let parent_population: i64 = populations.iter().map(|v| v.as_i64().unwrap_or(0)).sum();
        let floor = ratio_floor(parent_population, seats, right);
        let (discovery, selected_seed, screening) = floor_discovery(
            self.bisect,
            &context_path,
            seats,
            &node_dir,
            seed,
            floor,
            self.max_seed,
        )?;
        let labels = discovery
            .pointer("/objective/canonical_assignment")
            .and_then(Value::as_array)
            .context("canonical assignment")?;
        self.nodes.push(json!({
            "path":path,"seats":seats,"parent_population":parent_population,
            "discovery_id":discovery["discovery_id"],"seed":selected_seed,
            "seed_screening":screening,"objective":objective(&discovery)?,
            "population_proof":{"kind":"ratio-arithmetic-floor","lower_bound":floor},
            "context_sha256":sha256(&context_path)?
        }));
        for label in 0..=1usize {
            let child_seats = if label == 0 { left } else { right };
            let child_offset = if label == 0 { offset } else { offset + left };
            let local: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(i, v)| (v.as_u64() == Some(label as u64)).then_some(i))
                .collect();
            if local.is_empty() {
                bail!("node {path} produced empty child {label}");
            }
            let child_global = local.iter().map(|&unit| global[unit]).collect();
            let child_path = format!("{path}{label}");
            let child_context = subset_context(
                &context,
                &local,
                format!(
                    "operational-tree-node-{}-{label}",
                    if path.is_empty() { "root" } else { &path }
                ),
            )?;
            let child_context_path = self.out.join(format!("context-{child_path}.rctx"));
            write_json(&child_context_path, &child_context, false)?;
            let child_seed = if path.is_empty() {
                self.child_seeds[label]
            } else {
                seed + label as u64 + 1
            };
            self.visit(Visit {
                context: child_context,
                context_path: child_context_path,
                global: child_global,
                seats: child_seats,
                path: child_path,
                offset: child_offset,
                seed: child_seed,
            })?;
        }
        Ok(())
    }
}

fn connected(context: &Value, assignment: &[i64], label: i64) -> Result<bool> {
    let members: Vec<usize> = assignment
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| (v == label).then_some(i))
        .collect();
    let Some(&first) = members.first() else {
        return Ok(false);
    };
    let allowed: BTreeSet<_> = members.iter().copied().collect();
    let adjacency = context
        .pointer("/graph/adjacency")
        .and_then(Value::as_array)
        .context("adjacency")?;
    let mut seen = BTreeSet::from([first]);
    let mut queue = VecDeque::from([first]);
    while let Some(unit) = queue.pop_front() {
        for edge in adjacency[unit].as_array().context("edge list")? {
            let to = edge["to"].as_u64().context("edge.to")? as usize;
            if allowed.contains(&to) && seen.insert(to) {
                queue.push_back(to);
            }
        }
    }
    Ok(seen == allowed)
}

fn build(
    bisect: &Path,
    context_path: &Path,
    out: &Path,
    districts: usize,
    root_seed: u64,
    child_seeds: [u64; 2],
    max_seed: u64,
) -> Result<()> {
    let context = read_json(context_path)?;
    fs::create_dir_all(out)?;
    let unit_count = context
        .pointer("/units/unit_ids")
        .and_then(Value::as_array)
        .context("unit_ids")?
        .len();
    let mut state = BuildState {
        bisect,
        out,
        original: &context,
        child_seeds,
        max_seed,
        assignment: vec![-1; unit_count],
        nodes: Vec::new(),
        leaves: Vec::new(),
    };
    state.visit(Visit {
        context: context.clone(),
        context_path: context_path.to_path_buf(),
        global: (0..unit_count).collect(),
        seats: districts,
        path: String::new(),
        offset: 0,
        seed: root_seed,
    })?;
    if state.assignment.iter().any(|&v| v < 0) {
        bail!("operational tree left units unassigned");
    }
    for district in 0..districts {
        if !connected(&context, &state.assignment, district as i64)? {
            bail!("district {district} is disconnected");
        }
    }
    let population_total: i64 = context["populations"]
        .as_array()
        .context("populations")?
        .iter()
        .map(|v| v.as_i64().unwrap_or(0))
        .sum();
    let claim="Complete connected wall-to-wall recursive assignment. Node objectives are discovery incumbents unless separately proved.";
    let tree = json!({"schema_version":"certified-operational-recursive-tree-v1","status":"operational-unproved-objectives","context_sha256":sha256(context_path)?,"context_hash":context["context_hash"],"districts":districts,"nodes":state.nodes,"leaves":state.leaves,"assignment":state.assignment,"unit_count":unit_count,"population_total":population_total,"claim_boundary":claim});
    let tree_path = out.join("operational-tree.json");
    write_json(&tree_path, &tree, true)?;
    let source = out.join(BUILDER_SNAPSHOT);
    fs::write(&source, include_bytes!("main.rs"))?;
    let state_name = context
        .pointer("/units/state")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_lowercase();
    let manifest = json!({"schema_version":"certified-operational-recursive-tree-package-v1","package_id":format!("operational-tree-{state_name}-2020"),"status":"operational-unproved-objectives","files":[{"path":"operational-tree.json","sha256":sha256(&tree_path)?}],"builder_path":BUILDER_SNAPSHOT,"builder_sha256":sha256(&source)?,"seed_frontier_max":max_seed,"claim_boundary":claim});
    write_json(&out.join("manifest.json"), &manifest, true)?;
    for entry in fs::read_dir(out)? {
        let path = entry?.path();
        let keep = path.file_name().and_then(|v| v.to_str()).is_some_and(|v| {
            v == "manifest.json" || v == "operational-tree.json" || v == BUILDER_SNAPSHOT
        });
        if !keep {
            remove_path(&path)?;
        }
    }
    println!("Operational recursive tree: VERIFIED");
    Ok(())
}

fn batch(bisect: &Path, limit: Option<usize>, retry_failed: bool, max_seed: u64) -> Result<()> {
    let root = std::env::current_dir()?;
    let inventory_path = root.join("docs/experiments/nationwide-2020/inventory.json");
    let ledger_path = root.join("docs/experiments/nationwide-2020/tree-build-ledger.json");
    let inventory = read_json(&inventory_path)?;
    let mut prior: BTreeMap<String, Value> = if ledger_path.is_file() {
        read_json(&ledger_path)?["results"]
            .as_array()
            .context("ledger results")?
            .iter()
            .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
            .collect()
    } else {
        BTreeMap::new()
    };
    let package_root = root.join("data/2020/certified/operational-trees");
    for source in inventory["states"].as_array().context("inventory states")? {
        let state = source["state"].as_str().context("state")?;
        let package = package_root.join(state.to_lowercase());
        if package.join("manifest.json").is_file() {
            prior.insert(state.into(), json!({"state":state,"districts":source["districts"],"block_count":source["block_count"],"status":"built","exit_code":0,"command":["recovered-from-package"],"output":"Recovered from verified package manifest."}));
        }
    }
    let failed: BTreeSet<String> = prior
        .iter()
        .filter_map(|(state, row)| (row["status"] == "failed").then_some(state.clone()))
        .collect();
    let mut states: Vec<Value> = inventory["states"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            let state = row["state"].as_str().unwrap_or("");
            row["districts"].as_u64().unwrap_or(0) > 1
                && (retry_failed || !failed.contains(state))
                && !package_root
                    .join(state.to_lowercase())
                    .join("manifest.json")
                    .is_file()
        })
        .cloned()
        .collect();
    states.sort_by_key(|row| {
        (
            row["block_count"].as_u64().unwrap_or(u64::MAX),
            row["state"].as_str().unwrap_or("").to_owned(),
        )
    });
    if let Some(limit) = limit {
        states.truncate(limit);
    }
    for row in states {
        let state = row["state"].as_str().context("state")?.to_owned();
        let lower = state.to_lowercase();
        let districts = row["districts"].as_u64().context("districts")? as usize;
        let context = root.join(format!("data/2020/certified/{lower}_blocks_2020.rctx"));
        let out = package_root.join(&lower);
        let result = build(bisect, &context, &out, districts, 1, [2, 3], max_seed);
        let (status, exit_code, output) = match result {
            Ok(()) => ("built", 0, "Built and verified by bisect-ops.".into()),
            Err(error) => ("failed", 1, format!("{error:#}")),
        };
        println!("{state}: {status}");
        prior.insert(state.clone(),json!({"state":state,"districts":districts,"block_count":row["block_count"],"status":status,"exit_code":exit_code,"command":["bisect-ops","batch",format!("--max-seed={max_seed}")],"output":output}));
        let results: Vec<_> = prior.values().cloned().collect();
        let built = results
            .iter()
            .filter(|row| row["status"] == "built")
            .count();
        let failed_count = results
            .iter()
            .filter(|row| row["status"] == "failed")
            .count();
        write_json(
            &ledger_path,
            &json!({"schema_version":"certified-national-tree-build-ledger-v1","results":results,"built_count":built,"failed_count":failed_count,"claim_boundary":"Resumable operational tree build ledger; national coverage verification is separate."}),
            true,
        )?;
    }
    Ok(())
}

fn audit_python(staged: bool, base: Option<&str>) -> Result<()> {
    let mut command = Command::new("git");
    command.args(["diff", "--name-only", "--diff-filter=AM"]);
    if staged {
        command.arg("--cached");
    } else if let Some(base) = base {
        command.arg(format!("{base}...HEAD"));
    } else {
        command.arg("HEAD");
    }
    let output = command.output().context("run git diff for Python policy")?;
    if !output.status.success() {
        bail!("git diff failed while enforcing Rust-first policy");
    }
    let blocked: Vec<_> = String::from_utf8(output.stdout)?
        .lines()
        .filter(|path| path.ends_with(".py"))
        .filter(|path| {
            *path != "setup_data.py"
                && !path.starts_with("python/bisect_py/")
                && !path.starts_with("scripts/data/")
        })
        .map(str::to_owned)
        .collect();
    if !blocked.is_empty() {
        bail!(
            "Rust-first policy blocks new or modified Python:\n{}",
            blocked.join("\n")
        );
    }
    println!("Rust-first Python boundary: PASS");
    Ok(())
}

fn write_source_snapshot(dir: &Path, name: &str) -> Result<(PathBuf, String)> {
    let path = dir.join(name);
    fs::create_dir_all(dir)?;
    fs::write(&path, include_bytes!("main.rs"))?;
    Ok((path.clone(), sha256(&path)?))
}

fn analyze_tree(
    state: &str,
    package: &Path,
    rctx_report_path: &Path,
    report_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    let package_manifest = read_json(&package.join("manifest.json"))?;
    let tree_path = package.join(
        package_manifest["files"][0]["path"]
            .as_str()
            .context("tree path")?,
    );
    if sha256(&tree_path)?
        != package_manifest["files"][0]["sha256"]
            .as_str()
            .context("tree hash")?
    {
        bail!("operational tree package hash mismatch");
    }
    let tree = read_json(&tree_path)?;
    let rctx = read_json(rctx_report_path)?;
    let leaves = tree["leaves"].as_array().context("leaves")?;
    let nodes = tree["nodes"].as_array().context("nodes")?;
    let leaf_units: u64 = leaves
        .iter()
        .map(|v| v["unit_count"].as_u64().unwrap_or(0))
        .sum();
    let leaf_population: i64 = leaves
        .iter()
        .map(|v| v["population"].as_i64().unwrap_or(0))
        .sum();
    let floors = nodes.iter().all(|node| {
        node.pointer("/objective/max_population_deviation_scaled")
            == node.pointer("/population_proof/lower_bound")
    });
    if tree["unit_count"] != rctx["unit_count"]
        || leaves.len() != tree["districts"].as_u64().unwrap_or(0) as usize
        || leaf_units != tree["unit_count"].as_u64().unwrap_or(u64::MAX)
        || leaf_population != tree["population_total"].as_i64().unwrap_or(i64::MIN)
        || !floors
    {
        bail!("operational tree coverage or population proof mismatch");
    }
    let claim="Complete connected wall-to-wall recursive tree with arithmetic population optimality at every node; boundary and canonical optimality are unproved.";
    let report = json!({"schema_version":"certified-operational-tree-frontier-v1","status":"operational-complete-population-proved","state":state,"year":2020,"districts":tree["districts"],"unit_count":tree["unit_count"],"population_total":tree["population_total"],"bridge_edge_count":rctx["bridge_edge_count"],"tree_sha256":sha256(&tree_path)?,"package_manifest_sha256":sha256(&package.join("manifest.json"))?,"nodes":nodes,"leaves":leaves,"boundary_proof":"not-run","canonical_proof":"blocked-by-boundary","claim_boundary":claim});
    write_json(report_path, &report, true)?;
    let parent = manifest_path.parent().context("manifest parent")?;
    let (source, source_hash) = write_source_snapshot(parent, "bisect-ops-analyzer-source.rs")?;
    let report_name = report_path
        .file_name()
        .context("report filename")?
        .to_string_lossy()
        .into_owned();
    let source_name = source
        .file_name()
        .context("source filename")?
        .to_string_lossy()
        .into_owned();
    let manifest = json!({"schema_version":"certified-operational-tree-frontier-package-v1","package_id":format!("{}-operational-tree-2020",state.to_lowercase()),"status":"operational-complete-population-proved","files":[{"path":report_name,"sha256":sha256(report_path)?}],"analyzer_path":source_name,"analyzer_sha256":source_hash,"claim_boundary":claim});
    write_json(manifest_path, &manifest, true)?;
    println!("{state} operational tree frontier: VERIFIED");
    Ok(())
}

fn verify_tree_report(manifest_path: &Path) -> Result<()> {
    let manifest = read_json(manifest_path)?;
    let parent = manifest_path.parent().context("manifest parent")?;
    let analyzer = PathBuf::from(
        manifest["analyzer_path"]
            .as_str()
            .context("analyzer path")?,
    );
    let source = if analyzer.components().count() == 1 {
        parent.join(analyzer)
    } else {
        custody_source(&analyzer)
    };
    if sha256(&source)?
        != manifest["analyzer_sha256"]
            .as_str()
            .context("analyzer hash")?
    {
        bail!("operational tree analyzer hash mismatch");
    }
    let report_path = parent.join(
        manifest["files"][0]["path"]
            .as_str()
            .context("report path")?,
    );
    if sha256(&report_path)?
        != manifest["files"][0]["sha256"]
            .as_str()
            .context("report hash")?
    {
        bail!("operational tree report hash mismatch");
    }
    let report = read_json(&report_path)?;
    let leaves = report["leaves"].as_array().context("leaves")?;
    let units: u64 = leaves
        .iter()
        .map(|v| v["unit_count"].as_u64().unwrap_or(0))
        .sum();
    if report["status"] != "operational-complete-population-proved"
        || leaves.len() != report["districts"].as_u64().unwrap_or(0) as usize
        || units != report["unit_count"].as_u64().unwrap_or(u64::MAX)
    {
        bail!("operational tree report posture drift");
    }
    println!("Operational tree frontier report verification: PASS");
    Ok(())
}

fn verify_national_rctx(out: &Path) -> Result<()> {
    let root = std::env::current_dir()?;
    let inventory = read_json(&out.join("inventory.json"))?;
    let mut states: Vec<String> = inventory["states"]
        .as_array()
        .context("states")?
        .iter()
        .filter_map(|v| v["state"].as_str().map(str::to_owned))
        .collect();
    states.sort();
    let mut rows = Vec::new();
    for state in states {
        let path = root.join(format!(
            "data/2020/certified/{}_blocks_2020.rctx",
            state.to_lowercase()
        ));
        let context = read_json(&path)?;
        let projection = json!({"units":context["units"],"graph":context["graph"],"populations":context["populations"],"source_hashes":context["source_hashes"]});
        if context["context_hash"] != canonical_hash(&projection)? {
            bail!("{state} context hash mismatch");
        }
        let adjacency = context
            .pointer("/graph/adjacency")
            .and_then(Value::as_array)
            .context("adjacency")?;
        let mut seen = BTreeSet::from([0usize]);
        let mut queue = VecDeque::from([0usize]);
        while let Some(unit) = queue.pop_front() {
            for edge in adjacency[unit].as_array().context("edges")? {
                let to = edge["to"].as_u64().context("edge.to")? as usize;
                if seen.insert(to) {
                    queue.push_back(to);
                }
            }
        }
        if seen.len() != adjacency.len() {
            bail!("{state} context is disconnected");
        }
        let directed: usize = adjacency
            .iter()
            .map(|v| v.as_array().map_or(0, Vec::len))
            .sum();
        let bridges: usize = adjacency
            .iter()
            .flat_map(|v| v.as_array().into_iter().flatten())
            .filter(|edge| edge["kind"] == "bridge")
            .count()
            / 2;
        let population: i64 = context["populations"]
            .as_array()
            .context("populations")?
            .iter()
            .map(|v| v.as_i64().unwrap_or(0))
            .sum();
        rows.push(json!({"state":state,"unit_count":context.pointer("/units/unit_ids").and_then(Value::as_array).context("unit_ids")?.len(),"population_total":population,"edge_count":directed/2,"bridge_edge_count":bridges,"rctx_bytes":fs::metadata(&path)?.len(),"rctx_sha256":sha256(&path)?,"context_hash":context["context_hash"],"status":"verified"}));
        println!(
            "{}: verified",
            rows.last().unwrap()["state"].as_str().unwrap()
        );
    }
    let sum = |key: &str| {
        rows.iter()
            .map(|v| v[key].as_u64().unwrap_or(0))
            .sum::<u64>()
    };
    let population: i64 = rows
        .iter()
        .map(|v| v["population_total"].as_i64().unwrap_or(0))
        .sum();
    let claim="All 50 local 2020 block contexts are hash-valid and connected; no district assignments are claimed.";
    let report = json!({"schema_version":"certified-national-rctx-verification-v1","status":"verified","state_count":rows.len(),"unit_count":sum("unit_count"),"population_total":population,"edge_count":sum("edge_count"),"bridge_edge_count":sum("bridge_edge_count"),"rctx_bytes":sum("rctx_bytes"),"states":rows,"claim_boundary":claim});
    let report_path = out.join("rctx-verification.json");
    write_json(&report_path, &report, true)?;
    let (source, source_hash) = write_source_snapshot(out, "bisect-ops-rctx-verifier-source.rs")?;
    let source_name = source
        .file_name()
        .context("source filename")?
        .to_string_lossy()
        .into_owned();
    let manifest = json!({"schema_version":"certified-national-rctx-verification-package-v1","package_id":"nationwide-2020-rctx-verification","status":"verified","files":[{"path":"rctx-verification.json","sha256":sha256(&report_path)?}],"verifier_path":source_name,"verifier_sha256":source_hash,"claim_boundary":claim});
    write_json(&out.join("rctx-manifest.json"), &manifest, true)?;
    println!(
        "National RCTX verification: {} States, {} blocks, {} bridges",
        report["state_count"], report["unit_count"], report["bridge_edge_count"]
    );
    Ok(())
}

fn rctx_batch(workers: usize, limit: Option<usize>, runtime: &Path, adapter: &Path) -> Result<()> {
    if workers == 0 {
        bail!("workers must be positive");
    }
    let root = std::env::current_dir()?;
    let inventory = read_json(&root.join("docs/experiments/nationwide-2020/inventory.json"))?;
    let rows = inventory["states"].as_array().context("states")?;
    let by_state: BTreeMap<_, _> = rows
        .iter()
        .filter_map(|row| Some((row["state"].as_str()?.to_owned(), row.clone())))
        .collect();
    let mut pending: Vec<Value> = inventory["batch_order"]
        .as_array()
        .context("batch_order")?
        .iter()
        .filter_map(|state| by_state.get(state.as_str()?).cloned())
        .filter(|row| {
            !root
                .join(format!(
                    "data/2020/certified/{}_blocks_2020.rctx",
                    row["state"].as_str().unwrap().to_lowercase()
                ))
                .is_file()
        })
        .collect();
    if let Some(limit) = limit {
        pending.truncate(limit);
    }
    let ledger_path = root.join("docs/experiments/nationwide-2020/rctx-build-ledger.json");
    if pending.is_empty() {
        let ledger = read_json(&ledger_path)?;
        println!(
            "National RCTX batch: {} built, {} failed, {} remaining",
            ledger["built_count"], ledger["failed_count"], ledger["remaining_count"]
        );
        return Ok(());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()?;
    let results:Vec<Value>=pool.install(||pending.par_iter().map(|row|{
        let state=row["state"].as_str().unwrap(); let lower=state.to_lowercase(); let state_name=row["name"].as_str().unwrap_or(state).to_lowercase().replace(' ',"_");
        let args=vec![adapter.to_string_lossy().into_owned(),"--state-code".into(),state.into(),"--state-fips".into(),row["fips"].as_str().unwrap_or("").into(),"--state-name".into(),state_name,"--rctx".into(),format!("data/2020/certified/{lower}_blocks_2020.rctx"),"--report".into(),format!("docs/experiments/nationwide-2020/rctx/{lower}.json"),"--manifest".into(),format!("docs/experiments/nationwide-2020/rctx/{lower}-manifest.json")];
        let output=Command::new(runtime).args(&args).current_dir(&root).output(); match output { Ok(out)=>{let mut text=String::from_utf8_lossy(&out.stdout).into_owned()+&String::from_utf8_lossy(&out.stderr);if text.len()>4000{text=text[text.len()-4000..].into();}json!({"state":state,"block_count":row["block_count"],"status":if out.status.success(){"built"}else{"failed"},"exit_code":out.status.code().unwrap_or(1),"command":std::iter::once(runtime.to_string_lossy().into_owned()).chain(args).collect::<Vec<_>>(),"output":text})},Err(error)=>json!({"state":state,"block_count":row["block_count"],"status":"failed","exit_code":1,"command":[runtime.to_string_lossy(),adapter.to_string_lossy()],"output":error.to_string()}) }
    }).collect());
    let mut merged: BTreeMap<String, Value> = if ledger_path.is_file() {
        read_json(&ledger_path)?["results"]
            .as_array()
            .context("results")?
            .iter()
            .filter_map(|row| Some((row["state"].as_str()?.into(), row.clone())))
            .collect()
    } else {
        BTreeMap::new()
    };
    for row in results {
        println!(
            "{}: {}",
            row["state"].as_str().unwrap(),
            row["status"].as_str().unwrap()
        );
        merged.insert(row["state"].as_str().unwrap().into(), row);
    }
    let results: Vec<_> = merged.values().cloned().collect();
    let built = results.iter().filter(|v| v["status"] == "built").count();
    let failed = results.iter().filter(|v| v["status"] == "failed").count();
    let remaining = inventory["batch_order"]
        .as_array()
        .unwrap()
        .len()
        .saturating_sub(results.len());
    let ledger = json!({"schema_version":"certified-national-rctx-build-ledger-v1","results":results,"built_count":built,"failed_count":failed,"remaining_count":remaining,"claim_boundary":"Resumable engineering ledger; aggregate verification occurs after all State contexts exist."});
    write_json(&ledger_path, &ledger, true)?;
    println!("National RCTX batch: {built} built, {failed} failed, {remaining} remaining");
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::Build {
            bisect,
            context,
            out_dir,
            districts,
            root_seed,
            child_seed_0,
            child_seed_1,
            max_seed,
        } => {
            if max_seed == 0 {
                bail!("max-seed must be positive");
            }
            build(
                &bisect,
                &context,
                &out_dir,
                districts,
                root_seed,
                [child_seed_0, child_seed_1],
                max_seed,
            )
        }
        Action::Verify { package } => {
            let manifest = read_json(&package.join("manifest.json"))?;
            let builder = PathBuf::from(manifest["builder_path"].as_str().context("builder path")?);
            let builder_source = if builder.components().count() == 1 {
                package.join(&builder)
            } else {
                custody_source(&builder)
            };
            if sha256(&builder_source)?
                != manifest["builder_sha256"]
                    .as_str()
                    .context("builder hash")?
            {
                bail!("operational tree builder hash mismatch");
            }
            if let Some(base) = manifest["base_builder_path"].as_str() {
                if sha256(&custody_source(Path::new(base)))?
                    != manifest["base_builder_sha256"]
                        .as_str()
                        .context("base builder hash")?
                {
                    bail!("operational tree base-builder hash mismatch");
                }
            }
            let tree_path =
                package.join(manifest["files"][0]["path"].as_str().context("tree path")?);
            if sha256(&tree_path)?
                != manifest["files"][0]["sha256"]
                    .as_str()
                    .context("tree hash")?
            {
                bail!("operational tree hash mismatch");
            }
            let tree = read_json(&tree_path)?;
            let districts = tree["districts"].as_u64().context("districts")?;
            let assignment = tree["assignment"].as_array().context("assignment")?;
            let unit_count = tree["unit_count"].as_u64().context("unit_count")?;
            let labels: BTreeSet<_> = assignment.iter().filter_map(Value::as_u64).collect();
            let expected: BTreeSet<_> = (0..districts).collect();
            let leaf_units: u64 = tree["leaves"]
                .as_array()
                .context("leaves")?
                .iter()
                .map(|leaf| leaf["unit_count"].as_u64().unwrap_or(0))
                .sum();
            let leaf_population: i64 = tree["leaves"]
                .as_array()
                .unwrap()
                .iter()
                .map(|leaf| leaf["population"].as_i64().unwrap_or(0))
                .sum();
            let floors_match = tree["nodes"]
                .as_array()
                .context("nodes")?
                .iter()
                .all(|node| {
                    node.pointer("/objective/max_population_deviation_scaled")
                        == node.pointer("/population_proof/lower_bound")
                });
            if districts < 2
                || assignment.len() as u64 != unit_count
                || labels != expected
                || leaf_units != unit_count
                || leaf_population != tree["population_total"].as_i64().unwrap_or(i64::MIN)
                || !floors_match
            {
                bail!("operational tree coverage drift");
            }
            println!("Operational recursive tree package verification: PASS");
            Ok(())
        }
        Action::Batch {
            bisect,
            limit,
            retry_failed,
            max_seed,
        } => batch(&bisect, limit, retry_failed, max_seed),
        Action::AuditPython { staged, base } => audit_python(staged, base.as_deref()),
        Action::AnalyzeTree {
            state,
            package,
            rctx_report,
            report,
            manifest,
        } => analyze_tree(&state, &package, &rctx_report, &report, &manifest),
        Action::VerifyTreeReport { manifest } => verify_tree_report(&manifest),
        Action::VerifyNationalRctx { out_dir } => verify_national_rctx(&out_dir),
        Action::RctxBatch {
            workers,
            limit,
            adapter_runtime,
            adapter,
        } => rctx_batch(workers, limit, &adapter_runtime, &adapter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_context() -> Value {
        json!({
            "rctx_version":"0.1","context_hash":"old",
            "units":{"unit_ids":["a","b","c"],"source_id":"fixture","state":"XY"},
            "graph":{"edge_semantics":"undirected","adjacency":[
                [{"to":1,"weight":4}],
                [{"to":0,"weight":4},{"to":2,"weight":5}],
                [{"to":1,"weight":5}]
            ]},
            "populations":[10,20,30],"source_hashes":{}
        })
    }

    #[test]
    fn arithmetic_floor_matches_ratio_bound() {
        assert_eq!(ratio_floor(101, 3, 2), 1);
        assert_eq!(ratio_floor(100, 2, 1), 0);
    }

    #[test]
    fn subset_remaps_edges_and_units() {
        let subset = subset_context(&tiny_context(), &[1, 2], "child".into()).unwrap();
        assert_eq!(subset["units"]["unit_ids"], json!(["b", "c"]));
        assert_eq!(subset["populations"], json!([20, 30]));
        assert_eq!(subset["graph"]["adjacency"][0][0]["to"], 1);
        assert!(subset["context_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
    }

    #[test]
    fn connectivity_detects_split_components() {
        let context = tiny_context();
        assert!(connected(&context, &[0, 0, 1], 0).unwrap());
        assert!(!connected(&context, &[0, 1, 0], 0).unwrap());
    }
}
