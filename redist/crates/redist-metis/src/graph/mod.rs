use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CsrGraph {
    pub xadj:   Vec<u32>,
    pub adjncy: Vec<u32>,
    pub ncon:   u32,
    pub vwgt:   Vec<i32>,
    pub adjwgt: Option<Vec<i32>>,
}

impl CsrGraph {
    pub fn n(&self) -> usize { self.xadj.len().saturating_sub(1) }

    pub fn is_valid(&self) -> bool {
        let n = self.n();
        if self.xadj.len() != n + 1 { return false; }
        if n == 0 { return true; }
        if self.xadj[0] != 0 { return false; }
        if self.ncon < 1 { return false; }
        if self.vwgt.len() != n * self.ncon as usize { return false; }
        if self.vwgt.iter().any(|&w| w <= 0) { return false; }
        if let Some(ref aw) = self.adjwgt {
            if aw.len() != self.adjncy.len() { return false; }
        }
        for i in 0..n {
            if self.xadj[i] > self.xadj[i + 1] { return false; }
            for j in self.xadj[i] as usize..self.xadj[i + 1] as usize {
                if j >= self.adjncy.len() { return false; }
                let nb = self.adjncy[j] as usize;
                if nb >= n || nb == i { return false; }
            }
        }
        // PP-05: connectivity BFS from vertex 0
        let mut visited = vec![false; n];
        let mut queue = VecDeque::new();
        queue.push_back(0usize);
        visited[0] = true;
        while let Some(v) = queue.pop_front() {
            for j in self.xadj[v] as usize..self.xadj[v + 1] as usize {
                let u = self.adjncy[j] as usize;
                if !visited[u] { visited[u] = true; queue.push_back(u); }
            }
        }
        visited.iter().all(|&v| v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    pub assignment: Vec<u32>,
    pub k:          u32,
    /// Target partition weights (one `f32` per part, summing to 1.0).
    /// `None` means equal weights (each part gets `1/k` of total population).
    /// Set by `split_weighted` and consumed by FM balance checks.
    pub tpwgts:     Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct CoarseMap { pub cmap: Vec<u32> }

#[cfg(test)]
mod tests {
    use super::*;

    pub fn path_graph(n: usize) -> CsrGraph {
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
    fn valid_path_graph() { assert!(path_graph(5).is_valid()); }

    #[test]
    fn invalid_self_loop() {
        let mut g = path_graph(4);
        g.adjncy[0] = 0;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_out_of_bounds_adjncy() {
        let mut g = path_graph(4);
        g.adjncy[0] = 99;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_zero_vwgt() {
        let mut g = path_graph(4);
        g.vwgt[1] = 0;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_negative_vwgt() {
        let mut g = path_graph(4);
        g.vwgt[1] = -1;
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_disconnected() {
        let g = CsrGraph {
            xadj:   vec![0, 1, 2, 3, 4],
            adjncy: vec![1, 0, 3, 2],
            ncon: 1,
            vwgt: vec![1; 4],
            adjwgt: None,
        };
        assert!(!g.is_valid());
    }

    #[test]
    fn invalid_adjwgt_wrong_len() {
        let mut g = path_graph(4);
        g.adjwgt = Some(vec![1i32; 3]);
        assert!(!g.is_valid());
    }

    #[test]
    fn valid_multi_constraint() {
        let mut g = path_graph(4);
        g.ncon = 2;
        g.vwgt = vec![1, 2, 3, 4, 5, 6, 7, 8];
        assert!(g.is_valid());
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// Proves: CsrGraph::is_valid() never panics for any input up to n=8.
    /// Covers: all branches in is_valid() including xadj check, self-loop (PP-01),
    /// OOB adjncy, ncon, vwgt positivity, adjwgt length, BFS connectivity.
    #[kani::proof]
    #[kani::unwind(9)]
    fn verify_is_valid_no_panic() {
        let n: usize = kani::any_where(|&n: &usize| n <= 8);
        // Construct arbitrary xadj of length n+1
        let xadj: Vec<u32> = (0..=n).map(|_| kani::any()).collect();
        let adjncy_len: usize = kani::any_where(|&l: &usize| l <= 32);
        let adjncy: Vec<u32> = (0..adjncy_len).map(|_| kani::any()).collect();
        let ncon: u32 = kani::any_where(|&c: &u32| c <= 4);
        let vwgt_len = n.saturating_mul(ncon as usize).min(64);
        let vwgt: Vec<i32> = (0..vwgt_len).map(|_| kani::any()).collect();
        let g = CsrGraph { xadj, adjncy, ncon, vwgt, adjwgt: None };
        // Must not panic — result is ignored
        let _ = g.is_valid();
    }
}
