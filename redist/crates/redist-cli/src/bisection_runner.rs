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
    // Format: "partition_id = weight\n" for n-1 partitions (METIS infers last)
    let tpwgts_file = if let Some((left_frac, _right_frac)) = target_weights {
        let tpwgts = tmp_dir.path().join("tpwgts.txt");
        // Only write partition 0; METIS infers partition 1 = 1 - left_frac
        std::fs::write(&tpwgts, format!("0 = {left_frac:.6}\n"))
            .map_err(|e| e.to_string())?;
        Some(tpwgts)
    } else {
        None
    };

    let mut cmd = Command::new(&gpmetis);
    cmd.args([
        "-contig",
        "-minconn",
        &format!("-niter={niter}"),
        &format!("-ufactor={ufactor:.4}"),
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

/// Run the full level-parallel bisection for k districts.
/// Returns HashMap<tract_index, district_id> (1-based district IDs).
///
/// RACE CONDITION FIX: tract data extracted from node_tracts sequentially
/// BEFORE par_iter, so closures own their data with no shared references.
///
/// SORT FIX: leaves sorted by (depth, path) not plain lex, which gives
/// correct BFS order for mixed-length binary paths.
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

        // Use depth-based ufactor matching Python's recursive_bisection.py logic:
        //   depth 1 → 1.001 (first split: tightest)
        //   depth 2 → 1.002, depth 3 → 1.003, depth 4+ → 1.005
        // Rust BFS depth 0 = Python's depth 1 (first actual split), hence depth+1.
        let depth_ufactor = ufactor_for_depth(depth + 1);

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
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
                        depth_ufactor, niter, seed, tw
                    ).map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?;
                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        for (path, left, right) in split_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
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
        let assignments = run_all_splits(&adj, &vw, &ew, 1, 5, 100, None)
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

        let assignments = run_all_splits(&adj, &vw, &ew, 2, 5, 100, Some(42)).unwrap(); // ufactor=5 converted internally

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
}
