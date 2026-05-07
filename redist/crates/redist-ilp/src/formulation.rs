//! ILP formulation for minimum-edge-cut redistricting.
//!
//! Generates a human-readable formulation summary (Phase 1).
//! Full MPS output is deferred to Phase 2 (requires solver integration).
//!
//! Formulation variables:
//!   x[t][d] in {0,1}   — tract t assigned to district d
//!   z[t][t'] in {0,1}  — undirected edge (t,t') with t < t' is cut
//!   f[t][t'][d] >= 0   — MTZ flow on directed arc (t->t') for district d
//!
//! Objective (CORRECT — one z per undirected edge, no double-counting):
//!   minimize: sum_{(t,t') : t < t'} z[t][t']
//!
//! Constraints:
//!   1. Coverage:       sum_d x[t][d] = 1                       for all t
//!   2. Pop balance:    ideal*(1-eps) <= sum_t pop[t]*x[t][d]
//!                                    <= ideal*(1+eps)           for all d
//!   3. Cut linearize:  z[t][t'] >= x[t][d] - x[t'][d]         for all edges, districts
//!                      z[t][t'] >= x[t'][d] - x[t][d]         for all edges, districts
//!   4. MTZ contiguity (both-endpoint flow bound):
//!      f[t][t'][d] <= (n-1) * min(x[t][d], x[t'][d])
//!      flow conservation at non-root tracts
//!      flow at root supplies (n_d - 1) units

/// Counts and summary of an ILP formulation (Phase 1: no actual MPS text).
#[derive(Debug, Clone)]
pub struct IlpFormulation {
    /// Number of tracts (nodes in the adjacency graph).
    pub n_tracts: usize,
    /// Number of districts to partition into.
    pub k: usize,
    /// Number of binary x variables: n_tracts * k.
    pub n_binary_x: usize,
    /// Number of binary z variables: |E| undirected edges (t < t').
    pub n_binary_z: usize,
    /// Number of continuous flow variables: 2*|E|*k (directed arcs per district).
    pub n_flow_vars: usize,
    /// Total constraint count: coverage + balance + cut-linearization + MTZ.
    pub n_constraints: usize,
    /// Number of undirected edges in the graph.
    pub n_edges: usize,
    /// Human-readable summary of the formulation structure.
    pub formulation_summary: String,
}

impl IlpFormulation {
    /// Total variable count: binary x + binary z + continuous flow.
    pub fn n_variables(&self) -> usize {
        self.n_binary_x + self.n_binary_z + self.n_flow_vars
    }
}

/// Build the ILP formulation for minimum-edge-cut redistricting.
///
/// `adjacency[t]` is the list of neighbours of tract `t` (undirected graph).
/// `pop[t]` is the population of tract `t`.
/// `k` is the number of districts.
/// `pop_tolerance` is the fractional population tolerance (e.g. 0.005 = 0.5%).
///
/// Returns the formulation variable counts and a human-readable summary.
/// Phase 1: does not produce a valid MPS file; that is deferred to Phase 2.
pub fn build_formulation(
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    pop_tolerance: f64,
) -> IlpFormulation {
    let n = adjacency.len();

    // Count undirected edges (t < t' only).
    let mut n_edges: usize = 0;
    for t in 0..n {
        for &t2 in &adjacency[t] {
            if t < t2 {
                n_edges += 1;
            }
        }
    }

    // Variable counts.
    let n_binary_x = n * k;
    let n_binary_z = n_edges;
    // Directed arcs: 2 * n_edges (each undirected edge becomes 2 directed arcs).
    let n_flow_vars = 2 * n_edges * k;

    // Constraint counts.
    // 1. Coverage: n constraints (one per tract).
    let n_coverage = n;
    // 2. Population balance: 2*k constraints (lower + upper bound per district).
    let n_balance = 2 * k;
    // 3. Cut linearization: 2 * n_edges * k constraints.
    //    For each undirected edge and each district: 2 inequalities.
    let n_cut = 2 * n_edges * k;
    // 4. MTZ flow conservation: 2*n_edges*k bound constraints
    //    + (n-1)*k non-root conservation
    //    + k root supply equations.
    let n_mtz_bounds = 2 * n_edges * k;
    let n_mtz_conservation = (n.saturating_sub(1)) * k;
    let n_mtz_root = k;
    let n_mtz = n_mtz_bounds + n_mtz_conservation + n_mtz_root;

    let n_constraints = n_coverage + n_balance + n_cut + n_mtz;

    // Compute ideal population for the summary.
    let total_pop: i64 = pop.iter().sum();
    let ideal_pop = if k > 0 { total_pop / k as i64 } else { 0 };

    let summary = format!(
        "ILP Formulation Summary\n\
         ========================\n\
         Tracts (n):          {n}\n\
         Districts (k):       {k}\n\
         Edges (|E|):         {n_edges}\n\
         \n\
         Variables\n\
         ---------\n\
         x[t][d] binary:      {n_binary_x}  (n*k = {n}*{k})\n\
         z[t][t'] binary:     {n_binary_z}  (|E|, one per undirected edge t<t')\n\
         f[t][t'][d] cont.:   {n_flow_vars} (2*|E|*k = 2*{n_edges}*{k})\n\
         Total variables:     {total_vars}\n\
         \n\
         Constraints\n\
         -----------\n\
         Coverage (sum_d x[t][d]=1):          {n_coverage}\n\
         Pop balance (lb+ub per district):    {n_balance}\n\
         Cut linearization (2*|E|*k):         {n_cut}\n\
         MTZ flow bounds (2*|E|*k):           {n_mtz_bounds}\n\
         MTZ conservation ((n-1)*k):          {n_mtz_conservation}\n\
         MTZ root supply (k):                 {n_mtz_root}\n\
         Total constraints:                   {n_constraints}\n\
         \n\
         Parameters\n\
         ----------\n\
         Total population:    {total_pop}\n\
         Ideal district pop:  {ideal_pop}\n\
         Pop tolerance:       {pop_tolerance:.4} ({tol_pct:.2}%)\n\
         \n\
         Objective\n\
         ---------\n\
         minimize sum_{{(t,t'): t<t'}} z[t][t']\n\
         (minimum edge cut — each cut edge counted exactly once)\n\
         \n\
         Contiguity\n\
         ----------\n\
         MTZ flow formulation with BOTH-ENDPOINT bound:\n\
           f[t][t'][d] <= (n-1) * min(x[t][d], x[t'][d])\n\
         Root per district: tract 0 (if in district) or first assigned tract.\n\
         Flow conservation enforces each district forms a connected subtree.\n",
        n = n,
        k = k,
        n_edges = n_edges,
        n_binary_x = n_binary_x,
        n_binary_z = n_binary_z,
        n_flow_vars = n_flow_vars,
        total_vars = n_binary_x + n_binary_z + n_flow_vars,
        n_coverage = n_coverage,
        n_balance = n_balance,
        n_cut = n_cut,
        n_mtz_bounds = n_mtz_bounds,
        n_mtz_conservation = n_mtz_conservation,
        n_mtz_root = n_mtz_root,
        n_constraints = n_constraints,
        total_pop = total_pop,
        ideal_pop = ideal_pop,
        pop_tolerance = pop_tolerance,
        tol_pct = pop_tolerance * 100.0,
    );

    IlpFormulation {
        n_tracts: n,
        k,
        n_binary_x,
        n_binary_z,
        n_flow_vars,
        n_constraints,
        n_edges,
        formulation_summary: summary,
    }
}

// ── L0 inline unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Path graph: 0-1-2-3 (3 undirected edges), k=2.
    /// Expected: n_binary_x=8, n_binary_z=3, n_flow_vars=6*2=12.
    fn path_4_adjacency() -> Vec<Vec<usize>> {
        vec![
            vec![1],          // 0 -- 1
            vec![0, 2],       // 1 -- 0, 2
            vec![1, 3],       // 2 -- 1, 3
            vec![2],          // 3 -- 2
        ]
    }

    fn uniform_pop(n: usize, pop_each: i64) -> Vec<i64> {
        vec![pop_each; n]
    }

    #[test]
    fn formulation_4_node_path_k2_variable_counts() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);

        assert_eq!(f.n_binary_x, 8, "n_binary_x should be n*k = 4*2 = 8");
        assert_eq!(f.n_binary_z, 3, "n_binary_z should equal |E| = 3 (path 0-1-2-3)");
        // 2 * |E| * k = 2 * 3 * 2 = 12
        assert_eq!(f.n_flow_vars, 12, "n_flow_vars should be 2*|E|*k = 2*3*2 = 12");
    }

    #[test]
    fn formulation_covers_all_tracts() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        // Coverage constraints: one per tract.
        let n_coverage = 4; // n = 4
        // The coverage constraints are embedded in n_constraints; verify via formula:
        // n_coverage = n, n_balance = 2*k, n_cut = 2*|E|*k, n_mtz = 2*|E|*k + (n-1)*k + k
        // Total = 4 + 4 + 12 + 12 + 3 + 2 = 37
        assert!(
            f.n_constraints >= n_coverage,
            "n_constraints ({}) must be >= n_coverage ({})",
            f.n_constraints,
            n_coverage
        );
        // Verify the coverage part explicitly equals n_tracts.
        assert_eq!(f.n_tracts, 4);
    }

    #[test]
    fn formulation_n_constraints_positive() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        assert!(f.n_constraints > 0, "formulation must have at least one constraint");
    }

    #[test]
    fn formulation_4_node_path_k2_constraint_total() {
        // Verify the constraint count arithmetic for the 4-node path, k=2.
        // n=4, |E|=3, k=2:
        //   coverage      = 4
        //   balance       = 2*2 = 4
        //   cut           = 2*3*2 = 12
        //   mtz_bounds    = 2*3*2 = 12
        //   mtz_conserv   = (4-1)*2 = 6
        //   mtz_root      = 2
        //   total         = 4+4+12+12+6+2 = 40
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        assert_eq!(f.n_constraints, 40, "expected 40 constraints for 4-node path k=2");
    }

    #[test]
    fn formulation_summary_is_nonempty() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        assert!(!f.formulation_summary.is_empty());
        assert!(f.formulation_summary.contains("ILP Formulation Summary"));
    }

    #[test]
    fn formulation_single_node_single_district() {
        // Trivial: 1 tract, 1 district. 0 edges.
        let adj = vec![vec![]];
        let pop = vec![1000i64];
        let f = build_formulation(&adj, &pop, 1, 0.0);
        assert_eq!(f.n_binary_x, 1);
        assert_eq!(f.n_binary_z, 0);
        assert_eq!(f.n_flow_vars, 0);
        assert_eq!(f.n_edges, 0);
        // coverage=1, balance=2, cut=0, mtz_bounds=0, mtz_conserv=0, mtz_root=1 => 4
        assert_eq!(f.n_constraints, 4);
    }

    #[test]
    fn formulation_n_variables_matches_sum() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        assert_eq!(
            f.n_variables(),
            f.n_binary_x + f.n_binary_z + f.n_flow_vars
        );
    }
}
