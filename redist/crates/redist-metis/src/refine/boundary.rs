use std::collections::HashSet;
use crate::graph::{CsrGraph, Partition};

pub struct BoundarySet { inner: HashSet<u32> }

impl BoundarySet {
    pub fn from_partition(g: &CsrGraph, p: &Partition) -> Self {
        let mut inner = HashSet::new();
        for v in 0..g.n() {
            for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
                let u = g.adjncy[j] as usize;
                if p.assignment[v] != p.assignment[u] {
                    inner.insert(v as u32);
                    break;
                }
            }
        }
        Self { inner }
    }

    pub fn contains(&self, v: u32) -> bool { self.inner.contains(&v) }
    pub fn insert(&mut self, v: u32) { self.inner.insert(v); }
    pub fn remove(&mut self, v: u32) { self.inner.remove(&v); }
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ { self.inner.iter().copied() }
    pub fn len(&self) -> usize { self.inner.len() }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_graph(n: usize) -> CsrGraph {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i - 1) as u32); }
            if i < n - 1 { adjncy.push((i + 1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    #[test]
    fn boundary_contains_cross_part_vertices() {
        // path 0-1-2, partition [0,0,1]
        // vertex 1 (part 0) adj to vertex 2 (part 1) → on boundary
        // vertex 2 (part 1) adj to vertex 1 (part 0) → on boundary
        // vertex 0 (part 0) only adj to vertex 1 (same part) → NOT on boundary
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
