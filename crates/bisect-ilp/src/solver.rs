//! ILP solver dispatch.
//!
//! `FormulationOnly` is always available. U.16 branch-and-cut modes solve
//! bounded k=2 instances exactly; subprocess solvers (GLPK, HiGHS) are
//! stubbed and return `SubprocessNotImplemented`.
//!
//! Phase 2 will implement MPS file generation + subprocess invocation.

use crate::certificates::{branch_and_cut_certificate, BranchAndCutMode, ExactSearchStats};
use crate::formulation::{build_formulation, IlpFormulation};
use crate::result::{IlpResult, SolverStatus};
use std::collections::HashMap;
use std::time::Instant;

const EXACT_BRANCH_AND_BOUND_LIMIT: usize = 24;
const UNASSIGNED_DISTRICT: usize = usize::MAX;

/// Which solver backend to use.
#[derive(Debug, Clone)]
pub enum IlpSolver {
    /// No solver — only generate the formulation summary. Always available.
    FormulationOnly,
    /// Call `glpsol` subprocess if in PATH.
    Glpk { time_limit_secs: u64 },
    /// Call `highs` subprocess if in PATH.
    Highs { time_limit_secs: u64 },
    /// U.16 branch-and-cut mode. Phase 1 records separation metadata only.
    BranchAndCut {
        mode: BranchAndCutMode,
        incumbent_assignment: Option<Vec<usize>>,
        solver_name: Option<String>,
    },
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
/// Branch-and-cut modes solve bounded k=2 instances exactly. Subprocess
/// solvers return `SubprocessNotImplemented`.
pub fn solve(
    formulation: &IlpFormulation,
    adjacency: &[Vec<usize>],
    _pop: &[i64],
    k: usize,
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
            branch_and_cut: None,
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
            branch_and_cut: None,
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
            branch_and_cut: None,
        },
        IlpSolver::BranchAndCut {
            mode,
            incumbent_assignment,
            solver_name,
        } => {
            if k == 2 && adjacency.len() <= EXACT_BRANCH_AND_BOUND_LIMIT {
                return solve_k2_exact_branch_and_bound(
                    formulation,
                    adjacency,
                    _pop,
                    _pop_tolerance,
                    mode,
                    solver_name,
                );
            }

            let branch_and_cut = incumbent_assignment.map(|assignment| {
                let mut certificate = branch_and_cut_certificate(adjacency, &assignment, k, mode);
                certificate.solver_name = solver_name.clone();
                certificate
            });
            IlpResult {
                status: SolverStatus::BranchAndCutNotImplemented,
                plan: None,
                optimal_ec: None,
                solve_time_secs: 0.0,
                n_variables: formulation.n_variables(),
                n_constraints: formulation.n_constraints,
                solver_used: solver_name.unwrap_or_else(|| "branch_and_cut".to_string()),
                solver_version: String::new(),
                branch_and_cut,
            }
        }
    }
}

fn solve_k2_exact_branch_and_bound(
    formulation: &IlpFormulation,
    adjacency: &[Vec<usize>],
    pop: &[i64],
    pop_tolerance: f64,
    mode: BranchAndCutMode,
    solver_name: Option<String>,
) -> IlpResult {
    let started = Instant::now();
    let n = adjacency.len();
    if n == 0 || pop.len() != n {
        return branch_and_cut_result(
            SolverStatus::Infeasible,
            None,
            None,
            started.elapsed().as_secs_f64(),
            formulation,
            solver_name.unwrap_or_else(|| "branch_and_cut_exact_branch_and_bound".to_string()),
            None,
        );
    }

    let total_pop: i64 = pop.iter().sum();
    let ideal = total_pop as f64 / 2.0;
    let lower = ideal * (1.0 - pop_tolerance);
    let upper = ideal * (1.0 + pop_tolerance);

    let search = ExactK2Search::new(adjacency, pop, lower, upper, total_pop);

    let Some(assignment) = search.best_assignment else {
        return branch_and_cut_result(
            SolverStatus::Infeasible,
            None,
            None,
            started.elapsed().as_secs_f64(),
            formulation,
            solver_name.unwrap_or_else(|| "branch_and_cut_exact_branch_and_bound".to_string()),
            None,
        );
    };
    let best_cut = search.best_cut;

    let mut certificate = branch_and_cut_certificate(adjacency, &assignment, 2, mode);
    certificate.solver_name = solver_name.clone();
    certificate.lower_bound = best_cut.map(|cut| cut as f64);
    certificate.incumbent_objective = best_cut.map(|cut| cut as f64);
    certificate.optimality_gap = Some(0.0);
    certificate.exact_search = Some(search.stats);
    certificate
        .solver_mode_note
        .push_str("; solved exactly by bounded branch-and-bound for k=2");

    branch_and_cut_result(
        SolverStatus::Optimal,
        Some(
            assignment
                .iter()
                .enumerate()
                .map(|(idx, &district)| (idx, district))
                .collect(),
        ),
        best_cut,
        started.elapsed().as_secs_f64(),
        formulation,
        solver_name.unwrap_or_else(|| "branch_and_cut_exact_branch_and_bound".to_string()),
        Some(certificate),
    )
}

fn branch_and_cut_result(
    status: SolverStatus,
    plan: Option<HashMap<usize, usize>>,
    optimal_ec: Option<usize>,
    solve_time_secs: f64,
    formulation: &IlpFormulation,
    solver_used: String,
    branch_and_cut: Option<crate::certificates::BranchAndCutCertificate>,
) -> IlpResult {
    IlpResult {
        status,
        plan,
        optimal_ec,
        solve_time_secs,
        n_variables: formulation.n_variables(),
        n_constraints: formulation.n_constraints,
        solver_used,
        solver_version: String::new(),
        branch_and_cut,
    }
}

struct ExactK2Search<'a> {
    adjacency: &'a [Vec<usize>],
    pop: &'a [i64],
    lower: f64,
    upper: f64,
    assignment: Vec<usize>,
    best_assignment: Option<Vec<usize>>,
    best_cut: Option<usize>,
    stats: ExactSearchStats,
}

impl<'a> ExactK2Search<'a> {
    fn new(
        adjacency: &'a [Vec<usize>],
        pop: &'a [i64],
        lower: f64,
        upper: f64,
        total_pop: i64,
    ) -> Self {
        let mut assignment = vec![UNASSIGNED_DISTRICT; adjacency.len()];
        assignment[0] = 0;
        let mut search = Self {
            adjacency,
            pop,
            lower,
            upper,
            assignment,
            best_assignment: None,
            best_cut: None,
            stats: ExactSearchStats {
                search_strategy: "k2-branch-and-bound".to_string(),
                search_limit_vertices: EXACT_BRANCH_AND_BOUND_LIMIT,
                nodes_visited: 0,
                pruned_by_bound: 0,
                pruned_by_population: 0,
                complete_assignments_checked: 0,
                connected_feasible_assignments: 0,
                incumbent_updates: 0,
            },
        };
        search.run(total_pop);
        search
    }

    fn run(&mut self, total_pop: i64) {
        let remaining = total_pop - self.pop[0];
        self.dfs(1, self.pop[0], 0, remaining, 0);
    }

    fn dfs(
        &mut self,
        vertex: usize,
        left_pop: i64,
        right_pop: i64,
        remaining_pop: i64,
        cut_so_far: usize,
    ) {
        self.stats.nodes_visited += 1;
        if self.best_cut.is_some_and(|best| cut_so_far >= best) {
            self.stats.pruned_by_bound += 1;
            return;
        }
        if !self.population_can_still_balance(left_pop, right_pop, remaining_pop) {
            self.stats.pruned_by_population += 1;
            return;
        }
        if vertex == self.adjacency.len() {
            self.stats.complete_assignments_checked += 1;
            if self.is_complete_candidate_connected() {
                self.stats.connected_feasible_assignments += 1;
                self.stats.incumbent_updates += 1;
                self.best_cut = Some(cut_so_far);
                self.best_assignment = Some(self.assignment.clone());
            }
            return;
        }

        let next_remaining = remaining_pop - self.pop[vertex];
        for district in [0usize, 1usize] {
            self.assignment[vertex] = district;
            let added_cut = self.assigned_neighbor_cut_increment(vertex, district);
            let (next_left, next_right) = if district == 0 {
                (left_pop + self.pop[vertex], right_pop)
            } else {
                (left_pop, right_pop + self.pop[vertex])
            };
            self.dfs(
                vertex + 1,
                next_left,
                next_right,
                next_remaining,
                cut_so_far + added_cut,
            );
            self.assignment[vertex] = UNASSIGNED_DISTRICT;
        }
    }

    fn population_can_still_balance(
        &self,
        left_pop: i64,
        right_pop: i64,
        remaining_pop: i64,
    ) -> bool {
        let left = left_pop as f64;
        let right = right_pop as f64;
        let remaining = remaining_pop as f64;
        left <= self.upper
            && right <= self.upper
            && left + remaining >= self.lower
            && right + remaining >= self.lower
    }

    fn assigned_neighbor_cut_increment(&self, vertex: usize, district: usize) -> usize {
        self.adjacency[vertex]
            .iter()
            .filter(|&&neighbor| {
                let neighbor_district = self.assignment[neighbor];
                neighbor_district != UNASSIGNED_DISTRICT && neighbor_district != district
            })
            .count()
    }

    fn is_complete_candidate_connected(&self) -> bool {
        district_connected(self.adjacency, &self.assignment, 0)
            && district_connected(self.adjacency, &self.assignment, 1)
    }
}

fn district_connected(adjacency: &[Vec<usize>], assignment: &[usize], district: usize) -> bool {
    let Some(start) = assignment
        .iter()
        .enumerate()
        .find_map(|(idx, &assigned)| (assigned == district).then_some(idx))
    else {
        return false;
    };
    let member_count = assignment
        .iter()
        .filter(|&&assigned| assigned == district)
        .count();
    let mut seen = vec![false; adjacency.len()];
    let mut stack = vec![start];
    seen[start] = true;
    let mut reached = 0usize;
    while let Some(node) = stack.pop() {
        reached += 1;
        for &neighbor in &adjacency[node] {
            if assignment[neighbor] == district && !seen[neighbor] {
                seen[neighbor] = true;
                stack.push(neighbor);
            }
        }
    }
    reached == member_count
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
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2]]
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
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::Glpk {
                time_limit_secs: 60,
            },
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
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::Highs {
                time_limit_secs: 300,
            },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::SubprocessNotImplemented);
        assert_eq!(result.solver_used, "highs");
    }

    #[test]
    fn build_and_solve_formulation_only_roundtrip() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let (form, result) =
            build_and_solve(&adj, &pop, 2, 0.005, IlpSolver::FormulationOnly, 0.01);
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

    #[test]
    fn solve_branch_and_cut_records_iterative_certificate() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::BranchAndCut {
                mode: BranchAndCutMode::IterativeSeparation,
                incumbent_assignment: Some(vec![0, 1, 0, 1, 1]),
                solver_name: Some("highs".to_string()),
            },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::Optimal);
        assert_eq!(result.solver_used, "highs");
        assert_eq!(result.optimal_ec, Some(1));
        assert!(result.plan.is_some());
        let cert = result.branch_and_cut.unwrap();
        assert_eq!(cert.cut_count, 0);
        assert_eq!(cert.incumbent_objective, Some(1.0));
        assert_eq!(cert.optimality_gap, Some(0.0));
        let stats = cert.exact_search.unwrap();
        assert_eq!(stats.search_strategy, "k2-branch-and-bound");
        assert!(stats.nodes_visited > 0);
        assert!(stats.complete_assignments_checked > 0);
        assert!(stats.connected_feasible_assignments > 0);
        assert!(cert.solver_mode_note.contains("iterative"));
    }

    #[test]
    fn branch_and_cut_result_json_round_trips() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::BranchAndCut {
                mode: BranchAndCutMode::IterativeSeparation,
                incumbent_assignment: Some(vec![0, 1, 0, 1, 1]),
                solver_name: Some("highs".to_string()),
            },
            0.01,
        );
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""status":"optimal""#));
        assert!(json.contains(r#""branch_and_cut""#));
        let decoded: IlpResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, result);
    }

    #[test]
    fn solve_branch_and_cut_without_incumbent_solves_small_k2_instance() {
        let adj = path_4_adjacency();
        let pop = uniform_pop(4, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::BranchAndCut {
                mode: BranchAndCutMode::LazyCallback,
                incumbent_assignment: None,
                solver_name: None,
            },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::Optimal);
        assert_eq!(result.optimal_ec, Some(1));
        let cert = result.branch_and_cut.unwrap();
        assert!(cert.exact_search.unwrap().search_limit_vertices >= 4);
    }

    #[test]
    fn solve_branch_and_cut_over_exact_limit_keeps_stub_status() {
        let n = EXACT_BRANCH_AND_BOUND_LIMIT + 1;
        let mut adj = vec![Vec::new(); n];
        for idx in 0..(n - 1) {
            adj[idx].push(idx + 1);
            adj[idx + 1].push(idx);
        }
        let pop = uniform_pop(n, 100);
        let f = build_formulation(&adj, &pop, 2, 0.005);
        let result = solve(
            &f,
            &adj,
            &pop,
            2,
            0.005,
            IlpSolver::BranchAndCut {
                mode: BranchAndCutMode::LazyCallback,
                incumbent_assignment: None,
                solver_name: None,
            },
            0.01,
        );
        assert_eq!(result.status, SolverStatus::BranchAndCutNotImplemented);
        assert!(result.branch_and_cut.is_none());
    }
}
