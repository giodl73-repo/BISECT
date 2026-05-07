# Spec: ILP Redistricting — Exact Optimization via Integer Linear Programming

**Status**: Proposed (R1 reviewed, P0+P1 fixes applied)  
**Date**: 2026-05-07  
**Layer**: Structure (SplitStrategy) — alternative to METIS for small instances  
**Reviewed R1**: MERIDIAN 2/4, BENCHMARK 3/4, SURVEY 2/4, COVENANT 3/4 → avg 2.5/4  
**Related paper**: B.24  
**New crate**: `redist-ilp` (Rust ILP formulation; output MPS format for external solver)

---

## Overview

All current structure-layer algorithms (METIS, SA, CVD, BFS Growth) are heuristic or approximate. ILP redistricting finds the **provably optimal** edge-cut plan — no valid plan has lower edge cuts. This is the gold standard for compactness certification: "the submitted plan is optimal among all valid plans of this type."

Tractable only for small instances (n ≤ 500 tracts). Primary use cases:

1. **Small states**: VT (k=1), ND (k=1), WY (k=1), AK (k=1), SD (k=1) — trivial; DE (k=1), ND (k=1)
2. **Certification of existing plans**: given a plan, prove it is within X% of optimal
3. **Bisection nodes**: each bisection node has a small subgraph (n/k tracts ≤ 500 for large k)

**Relationship to existing structure strategies**:

| Property | ILP (B.24) | METIS | BFS Growth (B.23) |
|---|---|---|---|
| Optimality | Exact (provably optimal) | Heuristic | None (greedy) |
| Contiguity guarantee | Yes (MTZ constraints) | No (post-processing) | Yes (BFS) |
| Tractability | n ≤ 500 | Any size | Any size |
| Runtime | Exponential worst-case | O(n log n) multilevel | O(n log n) |
| Compactness quality | Gold standard | High | Low baseline |
| Use case | Certification, small states | Production | Baseline |

---

## 1. ILP Formulation

**Binary decision variables**:
- $x_{t,d} \in \{0,1\}$: equals 1 iff tract $t$ is assigned to district $d$
- $z_{t,t'} \in \{0,1\}$: equals 1 iff undirected edge $\{t,t'\}$ is cut (endpoints in different districts)

**Objective** (minimize total edge cuts, counting each cut edge exactly once):

$$\min \sum_{\{t,t'\} \in E,\; t < t'} z_{t,t'}$$

**Constraints**:

1. **Coverage** — every tract assigned to exactly one district:

$$\sum_d x_{t,d} = 1 \quad \forall t$$

2. **Population balance** — each district within tolerance of ideal:

$$\left|\sum_t \text{pop}(t) \cdot x_{t,d} - \text{ideal\_pop}\right| \le \varepsilon \cdot \text{ideal\_pop} \quad \forall d$$

3. **Cut edge linearization** — $z_{t,t'}$ is forced to 1 whenever the two endpoints are in different districts:

$$z_{t,t'} \ge x_{t,d} - x_{t',d} \quad \forall \{t,t'\} \in E, \forall d$$
$$z_{t,t'} \ge x_{t',d} - x_{t,d} \quad \forall \{t,t'\} \in E, \forall d$$

Each cut edge is counted once in the objective (indexed $t < t'$). The previous $y_{t,t',d}$ auxiliary variables (one per district per edge) are replaced by the single $z_{t,t'}$ cut indicator; the original formulation over-counted each cut edge $k$ times.

4. **Contiguity** — Miller-Tucker-Zemlin (MTZ) flow-based constraints. For each district $d$, designate an arbitrary root $r_d$. Introduce flow variables $f_{t,t',d} \ge 0$ on directed edges:

$$f_{t,t',d} \ge 0 \quad \forall (t,t') \in E_{\text{directed}}, \forall d$$
$$f_{t,t',d} \le (n-1) \cdot \min(x_{t,d},\, x_{t',d}) \quad \forall (t,t') \in E_{\text{directed}}, \forall d$$
$$\sum_{t': (t,t') \in E} f_{t,t',d} - \sum_{t': (t',t) \in E} f_{t',t,d} = x_{t,d} \quad \forall t \ne r_d, \forall d$$
$$\sum_{t: (r_d,t) \in E} f_{r_d,t,d} = \sum_t x_{t,d} - 1 \quad \forall d$$

The $\min(x_{t,d}, x_{t',d})$ upper bound ensures flow is zero whenever either endpoint is outside district $d$, preventing cross-district flow paths. The root $r_d$ supplies $n_d - 1$ units of flow (one per non-root tract in district $d$), enforcing that the district forms a connected subtree.

**Variable counts** (order of magnitude for reference):
- Binary $x$: $n \times k$
- Binary $z$: $|E|$ (one per undirected edge, shared across districts)
- Continuous $f$: $2|E| \times k$ (directed)
- Constraints: $O(n \times k + |E| \times k)$

For a typical small state (n=250 tracts, k=2): ~2 000 variables, ~7 000 constraints.

---

## 2. Solver interface

**Solver selection** (priority order):

1. `good_lp` with GLPK backend (pure Rust + C, no subprocess) — default for n ≤ 200
2. `HiGHS` subprocess (`highs` in PATH) — for n ≤ 500
3. Output MPS file only (user runs solver manually) — for n > 500

The solver interface is abstracted behind a `Solver` trait so that new backends (Gurobi, CPLEX) can be added without touching formulation code.

**Output format**: LP/MPS file written to `runs/{label}/{year}/ilp/{node_id}.mps` before solving. This file is retained in the audit directory for external verification regardless of solver outcome.

---

## 3. Architecture

**New crate**: `redist-ilp`

```
redist-ilp/
  src/
    lib.rs
    formulation.rs    // build ILP formulation as LP/MPS file
    solver.rs         // call external solver subprocess or good_lp
    result.rs         // IlpResult: optimal plan, objective value, gap, status
  Cargo.toml
```

**Minimal dependencies**:
- `good_lp` — LP/ILP solver abstraction (Rust; GLPK backend by default)
- `highs` (optional feature flag) — HiGHS subprocess integration

**`IlpResult`**:

```rust
pub struct IlpResult {
    pub assignment: Vec<usize>,     // tract -> district index
    pub solver_status: SolverStatus,
    pub optimal_ec: Option<u64>,    // edge cuts in solution (None if infeasible)
    pub achieved_gap: f64,          // 0.0 = proven optimal
    pub solve_time_secs: f64,
    pub n_variables: usize,
    pub n_constraints: usize,
    pub solver_used: String,        // "glpk" | "highs" | "fallback_metis"
}

pub enum SolverStatus {
    Optimal,       // provably optimal solution found
    GapReached,    // solution within optimality_gap of optimal (not proven exact)
    Timeout,       // time_limit_secs reached, best incumbent returned
    Infeasible,    // no valid plan exists under given constraints
    FallbackMetis, // n > max_tracts; METIS used instead
}
```

---

## 4. Compositor integration

```rust
SplitStrategy::Ilp {
    time_limit_secs: u64,   // solver time limit (default: 300)
    optimality_gap: f64,    // acceptable gap from optimal (default: 0.01 = 1%)
    max_tracts: usize,      // size guard — fallback to METIS if exceeded (default: 500)
}
```

**Calibration note**: the 300s default is calibrated for n ≤ 200 tract instances with `good_lp`/GLPK. For n = 300–500 (NH-sized instances with k=2), branch-and-bound with MTZ contiguity constraints may require 600–900s on commodity hardware. Increase `--ilp-time-limit` to 900 for NH-sized instances. The size-guard fallback at n > 500 prevents timeouts for larger states.

**Size guard**: if the subgraph at a bisection node exceeds `max_tracts` tracts, fall back to METIS:

```
WARNING: ILP solver skipped for node at depth 2 (847 tracts > 500 limit). Falling back to METIS.
         Increase --ilp-max-tracts or use --structure metis to suppress this warning.
```

The fallback is recorded in the audit JSON as `solver_status: "fallback_metis"`.

---

## 5. CLI

```bash
--structure ilp --ilp-time-limit 300 --ilp-gap 0.01 --ilp-max-tracts 500
```

**YAML**:

```yaml
algorithm:
  structure: ilp
  weights: geographic
  search: single
  balance_tolerance: 0.5
  ilp:
    time_limit_secs: 300
    optimality_gap: 0.01
    max_tracts: 500
workers: 8
years: ["2020"]
```

---

## 6. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"structure": "ilp",
"time_limit_secs": 300,
"optimality_gap": 0.01,
"solve_time_secs": 47.3,
"solver_status": "optimal",
"optimal_ec": 847,
"achieved_gap": 0.0,
"n_variables": 2418,
"n_constraints": 8931,
"solver_used": "glpk",
"solver_version": "GLPK 5.0"
```

**Note on `solver_version`**: recorded because numerical tolerances and branching decisions differ across versions; two runs with different solver versions may produce different optimal solutions in the presence of ties. The `solver_version` field enables auditors to reproduce the exact solution rather than merely an equivalent-cost one.

**`solver_status` semantics**:
- `"optimal"` — gold standard. Certifies that no valid plan achieves lower edge cuts.
- `"gap_reached"` — solution is within `optimality_gap` of optimal, but not proven exactly optimal.
- `"timeout"` — best incumbent returned; `achieved_gap` records the remaining bound gap.
- `"infeasible"` — no valid plan exists under the given population balance constraints.
- `"fallback_metis"` — subgraph exceeded `max_tracts`; METIS was used.

The MPS file is retained at `runs/{label}/{year}/ilp/{node_id}.mps` for external verification. An auditor can reload the MPS file into any solver and confirm that the committed plan matches the optimal solution.

---

## 7. Test invariants

### L0 (inline unit tests)

- 4-node path k=2: ILP finds optimal solution EC=1 (single cut at midpoint), `solver_status: "optimal"`
- 4-node path k=2: optimal EC ≤ METIS EC on the same instance (ILP is at least as good as METIS)
- `max_tracts` guard: subgraph with 600 tracts returns `solver_status: "fallback_metis"`
- Infeasible instance (impossible population balance): `solver_status: "infeasible"`
- Same `base_seed` → same LP file generated (deterministic formulation)

### L1 (integration, synthetic data)

- 4x4 grid k=2: ILP optimal EC = known value (verifiable by exhaustive enumeration for this graph size)
- VT synthetic k=1: trivial — single district, EC=0, `solver_status: "optimal"`, solve time < 1s
- Population balance constraint enforced: all districts within tolerance for every solved instance
- **MPS round-trip**: write MPS for 4-node path k=2 instance, re-parse with `good_lp`, resolve, confirm objective value = 1 (single cut). This validates that the retained MPS file can be independently solved by an auditor.

### L2 (`#[ignore]`, real data)

- VT 2020 k=1: trivially optimal, solve time < 1s
- ND 2020 k=1: trivially optimal, solve time < 1s
- DE 2020 k=1: trivially optimal, solve time < 1s
- NH 2020 k=2: ILP optimal EC ≤ METIS EC; solve time < 60s

---

## 8. Open questions (deferred)

1. Should MTZ contiguity constraints be replaced with stronger Dantzig-Fulkerson-Johnson cuts? DFJ cuts yield a tighter LP relaxation and typically faster branch-and-bound convergence, but require cut generation at each node (branch-and-cut), which complicates the implementation.
2. Can the ILP be warm-started with a METIS solution to improve the initial incumbent and reduce solve time? For `good_lp`, this requires checking whether the backend supports MIP start hints.
3. Should `good_lp` be the only dependency (keeping the crate pure-Rust without requiring GLPK)? This would require the `lp-solvers` feature of `good_lp` that ships an embedded solver, at the cost of a larger binary.
4. Should the MPS output be committed to the label audit directory for external verification? Currently the spec retains it in `runs/` (gitignored). Committing would allow reproducibility without re-running the formulation, but adds large binary files.

---

## References

- `nemhauser1988`: Nemhauser, G.L., & Wolsey, L.A. (1988). *Integer and Combinatorial Optimization*. Wiley.
- `miller1960`: Miller, C.E., Tucker, A.W., & Zemlin, R.A. (1960). "Integer programming formulation of traveling salesman problems." *Journal of the ACM*, 7(4), 326–329.
- `gurnee2021`: Gurnee, W., & Shmoys, D.B. (2021). "Fairmandering: A column generation heuristic for fairness-optimized political districting." *AAAI Workshop on AI for Social Good*.
- `king2015`: King, D.M., Jacobson, S.H., & Sewell, E.C. (2015). "Efficient geo-graph contiguity and hole algorithms for geographic zoning and dynamic plane graph partitioning." *Management Science*, 61(5), 1115–1131.
