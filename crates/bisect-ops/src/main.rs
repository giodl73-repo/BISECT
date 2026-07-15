use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
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
