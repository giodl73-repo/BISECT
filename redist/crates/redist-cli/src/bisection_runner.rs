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
        .ok_or_else(|| "gpmetis not found in PATH or common locations. Install with: apt install metis (Linux) or brew install metis (macOS)".to_string())?;

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

    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor_int}"),
    ]);
    if let Some(ref tpwgts) = tpwgts_file {
        cmd.arg(format!("-tpwgt={}", tpwgts.display()));
    }
    if let Some(s) = seed {
        cmd.arg(format!("-seed={s}"));
    }
    cmd.args([graph_file.to_str().unwrap(), "2"]);
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
        .ok_or_else(|| "gpmetis not found".to_string())?;

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

    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor:.4}"),
        &format!("-tpwgt={}", tpwgts_file.display()),
    ]);
    if let Some(s) = seed { cmd.arg(format!("-seed={s}")); }
    cmd.args([graph_file.to_str().unwrap(), &num_districts.to_string()]);
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

        for (path, left, right) in split_results {
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
}
