/// Bisection tree: pure split-schedule computation.
///
/// Given k districts, computes the binary tree of (k_left, k_right) splits
/// by depth level. This is the scheduling layer — which k goes where.
/// Actual METIS calls happen in the Python pipeline or (Phase 3) in redist-cli.
///
/// Split rule: k_left = k/2 (floor), k_right = k - k_left.
/// Example for k=7: [4,3] → [2,2],[2,1] → [1,1],[1,1],[1,1]

/// A single non-leaf node in the bisection tree.
#[derive(Debug, Clone, PartialEq)]
pub struct BisectionNode {
    /// Districts assigned to this region.
    pub k: usize,
    /// Districts going to the left child.
    pub k_left: usize,
    /// Districts going to the right child.
    pub k_right: usize,
    /// Depth in tree (root = 0).
    pub depth: usize,
    /// Binary path from root, e.g. "" → "0" → "01". Used for node naming.
    pub path: String,
}

/// Complete binary split schedule for k districts.
///
/// `nodes` contains all non-leaf nodes in BFS order (root first, then each
/// level left-to-right). Leaf nodes (k=1) are not stored.
///
/// **Level-parallel semantics**: all nodes at a given depth are processed
/// simultaneously. The Python pipeline uses `ProcessPoolExecutor` to run all
/// splits at the same depth in parallel; the Rust CLI (Phase 3c) uses Rayon.
/// Depth-first processing would produce different intermediate partitions.
#[derive(Debug, Clone)]
pub struct BisectionTree {
    pub k: usize,
    pub max_depth: usize,
    pub nodes: Vec<BisectionNode>,
}

impl BisectionTree {
    /// Build the full split schedule for k districts.
    pub fn from_k(k: usize) -> Self {
        if k == 0 {
            panic!("k must be >= 1");
        }
        let max_depth = max_depth_for_k(k);
        let mut nodes = Vec::new();
        // BFS queue: (k, depth, path)
        let mut queue: std::collections::VecDeque<(usize, usize, String)> =
            std::collections::VecDeque::new();
        queue.push_back((k, 0, String::new()));

        while let Some((k_curr, depth, path)) = queue.pop_front() {
            if k_curr <= 1 {
                continue; // leaf — no split
            }
            let k_left = k_curr / 2;
            let k_right = k_curr - k_left;
            nodes.push(BisectionNode {
                k: k_curr,
                k_left,
                k_right,
                depth,
                path: path.clone(),
            });
            queue.push_back((k_left, depth + 1, format!("{path}0")));
            queue.push_back((k_right, depth + 1, format!("{path}1")));
        }

        BisectionTree { k, max_depth, nodes }
    }

    /// Nodes that need splitting at a given depth level.
    pub fn nodes_at_depth(&self, depth: usize) -> Vec<&BisectionNode> {
        self.nodes.iter().filter(|n| n.depth == depth).collect()
    }

    /// Number of splits at each depth level.
    pub fn splits_per_depth(&self) -> Vec<usize> {
        let mut counts = vec![0usize; self.max_depth + 1];
        for n in &self.nodes {
            counts[n.depth] += 1;
        }
        counts
    }

    /// Total splits across all levels = k - 1.
    pub fn total_splits(&self) -> usize {
        self.nodes.len()
    }
}

/// Number of bisection rounds needed to produce k districts.
/// Equivalent to ceil(log2(k)).
pub fn max_depth_for_k(k: usize) -> usize {
    if k <= 1 {
        return 0;
    }
    let mut depth = 0;
    let mut current = 1usize;
    while current < k {
        current *= 2;
        depth += 1;
    }
    depth
}

/// METIS ufactor (imbalance tolerance) for a given bisection depth and user-specified base ufactor.
///
/// Tighter at early depths to maintain global balance.
/// Depth 0 (root node) is never split by METIS — it represents the whole state.
///
/// # Scaling
/// - `base_ufactor=5`  → 0.5% balance per split (congressional default)
/// - `base_ufactor=50` → 5% balance per split (state legislative)
/// - depth 1 gets the tightest constraint (20% of full base), depth 4+ gets 100%.
///
/// # Backward compatibility
/// When `base_ufactor=5` the results are identical to the old hardcoded values:
/// depth 1 → 1.001, depth 2 → 1.002, depth 3 → 1.003, depth 4+ → 1.005.
pub fn ufactor_for_depth(depth: usize, base_ufactor: u32) -> f64 {
    let base = 1.0 + base_ufactor as f64 / 1000.0;
    let scale = match depth {
        0 => 0.2, // Root never split; unreachable in normal operation
        1 => 0.2, // tightest at first split
        2 => 0.4,
        3 => 0.6,
        _ => 1.0, // full ufactor at deep levels
    };
    1.0 + (base - 1.0) * scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth_cases() {
        assert_eq!(max_depth_for_k(1), 0);
        assert_eq!(max_depth_for_k(2), 1);
        assert_eq!(max_depth_for_k(3), 2);
        assert_eq!(max_depth_for_k(4), 2);
        assert_eq!(max_depth_for_k(7), 3);
        assert_eq!(max_depth_for_k(8), 3);
        assert_eq!(max_depth_for_k(14), 4);
        assert_eq!(max_depth_for_k(52), 6);
    }

    #[test]
    fn test_total_splits_equals_k_minus_1() {
        for k in [2, 3, 4, 7, 8, 14, 52] {
            let tree = BisectionTree::from_k(k);
            assert_eq!(tree.total_splits(), k - 1, "k={k}");
        }
    }

    #[test]
    fn test_k2_single_split() {
        let tree = BisectionTree::from_k(2);
        assert_eq!(tree.max_depth, 1);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].k_left, 1);
        assert_eq!(tree.nodes[0].k_right, 1);
    }

    #[test]
    fn test_k7_split_schedule() {
        let tree = BisectionTree::from_k(7);
        assert_eq!(tree.max_depth, 3);
        // Depth 0: [7] → [4,3]
        let d0 = tree.nodes_at_depth(0);
        assert_eq!(d0.len(), 1);
        assert_eq!((d0[0].k_left, d0[0].k_right), (3, 4));
        // Depth 1: [3,4] → [1,2] + [2,2]
        let d1 = tree.nodes_at_depth(1);
        assert_eq!(d1.len(), 2);
        // Depth 2: [1,2,2,2] — only k>1 get split → [2] → [1,1], [2] → [1,1], [2] → [1,1]
        let d2 = tree.nodes_at_depth(2);
        assert_eq!(d2.len(), 3);
    }

    #[test]
    fn test_k52_depth_and_splits() {
        let tree = BisectionTree::from_k(52);
        assert_eq!(tree.max_depth, 6);
        assert_eq!(tree.total_splits(), 51);
    }

    #[test]
    fn test_k14_depth_and_splits() {
        let tree = BisectionTree::from_k(14);
        assert_eq!(tree.max_depth, 4);
        assert_eq!(tree.total_splits(), 13);
    }

    #[test]
    fn test_splits_per_depth_sums_to_total() {
        let tree = BisectionTree::from_k(7);
        let per_depth: usize = tree.splits_per_depth().iter().sum();
        assert_eq!(per_depth, tree.total_splits());
    }

    #[test]
    fn test_k_left_plus_k_right_equals_k() {
        for k in [2, 3, 4, 7, 8, 14, 52] {
            let tree = BisectionTree::from_k(k);
            for node in &tree.nodes {
                assert_eq!(node.k_left + node.k_right, node.k,
                    "k={k} node.k={}", node.k);
            }
        }
    }

    #[test]
    fn test_paths_are_unique() {
        for k in [2, 7, 14, 52] {
            let tree = BisectionTree::from_k(k);
            let paths: std::collections::HashSet<&str> =
                tree.nodes.iter().map(|n| n.path.as_str()).collect();
            assert_eq!(paths.len(), tree.nodes.len(), "duplicate paths for k={k}");
        }
    }

    #[test]
    fn test_ufactor_by_depth_default() {
        // base_ufactor=5 (congressional default) must be backward-compatible
        assert!((ufactor_for_depth(0, 5) - 1.001).abs() < 1e-9); // root — unreachable but safe
        assert!((ufactor_for_depth(1, 5) - 1.001).abs() < 1e-9);
        assert!((ufactor_for_depth(2, 5) - 1.002).abs() < 1e-9);
        assert!((ufactor_for_depth(3, 5) - 1.003).abs() < 1e-9);
        assert!((ufactor_for_depth(4, 5) - 1.005).abs() < 1e-9);
        assert!((ufactor_for_depth(99, 5) - 1.005).abs() < 1e-9);
    }

    #[test]
    fn test_ufactor_by_depth_state_legislative() {
        // base_ufactor=50 (state legislative): depth 1 → 1.010, depth 4+ → 1.050
        assert!((ufactor_for_depth(1, 50) - 1.010).abs() < 1e-9);
        assert!((ufactor_for_depth(2, 50) - 1.020).abs() < 1e-9);
        assert!((ufactor_for_depth(3, 50) - 1.030).abs() < 1e-9);
        assert!((ufactor_for_depth(4, 50) - 1.050).abs() < 1e-9);
        assert!((ufactor_for_depth(99, 50) - 1.050).abs() < 1e-9);
    }

    #[test]
    fn test_single_district_no_splits() {
        let tree = BisectionTree::from_k(1);
        assert_eq!(tree.total_splits(), 0);
        assert_eq!(tree.max_depth, 0);
    }
}
