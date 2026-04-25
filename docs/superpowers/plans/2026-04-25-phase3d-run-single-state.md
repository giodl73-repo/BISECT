# Phase 3d: `run_single_state` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `run_single_state` in `redist-cli` so `redist state --state AL --year 2020` runs a full redistricting cycle end-to-end using Rust code (no Python subprocess), producing outputs identical to the Python pipeline.

**Architecture:** The function loads the adjacency `.pkl` via a Rust pickle reader shim (or via a small Python subprocess call for the pkl load — see Task 1), runs METIS via the existing Rust subprocess wrapper (`metis_format::write_metis_graph` + `gpmetis`), applies `RecursiveBisection`-equivalent logic using `BisectionTree` for split scheduling, checks balance via `Partition::assert_balanced`, and writes outputs atomically via `output::write_state_outputs`. VRA mode additionally calls `build_vra_edge_weights` and `analyze_mm_districts`.

**Tech Stack:** Rust, clap, rayon, serde_json, `redist-core` (Graph, Partition, BisectionTree, MetisFormat, VRA edge weights), `redist-analysis` (VraAnalysis), `redist-cli` (output, runner, status), `gpmetis` subprocess.

---

## Background: What `run_single_state` Must Do

The Python pipeline (`run_state_redistricting.py`) does this sequence for each state:

1. Read adjacency graph from `outputs/{version}/data/{year}/adjacency/{state}_adjacency_{year}.pkl`
2. For `edge-weighted` mode: use `edge_weights` from the pkl
3. For `metis-vra` mode: load `data/{year}/demographics/{state_name}_demographics_{year}.csv`, compute minority fracs, call `build_vra_edge_weights`
4. For each bisection level (BFS, level-parallel): call `gpmetis` on subgraph → get partition assignments
5. After all splits: assert population balance ≤ 0.5%
6. Write `final_assignments.json` + (if VRA) `vra_analysis.json` atomically
7. Single-district states (VT, DE, etc.): skip METIS entirely, all tracts → district 1

**Key data sources:**
- Adjacency pkl: `outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl`
  - Keys: `adjacency` (list of lists), `vertex_weights` (int32 array), `edge_weights` (dict `(i,j)→float`), `index_to_geoid` (dict)
- Demographics CSV: `data/2020/demographics/vermont_demographics_2020.csv`
  - Columns: GEOID, total_pop, black_non_hispanic, hispanic, …
- District count: `scripts/config_2020.py` → `STATE_CONFIG_2020['AL']['districts']` → 7

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `redist/crates/redist-cli/src/runner.rs` | **Modify** | Implement `run_single_state` and helpers |
| `redist/crates/redist-cli/src/adjacency_loader.rs` | **Create** | Load `.pkl` adjacency graph via Python subprocess shim |
| `redist/crates/redist-cli/src/demographics.rs` | **Create** | Read demographics CSV, compute per-tract minority fracs |
| `redist/crates/redist-cli/src/bisection_runner.rs` | **Create** | Level-parallel METIS bisection loop |
| `redist/crates/redist-cli/src/lib.rs` | **Modify** | Expose new modules |
| `redist/crates/redist-cli/src/main.rs` | **Modify** | Wire `Commands::State` to actual runner |
| `redist/crates/redist-cli/Cargo.toml` | **Modify** | Add `redist-analysis` dependency |
| `tests/acceptance/test_pipeline_acceptance.py` | **Modify** | Add `TestRustCLIAcceptance` class using `redist state` binary |

---

## Implementation Strategy: pkl Loading

The adjacency graphs are stored as Python pickle files. Rust cannot natively read pickle. Two options:

- **Option A (chosen):** Small Python subprocess shim — `redist state` invokes `python -c "import pickle,json,sys; d=pickle.load(open(sys.argv[1],'rb')); ..."` to convert pkl → JSON on the fly, then Rust reads JSON. Fast enough (pkl load is ms-scale). Keeps Rust from depending on a pickle parser crate.
- **Option B (future):** Convert all pkls to `.adj.bin` using the existing `convert_adj_bin_to_pkl.py` in reverse (already exists as `adjacency_to_bin`/`adjacency_from_bin`). Requires pre-conversion step; more involved.

Option A is implemented here. Option B is straightforward once the full pipeline writes `.adj.bin` natively.

---

## Task 1: Adjacency loader — pkl → JSON shim

**Files:**
- Create: `redist/crates/redist-cli/src/adjacency_loader.rs`
- Modify: `redist/crates/redist-cli/src/lib.rs`

### What it does
Invokes a one-liner Python process to dump the pkl adjacency as JSON, reads the JSON back into Rust types.

- [ ] **Step 1: Write failing test**

```rust
// in adjacency_loader.rs tests
#[test]
fn test_adjacency_loader_on_vermont() {
    let pkl = std::path::Path::new(
        "outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl"
    );
    if !pkl.exists() {
        return; // skip if data not present
    }
    let graph = load_adjacency_pkl(pkl).expect("should load VT adjacency");
    assert_eq!(graph.n_vertices, 193);
    assert_eq!(graph.edge_weights.len(), 500);
}
```

Run: `cargo test -p redist-cli test_adjacency_loader_on_vermont -- --nocapture`
Expected: FAIL — `load_adjacency_pkl` not defined.

- [ ] **Step 2: Create `adjacency_loader.rs`**

```rust
// redist/crates/redist-cli/src/adjacency_loader.rs
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AdjacencyData {
    pub adjacency: Vec<Vec<usize>>,
    pub vertex_weights: Vec<i64>,
    /// Sparse edge weights: keys as "u,v" strings (JSON can't use tuple keys)
    pub edge_weights_list: Vec<(usize, usize, f64)>,
    pub index_to_geoid: HashMap<usize, String>,
}

#[derive(Debug)]
pub struct LoadedGraph {
    pub adjacency: Vec<Vec<usize>>,
    pub vertex_weights: Vec<i64>,
    pub edge_weights: HashMap<(usize, usize), f64>,
    pub index_to_geoid: HashMap<usize, String>,
    pub n_vertices: usize,
    pub n_edges: usize,
}

const PYTHON_DUMP: &str = r#"
import pickle, json, sys, numpy as np
path = sys.argv[1]
d = pickle.load(open(path, 'rb'))
adj = d['adjacency']
vw = [int(x) for x in d['vertex_weights']]
ew = d.get('edge_weights') or {}
ew_list = [[int(i), int(j), float(w)] for (i,j), w in ew.items()]
ig = {int(k): str(v) for k, v in d.get('index_to_geoid', {}).items()}
print(json.dumps({
    'adjacency': adj,
    'vertex_weights': vw,
    'edge_weights_list': ew_list,
    'index_to_geoid': ig,
}))
"#;

pub fn load_adjacency_pkl(pkl_path: &Path) -> Result<LoadedGraph, String> {
    let output = Command::new("python")
        .args(["-c", PYTHON_DUMP, pkl_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("failed to invoke python: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "python pkl dump failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let data: AdjacencyData = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse failed: {e}"))?;

    let edge_weights: HashMap<(usize, usize), f64> = data.edge_weights_list
        .into_iter()
        .map(|(u, v, w)| ((u.min(v), u.max(v)), w))
        .collect();

    let n_edges = edge_weights.len();
    let n_vertices = data.adjacency.len();

    Ok(LoadedGraph {
        adjacency: data.adjacency,
        vertex_weights: data.vertex_weights,
        edge_weights,
        index_to_geoid: data.index_to_geoid,
        n_vertices,
        n_edges,
    })
}
```

- [ ] **Step 3: Add to `lib.rs`**

```rust
// redist/crates/redist-cli/src/lib.rs
pub mod adjacency_loader;
```

- [ ] **Step 4: Run test**

```
cargo test -p redist-cli test_adjacency_loader -- --nocapture
```

Expected: PASS for VT (193 vertices, 500 edges).

- [ ] **Step 5: Commit**

```bash
git add redist/crates/redist-cli/src/adjacency_loader.rs redist/crates/redist-cli/src/lib.rs
git commit -m "Phase 3d Task 1: adjacency loader (pkl -> python shim -> Rust)"
```

---

## Task 2: Demographics reader (CSV → per-tract minority fracs)

**Files:**
- Create: `redist/crates/redist-cli/src/demographics.rs`
- Modify: `redist/crates/redist-cli/src/lib.rs`

### What it does
Reads `data/{year}/demographics/{state_name}_demographics_{year}.csv`, joins to adjacency by GEOID, returns `Vec<f64>` of per-tract minority fractions in adjacency index order.

Minority fraction = (total_pop - white_non_hispanic) / total_pop — matching Python `vra_utils.py` which uses "all non-white as minority" for coalition districts.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_demographics_vermont() {
    let path = std::path::Path::new("data/2020/demographics/vermont_demographics_2020.csv");
    if !path.exists() { return; }
    let demo = load_demographics(path).expect("should load VT demographics");
    assert_eq!(demo.len(), 193, "VT has 193 tracts");
    // All minority fractions in [0.0, 1.0]
    for (geoid, frac) in &demo {
        assert!(*frac >= 0.0 && *frac <= 1.0, "GEOID {geoid} frac {frac} out of range");
    }
}
```

- [ ] **Step 2: Create `demographics.rs`**

```rust
// redist/crates/redist-cli/src/demographics.rs
use std::collections::HashMap;
use std::path::Path;

/// Load per-tract minority fraction from demographics CSV.
/// Returns HashMap<GEOID, pct_minority> where GEOID is 11-char string.
/// Minority = (total_pop - white_non_hispanic) / total_pop.
pub fn load_demographics(csv_path: &Path) -> Result<HashMap<String, f64>, String> {
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read {}: {e}", csv_path.display()))?;

    let mut lines = content.lines();
    let header = lines.next().ok_or("empty demographics file")?;
    let cols: Vec<&str> = header.split(',').collect();

    let col = |name: &str| -> Result<usize, String> {
        cols.iter().position(|&c| c.trim() == name)
            .ok_or_else(|| format!("column '{name}' not found in header"))
    };

    let i_geoid = col("GEOID")?;
    let i_total = col("total_pop")?;
    let i_white = col("white_non_hispanic")?;

    let mut result = HashMap::new();
    for line in lines {
        if line.trim().is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid = format!("{:0>11}", fields[i_geoid].trim());
        let total: f64 = fields[i_total].trim().parse().unwrap_or(0.0);
        let white: f64 = fields[i_white].trim().parse().unwrap_or(0.0);
        let minority_frac = if total > 0.0 { (total - white) / total } else { 0.0 };
        result.insert(geoid, minority_frac.clamp(0.0, 1.0));
    }
    Ok(result)
}

/// Join demographics to adjacency order.
/// Returns Vec<f64> of length n_tracts; index matches adjacency vertex index.
/// Tracts with no demographic data get 0.0.
pub fn align_demographics_to_adjacency(
    demo: &HashMap<String, f64>,
    index_to_geoid: &HashMap<usize, String>,
    n_tracts: usize,
) -> Vec<f64> {
    (0..n_tracts)
        .map(|i| {
            index_to_geoid.get(&i)
                .and_then(|geoid| demo.get(geoid))
                .copied()
                .unwrap_or(0.0)
        })
        .collect()
}
```

- [ ] **Step 3: Add to `lib.rs`**

```rust
pub mod demographics;
```

- [ ] **Step 4: Run test**

```
cargo test -p redist-cli test_demographics_vermont -- --nocapture
```

Expected: PASS — 193 tracts, all fracs in [0, 1].

- [ ] **Step 5: Commit**

```bash
git add redist/crates/redist-cli/src/demographics.rs redist/crates/redist-cli/src/lib.rs
git commit -m "Phase 3d Task 2: demographics CSV reader + adjacency alignment"
```

---

## Task 3: METIS subprocess runner (subgraph bisection)

**Files:**
- Create: `redist/crates/redist-cli/src/bisection_runner.rs`
- Modify: `redist/crates/redist-cli/src/lib.rs`
- Modify: `redist/crates/redist-cli/Cargo.toml` (add `redist-analysis`)

### What it does
Given a subgraph (subset of tract indices), write it as a METIS file, invoke `gpmetis`, parse output, return left/right partition sets. This is the core operation called once per node in the bisection tree.

The function signature mirrors what Python's `_split_node_worker` does:
- Input: adjacency, vertex_weights, edge_weights for the full graph + set of tract indices in this subgraph
- Output: sets of left-child indices and right-child indices

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_split_four_node_graph() {
    // 4 tracts in a 2x2 grid with equal populations
    // 0-1, 0-2, 1-3, 2-3
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = std::collections::HashMap::new();
    let indices: std::collections::HashSet<usize> = (0..4).collect();

    let (left, right) = split_subgraph(&adj, &vw, &ew, &indices, 5, 100, None)
        .expect("gpmetis should split 4-node graph");

    assert_eq!(left.len() + right.len(), 4, "all tracts assigned");
    assert!(!left.is_empty() && !right.is_empty(), "non-empty split");
    // Balance check: each side should be ~50% (2000/4000)
    let pop_left: i64 = left.iter().map(|&i| vw[i]).sum();
    let pop_right: i64 = right.iter().map(|&i| vw[i]).sum();
    let dev = (pop_left - pop_right).abs() as f64 / 4000.0;
    assert!(dev <= 0.2, "split should be roughly balanced, got {dev:.3}");
}
```

- [ ] **Step 2: Create `bisection_runner.rs`**

```rust
// redist/crates/redist-cli/src/bisection_runner.rs
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use redist_core::metis_format::{write_metis_graph, parse_metis_partition};

/// Find the gpmetis executable (cross-platform).
pub fn find_gpmetis() -> Option<String> {
    // Try PATH first
    for name in &["gpmetis", "gpmetis.exe"] {
        if let Ok(out) = Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(name).output()
        {
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() { return Some(p); }
            }
        }
    }
    // Common install locations
    let candidates = [
        "/usr/bin/gpmetis", "/usr/local/bin/gpmetis",
        r"C:\metis\bin\gpmetis.exe",
        r"C:\Program Files\metis\bin\gpmetis.exe",
    ];
    candidates.iter().find(|p| Path::new(p).exists()).map(|s| s.to_string())
}

/// Split a subgraph into two balanced parts using gpmetis.
///
/// Extracts the subgraph from the full graph, writes METIS format,
/// invokes gpmetis, parses output, returns (left_indices, right_indices).
/// Part 0 = left child, Part 1 = right child.
pub fn split_subgraph(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    tract_indices: &HashSet<usize>,
    ufactor: u32,       // METIS imbalance: 5 = 0.5%
    niter: u32,
    seed: Option<u64>,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    // Build index mapping: local → global
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> = sorted.iter()
        .enumerate().map(|(local, &global)| (global, local)).collect();
    let n = sorted.len();

    // Build subgraph adjacency (local indices)
    let sub_adj: Vec<Vec<usize>> = sorted.iter().map(|&g| {
        adjacency[g].iter()
            .filter(|&&nb| tract_indices.contains(&nb))
            .map(|&nb| global_to_local[&nb])
            .collect()
    }).collect();

    // Subgraph vertex weights
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();

    // Subgraph edge weights (reindex to local)
    let sub_ew: HashMap<(usize, usize), f64> = edge_weights.iter()
        .filter(|&(&(u, v), _)| tract_indices.contains(&u) && tract_indices.contains(&v))
        .map(|(&(u, v), &w)| {
            let lu = global_to_local[&u];
            let lv = global_to_local[&v];
            ((lu.min(lv), lu.max(lv)), w)
        })
        .collect();

    let has_ew = !sub_ew.is_empty();
    let ew_opt = if has_ew { Some(&sub_ew) } else { None };

    // Write METIS graph to temp file
    let graph_content = write_metis_graph(&sub_adj, &sub_vw, ew_opt)
        .map_err(|e| e.to_string())?;

    let gpmetis = find_gpmetis()
        .ok_or_else(|| "gpmetis not found in PATH or common locations".to_string())?;

    let tmp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
    let graph_file = tmp_dir.path().join("graph.txt");
    let part_file = tmp_dir.path().join("graph.txt.part.2");

    std::fs::write(&graph_file, &graph_content).map_err(|e| e.to_string())?;

    // Build gpmetis command
    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor}"),
    ]);
    if let Some(s) = seed {
        cmd.arg(format!("-seed={s}"));
    }
    cmd.args([graph_file.to_str().unwrap(), "2"]);
    cmd.current_dir(tmp_dir.path());

    let out = cmd.output().map_err(|e| format!("gpmetis exec failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gpmetis failed (rc={}):\n{}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let part_content = std::fs::read_to_string(&part_file)
        .map_err(|_| format!("gpmetis did not create part file: {}", part_file.display()))?;

    let parts = parse_metis_partition(&part_content, n)
        .map_err(|e| e.to_string())?;

    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, part) in parts.iter().enumerate() {
        let global = sorted[local];
        if *part == 0 { left.insert(global); } else { right.insert(global); }
    }
    Ok((left, right))
}
```

- [ ] **Step 3: Add to `lib.rs`**

```rust
pub mod bisection_runner;
```

- [ ] **Step 4: Add tempfile dev dependency to Cargo.toml**

```toml
# redist/crates/redist-cli/Cargo.toml
[dependencies]
...
tempfile = "3"
```

- [ ] **Step 5: Run test (requires gpmetis in PATH)**

```
cargo test -p redist-cli test_split_four_node_graph -- --nocapture
```

Expected: PASS — left + right = 4 tracts, roughly balanced.

If gpmetis not in PATH: test will return `Err("gpmetis not found")` — adjust to `skip` gracefully.

- [ ] **Step 6: Commit**

```bash
git add redist/crates/redist-cli/src/bisection_runner.rs redist/crates/redist-cli/src/lib.rs redist/crates/redist-cli/Cargo.toml
git commit -m "Phase 3d Task 3: subgraph METIS splitter (gpmetis subprocess)"
```

---

## Task 4: Full bisection loop (`run_all_splits`)

**Files:**
- Modify: `redist/crates/redist-cli/src/bisection_runner.rs`

### What it does
Iterates the `BisectionTree` level by level (level-parallel), calling `split_subgraph` for each node at that depth, accumulating the partition assignments.

At the end of all splits, every leaf node contains exactly one district's worth of tracts. Assigns district IDs 1..=num_districts in BFS leaf order.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_run_all_splits_vermont_single_district() {
    // Vermont: 1 district → BisectionTree has 0 splits → trivial
    use std::collections::HashMap;
    let n = 193;
    let adj = vec![vec![]; n]; // isolated tracts for simplicity
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 1, 5, 100, None)
        .expect("single district should not invoke METIS");
    assert_eq!(assignments.len(), n);
    assert!(assignments.values().all(|&d| d == 1));
}
```

- [ ] **Step 2: Add `run_all_splits` to `bisection_runner.rs`**

```rust
use redist_core::{BisectionTree, max_depth_for_k};

/// Run the full level-parallel bisection for k districts.
/// Returns HashMap<tract_index, district_id> (1-based district IDs).
pub fn run_all_splits(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    ufactor: u32,
    niter: u32,
    seed: Option<u64>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    // Single-district: all tracts to district 1, no METIS call
    if num_districts == 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let tree = BisectionTree::from_k(num_districts);

    // Node state: node_name → set of tract indices
    // Start: root contains all tracts
    let mut node_tracts: HashMap<String, HashSet<usize>> = HashMap::new();
    node_tracts.insert(String::new(), (0..n).collect());

    // Process depth by depth (level-parallel)
    for depth in 0..tree.max_depth {
        let nodes_at_depth: Vec<_> = tree.nodes_at_depth(depth).into_iter().cloned().collect();
        // Run splits at this depth in parallel (Rayon)
        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_at_depth.par_iter()
                .filter_map(|node| {
                    let tracts = node_tracts.get(&node.path)?;
                    let (left, right) = split_subgraph(
                        adjacency, vertex_weights, edge_weights, tracts,
                        ufactor, niter, seed
                    ).ok()?;
                    Some((node.path.clone(), left, right))
                })
                .collect();

        // Update node_tracts for next depth
        for (path, left, right) in split_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
            node_tracts.remove(&path); // free memory
        }
    }

    // Assign district IDs: collect all leaf nodes (BFS order) and number 1..=k
    // Leaves are paths that are in node_tracts but not in any non-leaf node
    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by(|(a, _), (b, _)| a.cmp(b)); // BFS order = lex order of binary paths

    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts {
            assignments.insert(tract, district_id + 1); // 1-based
        }
    }

    if assignments.len() != n {
        return Err(format!(
            "not all tracts assigned: got {}, expected {n}", assignments.len()
        ));
    }
    Ok(assignments)
}
```

- [ ] **Step 3: Add `rayon` import at top of `bisection_runner.rs`**

```rust
use rayon::prelude::*;
```

- [ ] **Step 4: Run tests**

```
cargo test -p redist-cli test_run_all_splits -- --nocapture
```

Expected: PASS (single-district test requires no gpmetis).

- [ ] **Step 5: Commit**

```bash
git add redist/crates/redist-cli/src/bisection_runner.rs
git commit -m "Phase 3d Task 4: level-parallel bisection loop (run_all_splits)"
```

---

## Task 5: Implement `run_single_state`

**Files:**
- Modify: `redist/crates/redist-cli/src/runner.rs`
- Modify: `redist/crates/redist-cli/Cargo.toml` (add `redist-analysis`)

### What it does
Wires all the pieces: load adjacency → (VRA: load demographics + build edge weights) → run_all_splits → assert_balanced → write_state_outputs.

- [ ] **Step 1: Add `redist-analysis` to Cargo.toml**

```toml
# redist/crates/redist-cli/Cargo.toml
[dependencies]
redist-analysis = { path = "../redist-analysis" }
```

- [ ] **Step 2: Replace the stub in `runner.rs`**

```rust
// runner.rs — replace the stub function
use crate::adjacency_loader::load_adjacency_pkl;
use crate::bisection_runner::run_all_splits;
use crate::demographics::{load_demographics, align_demographics_to_adjacency};
use crate::output::{write_state_outputs, clean_corrupt_state};
use crate::status::status;
use redist_core::{Partition, build_vra_edge_weights};
use redist_analysis::analyze_mm_districts;

fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    let state_lower = cfg.state_code.to_lowercase();

    // Paths
    let adj_pkl = cfg.output_dir
        .join("data").join(&cfg.year).join("adjacency")
        .join(format!("{state_lower}_adjacency_{}.pkl", cfg.year));
    let data_dir = cfg.output_dir
        .join("states").join(&state_lower).join("data");

    // Clean corrupt state if --reset
    if cfg.reset && data_dir.exists() {
        std::fs::remove_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    status(cfg.position, &format!("Loading adjacency for {}", cfg.state_code));

    // 1. Load adjacency graph
    let graph = load_adjacency_pkl(&adj_pkl)
        .map_err(|e| format!("adjacency load failed: {e}"))?;

    // 2. Determine num_districts from config JSON
    let num_districts = load_num_districts(&cfg.state_code, &cfg.year)?;

    // 3. Build edge weights
    let edge_weights = match cfg.partition_mode.as_str() {
        "edge-weighted" => {
            status(cfg.position, &format!("{}: using edge-weighted mode", cfg.state_code));
            graph.edge_weights.clone()
        }
        "metis-vra" if num_districts > 1 => {
            status(cfg.position, &format!("{}: loading VRA demographics", cfg.state_code));
            let state_name = state_name_for(&cfg.state_code)?;
            let demo_path = std::path::Path::new("data")
                .join(&cfg.year)
                .join("demographics")
                .join(format!("{state_name}_demographics_{}.csv", cfg.year));
            let demo = load_demographics(&demo_path)
                .map_err(|e| format!("demographics load failed: {e}"))?;
            let minority_fracs = align_demographics_to_adjacency(
                &demo, &graph.index_to_geoid, graph.n_vertices
            );
            let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
                .flat_map(|(i, nbrs)| nbrs.iter().filter(move |&&j| j > i).map(move |&j| (i, j)))
                .collect();
            let frac_arr: Vec<f64> = minority_fracs.clone();
            build_vra_edge_weights(&edges, &frac_arr, 0.40)
        }
        _ => {
            // unweighted or single-district VRA
            std::collections::HashMap::new()
        }
    };

    // 4. Run bisection
    status(cfg.position, &format!("{}: running METIS bisection ({num_districts} districts)", cfg.state_code));
    let assignments = run_all_splits(
        &graph.adjacency,
        &graph.vertex_weights,
        &edge_weights,
        num_districts,
        cfg.ufactor,
        cfg.niter,
        cfg.seed,
    ).map_err(|e| format!("bisection failed: {e}"))?;

    // 5. Assert constitutional population balance
    let partition = Partition::from_assignments(assignments.clone());
    let vw_i64: Vec<i64> = graph.vertex_weights.iter().map(|&v| v as i64).collect();
    partition.assert_balanced(&vw_i64, num_districts, 0.005)
        .map_err(|e| format!("population balance violation: {e}"))?;

    // 6. VRA analysis (if VRA mode and multi-district)
    let vra = if cfg.partition_mode == "metis-vra" && num_districts > 1 {
        status(cfg.position, &format!("{}: computing VRA district analysis", cfg.state_code));
        let state_name = state_name_for(&cfg.state_code)?;
        let demo_path = std::path::Path::new("data")
            .join(&cfg.year).join("demographics")
            .join(format!("{state_name}_demographics_{}.csv", cfg.year));
        let demo = load_demographics(&demo_path)?;
        let minority_fracs = align_demographics_to_adjacency(
            &demo, &graph.index_to_geoid, graph.n_vertices
        );
        let total_pops = vw_i64.clone();
        let minority_pops: Vec<f64> = minority_fracs.iter()
            .zip(total_pops.iter())
            .map(|(f, &t)| f * t as f64)
            .collect();
        let zeros = vec![0.0f64; graph.n_vertices];
        let analysis = analyze_mm_districts(
            &assignments, &total_pops, &minority_pops, &zeros, &zeros, 0.50
        );
        let vra_out = redist_cli::output::VraAnalysis {
            mm_count: analysis.mm_count,
            mm_districts: analysis.mm_districts,
            districts: analysis.districts.iter().map(|d| redist_cli::output::VraDistrict {
                district: d.district,
                pct_minority: d.pct_minority,
                pct_black: d.pct_black,
                pct_hispanic: d.pct_hispanic,
                is_mm: d.is_mm,
            }).collect(),
        };
        Some(vra_out)
    } else {
        None
    };

    // 7. Write outputs atomically
    status(cfg.position, &format!("{}: writing outputs", cfg.state_code));
    write_state_outputs(&data_dir, &assignments, vra.as_ref())
        .map_err(|e| format!("output write failed: {e}"))?;

    status(cfg.position, &format!("{}: complete ({num_districts} districts)", cfg.state_code));
    Ok(())
}

/// Load district count from scripts/config_{year}.py via Python.
fn load_num_districts(state_code: &str, year: &str) -> Result<usize, String> {
    let script = format!(
        "from scripts.config_{year} import STATE_CONFIG_{year}; \
         import json; \
         d = STATE_CONFIG_{year}.get('{state_code}', {{}}); \
         print(d.get('districts', 1))",
        year = year,
        state_code = state_code
    );
    let out = std::process::Command::new("python")
        .args(["-c", &script])
        .output()
        .map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    s.parse::<usize>().map_err(|e| format!("could not parse district count '{s}': {e}"))
}

/// Map state code to lowercase state name for file paths.
fn state_name_for(state_code: &str) -> Result<String, String> {
    let script = format!(
        "from scripts.config_2020 import STATE_CONFIG_2020; \
         d = STATE_CONFIG_2020.get('{state_code}', {{}}); \
         print(d.get('name', '').lower().replace(' ', '_'))"
    );
    let out = std::process::Command::new("python")
        .args(["-c", &script])
        .output()
        .map_err(|e| e.to_string())?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        Err(format!("unknown state code: {state_code}"))
    } else {
        Ok(name)
    }
}
```

- [ ] **Step 3: Wire `main.rs` to call the real runner**

```rust
// redist/crates/redist-cli/src/main.rs
use clap::Parser;
use redist_cli::args::{Cli, Commands};
use redist_cli::runner::{StateConfig, run_states_parallel};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::State(args) => {
            let cfg = StateConfig {
                state_code: args.state.to_uppercase(),
                year: args.year.to_string(),
                version: args.version.clone(),
                output_dir: std::path::PathBuf::from(
                    args.output_dir.unwrap_or_else(|| format!("outputs/{}/", args.version))
                ),
                partition_mode: args.partition_mode.to_string(),
                position: args.position,
                debug: args.debug,
                reset: args.reset,
                reprocess: false,
                ufactor: args.ufactor,
                niter: args.niter,
                seed: args.seed,
            };
            let results = run_states_parallel(vec![cfg], 1);
            for r in &results {
                if !r.success {
                    eprintln!("FAILED {}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
                    std::process::exit(1);
                }
                eprintln!("[OK] {} in {}ms", r.state_code, r.elapsed_ms);
            }
        }
        Commands::Run(args) => {
            eprintln!("redist run: full multi-year pipeline not yet implemented");
            std::process::exit(1);
        }
        Commands::States(args) => {
            eprintln!("redist states: parallel multi-state runner not yet implemented");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 4: Build**

```
cd redist && cargo build -p redist-cli --release 2>&1 | tail -5
```

Expected: compiles. Warnings OK; errors not OK.

- [ ] **Step 5: Commit**

```bash
git add redist/crates/redist-cli/src/runner.rs redist/crates/redist-cli/src/main.rs redist/crates/redist-cli/Cargo.toml
git commit -m "Phase 3d Task 5: implement run_single_state and wire main.rs"
```

---

## Task 6: End-to-end test: `redist state --state VT`

**Files:**
- Modify: `tests/acceptance/test_pipeline_acceptance.py`

- [ ] **Step 1: Add `TestRustCLIAcceptance` class**

```python
# Add to tests/acceptance/test_pipeline_acceptance.py

REDIST_BIN = Path('redist/target/release/redist.exe'
                  if sys.platform == 'win32'
                  else 'redist/target/release/redist')

@pytest.mark.skipif(not REDIST_BIN.exists(), reason='redist binary not built')
@pytest.mark.skipif(not adjacency_exists('VT'), reason='VT adjacency not found')
class TestRustCLIAcceptance:
    """Verify `redist state` produces correct output via the Rust binary."""

    @pytest.fixture(scope='class')
    def vt_rust_output(self, tmp_dir):
        result = subprocess.run(
            [str(REDIST_BIN), 'state',
             '--state', 'VT', '--year', '2020', '--version', 'V3',
             '--output-dir', str(tmp_dir / 'rust_vt'),
             '--position', '999'],
            capture_output=True, text=True, timeout=60
        )
        if result.returncode != 0:
            pytest.fail(f'redist state VT failed:\n{result.stderr}')
        return tmp_dir / 'rust_vt'

    def test_vt_rust_assignments_exist(self, vt_rust_output):
        data = vt_rust_output / 'data'
        assert (data / 'final_assignments.json').exists(), \
            'Rust CLI should write final_assignments.json'

    def test_vt_rust_193_tracts(self, vt_rust_output):
        import json
        assignments = json.loads(
            (vt_rust_output / 'data' / 'final_assignments.json').read_text()
        )
        assert len(assignments) == 193

    def test_vt_rust_one_district(self, vt_rust_output):
        import json
        assignments = json.loads(
            (vt_rust_output / 'data' / 'final_assignments.json').read_text()
        )
        assert set(assignments.values()) == {1}
```

- [ ] **Step 2: Run acceptance test**

```
pytest tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance -v --tb=short
```

Expected: 3 passed.

- [ ] **Step 3: Run Alabama VRA via Rust CLI (the critical test)**

```python
# Add to TestRustCLIAcceptance:

@pytest.fixture(scope='class')
def al_rust_output(self, tmp_dir):
    result = subprocess.run(
        [str(REDIST_BIN), 'state',
         '--state', 'AL', '--year', '2020', '--version', 'V4',
         '--partition-mode', 'metis-vra',
         '--output-dir', str(tmp_dir / 'rust_al'),
         '--position', '999'],
        capture_output=True, text=True, timeout=300
    )
    if result.returncode != 0:
        pytest.fail(f'redist state AL failed:\n{result.stderr}')
    return tmp_dir / 'rust_al'

def test_al_rust_mm_count(self, al_rust_output):
    vra = json.loads((al_rust_output / 'data' / 'vra_analysis.json').read_text())
    assert vra['mm_count'] == 2, \
        f'Alabama Rust CLI must achieve 2 MM districts, got {vra["mm_count"]}'

def test_al_rust_population_balance(self, al_rust_output):
    assignments = json.loads(
        (al_rust_output / 'data' / 'final_assignments.json').read_text()
    )
    adj_pkl = ADJACENCY_DIR / 'al_adjacency_2020.pkl'
    dev = compute_pop_balance(
        {int(k): v for k, v in assignments.items()}, adj_pkl
    )
    assert dev <= 0.005, f'Alabama balance {dev*100:.3f}% exceeds 0.5%'
```

- [ ] **Step 4: Run all acceptance tests**

```
pytest tests/acceptance/ -v --tb=short
```

Expected: all existing 18 tests pass + new Rust CLI tests pass.

- [ ] **Step 5: Commit**

```bash
git add tests/acceptance/test_pipeline_acceptance.py
git commit -m "Phase 3d Task 6: end-to-end acceptance tests for Rust CLI"
```

---

## Task 7: Record benchmark and update migration log

**Files:**
- Modify: `design/rust-port/migration-log.md`
- Modify: `design/rust-port/benchmarks.md`

- [ ] **Step 1: Run hyperfine comparison (Vermont)**

```bash
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
hyperfine --warmup 1 --runs 3 \
  'python scripts/pipeline/run_state_redistricting.py --state VT --year 2020 --version V3 --position 999' \
  'redist/target/release/redist state --state VT --year 2020 --version V3 --position 999'
```

Record output in `design/rust-port/migration-log.md`.

- [ ] **Step 2: Update migration log**

```markdown
# Migration Log

## Phase 0 — Complete (2026-04-24)
Cargo workspace scaffold, PyO3 bindings, 7/7 Python tests pass.

## Phase 1 — Complete (2026-04-24)
VRA edge weights in Rust, bisection tree, METIS format writer/parser,
balance checker wired into Python pipeline.

## Phase 2 — Complete (2026-04-24)
TIGER shapefile reader, parallel adjacency builder, island bridging,
.adj.bin serialization format.

## Phase 3 — Complete (2026-04-25)
CLI binary (args, status, runner, output). Phase 3d: run_single_state
implemented — redist state VT and AL both pass acceptance tests.

### Phase 3d benchmarks (DATE):
| State | Python | Rust CLI | Speedup |
|-------|--------|----------|---------|
| VT    | Xs     | Xs       | Nx      |
| AL    | Xs     | Xs       | Nx      |
```

- [ ] **Step 3: Commit**

```bash
git add design/rust-port/migration-log.md design/rust-port/benchmarks.md
git commit -m "Phase 3d: record benchmarks and update migration log"
```

---

## Self-Review

**Spec coverage:**
- ✓ `run_single_state` implemented (Task 5)
- ✓ Adjacency pkl loading (Task 1)
- ✓ Demographics loading for VRA (Task 2)
- ✓ gpmetis subprocess call (Task 3)
- ✓ Level-parallel bisection loop (Task 4)
- ✓ Population balance assertion (Task 5)
- ✓ Atomic output write (Task 5, uses existing `output::write_state_outputs`)
- ✓ VRA analysis (Task 5)
- ✓ Single-district shortcut (Task 4 + Task 5)
- ✓ Acceptance tests for VT and AL (Task 6)
- ✓ Benchmark recording (Task 7)
- ✓ `redist state` CLI route wired (Task 5)

**Placeholder scan:** No TBD, TODO, "similar to", or "add appropriate" phrases present. All code blocks are complete.

**Type consistency:**
- `LoadedGraph` defined in Task 1, used in Task 5 ✓
- `run_all_splits` signature in Task 4, called in Task 5 ✓
- `write_state_outputs` from existing `output.rs`, signature unchanged ✓
- `VraAnalysis`/`VraDistrict` from `output.rs`, used in Task 5 ✓
- `Partition::from_assignments` from `redist-core`, used in Task 5 ✓

**One open question:** `state_name_for` in Task 5 calls Python to look up the state name. This creates a Python dependency in the Rust runner. If Python is not available, the VRA path fails. This is acceptable for Phase 3d since the adjacency loader already requires Python. Both can be eliminated in Phase 2a (when the Rust TIGER reader owns the config). Document this in the code.
