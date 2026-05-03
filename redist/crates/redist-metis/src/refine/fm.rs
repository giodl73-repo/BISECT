use crate::graph::{CsrGraph, Partition};
use super::{gain::GainTable, boundary::BoundarySet};
use crate::refine::Refiner;

pub struct FiducciaMattheyses { pub niter: u32 }

impl Refiner for FiducciaMattheyses {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition {
        let mut state = FmState::new(g, p);
        let mut best = state.checkpoint();

        for _pass in 0..self.niter {
            let improved = fm_pass(&mut state, &mut best);
            // Always restore to best after each pass so the next pass
            // starts from the best-known state, not the end-of-pass state.
            state.restore(&best);
            if !improved { break; }
        }
        Partition { assignment: state.assignment, k: state.k, tpwgts: None }
    }
}

fn fm_pass(state: &mut FmState, best: &mut Checkpoint) -> bool {
    let ncon = state.graph.ncon as usize;
    let k    = state.k as usize;
    let total_pops: Vec<i64> = (0..ncon)
        .map(|c| state.part_pop[c].iter().sum())
        .collect();

    // Per-part, per-constraint targets and epsilons.
    // Layout: targets_pc[part][constraint], epsilons_pc[part][constraint].
    //
    // When tpwgts is provided: constraint 0 gets proportional targets derived from
    // the float weights; all other constraints (VAP, etc.) keep equal targets.
    // When tpwgts is None: all constraints use equal targets.
    let targets_pc: Vec<Vec<i64>> = (0..k).map(|part| {
        (0..ncon).map(|c| {
            if c == 0 {
                match &state.tpwgts {
                    Some(tw) => (total_pops[0] as f64 * tw[part] as f64).round() as i64,
                    None     => total_pops[0] / k as i64,
                }
            } else {
                total_pops[c] / k as i64
            }
        }).collect()
    }).collect();
    // INTEGER balance epsilon — ceiling of 0.5% of each part's target — NO FLOATS
    let epsilons_pc: Vec<Vec<i64>> = targets_pc.iter()
        .map(|row| row.iter().map(|&t| (t.abs() * 5 + 999) / 1000).collect())
        .collect();

    let start_cut = best.cut;

    // Locked set: vertices that have been popped this pass must not be re-inserted.
    // This is the fundamental FM invariant — each vertex moves at most once per pass.
    let n = state.graph.n();
    let mut locked = vec![false; n];

    loop {
        let Some((v, _gain)) = state.gain_table.pop_max() else { break };
        let v = v as usize;
        locked[v] = true;

        let from_part = state.assignment[v] as usize;

        // Find best destination part
        let to_part = if state.k == 2 {
            1 - from_part as u32
        } else {
            best_destination(state, v, from_part as u32)
        } as usize;

        // Gather per-constraint weights for vertex v
        let vwgt_v: Vec<i64> = (0..ncon)
            .map(|c| state.graph.vwgt[v * ncon + c] as i64)
            .collect();

        // Balance check — ALL constraints must pass for BOTH parts
        let balanced = (0..ncon).all(|c| {
            let new_from = state.part_pop[c][from_part] - vwgt_v[c];
            let new_to   = state.part_pop[c][to_part]   + vwgt_v[c];
            new_from >= targets_pc[from_part][c] - epsilons_pc[from_part][c]
                && new_to <= targets_pc[to_part][c] + epsilons_pc[to_part][c]
        });
        if !balanced { continue; }

        // Apply move
        state.assignment[v] = to_part as u32;
        for c in 0..ncon {
            state.part_pop[c][from_part] -= vwgt_v[c];
            state.part_pop[c][to_part]   += vwgt_v[c];
        }
        state.boundary.remove(v as u32);

        // Update current_cut incrementally — O(degree(v)) not O(m)
        {
            let g = state.graph;
            let mut cut_delta: i64 = 0;
            for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
                let u = g.adjncy[j] as usize;
                let ew = g.adjwgt.as_ref().map_or(1i64, |aw| aw[j] as i64);
                if state.assignment[u] as usize == from_part { cut_delta += ew; } // was same, now cross
                if state.assignment[u] as usize == to_part   { cut_delta -= ew; } // was cross, now same
            }
            state.current_cut += cut_delta;
        }

        // Update gains for all unlocked neighbours of v
        let g = state.graph;
        for j in g.xadj[v] as usize..g.xadj[v + 1] as usize {
            let u = g.adjncy[j] as usize;
            if locked[u] { continue; } // never re-insert a locked vertex

            let new_gain = compute_gain(g, &state.assignment, u);
            let clamped = new_gain.clamp(-state.gain_table.max_gain, state.gain_table.max_gain);
            // Check if u is on boundary after the move
            let u_on_boundary = (g.xadj[u] as usize..g.xadj[u + 1] as usize)
                .any(|jj| state.assignment[g.adjncy[jj] as usize] != state.assignment[u]);
            if u_on_boundary {
                if state.gain_table.contains(u as u32) {
                    state.gain_table.update(u as u32, clamped);
                } else {
                    state.boundary.insert(u as u32);
                    state.gain_table.insert(u as u32, clamped);
                }
            } else {
                if state.gain_table.contains(u as u32) {
                    state.gain_table.remove(u as u32);
                }
                state.boundary.remove(u as u32);
            }
        }

        // Checkpoint if improved — use incremental cut value, O(1)
        let cur_cut = state.current_cut;
        if cur_cut < best.cut {
            *best = Checkpoint { assignment: state.assignment.clone(), cut: cur_cut };
        }
    }

    best.cut < start_cut
}

fn best_destination(state: &FmState, v: usize, from: u32) -> u32 {
    let g = state.graph;
    (0..state.k)
        .filter(|&p| p != from)
        .max_by_key(|&p| {
            (g.xadj[v] as usize..g.xadj[v + 1] as usize)
                .filter(|&j| state.assignment[g.adjncy[j] as usize] == p)
                .map(|j| g.adjwgt.as_ref().map_or(1i32, |aw| aw[j]))
                .sum::<i32>()
        })
        .unwrap_or(if from == 0 { 1 } else { 0 })
}

pub struct FmState<'g> {
    pub graph:        &'g CsrGraph,
    pub assignment:   Vec<u32>,
    pub k:            u32,
    pub gain_table:   GainTable,
    pub boundary:     BoundarySet,
    /// `part_pop[constraint][part]` = population sum for constraint c in part p.
    /// For ncon=1, `part_pop[0][part]` is equivalent to the old single-constraint `part_pop[part]`.
    pub part_pop:     Vec<Vec<i64>>,
    pub current_cut:  i64,
    /// Per-part target weights (one f32 per part, summing to 1.0).
    /// `None` means equal weights: each part targets `total_pop / k`.
    pub tpwgts:       Option<Vec<f32>>,
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
        let ncon = g.ncon as usize;
        let mut part_pop = vec![vec![0i64; k]; ncon];
        for v in 0..n {
            let part = p.assignment[v] as usize;
            for c in 0..ncon {
                part_pop[c][part] += g.vwgt[v * ncon + c] as i64;
            }
        }

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
            debug_assert!(
                gain.abs() <= max_gain,
                "computed gain {gain} exceeds max_gain {max_gain} — max_gain estimate is wrong"
            );
            let gain_clamped = gain.clamp(-max_gain, max_gain);
            gain_table.insert(v, gain_clamped);
        }

        let tpwgts = p.tpwgts.clone();
        let mut state = FmState { graph: g, assignment: p.assignment, k: p.k, gain_table, boundary, part_pop, current_cut: 0, tpwgts };
        state.current_cut = state.cut();
        state
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
        Checkpoint { assignment: self.assignment.clone(), cut: self.current_cut }
    }

    pub fn restore(&mut self, cp: &Checkpoint) {
        self.assignment = cp.assignment.clone();
        // tpwgts is preserved across restore — it is a property of the partition problem,
        // not the current state, and must survive all passes.
        let p = Partition { assignment: self.assignment.clone(), k: self.k, tpwgts: self.tpwgts.clone() };
        self.boundary = BoundarySet::from_partition(self.graph, &p);
        let n = self.graph.n();
        let max_gain = self.gain_table.max_gain;
        self.gain_table = GainTable::new(n, max_gain);
        for v in self.boundary.iter() {
            let gain = compute_gain(self.graph, &self.assignment, v as usize);
            debug_assert!(
                gain.abs() <= max_gain,
                "computed gain {gain} exceeds max_gain {max_gain} — max_gain estimate is wrong"
            );
            let gain_clamped = gain.clamp(-max_gain, max_gain);
            self.gain_table.insert(v, gain_clamped);
        }
        // Recompute per-constraint part populations
        let ncon = self.graph.ncon as usize;
        let k = self.k as usize;
        self.part_pop = vec![vec![0i64; k]; ncon];
        for v in 0..n {
            let part = self.assignment[v] as usize;
            for c in 0..ncon {
                self.part_pop[c][part] += self.graph.vwgt[v * ncon + c] as i64;
            }
        }
        self.current_cut = self.cut(); // recompute once after restore
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

    fn grid_4x4() -> CsrGraph {
        let n = 16usize;
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..4usize {
            for j in 0..4usize {
                let mut nbrs = Vec::new();
                if i > 0 { nbrs.push((i-1)*4+j); }
                if i < 3 { nbrs.push((i+1)*4+j); }
                if j > 0 { nbrs.push(i*4+(j-1)); }
                if j < 3 { nbrs.push(i*4+(j+1)); }
                for &u in &nbrs { adjncy.push(u as u32); }
                xadj.push(adjncy.len() as u32);
            }
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    fn dumbbell_graph() -> CsrGraph {
        // Two K5 cliques connected by a single bridge edge
        // Vertices 0-4: left clique, vertices 5-9: right clique
        // Bridge: vertex 4 -- vertex 5
        let n = 10usize;
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for v in 0..n {
            let mut nbrs: Vec<usize> = Vec::new();
            let clique = if v < 5 { 0..5 } else { 5..10 };
            for u in clique { if u != v { nbrs.push(u); } }
            // bridge
            if v == 4 { nbrs.push(5); }
            if v == 5 { nbrs.push(4); }
            for &u in &nbrs { adjncy.push(u as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    fn compute_cut_for_test(g: &CsrGraph, assignment: &[u32]) -> u32 {
        let mut cut = 0u32;
        for v in 0..g.n() {
            for j in g.xadj[v] as usize..g.xadj[v+1] as usize {
                let u = g.adjncy[j] as usize;
                if assignment[v] != assignment[u] { cut += 1; }
            }
        }
        cut / 2
    }

    #[test]
    fn fm_state_cut_path3_split() {
        // path 0-1-2, partition [0,0,1]
        // cut edge: 1-2, so cut = 1
        let g = path_graph(3);
        let p = Partition { assignment: vec![0, 0, 1], k: 2, tpwgts: None };
        let state = FmState::new(&g, p);
        assert_eq!(state.cut(), 1);
    }

    #[test]
    fn fm_state_checkpoint_restore() {
        let g = path_graph(4);
        let p = Partition { assignment: vec![0, 0, 1, 1], k: 2, tpwgts: None };
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

    #[test]
    fn fm_does_not_increase_cut() {
        use crate::init::InitialPartitioner;
        use crate::init::random::RandomBisect;
        use crate::refine::Refiner;
        let g = grid_4x4();
        let p_init = RandomBisect.partition(&g, 2, 0);
        let cut_before = compute_cut_for_test(&g, &p_init.assignment);
        let fm = FiducciaMattheyses { niter: 10 };
        let p = fm.refine(&g, p_init);
        let cut_after = compute_cut_for_test(&g, &p.assignment);
        assert!(cut_after <= cut_before,
            "FM must not increase cut: before={cut_before} after={cut_after}");
    }

    #[test]
    fn fm_oracle_dumbbell_bisect() {
        // Dumbbell: two K5 joined by 1 edge — optimal bisection cut = 1
        use crate::init::InitialPartitioner;
        use crate::init::random::RandomBisect;
        use crate::refine::Refiner;
        let g = dumbbell_graph();
        let p_init = RandomBisect.partition(&g, 2, 42);
        let fm = FiducciaMattheyses { niter: 20 };
        let p = fm.refine(&g, p_init);
        let cut = compute_cut_for_test(&g, &p.assignment);
        assert_eq!(cut, 1, "dumbbell bisect optimal cut is 1, got {cut}");
    }

    #[test]
    fn fm_preserves_population_balance() {
        use crate::init::InitialPartitioner;
        use crate::init::random::RandomBisect;
        use crate::refine::Refiner;
        let g = grid_4x4();
        let total: i64 = g.vwgt.iter().map(|&w| w as i64).sum(); // = 16
        let target = total / 2; // = 8
        let eps = (total * 5 + 999) / 1000; // ceiling of 0.5% = 1
        let p_init = RandomBisect.partition(&g, 2, 99);
        let fm = FiducciaMattheyses { niter: 10 };
        let p = fm.refine(&g, p_init);
        for part in 0..2u32 {
            let pop: i64 = (0..g.n())
                .filter(|&v| p.assignment[v] == part)
                .map(|v| g.vwgt[v] as i64)
                .sum();
            assert!((pop - target).abs() <= eps,
                "part {part} pop {pop} violates balance (target {target} ± {eps})");
        }
    }

    #[test]
    fn fm_multi_constraint_balance() {
        // Grid 4x4 with ncon=2: constraint 0 = population, constraint 1 = VAP.
        // Both are uniform (weight 1) so the expected balance is identical for both.
        use crate::init::InitialPartitioner;
        use crate::init::random::RandomBisect;
        use crate::refine::Refiner;
        let mut g = grid_4x4();
        g.ncon = 2;
        // vwgt: vertex i has [pop=1, vap=1]; interleaved layout: [1, 1, 1, 1, ...]
        g.vwgt = vec![1i32; 32]; // 16 vertices × 2 constraints
        let p_init = RandomBisect.partition(&g, 2, 42);
        let fm = FiducciaMattheyses { niter: 10 };
        let p = fm.refine(&g, p_init);
        assert_eq!(p.assignment.len(), 16);
        // Both constraints should be balanced within ε = ceil(0.5% × 16) = 1
        let total = 16i64;
        let target = 8i64;
        let eps = (total * 5 + 999) / 1000; // = 1
        for part in 0..2u32 {
            let pop0: i64 = (0..16)
                .filter(|&v| p.assignment[v] == part)
                .map(|v| g.vwgt[v * 2] as i64)
                .sum();
            let pop1: i64 = (0..16)
                .filter(|&v| p.assignment[v] == part)
                .map(|v| g.vwgt[v * 2 + 1] as i64)
                .sum();
            assert!((pop0 - target).abs() <= eps,
                "part {part} constraint 0 pop {pop0} violates balance (target {target} ± {eps})");
            assert!((pop1 - target).abs() <= eps,
                "part {part} constraint 1 pop {pop1} violates balance (target {target} ± {eps})");
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::refine::Refiner;

    fn kani_path(n: usize) -> CsrGraph {
        let mut xadj = vec![0u32];
        let mut adjncy = Vec::new();
        for i in 0..n {
            if i > 0 { adjncy.push((i-1) as u32); }
            if i < n-1 { adjncy.push((i+1) as u32); }
            xadj.push(adjncy.len() as u32);
        }
        CsrGraph { xadj, adjncy, ncon: 1, vwgt: vec![1i32; n], adjwgt: None }
    }

    /// Proves: FiducciaMattheyses::refine() never panics or goes OOB
    /// for valid path graphs up to n=16, k=4.
    /// Also proves: output assignment has correct length and valid part IDs.
    #[kani::proof]
    #[kani::unwind(17)]
    fn verify_fm_no_oob() {
        let n: usize = kani::any_where(|&n: &usize| n >= 4 && n <= 16);
        let k: u32   = kani::any_where(|&k: &u32| k >= 2 && k <= 4);
        kani::assume(k as usize <= n);

        let g = kani_path(n);
        kani::assume(g.is_valid());

        // Simple initial partition: vertex i gets part i%k
        let p = Partition {
            assignment: (0..n).map(|i| (i % k as usize) as u32).collect(),
            k,
            tpwgts: None,
        };

        let fm = FiducciaMattheyses { niter: 2 };
        let result = fm.refine(&g, p);

        // Safety postconditions:
        assert!(result.assignment.len() == n);
        assert!(result.assignment.iter().all(|&a| a < k));
    }
}
