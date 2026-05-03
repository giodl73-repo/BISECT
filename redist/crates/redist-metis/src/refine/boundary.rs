use crate::graph::{CsrGraph, Partition};

/// Cache-friendly boundary set using a boolean array indexed by vertex ID.
/// O(1) contains/insert/remove with sequential memory access.
/// iter() is O(n) (scans full array) — acceptable since boundary is ~20% of n.
pub struct BoundarySet {
    inner: Vec<bool>,
}

impl BoundarySet {
    pub fn from_partition(g: &CsrGraph, p: &Partition) -> Self {
        let n = g.n();
        let mut inner = vec![false; n];
        for v in 0..n {
            for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
                let u = g.adjncy[j] as usize;
                if p.assignment[v] != p.assignment[u] {
                    inner[v] = true;
                    break;
                }
            }
        }
        Self { inner }
    }

    pub fn contains(&self, v: u32) -> bool {
        self.inner[v as usize]
    }

    pub fn insert(&mut self, v: u32) {
        self.inner[v as usize] = true;
    }

    pub fn remove(&mut self, v: u32) {
        self.inner[v as usize] = false;
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.inner.iter().enumerate()
            .filter_map(|(i, &b)| if b { Some(i as u32) } else { None })
    }

    /// O(n) — only call in non-hot paths (tests, debug)
    pub fn len(&self) -> usize {
        self.inner.iter().filter(|&&b| b).count()
    }

    pub fn is_empty(&self) -> bool {
        !self.inner.iter().any(|&b| b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Partition;

    fn path_graph(n: usize) -> CsrGraph {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i-1) as u32); }
            if i < n-1 { adjncy.push((i+1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    #[test]
    fn boundary_contains_cross_part_vertices() {
        let g = path_graph(3);
        let p = Partition { assignment: vec![0, 0, 1], k: 2 };
        let b = BoundarySet::from_partition(&g, &p);
        assert!(b.contains(1), "vertex 1 should be on boundary");
        assert!(b.contains(2), "vertex 2 should be on boundary");
        assert!(!b.contains(0), "vertex 0 should NOT be on boundary");
    }

    #[test]
    fn boundary_all_same_part_empty() {
        let g = path_graph(5);
        let p = Partition { assignment: vec![0; 5], k: 1 };
        let b = BoundarySet::from_partition(&g, &p);
        for v in 0..5u32 { assert!(!b.contains(v)); }
    }
}
