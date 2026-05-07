//! ILP solver result types.
//!
//! `IlpResult` carries the outcome of a solve attempt, including the
//! assignment plan (if solved), solver status, and audit metadata.

use std::collections::HashMap;

/// Status of the ILP solve attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum SolverStatus {
    /// Provably optimal solution found. No valid plan has lower edge cuts.
    Optimal,
    /// Solve timed out but a feasible incumbent was found.
    /// `achieved_gap` in `IlpResult` records the remaining bound gap.
    TimeoutWithSolution,
    /// Solution is within the requested optimality gap (not proven exactly optimal).
    GapReached { achieved_gap: f64 },
    /// No valid plan exists under the given population balance constraints.
    Infeasible,
    /// Subgraph exceeded `max_tracts`; METIS was used instead.
    FallbackMetis,
    /// Phase 1 only: formulation generated and summarised, not solved.
    FormulationOnly,
    /// Subprocess solver not yet implemented (Phase 1 stub for GLPK/HiGHS).
    SubprocessNotImplemented,
}

/// Result of an ILP redistricting solve.
#[derive(Debug, Clone)]
pub struct IlpResult {
    /// Solver status.
    pub status: SolverStatus,
    /// Tract-to-district assignment (0-indexed). `None` if no solution was found.
    pub plan: Option<HashMap<usize, usize>>,
    /// Edge-cut count of the optimal/incumbent solution. `None` if not solved.
    pub optimal_ec: Option<usize>,
    /// Wall-clock solve time in seconds (0.0 if not attempted).
    pub solve_time_secs: f64,
    /// Total variable count from the formulation.
    pub n_variables: usize,
    /// Total constraint count from the formulation.
    pub n_constraints: usize,
    /// Identifier of the solver used (e.g. "glpk", "highs", "formulation_only").
    pub solver_used: String,
    /// Solver version string (empty if not available).
    pub solver_version: String,
}

// ── L0 inline unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(status: SolverStatus, plan: Option<HashMap<usize, usize>>) -> IlpResult {
        IlpResult {
            status,
            plan,
            optimal_ec: None,
            solve_time_secs: 0.0,
            n_variables: 0,
            n_constraints: 0,
            solver_used: String::new(),
            solver_version: String::new(),
        }
    }

    #[test]
    fn ilp_result_debug_derives() {
        // Verify that Debug is derived and produces non-empty output.
        let r = make_result(SolverStatus::FormulationOnly, None);
        let s = format!("{:?}", r);
        assert!(!s.is_empty());
        assert!(s.contains("FormulationOnly"));
    }

    #[test]
    fn solver_status_clone_and_eq() {
        let s1 = SolverStatus::Optimal;
        let s2 = s1.clone();
        assert_eq!(s1, s2);
    }

    #[test]
    fn solver_status_gap_reached_carries_value() {
        let s = SolverStatus::GapReached { achieved_gap: 0.005 };
        if let SolverStatus::GapReached { achieved_gap } = s {
            assert!((achieved_gap - 0.005).abs() < 1e-10);
        } else {
            panic!("expected GapReached");
        }
    }

    #[test]
    fn ilp_result_with_plan_is_some() {
        let mut plan = HashMap::new();
        plan.insert(0usize, 0usize);
        plan.insert(1, 1);
        let r = make_result(SolverStatus::Optimal, Some(plan.clone()));
        assert!(r.plan.is_some());
        assert_eq!(r.plan.unwrap().len(), 2);
    }

    #[test]
    fn ilp_result_no_plan_is_none() {
        let r = make_result(SolverStatus::Infeasible, None);
        assert!(r.plan.is_none());
    }

    #[test]
    fn all_solver_status_variants_debug() {
        // Smoke test: all variants can be formatted.
        let variants: Vec<SolverStatus> = vec![
            SolverStatus::Optimal,
            SolverStatus::TimeoutWithSolution,
            SolverStatus::GapReached { achieved_gap: 0.01 },
            SolverStatus::Infeasible,
            SolverStatus::FallbackMetis,
            SolverStatus::FormulationOnly,
            SolverStatus::SubprocessNotImplemented,
        ];
        for v in variants {
            let s = format!("{:?}", v);
            assert!(!s.is_empty());
        }
    }
}
