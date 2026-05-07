//! ILP solver dispatch.
//!
//! Phase 1: only `FormulationOnly` is implemented. Subprocess solvers
//! (GLPK, HiGHS) are stubbed and return `SubprocessNotImplemented`.
//!
//! Phase 2 will implement MPS file generation + subprocess invocation.

use crate::formulation::{IlpFormulation, build_formulation};
use crate::result::{IlpResult, SolverStatus};

/// Which solver backend to use.
#[derive(Debug, Clone)]
pub enum IlpSolver {
    /// No solver — only generate the formulation summary. Always available.
    FormulationOnly,
    /// Call `glpsol` subprocess if in PATH.
    Glpk { time_limit_secs: u64 },
    /// Call `highs` subprocess if in PATH.
    Highs { time_limit_secs: u64 },
}

/// Solve (or summarise) an ILP redistricting instance.
///
/// `adjacency[t]` is the neighbour list of tract `t`.
/// `pop[t]` is the population of tract `t`.
/// `k` is the number of districts.
/// `pop_tolerance` is the fractional population tolerance (e.g. 0.005).
/// `solver` selects the backend.
/// `optimality_gap` is the acceptable gap from optimal (e.g. 0.01 = 1%).
///
/// Phase 1: only `FormulationOnly` produces a meaningful result.
/// Subprocess solvers return `SubprocessNotImplemented`.
pub fn solve(
    formulation: &IlpFormulation,
    _adjacency: &[Vec<usize>],
    _pop: &[i64],
    _k: usize,
    _pop_tolerance: f64,
    solver: IlpSolver,
    _optimality_gap: f64,
) -> IlpResult {
    match solver {
        IlpSolver::FormulationOnly => IlpResult {
            status: SolverStatus::FormulationOnly,
            plan: None,
            optimal_ec: None,
            solve_time_secs: 0.0,
            n_variables: formulation.n_variables(),
            n_constraints: formulation.n_constraints,
            solver_used: "formulation_only".to_string(),
            solver_version: String::new(),
        },
        IlpSolver::Glpk { .. } => IlpResult {
            status: SolverStatus::SubprocessNotImplemented,
            plan: None,
            optimal_ec: None,
            solve_time_secs: 0.0,
            n_variables: formulation.n_variables(),
            n_constraints: formulation.n_constraints,
            solver_used: "glpk".to_string(),
            solver_version: String::new(),
        },
        IlpSolver::Highs { .. } => IlpResult {
            status: SolverStatus::SubprocessNotImplemented,
            plan: None,
            optimal_ec: None,
            solve_time_secs: 0.0,
            n_variables: formulation.n_variables(),
            n_constraints: formulation.n_constraints,
            solver_used: "highs".to_string(),
            solver_version: String::new(),
        },
    }
}

// ── Convenience wrapper: build formulation then solve ─────────────────────────

/// Build the formulation from raw adjacency/population data and immediately
/// dispatch to the chosen solver.  Equivalent to calling `build_formulation`
/// followed by `solve`.
pub fn build_and_solve(
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    pop_tolerance: f64,
    solver: IlpSolver,
    optimality_gap: f64,
) -> (IlpFormulation, IlpResult) {
    let formulation = build_formulation(adjacency, pop, k, pop_tolerance);
    let result = solve(
        &formulation,
        adjacency,
        pop,
        k,
        pop_tolerance,
        solver,
        optimality_gap,
    );
    (formulation, result)
}

// ── L0 inline unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::SolverStatus;

    fn path_4_adjacency() -> Vec<Vec<usize>> {
        vec![
            vec![1],
            vec![0, 2],
            vec![1, 3],
            vec![2],
        ]
    }

    fn uniform_pop(n: usize, pop_each: i64) -> Vec<i64> {
        vec![pop_each; n]
    }

    #[test]
    fn solve_formulation_only_returns_no_plan() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(&f, &adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
        assert!(
            result.plan.is_none(),
            "FormulationOnly solver must return plan = None"
        );
    }

    #[test]
    fn solve_formulation_only_status_correct() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(&f, &adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
        assert_eq!(
            result.status,
            SolverStatus::FormulationOnly,
            "FormulationOnly solver must return status FormulationOnly"
        );
    }

    #[test]
    fn solve_formulation_only_propagates_variable_count() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let expected_vars = f.n_variables();
        let result = solve(&f, &adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
        assert_eq!(result.n_variables, expected_vars);
    }

    #[test]
    fn solve_glpk_stub_returns_not_implemented() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f, &adj, &pop, 2, 0.005,
            IlpSolver::Glpk { time_limit_secs: 60 },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::SubprocessNotImplemented);
        assert_eq!(result.solver_used, "glpk");
    }

    #[test]
    fn solve_highs_stub_returns_not_implemented() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f, &adj, &pop, 2, 0.005,
            IlpSolver::Highs { time_limit_secs: 300 },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::SubprocessNotImplemented);
        assert_eq!(result.solver_used, "highs");
    }

    #[test]
    fn build_and_solve_formulation_only_roundtrip() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let (form, result) = build_and_solve(&adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
        // Formulation variable counts match result.
        assert_eq!(result.n_variables, form.n_variables());
        assert_eq!(result.n_constraints, form.n_constraints);
        assert_eq!(result.status, SolverStatus::FormulationOnly);
        assert!(result.plan.is_none());
    }

    #[test]
    fn solver_used_field_formulation_only() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(&f, &adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
        assert_eq!(result.solver_used, "formulation_only");
    }
}
