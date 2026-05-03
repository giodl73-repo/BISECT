use crate::graph::{CsrGraph, Partition};
use super::{gain::GainTable, boundary::BoundarySet};

pub struct FmState<'g> {
    pub graph:      &'g CsrGraph,
    pub assignment: Vec<u32>,
    pub k:          u32,
    pub gain_table: GainTable,
    pub boundary:   BoundarySet,
    pub part_pop:   Vec<i64>,
}

#[derive(Clone)]
pub struct Checkpoint {
    pub assignment: Vec<u32>,
    pub cut:        i64,
}

impl<'g> FmState<'g> {
    pub fn new(g: &'g CsrGraph, p: Partition) -> Self {
        let n = g.n();
        let k = p.k as usize;
        let mut part_pop = vec![0i64; k];
        for v in 0..n { part_pop[p.assignment[v] as usize] += g.vwgt[v] as i64; }

        // max_gain = max edge weight (or 1 if unweighted) × max degree
        let max_ew = g.adjwgt.as_ref()
            .and_then(|aw| aw.iter().copied().max())
            .unwrap_or(1);
        let max_deg = (0..n).map(|v| g.xadj[v + 1] - g.xadj[v]).max().unwrap_or(1) as i32;
        let max_gain = (max_ew * max_deg).max(1);

        let boundary = BoundarySet::from_partition(g, &p);
        let mut gain_table = GainTable::new(n, max_gain);
        for v in boundary.iter() {
            let gain = compute_gain(g, &p.assignment, v as usize);
            let gain_clamped = gain.clamp(-max_gain, max_gain);
            gain_table.insert(v, gain_clamped);
        }

        FmState { graph: g, assignment: p.assignment, k: p.k, gain_table, boundary, part_pop }
    }

    pub fn cut(&self) -> i64 {
        let g = self.graph;
        let mut c = 0i64;
        for v in 0..g.n() {
            for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
                let u = g.adjncy[j] as usize;
                if self.assignment[v] != self.assignment[u] {
                    c += g.adjwgt.as_ref().map_or(1i64, |aw| aw[j] as i64);
                }
            }
        }
        c / 2 // each edge counted twice
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { assignment: self.assignment.clone(), cut: self.cut() }
    }

    pub fn restore(&mut self, cp: &Checkpoint) {
        self.assignment = cp.assignment.clone();
        let p = Partition { assignment: self.assignment.clone(), k: self.k };
        self.boundary = BoundarySet::from_partition(self.graph, &p);
        let n = self.graph.n();
        let max_gain = self.gain_table.max_gain;
        self.gain_table = GainTable::new(n, max_gain);
        for v in self.boundary.iter() {
            let gain = compute_gain(self.graph, &self.assignment, v as usize);
            let gain_clamped = gain.clamp(-max_gain, max_gain);
            self.gain_table.insert(v, gain_clamped);
        }
        // Recompute part populations
        let k = self.k as usize;
        self.part_pop = vec![0i64; k];
        for v in 0..n { self.part_pop[self.assignment[v] as usize] += self.graph.vwgt[v] as i64; }
    }
}

/// Gain of moving vertex v to its "other" part (for k=2 bisection).
/// For k>2, returns gain of moving to the best adjacent part.
/// Gain = (edges to other parts) - (edges to same part).
pub fn compute_gain(g: &CsrGraph, assignment: &[u32], v: usize) -> i32 {
    let part_v = assignment[v];
    let mut gain = 0i32;
    for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
        let u = g.adjncy[j] as usize;
        let ew = g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]);
        if assignment[u] == part_v { gain -= ew; } else { gain += ew; }
    }
    gain
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
    fn fm_state_cut_path3_split() {
        // path 0-1-2, partition [0,0,1]
        // cut edge: 1-2, so cut = 1
        let g = path_graph(3);
        let p = Partition { assignment: vec![0, 0, 1], k: 2 };
        let state = FmState::new(&g, p);
        assert_eq!(state.cut(), 1);
    }

    #[test]
    fn fm_state_checkpoint_restore() {
        let g = path_graph(4);
        let p = Partition { assignment: vec![0, 0, 1, 1], k: 2 };
        let mut state = FmState::new(&g, p);
        let cp = state.checkpoint();
        assert_eq!(cp.cut, state.cut());
        // Modify assignment (simulate a move)
        state.assignment[1] = 1;
        assert_ne!(state.assignment[1], cp.assignment[1]);
        // Restore
        state.restore(&cp);
        assert_eq!(state.assignment, cp.assignment);
    }
}
