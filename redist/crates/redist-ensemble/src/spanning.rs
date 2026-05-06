//! Wilson's loop-erased random walk for uniform random spanning trees.
//!
//! Given a connected undirected graph on n vertices, samples a spanning tree
//! uniformly at random in O(cover time) expected time — O(n log n) for planar
//! graphs (Wilson 1996, Theorem 1.1).
//!
//! For redistricting regions of ~n/k tracts this runs in O((n/k) log(n/k)).

use rand::Rng;

/// A spanning tree represented as a parent array.
///
/// `parent[v] == u32::MAX` for the root; otherwise `parent[v]` is v's parent.
pub struct SpanningTree {
    pub parent: Vec<u32>,
    pub n: usize,
}

impl SpanningTree {
    /// Iterate over all tree edges as (child, parent) pairs.
    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.n as u32)
            .filter(|&v| self.parent[v as usize] != u32::MAX)
            .map(|v| (v, self.parent[v as usize]))
    }

    /// Given a cut edge (a, b), partition the tree into two components.
    /// Returns (component containing `a`, component containing `b`).
    pub fn split_on(&self, a: u32, b: u32) -> (Vec<u32>, Vec<u32>) {
        // Build children lists from parent array (excluding the cut edge a↔b).
        let n = self.n;
        let mut children: Vec<Vec<u32>> = vec![vec![]; n];
        for v in 0..n as u32 {
            let p = self.parent[v as usize];
            if p == u32::MAX { continue; }
            // Skip the cut edge in both directions.
            if (v == a && p == b) || (v == b && p == a) { continue; }
            children[p as usize].push(v);
            children[v as usize].push(p); // undirected — add both so BFS works
        }
        // But the children list built above is wrong for undirected traversal.
        // Rebuild as a proper adjacency list of the tree minus the cut edge.
        let mut adj: Vec<Vec<u32>> = vec![vec![]; n];
        for v in 0..n as u32 {
            let p = self.parent[v as usize];
            if p == u32::MAX { continue; }
            if (v == a && p == b) || (v == b && p == a) { continue; }
            adj[v as usize].push(p);
            adj[p as usize].push(v);
        }
        // BFS from a to find its component.
        let mut comp_a = Vec::new();
        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(a);
        visited[a as usize] = true;
        while let Some(v) = queue.pop_front() {
            comp_a.push(v);
            for &nb in &adj[v as usize] {
                if !visited[nb as usize] {
                    visited[nb as usize] = true;
                    queue.push_back(nb);
                }
            }
        }
        let comp_b: Vec<u32> = (0..n as u32).filter(|&v| !visited[v as usize]).collect();
        (comp_a, comp_b)
    }
}

/// Sample a uniform random spanning tree of the subgraph using Wilson's algorithm.
///
/// `adj[v]` is the list of neighbour indices of vertex `v` (local indices 0..n).
/// Returns a `SpanningTree` where `parent[root] == u32::MAX`.
pub fn random_spanning_tree<R: Rng>(adj: &[Vec<u32>], rng: &mut R) -> SpanningTree {
    let n = adj.len();
    assert!(n >= 2, "need at least 2 vertices for a spanning tree");

    let mut parent = vec![u32::MAX; n];
    let mut in_tree = vec![false; n];

    // Choose a random root.
    let root = rng.gen_range(0..n) as u32;
    in_tree[root as usize] = true;

    // Loop-erased random walk from each not-yet-in-tree vertex.
    for start in 0..n as u32 {
        if in_tree[start as usize] { continue; }

        // Walk from `start` until we hit the tree.
        let mut path: Vec<u32> = vec![start];
        // next[v] records the next step from v during the current walk
        // (used to detect and erase loops efficiently).
        let mut next: Vec<u32> = vec![u32::MAX; n];
        let mut cur = start;

        loop {
            let nbrs = &adj[cur as usize];
            let step = nbrs[rng.gen_range(0..nbrs.len())];

            if in_tree[step as usize] {
                // Reached the tree — record final step and stop.
                next[cur as usize] = step;
                break;
            }

            if next[step as usize] != u32::MAX {
                // Loop detected: erase it by truncating the path.
                // Find `step` in path and cut there.
                let pos = path.iter().position(|&v| v == step).unwrap();
                // Erase loop: clear next[] for the erased portion.
                for &v in &path[pos + 1..] {
                    next[v as usize] = u32::MAX;
                }
                path.truncate(pos + 1);
                cur = step;
            } else {
                next[cur as usize] = step;
                path.push(step);
                cur = step;
            }
        }

        // Add the walk to the tree.
        for &v in &path {
            let nxt = next[v as usize];
            if nxt != u32::MAX {
                parent[v as usize] = nxt;
                in_tree[v as usize] = true;
            }
        }
    }

    SpanningTree { parent, n }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn path_graph(n: usize) -> Vec<Vec<u32>> {
        let mut adj = vec![vec![]; n];
        for i in 0..n-1 {
            adj[i].push((i+1) as u32);
            adj[i+1].push(i as u32);
        }
        adj
    }

    fn grid_graph(rows: usize, cols: usize) -> Vec<Vec<u32>> {
        let n = rows * cols;
        let mut adj = vec![vec![]; n];
        for r in 0..rows {
            for c in 0..cols {
                let v = r * cols + c;
                if c + 1 < cols { adj[v].push((v+1) as u32); adj[v+1].push(v as u32); }
                if r + 1 < rows { adj[v].push((v+cols) as u32); adj[v+cols].push(v as u32); }
            }
        }
        adj
    }

    fn is_spanning_tree(parent: &[u32], n: usize) -> bool {
        // Exactly one root (parent == MAX).
        let roots = parent.iter().filter(|&&p| p == u32::MAX).count();
        if roots != 1 { return false; }
        // n-1 edges.
        let edges = parent.iter().filter(|&&p| p != u32::MAX).count();
        if edges != n - 1 { return false; }
        // All vertices reachable from root (no cycles ↔ connected tree).
        let root = parent.iter().position(|&p| p == u32::MAX).unwrap();
        let mut visited = vec![false; n];
        let mut adj_tree: Vec<Vec<usize>> = vec![vec![]; n];
        for v in 0..n {
            if parent[v] != u32::MAX {
                adj_tree[v].push(parent[v] as usize);
                adj_tree[parent[v] as usize].push(v);
            }
        }
        let mut stack = vec![root];
        visited[root] = true;
        while let Some(v) = stack.pop() {
            for &nb in &adj_tree[v] {
                if !visited[nb] { visited[nb] = true; stack.push(nb); }
            }
        }
        visited.iter().all(|&v| v)
    }

    #[test]
    fn spanning_tree_path_graph() {
        let adj = path_graph(10);
        let mut rng = SmallRng::seed_from_u64(42);
        let tree = random_spanning_tree(&adj, &mut rng);
        assert!(is_spanning_tree(&tree.parent, 10));
    }

    #[test]
    fn spanning_tree_grid_graph() {
        let adj = grid_graph(4, 4);
        let mut rng = SmallRng::seed_from_u64(99);
        let tree = random_spanning_tree(&adj, &mut rng);
        assert!(is_spanning_tree(&tree.parent, 16));
    }

    #[test]
    fn spanning_tree_split_is_partition() {
        let adj = path_graph(10);
        let mut rng = SmallRng::seed_from_u64(7);
        let tree = random_spanning_tree(&adj, &mut rng);
        // Cut the first tree edge.
        let (a, b) = tree.edges().next().unwrap();
        let (ca, cb) = tree.split_on(a, b);
        let mut all: Vec<u32> = ca.iter().chain(cb.iter()).copied().collect();
        all.sort_unstable();
        let expected: Vec<u32> = (0..10).collect();
        assert_eq!(all, expected, "split must partition all vertices");
        assert!(!ca.is_empty() && !cb.is_empty(), "both components must be non-empty");
    }

    #[test]
    fn spanning_tree_multiple_seeds_produce_different_trees() {
        let adj = grid_graph(5, 5);
        let mut rng1 = SmallRng::seed_from_u64(1);
        let mut rng2 = SmallRng::seed_from_u64(2);
        let t1 = random_spanning_tree(&adj, &mut rng1);
        let t2 = random_spanning_tree(&adj, &mut rng2);
        // With high probability two different seeds produce different trees.
        assert_ne!(t1.parent, t2.parent, "different seeds should produce different trees");
    }
}
