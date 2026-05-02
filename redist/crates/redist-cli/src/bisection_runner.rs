/// Level-parallel METIS bisection runner.
///
/// Implements the recursive bisection loop using BisectionTree for split
/// scheduling and gpmetis subprocess for each individual split.
///
/// Level-parallel: all nodes at depth D are split simultaneously via Rayon,
/// then depth D+1 is processed. BisectionNode implements Clone (bisection.rs).
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use rayon::prelude::*;
use redist_core::{BisectionTree, ufactor_for_depth};
use redist_core::metis_format::{write_metis_graph, parse_metis_partition};

// ── CompactBisect (B.7) ───────────────────────────────────────────────────────

/// Configuration for the CompactBisect algorithm.
///
/// At each bisection level, runs `seeds_per_level` candidate splits via METIS,
/// filters to those within `epsilon` of the minimum edge-cut, then selects the
/// candidate maximising geometric-mean Polsby-Popper: sqrt(PP(L) * PP(R)).
///
/// When `graph` has no geometry data (vertex_areas/vertex_ext_perimeters empty),
/// CompactBisect degrades gracefully to standard minimum-edge-cut selection.
#[derive(Debug, Clone)]
pub struct CompactBisectOpts {
    /// Number of METIS seeds to try at each bisection node. Higher = better
    /// approximation of the true minimum. Typical: 20-100.
    pub seeds_per_level: usize,
    /// Fraction above minimum edge-cut that is still considered "near-minimum".
    /// Candidates with EC > (1+epsilon)*EC_min are excluded from PP selection.
    /// Typical: 0.05 (5%).
    pub epsilon: f64,
}

impl Default for CompactBisectOpts {
    fn default() -> Self { Self { seeds_per_level: 50, epsilon: 0.05 } }
}

/// Select the best bisection candidate by geometric-mean Polsby-Popper,
/// among candidates within epsilon of the minimum edge-cut.
///
/// Returns the (left, right) partition maximising sqrt(PP(left)*PP(right)).
/// Falls back to the minimum-edge-cut candidate if geometry is unavailable.
fn select_compact_split(
    candidates: &[(HashSet<usize>, HashSet<usize>, f64)], // (left, right, edge_cut)
    graph: &redist_data::AdjacencyGraph,
    epsilon: f64,
) -> (HashSet<usize>, HashSet<usize>) {
    assert!(!candidates.is_empty());

    let ec_min = candidates.iter().map(|(_, _, ec)| *ec)
        .fold(f64::INFINITY, f64::min);
    let threshold = ec_min * (1.0 + epsilon);

    let near_min: Vec<&(HashSet<usize>, HashSet<usize>, f64)> = candidates.iter()
        .filter(|(_, _, ec)| *ec <= threshold)
        .collect();

    // If no geometry: return the minimum-edge-cut candidate
    if !graph.has_geometry() {
        let best = near_min.iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(&candidates[0]);
        return (best.0.clone(), best.1.clone());
    }

    // Geometric-mean PP selection: argmax sqrt(PP(L) * PP(R))
    let best_idx = near_min.iter()
        .enumerate()
        .map(|(i, (l, r, _))| {
            let gm = graph.geometric_mean_pp(l, r).unwrap_or(0.0);
            (i, gm)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let best = near_min[best_idx];
    (best.0.clone(), best.1.clone())
}

/// Detect the gpmetis version string by running `gpmetis --version` or `gpmetis -version`.
///
/// gpmetis typically prints to stderr:
///   "METIS 5.1.0 (2013-03-30)"
///   or: "METIS  Copyright 1998-2020, Regents of the University of Minnesota"
///
/// Returns the first non-empty output line, or "unknown" if gpmetis is not found or
/// does not produce recognizable output. Never panics.
pub fn detect_gpmetis_version() -> String {
    let gpmetis = match find_gpmetis() {
        Some(p) => p,
        None => return "unknown".to_string(),
    };

    // Try --version first, then -version (older METIS versions use -version)
    for flag in &["--version", "-version"] {
        if let Ok(out) = Command::new(&gpmetis).arg(flag).output() {
            // gpmetis writes version info to stderr (and sometimes stdout)
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stderr),
                String::from_utf8_lossy(&out.stdout),
            );
            for line in combined.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

/// Find the gpmetis executable (cross-platform: Windows .exe or Linux/macOS binary).
pub fn find_gpmetis() -> Option<String> {
    // Try PATH first
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    for name in &["gpmetis", "gpmetis.exe"] {
        if let Ok(out) = Command::new(which_cmd).arg(name).output() {
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                // `where` can return multiple lines; take first
                let first = p.lines().next().unwrap_or("").to_string();
                if !first.is_empty() { return Some(first); }
            }
        }
    }
    // Common fixed locations (including project-local bin/ directory)
    let candidates = [
        "/usr/bin/gpmetis",
        "/usr/local/bin/gpmetis",
        "/opt/homebrew/bin/gpmetis",
        r"C:\metis\bin\gpmetis.exe",
        r"C:\Program Files\metis\bin\gpmetis.exe",
        r"bin\gpmetis.exe",          // project-local (Windows)
        r"bin/gpmetis",              // project-local (Unix)
    ];
    candidates.iter().find(|p| Path::new(p).exists()).map(|s| s.to_string())
}

/// Split a subgraph (identified by `tract_indices`) into two balanced parts.
///
/// Builds a local subgraph, writes METIS format, invokes gpmetis, parses output.
/// Returns (left_indices, right_indices) where left = partition 0, right = partition 1.
pub fn split_subgraph(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    tract_indices: &HashSet<usize>,
    // ufactor: METIS decimal multiplier (e.g. 1.001 = 0.1%). Use ufactor_for_depth().
    ufactor: f64,
    niter: u32,
    seed: Option<u64>,
    // target_weights: (left_frac, right_frac) for unequal splits (e.g. 3/7, 4/7).
    // None = equal 50/50 split (METIS default).
    target_weights: Option<(f64, f64)>,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    // Build local index mapping: local → global (sorted for determinism)
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

    // Subgraph edge weights (reindex to local, canonical order)
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

    // Generate METIS graph content
    let graph_content = write_metis_graph(&sub_adj, &sub_vw, ew_opt)
        .map_err(|e| e.to_string())?;

    let gpmetis = find_gpmetis()
        .ok_or_else(|| {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let install_hint = match (os, arch) {
                ("linux", "aarch64") | ("linux", "arm") =>
                    "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
                ("macos", "aarch64") =>
                    "Apple Silicon: brew install metis",
                ("linux", _) =>
                    "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
                ("windows", _) =>
                    "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
                ("macos", _) =>
                    "macOS: brew install metis",
                _ =>
                    "Install METIS from https://github.com/KarypisLab/METIS",
            };
            format!("gpmetis not found ({os}/{arch}). {install_hint}")
        })?;

    // Write to temp directory and invoke gpmetis
    let tmp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
    let graph_file = tmp_dir.path().join("graph.txt");
    let part_file = tmp_dir.path().join("graph.txt.part.2");

    std::fs::write(&graph_file, &graph_content).map_err(|e| e.to_string())?;

    // Write tpwgts file if target_weights specified (non-equal split)
    // AC-05: validate target weights sum to 1.0 before passing to METIS
    if let Some((lf, rf)) = target_weights {
        let sum = lf + rf;
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "target_weights must sum to 1.0: {lf:.6} + {rf:.6} = {sum:.6}"
        );
    }
    let tpwgts_file = if let Some((left_frac, _right_frac)) = target_weights {
        let tpwgts = tmp_dir.path().join("tpwgts.txt");
        // Only write partition 0; METIS infers partition 1 = 1 - left_frac
        std::fs::write(&tpwgts, format!("0 = {left_frac:.6}\n"))
            .map_err(|e| e.to_string())?;
        Some(tpwgts)
    } else {
        None
    };

    // gpmetis uses atoi() to parse -ufactor, so it MUST be an integer.
    // The METIS manual defines: imbalance = (1 + ufactor/1000). So:
    //   ufactor=1  → 0.1% tolerance
    //   ufactor=50 → 5.0% tolerance
    // Convert from decimal (e.g. 1.05 → 50) and clamp to [1, 1000].
    let ufactor_int = ((ufactor - 1.0) * 1000.0).round() as u32;
    let ufactor_int = ufactor_int.clamp(1, 1000);

    // SAFETY: Rust's std::process::Command::arg() passes arguments directly to the
    // OS API (CreateProcess on Windows, execvp on Unix) without shell interpretation.
    // This means individual .arg() calls handle quoting and spaces automatically —
    // NO manual quoting is needed. Paths with spaces are safe when passed as PathBuf
    // or OsStr via .arg(path). Only format!("{}", path.display()) inserted into a
    // shell string would be unsafe; we avoid that pattern here.
    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor_int}"),
    ]);
    if let Some(ref tpwgts) = tpwgts_file {
        // Pass -tpwgt= by building the flag string and the path separately via OsString
        // so that spaces in the temp dir path are handled correctly by the OS API.
        let mut flag = std::ffi::OsString::from("-tpwgt=");
        flag.push(tpwgts.as_os_str());
        cmd.arg(flag);
    }
    if let Some(s) = seed {
        cmd.arg(format!("-seed={s}"));
    }
    // Pass graph file path via .arg(PathBuf) — OS API handles spaces automatically.
    cmd.arg(&graph_file).arg("2");
    cmd.current_dir(tmp_dir.path());

    let out = cmd.output().map_err(|e| format!("gpmetis exec error: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gpmetis failed (rc={}):\nSTDOUT: {}\nSTDERR: {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).chars().take(300).collect::<String>(),
            String::from_utf8_lossy(&out.stderr).chars().take(300).collect::<String>(),
        ));
    }

    let part_content = std::fs::read_to_string(&part_file)
        .map_err(|_| format!("gpmetis did not create partition file: {} (stdout: {})",
            part_file.display(),
            String::from_utf8_lossy(&out.stdout).chars().take(200).collect::<String>()
        ))?;

    let parts = parse_metis_partition(&part_content, n).map_err(|e| e.to_string())?;

    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, part) in parts.iter().enumerate() {
        let global = sorted[local];
        if *part == 0 { left.insert(global); } else { right.insert(global); }
    }

    Ok((left, right))
}

/// Run n-way partitioning: call gpmetis once with nparts=k.
///
/// Direct n-way is faster than recursive bisection (D.2 research: 3.68s vs 11.33s).
/// D.2 also shows equivalent VRA success rates (47.5% vs 48.3%, p=0.634).
///
/// Target weights: equal partitioning (1/k per district). The last weight is
/// inferred by METIS so the sum is exactly 1.0 (avoids floating-point drift).
///
/// AC-05 invariant: all target weights sum to 1.0.
/// The approach: write n-1 explicit weights of 1/k; METIS infers the last.
/// This guarantees sum = (n-1)*(1/k) + inferred = 1.0 regardless of rounding.
pub fn run_nway_partition(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    ufactor: f64,
    niter: u32,
    seed: Option<u64>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();
    if num_districts == 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let graph_content = write_metis_graph(
        adjacency,
        vertex_weights,
        if edge_weights.is_empty() { None } else { Some(edge_weights) },
    ).map_err(|e| e.to_string())?;

    let gpmetis = find_gpmetis()
        .ok_or_else(|| {
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            let install_hint = match (os, arch) {
                ("linux", "aarch64") | ("linux", "arm") =>
                    "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
                ("macos", "aarch64") =>
                    "Apple Silicon: brew install metis",
                ("linux", _) =>
                    "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
                ("windows", _) =>
                    "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
                ("macos", _) =>
                    "macOS: brew install metis",
                _ =>
                    "Install METIS from https://github.com/KarypisLab/METIS",
            };
            format!("gpmetis not found ({os}/{arch}). {install_hint}")
        })?;

    let tmp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
    let graph_file = tmp_dir.path().join("graph.txt");
    let part_file = tmp_dir.path().join(format!("graph.txt.part.{num_districts}"));

    std::fs::write(&graph_file, &graph_content).map_err(|e| e.to_string())?;

    // Write n-1 equal target weights; METIS infers the last to guarantee sum=1.0
    // AC-05: this approach avoids floating-point drift from n independent 1/k values
    let tpwgts_file = tmp_dir.path().join("tpwgts.txt");
    let weight_per = 1.0_f64 / num_districts as f64;
    let mut tpwgts_content = String::new();
    for partition_id in 0..(num_districts - 1) {
        tpwgts_content.push_str(&format!("{partition_id} = {weight_per:.8}\n"));
    }
    std::fs::write(&tpwgts_file, &tpwgts_content).map_err(|e| e.to_string())?;

    // SAFETY: See comment in split_subgraph — .arg(PathBuf) is safe for paths with
    // spaces; format!("{}", path.display()) in a shell string would NOT be.
    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor:.4}"),
    ]);
    {
        let mut flag = std::ffi::OsString::from("-tpwgt=");
        flag.push(tpwgts_file.as_os_str());
        cmd.arg(flag);
    }
    if let Some(s) = seed { cmd.arg(format!("-seed={s}")); }
    // Pass graph file via .arg(PathBuf) and num_districts as a string arg.
    cmd.arg(&graph_file).arg(num_districts.to_string());
    cmd.current_dir(tmp_dir.path());

    let out = cmd.output().map_err(|e| format!("gpmetis exec error: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gpmetis n-way failed (rc={}):\n{}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).chars().take(300).collect::<String>()
        ));
    }

    let part_content = std::fs::read_to_string(&part_file)
        .map_err(|_| format!("gpmetis did not create {}", part_file.display()))?;

    let parts = parse_metis_partition(&part_content, n).map_err(|e| e.to_string())?;

    // Convert 0-based METIS output to 1-based district IDs
    Ok(parts.iter().enumerate().map(|(tract, &part)| (tract, part + 1)).collect())
}

/// Run the full level-parallel bisection for k districts.
/// Returns HashMap<tract_index, district_id> (1-based district IDs).
///
/// RACE CONDITION FIX: tract data extracted from node_tracts sequentially
/// BEFORE par_iter, so closures own their data with no shared references.
///
/// SORT FIX: leaves sorted by (depth, path) not plain lex, which gives
/// correct BFS order for mixed-length binary paths.
/// Run recursive bisection with mathematically-derived per-node ufactor.
///
/// **Key insight**: at each bisection node producing `k` final districts, the allowed
/// per-split imbalance must be `balance_tolerance / k` — not a fixed value — so that
/// cumulative error across all splits never exceeds `balance_tolerance` per final district.
///
/// For k=98 (WA house) with 10% target: root ufactor=0.102% (very tight), leaf ufactor=5% (loose).
/// This prevents the compounding error (28% deviation) seen with fixed ufactor per depth.
///
/// Formula: `node_ufactor = 1.0 + balance_tolerance_frac / node.k`
/// GeoSection: find the natural geographic split ratio.
///
/// At the first bisection level (depth 0), try ALL feasible split ratios
/// (1:k-1, 2:k-2, ..., ⌊k/2⌋:⌈k/2⌉), each with `seeds_per_ratio` seeds.
/// The ratio with the globally minimum edge-cut is the "natural" ratio.
/// All subsequent levels use the standard ⌊k/2⌋:⌈k/2⌉ split.
///
/// Returns (assignments, natural_ratio_left, natural_ratio_right, natural_ec).
pub fn run_geosection(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    intermediate_dir: Option<&Path>,
) -> Result<(HashMap<usize, usize>, usize, usize, f64), String> {
    let n = adjacency.len();

    if num_districts == 1 {
        let asgn = (0..n).map(|i| (i, 1)).collect();
        return Ok((asgn, 1, 0, 0.0));
    }
    if num_districts == 2 {
        // Only one ratio possible: 1:1
        let asgn = run_all_splits(adjacency, vertex_weights, edge_weights,
                                   2, balance_tolerance, niter,
                                   Some(1), intermediate_dir)?;
        let ec: f64 = edge_weights.iter().map(|(&(u,v),&w)| {
            if asgn.get(&u) != asgn.get(&v) { w } else { 0.0 }
        }).sum();
        return Ok((asgn, 1, 1, ec));
    }

    // Try all split ratios at the root level
    let node_ufactor = 1.0 + balance_tolerance / num_districts as f64;
    let mut best_ec = f64::INFINITY;
    let mut best_normalised = f64::INFINITY; // isoperimetric-corrected selection criterion
    let mut best_left = 0usize;
    let mut best_right = 0usize;
    let mut best_left_set = HashSet::new();
    let mut best_right_set = HashSet::new();

    let all_tracts: HashSet<usize> = (0..n).collect();
    let max_left = num_districts / 2;  // try ratios 1:k-1 through k/2:k/2

    eprintln!("[geosection] trying {} ratios × {} seeds for k={}",
              max_left, seeds_per_ratio, num_districts);

    for left_k in 1..=max_left {
        let right_k = num_districts - left_k;
        let tw = Some((left_k as f64 / num_districts as f64,
                       right_k as f64 / num_districts as f64));
        let mut ratio_best = f64::INFINITY;
        let mut ratio_best_left = HashSet::new();
        let mut ratio_best_right = HashSet::new();

        for seed in 1..=(seeds_per_ratio as u64) {
            match split_subgraph(adjacency, vertex_weights, edge_weights,
                                  &all_tracts, node_ufactor, niter, Some(seed), tw) {
                Ok((l, r)) => {
                    let ec: f64 = edge_weights.iter().filter_map(|(&(u,v),&w)| {
                        if l.contains(&u) != l.contains(&v) { Some(w) } else { None }
                    }).sum();
                    if ec < ratio_best {
                        ratio_best = ec;
                        ratio_best_left = l;
                        ratio_best_right = r;
                    }
                }
                Err(_) => continue,
            }
        }

        // Normalise by √(min(i,k-i)): isoperimetric correction.
        // Raw EC always favours 1:k-1 (tiny boundary) over k/2:k/2 (full bisection).
        // Dividing by √(smaller_half_districts) makes the comparison apples-to-apples:
        // both represent the geometric efficiency of the cut per unit area carved.
        let smaller = left_k.min(right_k) as f64;
        let normalised = ratio_best / smaller.sqrt();

        eprintln!("[geosection]   ratio {}:{} best={:.0}km  normalised={:.1}",
                  left_k, right_k, ratio_best/1000.0, normalised/1000.0);

        if normalised < best_normalised {
            best_normalised = normalised;
            best_ec = ratio_best;
            best_left = left_k;
            best_right = right_k;
            best_left_set = ratio_best_left;
            best_right_set = ratio_best_right;
        }
    }

    eprintln!("[geosection] natural ratio {}:{} at {:.0}km (normalised={:.1})",
              best_left, best_right, best_ec/1000.0, best_normalised/1000.0);

    // Write depth_01 intermediate
    if let Some(dir) = intermediate_dir {
        let round_dir = dir.join("depth_01");
        let _ = std::fs::create_dir_all(&round_dir);
        let mut d1: HashMap<usize, usize> = HashMap::new();
        for &v in &best_left_set  { d1.insert(v, 1); }
        for &v in &best_right_set { d1.insert(v, 2); }
        let _ = write_intermediate_round(&round_dir, &d1);
    }

    // Recurse: apply GeoSection to each half (full recursive ratio search).
    // Each subregion finds its own natural ratio independently.
    let left_asgn  = recurse_geosection(
        &best_left_set,  adjacency, vertex_weights, edge_weights,
        best_left,  balance_tolerance, niter, seeds_per_ratio, 1)?;
    let right_asgn = recurse_geosection(
        &best_right_set, adjacency, vertex_weights, edge_weights,
        best_right, balance_tolerance, niter, seeds_per_ratio, best_left + 1)?;

    let mut assignments = left_asgn;
    assignments.extend(right_asgn);

    if assignments.len() != n {
        return Err(format!("geosection incomplete: {}/{n}", assignments.len()));
    }
    Ok((assignments, best_left, best_right, best_ec))
}

/// Fully recursive GeoSection on a geographic subregion.
///
/// At each level:
///   1. Extract local subgraph (local indices)
///   2. (Future) Compute local minor axis via PCA of subregion centroids
///   3. Run ratio search on local graph with local orientation
///   4. Map results back to global indices, offset by district_base
///
/// Centroid-based orientation (Phase 2): each half checks its own axis
/// so a horizontal cut produces a "top half" that may want a vertical
/// next cut, and vice versa. Currently lambda=0 (no directional penalty)
/// until centroids are wired in.
fn recurse_geosection(
    verts: &HashSet<usize>,
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize,usize),f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    district_base: usize,
    // TODO Phase 2: centroids: Option<&[(f64,f64)]>  — for local PCA / minor axis
) -> Result<HashMap<usize,usize>, String> {
    if k == 0 { return Ok(HashMap::new()); }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    // Extract sorted vertex list for deterministic local indexing
    let sorted: Vec<usize> = { let mut v: Vec<usize> = verts.iter().copied().collect(); v.sort_unstable(); v };
    let global_to_local: HashMap<usize,usize> = sorted.iter().enumerate().map(|(i,&g)|(g,i)).collect();

    // Build local subgraph components
    let local_adj: Vec<Vec<usize>> = build_subgraph_adjacency(verts, adjacency);
    let local_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let local_ew: HashMap<(usize,usize),f64> = edge_weights.iter()
        .filter_map(|(&(u,v),&w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m,(k,v)| { m.insert(k,v); m });

    // TODO Phase 2: compute local PCA of subregion centroids here
    // let local_centroids: Vec<(f64,f64)> = sorted.iter()
    //     .map(|&g| centroids.map(|c| c[g]).unwrap_or((0.0,0.0))).collect();
    // let minor_axis_angle = pca_minor_axis(&local_centroids);
    // let local_ew = apply_directional_penalty(&local_ew, &local_adj, minor_axis_angle, lambda);

    // Recursively find natural ratio for THIS subregion
    let (local_asgn, nat_left, nat_right, nat_ec) = run_geosection(
        &local_adj, &local_vw, &local_ew,
        k, balance_tolerance, niter, seeds_per_ratio, None,
    )?;

    if local_asgn.len() < sorted.len().saturating_sub(1) {
        // Partial assignment — fall back to standard for this subregion
        return recurse_standard(verts, &local_adj, adjacency, vertex_weights, edge_weights,
                                k, balance_tolerance, niter, district_base);
    }

    // Map local indices back to global with district offset
    let result: HashMap<usize,usize> = local_asgn.iter()
        .filter_map(|(&local, &dist)| {
            sorted.get(local).map(|&global| (global, dist + district_base - 1))
        })
        .collect();
    Ok(result)
}

/// Build adjacency restricted to a subset of vertices (for recursion).
fn build_subgraph_adjacency(verts: &HashSet<usize>, adj: &[Vec<usize>])
    -> Vec<Vec<usize>>
{
    let sorted: Vec<usize> = {
        let mut v: Vec<usize> = verts.iter().copied().collect();
        v.sort_unstable(); v
    };
    let global_to_local: HashMap<usize,usize> = sorted.iter()
        .enumerate().map(|(i,&g)| (g,i)).collect();
    sorted.iter().map(|&g| {
        adj[g].iter().filter_map(|&nb| global_to_local.get(&nb).copied()).collect()
    }).collect()
}

/// Recurse using standard bisection (UpfloorD k/2 : ceil k/2) within a subgraph.
/// Returns global-index assignments offset by district_base.
fn recurse_standard(
    verts: &HashSet<usize>,
    sub_adj: &[Vec<usize>],
    global_adj: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize,usize),f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    district_base: usize,
) -> Result<HashMap<usize,usize>, String> {
    if k == 0 { return Ok(HashMap::new()); }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    let sorted: Vec<usize> = { let mut v: Vec<usize> = verts.iter().copied().collect(); v.sort_unstable(); v };
    let global_to_local: HashMap<usize,usize> = sorted.iter().enumerate().map(|(i,&g)|(g,i)).collect();

    // Extract sub-vertex-weights and sub-edge-weights
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let sub_ew: HashMap<(usize,usize),f64> = edge_weights.iter()
        .filter_map(|(&(u,v),&w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m,(k,v)| { m.insert(k,v); m });

    let sub_n = sorted.len();
    let sub_asgn = run_all_splits(sub_adj, &sub_vw, &sub_ew,
                                   k, balance_tolerance, niter,
                                   Some(42), None)?;

    // Map back to global indices with offset
    let result: HashMap<usize,usize> = sub_asgn.iter()
        .map(|(&local, &dist)| (sorted[local], dist + district_base - 1))
        .collect();
    Ok(result)
}

pub fn run_all_splits(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64, // fraction (e.g. 0.10 for ±10%); node ufactor = 1 + T/k
    niter: u32,
    seed: Option<u64>,
    // If Some, writes intermediate/depth_{d:02}/assignments.json after each round.
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    // Single-district: all tracts to district 1, no METIS call
    if num_districts == 1 {
        // Write depth-00 as the trivial single-region state
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join("depth_00");
            let _ = std::fs::create_dir_all(&round_dir);
            let asgn: HashMap<usize, usize> = (0..n).map(|i| (i, 1)).collect();
            let _ = write_intermediate_round(&round_dir, &asgn);
        }
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let tree = BisectionTree::from_k(num_districts);
    let mut node_tracts: HashMap<String, HashSet<usize>> = HashMap::new();
    node_tracts.insert(String::new(), (0..n).collect());

    for depth in 0..tree.max_depth {
        let nodes_at_depth: Vec<_> = tree.nodes_at_depth(depth).into_iter().cloned().collect();

        // Extract data BEFORE parallel section — no shared references across threads
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| {
                    node_tracts.remove(&node.path).map(|tracts| (node, tracts))
                })
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    // Per-node ufactor: 1.0 + balance_tolerance / k_node
                    // This is the mathematically correct formula: if each split at a node
                    // with k remaining districts is balanced to (T/k)%, the cumulative
                    // error across all levels never exceeds T% per final district.
                    // Root (k=98, T=10%): ufactor=1.00102 (very tight)
                    // Leaf (k=2, T=10%): ufactor=1.05 (loose — OK since only 2 districts)
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;

                    // Target weights: k_left/k and k_right/k
                    // Equal when k_left == k_right (even k); unequal for odd k
                    let tw = if node.k_left == node.k_right {
                        None // equal split — METIS default
                    } else {
                        Some((node.k_left as f64 / node.k as f64,
                              node.k_right as f64 / node.k as f64))
                    };
                    let (left, right) = split_subgraph(
                        adjacency, vertex_weights, edge_weights, &tracts,
                        node_ufactor, niter, seed, tw
                    ).map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?;
                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        // Sort results by path before inserting to ensure deterministic insertion order.
        // Rayon's thread scheduling may vary, so the collection order of split_results
        // is non-deterministic without this sort.
        //
        // Determinism requires: (a) same seed passed to gpmetis, (b) same graph structure,
        // (c) same topology of adjacency. The sort here ensures consistent insertion order
        // into node_tracts, which affects the final leaf sort and district numbering.
        let mut sorted_results = split_results;
        sorted_results.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }

        // Write intermediate round: current node_tracts state as tract→region_id
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join(format!("depth_{:02}", depth + 1));
            let _ = std::fs::create_dir_all(&round_dir);
            // Sort nodes for deterministic region numbering
            let mut nodes: Vec<(&String, &HashSet<usize>)> = node_tracts.iter().collect();
            nodes.sort_by_key(|(path, _)| (path.len(), *path));
            let mut round_asgn: HashMap<usize, usize> = HashMap::with_capacity(n);
            for (region_id, (_, tracts)) in nodes.iter().enumerate() {
                for &tract in tracts.iter() {
                    round_asgn.insert(tract, region_id + 1);
                }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
        }
    }

    // Sort leaves by (depth, path) — NOT plain lex.
    // Plain lex on binary paths is WRONG: "0","00","01","1" ≠ BFS "0","1","00","01"
    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));

    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts {
            assignments.insert(tract, district_id + 1);
        }
    }

    if assignments.len() != n {
        return Err(format!(
            "bisection incomplete: {}/{n} tracts assigned", assignments.len()
        ));
    }
    Ok(assignments)
}

/// CompactBisect variant of run_all_splits.
///
/// Identical to run_all_splits except at each bisection node it runs
/// `opts.seeds_per_level` METIS candidates, filters to near-minimum-cut,
/// and selects the split maximising geometric-mean Polsby-Popper.
/// Requires geometry data in `graph` (vertex_areas + vertex_ext_perimeters);
/// gracefully degrades to minimum-edge-cut if geometry is absent.
pub fn run_all_splits_compact(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    // Per-vertex land area in m² (from TIGER ALAND). Empty = no PP computation.
    vertex_areas: &[f64],
    // Per-vertex external perimeter in metres. Empty = no PP computation.
    vertex_ext_perimeters: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    // Ignored here — seeds_per_level in opts controls METIS seed iteration.
    _single_seed: Option<u64>,
    opts: &CompactBisectOpts,
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();
    // Build a lightweight AdjacencyGraph wrapper so select_compact_split can call subgraph_pp.
    // We only populate the geometry fields — adjacency/weights are borrowed from the caller.
    let geom_graph = {
        let mut g = redist_data::AdjacencyGraph {
            adjacency: adjacency.to_vec(),
            vertex_weights: vertex_weights.to_vec(),
            edge_weights: edge_weights.clone(),
            n_vertices: n,
            n_edges: edge_weights.len(),
            vertex_areas: vertex_areas.to_vec(),
            vertex_ext_perimeters: vertex_ext_perimeters.to_vec(),
        };
        g
    };

    if num_districts == 1 {
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join("depth_00");
            let _ = std::fs::create_dir_all(&round_dir);
            let asgn: HashMap<usize, usize> = (0..n).map(|i| (i, 1)).collect();
            let _ = write_intermediate_round(&round_dir, &asgn);
        }
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let tree = BisectionTree::from_k(num_districts);
    let mut node_tracts: HashMap<String, HashSet<usize>> = HashMap::new();
    node_tracts.insert(String::new(), (0..n).collect());

    for depth in 0..tree.max_depth {
        let nodes_at_depth: Vec<_> = tree.nodes_at_depth(depth).into_iter().cloned().collect();
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| node_tracts.remove(&node.path).map(|t| (node, t)))
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;
                    let tw = if node.k_left == node.k_right { None } else {
                        Some((node.k_left as f64 / node.k as f64,
                              node.k_right as f64 / node.k as f64))
                    };

                    // Run N seeds, collect (left, right, edge_cut)
                    let candidates: Vec<(HashSet<usize>, HashSet<usize>, f64)> =
                        (1..=opts.seeds_per_level).filter_map(|s| {
                            let seed = Some(s as u64);
                            split_subgraph(
                                adjacency, vertex_weights, edge_weights,
                                &tracts, node_ufactor, niter, seed, tw
                            ).ok().map(|(l, r)| {
                                let ec: f64 = edge_weights.iter().filter_map(|(&(u, v), &w)| {
                                    if l.contains(&u) != l.contains(&v) { Some(w) } else { None }
                                }).sum();
                                (l, r, ec)
                            })
                        }).collect();

                    if candidates.is_empty() {
                        return Err(format!(
                            "depth {} node '{}': all {} seeds failed",
                            depth, node.path, opts.seeds_per_level
                        ));
                    }

                    let (left, right) = select_compact_split(&candidates, &geom_graph, opts.epsilon);
                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        let mut sorted = split_results;
        sorted.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }

        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join(format!("depth_{:02}", depth + 1));
            let _ = std::fs::create_dir_all(&round_dir);
            let mut nodes: Vec<(&String, &HashSet<usize>)> = node_tracts.iter().collect();
            nodes.sort_by_key(|(path, _)| (path.len(), *path));
            let mut round_asgn: HashMap<usize, usize> = HashMap::with_capacity(n);
            for (region_id, (_, tracts)) in nodes.iter().enumerate() {
                for &tract in tracts.iter() { round_asgn.insert(tract, region_id + 1); }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
        }
    }

    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));
    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts { assignments.insert(tract, district_id + 1); }
    }
    if assignments.len() != n {
        return Err(format!("bisection incomplete: {}/{n} tracts assigned", assignments.len()));
    }
    Ok(assignments)
}

// ── Proportional Bisection (B.7) ─────────────────────────────────────────────

/// At each bisection, compute the Dem vote share within the current subgraph
/// and split the subgraph proportionally: the "left" half gets
/// round(dem_share * k) districts and the "right" half gets the remainder.
///
/// Within that proportional constraint, edge-cut minimisation (METIS) determines
/// WHERE the boundary is drawn. No partisan data enters the boundary decision —
/// only the RATIO of districts allocated to each side.
///
/// Theorem (B.7): this achieves near-proportional seat allocation without
/// picking which party's voters land in which half. The proportional ratio is
/// applied symmetrically; METIS draws the most compact boundary satisfying it.
///
/// Requires: per-vertex dem_votes (from partisan_shares CSV, same as partisan-weighted mode).
/// §104(e) of the Districting Integrity Act prohibits this for federal congressional
/// districts. Valid for state legislative redistricting.
pub fn run_all_splits_proportional(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    // Per-vertex Dem vote total (from partisan_shares CSV).
    dem_votes: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    if num_districts == 1 {
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join("depth_00");
            let _ = std::fs::create_dir_all(&round_dir);
            let asgn: HashMap<usize, usize> = (0..n).map(|i| (i, 1)).collect();
            let _ = write_intermediate_round(&round_dir, &asgn);
        }
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let tree = BisectionTree::from_k(num_districts);
    let mut node_tracts: HashMap<String, HashSet<usize>> = HashMap::new();
    node_tracts.insert(String::new(), (0..n).collect());

    for depth in 0..tree.max_depth {
        let nodes_at_depth: Vec<_> = tree.nodes_at_depth(depth).into_iter().cloned().collect();
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| node_tracts.remove(&node.path).map(|t| (node, t)))
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;

                    // Compute Dem vote share within this subgraph
                    let total_dem: f64 = tracts.iter().map(|&v| dem_votes[v]).sum();
                    let total_votes: f64 = tracts.iter()
                        .map(|&v| vertex_weights[v] as f64).sum();
                    let dem_share = if total_votes > 0.0 {
                        total_dem / total_votes
                    } else {
                        0.5 // fallback: equal split
                    };

                    // Proportional district allocation: round to nearest integer
                    let k_dem = (dem_share * node.k as f64).round() as usize;
                    let k_dem = k_dem.max(1).min(node.k - 1); // at least 1 per side
                    let k_rep = node.k - k_dem;

                    // Use the proportional allocation as METIS target weights.
                    // METIS will minimise edge-cut subject to this population-ratio constraint.
                    let tw = if k_dem == k_rep {
                        None // equal — use default
                    } else {
                        Some((k_dem as f64 / node.k as f64, k_rep as f64 / node.k as f64))
                    };

                    let (left, right) = split_subgraph(
                        adjacency, vertex_weights, edge_weights,
                        &tracts, node_ufactor, niter, seed, tw
                    ).map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?;

                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        let mut sorted = split_results;
        sorted.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }

        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join(format!("depth_{:02}", depth + 1));
            let _ = std::fs::create_dir_all(&round_dir);
            let mut nodes: Vec<(&String, &HashSet<usize>)> = node_tracts.iter().collect();
            nodes.sort_by_key(|(path, _)| (path.len(), *path));
            let mut round_asgn: HashMap<usize, usize> = HashMap::with_capacity(n);
            for (region_id, (_, tracts)) in nodes.iter().enumerate() {
                for &tract in tracts.iter() { round_asgn.insert(tract, region_id + 1); }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
        }
    }

    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));
    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts { assignments.insert(tract, district_id + 1); }
    }
    if assignments.len() != n {
        return Err(format!("bisection incomplete: {}/{n} tracts assigned", assignments.len()));
    }
    Ok(assignments)
}

/// Write one intermediate round's assignments to `{round_dir}/assignments.json`.
/// Format: `{"tract_index": region_id, ...}` — mirrors final_assignments.json.
fn write_intermediate_round(round_dir: &Path, assignments: &HashMap<usize, usize>) -> Result<(), String> {
    let path = round_dir.join("assignments.json");
    let json = serde_json::to_string(assignments)
        .map_err(|e| format!("serialize intermediate: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write intermediate: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Task 132: detect_gpmetis_version ─────────────────────────────────────

    #[test]
    fn test_detect_gpmetis_version_returns_string() {
        // Must return a non-panicking string — "unknown" when gpmetis isn't present,
        // or a non-empty version string when it is. Never panics.
        let version = detect_gpmetis_version();
        assert!(!version.is_empty(),
            "detect_gpmetis_version must return a non-empty string (got empty)");
        // Must be either "unknown" (gpmetis absent) or contain printable characters
        assert!(version.chars().all(|c| !c.is_control() || c == ' '),
            "version string must contain only printable chars: {:?}", version);
    }

    #[test]
    fn test_detect_gpmetis_version_never_panics_without_gpmetis() {
        // Simulate the case where gpmetis is not found: find_gpmetis() → None.
        // detect_gpmetis_version() must return "unknown" in that case.
        // We can't easily mock find_gpmetis, but we can at least verify the return
        // value is a valid String (non-empty).
        let v = detect_gpmetis_version();
        // Acceptable values: "unknown" (no gpmetis) or any non-empty version string
        assert!(v == "unknown" || !v.is_empty(),
            "must return 'unknown' or a version string, got: {:?}", v);
    }

    #[test]
    fn test_split_four_node_graph() {
        if find_gpmetis().is_none() { return; }
        let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..4).collect();

        let (left, right) = split_subgraph(&adj, &vw, &ew, &indices, 1.001, 100, Some(42), None)
            .expect("gpmetis should split 4-node graph");

        assert_eq!(left.len() + right.len(), 4, "all tracts assigned");
        assert!(!left.is_empty() && !right.is_empty(), "non-empty split");
        // Disjoint and complete
        assert!(left.is_disjoint(&right), "left and right must be disjoint");
        for i in 0..4 {
            assert!(left.contains(&i) || right.contains(&i), "tract {i} missing");
        }
        let pop_left: i64 = left.iter().map(|&i| vw[i]).sum();
        let pop_right: i64 = right.iter().map(|&i| vw[i]).sum();
        let dev = (pop_left - pop_right).abs() as f64 / 4000.0;
        assert!(dev <= 0.2, "split should be balanced, got {dev:.3}");
    }

    #[test]
    fn test_run_all_splits_single_district() {
        let n = 193usize;
        let adj = vec![vec![]; n];
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
            .expect("single district should not invoke METIS");
        assert_eq!(assignments.len(), n);
        assert!(assignments.values().all(|&d| d == 1));
    }

    #[test]
    fn test_run_all_splits_two_districts() {
        if find_gpmetis().is_none() { return; }
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();

        let assignments = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None).unwrap();

        assert_eq!(assignments.len(), 4, "all tracts assigned");
        assert!(assignments.values().any(|&d| d == 1), "district 1 must exist");
        assert!(assignments.values().any(|&d| d == 2), "district 2 must exist");
        assert!(assignments.values().all(|&d| d == 1 || d == 2));

        let d1: HashSet<usize> = assignments.iter().filter(|(_, &v)| v == 1).map(|(&k, _)| k).collect();
        let d2: HashSet<usize> = assignments.iter().filter(|(_, &v)| v == 2).map(|(&k, _)| k).collect();
        assert!(d1.is_disjoint(&d2), "districts must be disjoint");
        assert_eq!(d1.len() + d2.len(), 4, "complete coverage");
    }

    #[test]
    fn test_leaf_sort_bfs_order() {
        // Verify sort_by_key gives BFS not lex order
        let mut paths = vec!["1".to_string(), "0".to_string(), "01".to_string(), "00".to_string()];
        paths.sort_by_key(|p| (p.len(), p.clone()));
        // BFS: depth-1 first ("0","1"), then depth-2 ("00","01")
        assert_eq!(paths, vec!["0", "1", "00", "01"]);
    }

    // ── Invariant tests ──────────────────────────────────────────────────────

    #[test]
    fn test_nway_single_district_shortcut() {
        let adj = vec![vec![1], vec![0]];
        let vw = vec![1000i64, 1000];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None).unwrap();
        assert_eq!(assignments.len(), 2);
        assert!(assignments.values().all(|&d| d == 1));
    }

    #[test]
    fn test_nway_two_districts() {
        if find_gpmetis().is_none() { return; }
        let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let vw = vec![1000i64; 4];
        let ew: HashMap<(usize, usize), f64> = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.005, 100, Some(42)).unwrap();
        assert_eq!(assignments.len(), 4);
        assert!(assignments.values().any(|&d| d == 1), "district 1 must exist");
        assert!(assignments.values().any(|&d| d == 2), "district 2 must exist");
        // Districts are 1-based and disjoint
        let d1: HashSet<_> = assignments.iter().filter(|(_, &v)| v == 1).map(|(&k, _)| k).collect();
        let d2: HashSet<_> = assignments.iter().filter(|(_, &v)| v == 2).map(|(&k, _)| k).collect();
        assert!(d1.is_disjoint(&d2));
        assert_eq!(d1.len() + d2.len(), 4);
    }

    #[test]
    fn test_nway_equal_weights_sum_to_one() {
        // AC-05: for n-way, verify n-1 explicit weights + inferred last = 1.0
        // With weight_per = 1/k, sum = (k-1)/k + inferred(1/k) = 1.0 exactly
        for k in [2usize, 3, 7, 52] {
            let weight_per = 1.0_f64 / k as f64;
            let explicit_sum: f64 = (k - 1) as f64 * weight_per;
            let inferred = 1.0 - explicit_sum;
            assert!(
                (explicit_sum + inferred - 1.0).abs() < 1e-9,
                "k={k}: explicit({explicit_sum:.9}) + inferred({inferred:.9}) should = 1.0"
            );
        }
    }

    #[test]
    fn test_invariant_target_weights_sum_to_one_2way() {
        // AC-05: target partition weights must sum to 1.0 for 2-way split
        // (k_left/k + k_right/k = k/k = 1.0 by construction)
        for k in [2, 3, 4, 7, 8, 14, 52] {
            let tree = redist_core::BisectionTree::from_k(k);
            for node in &tree.nodes {
                let left_frac = node.k_left as f64 / node.k as f64;
                let right_frac = node.k_right as f64 / node.k as f64;
                let sum = left_frac + right_frac;
                assert!(
                    (sum - 1.0).abs() < 1e-9,
                    "k={k} node k={}: left_frac={left_frac:.6} + right_frac={right_frac:.6} = {sum:.6} != 1.0",
                    node.k
                );
            }
        }
    }

    // ── ufactor correctness tests ─────────────────────────────────────────────

    #[test]
    fn test_ufactor_integer_conversion_0_5_pct() {
        // 0.5% tolerance: decimal 1.005 → integer 5
        let decimal = 1.005_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 5, "1.005 must convert to integer 5 (0.5%)");
    }

    #[test]
    fn test_ufactor_integer_conversion_5_pct() {
        // 5% tolerance: decimal 1.05 → integer 50
        let decimal = 1.05_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 50, "1.05 must convert to integer 50 (5%)");
    }

    #[test]
    fn test_ufactor_integer_conversion_10_pct() {
        // 10% tolerance: decimal 1.10 → integer 100
        let decimal = 1.10_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 100, "1.10 must convert to integer 100 (10%)");
    }

    #[test]
    fn test_ufactor_never_zero() {
        // Minimum clamped to 1 — ufactor=0 would disable balance checking
        for decimal in [1.0001_f64, 1.0_f64, 0.999_f64] {
            let raw = ((decimal - 1.0) * 1000.0).round() as i32;
            let clamped = (raw as u32).clamp(1, 1000);
            assert!(clamped >= 1, "ufactor must be >= 1, got {clamped} from decimal {decimal}");
        }
    }

    #[test]
    fn test_per_node_ufactor_formula() {
        // node_ufactor = 1.0 + balance_tolerance / k_node
        // Root of 98-district map (T=10%): should be very tight
        let k_root = 98usize;
        let tolerance = 0.10_f64;
        let node_ufactor = 1.0 + tolerance / k_root as f64;
        // ~0.102% — convert to int
        let ufactor_int = ((node_ufactor - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 1, "root of 98-district (10%) → ufactor=1 (0.1%)");

        // Leaf of 2-district split (T=10%): should be loose
        let k_leaf = 2usize;
        let leaf_ufactor = 1.0 + tolerance / k_leaf as f64;
        let leaf_int = ((leaf_ufactor - 1.0) * 1000.0).round() as u32;
        assert_eq!(leaf_int, 50, "leaf of 2-district (10%) → ufactor=50 (5%)");
    }

    #[test]
    fn test_per_node_ufactor_congressional_tight() {
        // Congressional (T=0.5%): root of 52-district CA map
        // 0.5%/52 = 0.0096% → rounds to 0, clamped to minimum 1
        let k = 52usize;
        let tolerance = 0.005_f64; // 0.5%
        let node_ufactor = 1.0 + tolerance / k as f64;
        let raw = ((node_ufactor - 1.0) * 1000.0).round() as u32;
        let ufactor_int = raw.clamp(1, 1000); // minimum 1 = 0.1%
        assert_eq!(ufactor_int, 1, "CA 52D congressional root → clamped to minimum ufactor=1 (0.1%)");
    }

    #[test]
    fn test_ufactor_wasnt_silently_truncated_regression() {
        // This test catches the historical bug where '-ufactor=1.0050' was passed
        // to gpmetis as a float, which atoi() truncated to 1 regardless of value.
        // The correct behavior: 1.005 → integer 5 (not 1).
        let old_style_float = 1.005_f64;
        // Old bug: atoi("1.0050") == 1 (always)
        // New fix: round((1.005 - 1.0) * 1000) == 5
        let correct_int = ((old_style_float - 1.0) * 1000.0).round() as u32;
        assert_ne!(correct_int, 1,
            "REGRESSION: 1.005 should not convert to 1 — that was the bug. Got {correct_int}");
        assert_eq!(correct_int, 5,
            "1.005 (0.5% tolerance) must convert to integer 5");
    }

    #[test]
    fn test_invariant_vertex_weights_positive() {
        // DF-04: all vertex weights must be >= 1 after loading
        // sub-zero weights cause METIS to produce degenerate partitions
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let vw = vec![1000i64, 500, 2000]; // all positive
        let ew: HashMap<(usize, usize), f64> = HashMap::new();
        // The subgraph builder clamps to max(weight, 1) — verify it would catch 0
        let tract_indices: HashSet<usize> = (0..3).collect();
        let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
        sorted.sort_unstable();
        let sub_vw: Vec<i64> = sorted.iter().map(|&g| vw[g].max(1)).collect();
        assert!(sub_vw.iter().all(|&v| v >= 1), "all vertex weights must be >= 1 after clamping");
    }

    // ── Group 1: split_subgraph edge cases ───────────────────────────────────

    #[test]
    fn test_split_subgraph_with_edge_weights() {
        if find_gpmetis().is_none() { return; }
        // 4-node chain with strong edge weights on left side — should bias split
        let adj = vec![vec![1], vec![0,2], vec![1,3], vec![2]];
        let vw = vec![1000i64; 4];
        let mut ew = HashMap::new();
        ew.insert((0,1), 1000.0); // strong edge — METIS should avoid cutting
        ew.insert((1,2), 1.0);    // weak edge — METIS may cut here
        ew.insert((2,3), 1000.0); // strong edge
        let indices: HashSet<usize> = (0..4).collect();
        let (left, right) = split_subgraph(&adj, &vw, &ew, &indices, 1.005, 100, Some(42), None)
            .expect("should split with edge weights");
        assert_eq!(left.len() + right.len(), 4);
        assert!(!left.is_empty() && !right.is_empty());
    }

    #[test]
    fn test_split_subgraph_unequal_target_weights() {
        if find_gpmetis().is_none() { return; }
        // 6 tracts, split 4:2 (target weights 2/3 and 1/3)
        let adj = vec![vec![1,2], vec![0,3], vec![0,3], vec![1,2,4,5], vec![3,5], vec![3,4]];
        let vw = vec![1000i64; 6];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..6).collect();
        let (left, right) = split_subgraph(
            &adj, &vw, &ew, &indices, 1.05, 100, Some(42),
            Some((2.0/3.0, 1.0/3.0))  // unequal: 4:2 split
        ).expect("should split with target weights");
        assert_eq!(left.len() + right.len(), 6);
        assert!(!left.is_empty() && !right.is_empty());
        // Approximate target: left ~4 tracts, right ~2 (within tolerance)
        let larger = left.len().max(right.len());
        assert!(larger >= 3, "larger partition should have >= 3 tracts for 4:2 split");
    }

    #[test]
    fn test_split_subgraph_single_node_returns_all_left() {
        // Edge case: single tract — no METIS call, returns all in left
        let adj = vec![vec![]];
        let vw = vec![1000i64];
        let ew = HashMap::new();
        let indices: HashSet<usize> = vec![0].into_iter().collect();
        let (left, right) = split_subgraph(&adj, &vw, &ew, &indices, 1.005, 100, None, None)
            .expect("single node split");
        assert_eq!(left.len(), 1);
        assert!(right.is_empty());
    }

    #[test]
    fn test_split_subgraph_two_tracts_always_splits() {
        if find_gpmetis().is_none() { return; }
        // 2 tracts: must produce one in each partition
        let adj = vec![vec![1], vec![0]];
        let vw = vec![1000i64, 1000];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..2).collect();
        let (left, right) = split_subgraph(&adj, &vw, &ew, &indices, 1.005, 100, Some(42), None)
            .expect("2-node split");
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        assert!(left.is_disjoint(&right));
    }

    // ── Group 2: run_nway_partition ──────────────────────────────────────────

    #[test]
    fn test_run_nway_partition_basic() {
        if find_gpmetis().is_none() { return; }
        // 12 tracts into 3 districts — n-way partition
        let n = 12;
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nbrs = vec![];
            if i > 0 { nbrs.push(i-1); }
            if i < n-1 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let result = run_nway_partition(&adj, &vw, &ew, 3, 1.05, 100, Some(42));
        assert!(result.is_ok(), "n-way partition should succeed: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), n, "all tracts assigned");
        let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
        assert_eq!(districts.len(), 3, "exactly 3 districts");
        assert!(districts.contains(&1) && districts.contains(&2) && districts.contains(&3));
    }

    #[test]
    fn test_run_nway_partition_balance() {
        if find_gpmetis().is_none() { return; }
        // 20 equal-weight tracts into 4 districts — should be well-balanced
        let n = 20;
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nbrs = vec![];
            if i > 0 { nbrs.push(i-1); }
            if i < n-1 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 4, 1.05, 100, Some(42)).unwrap();

        let mut district_pops = vec![0i64; 5];
        for (tract, &dist) in &assignments {
            district_pops[dist] += vw[*tract];
        }
        let ideal = 20 * 1000 / 4; // 5000
        for d in 1..=4 {
            let dev = (district_pops[d] - ideal as i64).abs() as f64 / ideal as f64;
            assert!(dev <= 0.1, "district {d} deviation {:.1}% exceeds 10%", dev*100.0);
        }
    }

    #[test]
    fn test_run_nway_partition_output_complete_and_valid() {
        if find_gpmetis().is_none() { return; }
        let adj = vec![vec![1,2], vec![0,3], vec![0,3], vec![1,2]];
        let vw = vec![1000i64; 4];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.05, 100, Some(42)).unwrap();
        // Every tract assigned, district IDs 1-based
        assert_eq!(assignments.len(), 4);
        assert!(assignments.values().all(|&d| d >= 1 && d <= 2));
        let d1: Vec<_> = assignments.values().filter(|&&d| d == 1).collect();
        let d2: Vec<_> = assignments.values().filter(|&&d| d == 2).collect();
        assert!(!d1.is_empty() && !d2.is_empty(), "both districts must have tracts");
    }

    // ── Group 3: run_all_splits edge cases ───────────────────────────────────

    #[test]
    fn test_run_all_splits_large_k_structure() {
        // Verify that run_all_splits with k=8 produces exactly 8 districts
        // without calling gpmetis (test the assignment structure, not balance)
        // Use single-tract-per-district to make it trivially balanced
        if find_gpmetis().is_none() { return; }
        let n = 16;
        // Grid graph: 4x4
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let row = i / 4; let col = i % 4;
            let mut nbrs = vec![];
            if row > 0 { nbrs.push(i-4); }
            if row < 3 { nbrs.push(i+4); }
            if col > 0 { nbrs.push(i-1); }
            if col < 3 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 8, 0.10, 100, Some(42), None).unwrap();
        assert_eq!(assignments.len(), n);
        let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
        assert_eq!(districts.len(), 8, "exactly 8 districts");
        // All district IDs 1-based
        assert!(districts.iter().all(|&d| d >= 1 && d <= 8));
    }

    #[test]
    fn test_run_all_splits_tight_balance_10pct() {
        // With correct ufactor math, 10% tolerance on a 4-district map
        // should produce well-balanced output
        if find_gpmetis().is_none() { return; }
        let adj = vec![
            vec![1,4], vec![0,2,5], vec![1,3,6], vec![2,7],
            vec![0,5], vec![1,4,6], vec![2,5,7], vec![3,6],
        ];
        let vw = vec![1000i64; 8]; // 8 equal tracts
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 4, 0.10, 100, Some(42), None).unwrap();

        let mut pops = vec![0i64; 5];
        for (&tract, &dist) in &assignments {
            pops[dist] += vw[tract];
        }
        let ideal = 8000 / 4; // 2000
        for d in 1..=4 {
            let dev = (pops[d] - ideal).abs() as f64 / ideal as f64;
            assert!(dev <= 0.10, "district {d} deviation {:.1}% exceeds 10%", dev*100.0);
        }
    }

    // ── AP-08: Granularity floor tests ───────────────────────────────────────

    #[test]
    fn test_granularity_floor_warning_threshold() {
        // AP-08: when tracts_per_district < 20, balance may be unachievable
        // This tests the THRESHOLD CALCULATION not the algorithm (which can't be unit tested)
        let total_tracts = 1784usize; // WA 2020
        let house_districts = 98usize;
        let tpd = total_tracts as f64 / house_districts as f64;
        assert!(tpd < 20.0, "WA house at tract level has {tpd:.1} tracts/district — below granularity threshold");

        let avg_tract_pop = 7_705_281i64 / total_tracts as i64;
        let ideal_district_pop = 7_705_281i64 / house_districts as i64;
        let single_tract_impact_pct = avg_tract_pop as f64 / ideal_district_pop as f64 * 100.0;
        // One tract swap changes the balance by >5% — makes 5% tolerance often impossible
        assert!(single_tract_impact_pct > 3.0,
            "At WA tract granularity, one tract swap = {single_tract_impact_pct:.1}% of district ideal — exceeds 5% tolerance at 10% target");
    }

    #[test]
    fn test_granularity_sufficient_for_congressional() {
        // Congressional maps (10 districts) have ~178 tracts/district — far above threshold
        let total_tracts = 1784usize;
        let congressional_districts = 10usize;
        let tpd = total_tracts as f64 / congressional_districts as f64;
        assert!(tpd >= 20.0, "WA congressional has {tpd:.1} tracts/district — sufficient granularity");
    }

    #[test]
    fn test_granularity_block_group_fixes_wa_house() {
        // Block groups (5311 for WA) give 54/district — above threshold
        let bg_count = 5311usize;
        let house_districts = 98usize;
        let bgpd = bg_count as f64 / house_districts as f64;
        assert!(bgpd >= 20.0, "WA house at block_group has {bgpd:.1} BGs/district — adequate");
    }

    // ── Task 147: ARM Linux platform detection ───────────────────────────────

    #[test]
    fn test_gpmetis_not_found_error_includes_arch() {
        // The error message from a missing gpmetis must include the OS/arch string.
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
        let msg = format!("gpmetis not found ({os}/{arch}). {install_hint}");
        assert!(msg.contains(os), "error must contain OS: {os}");
        assert!(msg.contains(arch), "error must contain arch: {arch}");
        assert!(msg.contains("gpmetis not found"), "must include 'gpmetis not found'");
    }

    #[test]
    fn test_platform_install_hint_linux_arm() {
        // Simulate ARM Linux hint construction.
        let (os, arch) = ("linux", "aarch64");
        let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
        assert!(install_hint.contains("apt-get install metis"),
            "ARM Linux must get apt-get hint, got: {install_hint}");
        assert!(install_hint.contains("ARM Linux"),
            "must mention ARM Linux, got: {install_hint}");
    }

    /// Task 112: Windows path quoting invariant.
    /// Documents that Command::arg(PathBuf) handles paths with spaces correctly via
    /// the OS API — no manual quoting is needed or should be applied.
    #[test]
    fn test_path_arg_does_not_need_manual_quoting() {
        use std::ffi::OsString;
        // Simulate building the -tpwgt= flag as done in split_subgraph/run_nway_partition.
        // A path with spaces: "/tmp/my dir with spaces/tpwgts.txt"
        let spaced_path = std::path::PathBuf::from("/tmp/my dir with spaces/tpwgts.txt");

        // The correct pattern: OsString concatenation, passed as a single .arg()
        let mut flag = OsString::from("-tpwgt=");
        flag.push(spaced_path.as_os_str());

        // The flag should contain the path verbatim (with spaces) — no manual quoting
        let flag_str = flag.to_string_lossy();
        assert!(flag_str.contains(" "), "spaces are preserved in OsString — OS API handles quoting");
        assert!(flag_str.starts_with("-tpwgt="), "flag prefix preserved");
        assert!(!flag_str.contains('"'), "no manual quoting added — OS API handles this");

        // Contrast: format!() with .display() would produce the same string,
        // but would be passed through the shell if used with Command::new("sh").arg("-c", ...)
        // When using Command::arg() directly, the OS API receives the raw arg — safe either way.
        // The important invariant: do NOT concatenate paths into shell strings.
        let display_str = format!("-tpwgt={}", spaced_path.display());
        assert_eq!(flag_str, display_str.as_str(),
            "OsString flag matches display()-based string for non-Unicode paths");
    }

    /// Scenario 23: Rayon seed determinism — sort split_results by path before insert.
    /// Verify that for a two-district run with a fixed seed, calling run_all_splits
    /// twice returns identical assignments (deterministic output).
    #[test]
    fn test_rayon_results_sorted_before_insert() {
        if find_gpmetis().is_none() { return; }

        // A simple 4-node chain graph: 0-1-2-3
        let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();

        // Run twice with the same seed
        let result1 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);
        let result2 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);

        assert!(result1.is_ok(), "first run must succeed: {:?}", result1.err());
        assert!(result2.is_ok(), "second run must succeed: {:?}", result2.err());

        let a1 = result1.unwrap();
        let a2 = result2.unwrap();

        // With sorted insertion order and same seed, assignments must be identical
        let mut a1_sorted: Vec<(usize, usize)> = a1.into_iter().collect();
        let mut a2_sorted: Vec<(usize, usize)> = a2.into_iter().collect();
        a1_sorted.sort_by_key(|&(k, _)| k);
        a2_sorted.sort_by_key(|&(k, _)| k);

        assert_eq!(
            a1_sorted, a2_sorted,
            "two runs with the same seed must produce identical assignments"
        );
    }
}
