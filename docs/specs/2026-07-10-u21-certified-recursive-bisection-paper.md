# U.21 Certified Recursive Bisection Paper Specification

**Status:** Accepted internal writing specification  
**Paper:** U.21  
**Track:** U — Search and Optimization  
**Claim posture:** Methods, certificate architecture, and bounded evidence

## Research Question

Can recursive bisection preserve its fixed district-count tree while replacing
heuristic per-node choices with uniquely selected, independently checkable
optimal cuts?

## Core Claims

1. The canonical `floor(k/2)` / `ceil(k/2)` tree can be certified without
   replacing BISECT with unrestricted statewide optimization.
2. Population, boundary, and canonical tie claims decompose into three exact
   decision problems at every node.
3. Parent-derived child instances and one-seat leaves provide a verifiable
   whole-plan certificate tree.
4. The bounded Rust implementation and hostile corpora validate the contract.
5. Rhode Island block input custody is ready, but scalable discovery and
   external PB proof checking remain open.

## Claims Not Yet Permitted

- nationwide exact readiness;
- first full-State optimality proof;
- faster runtime than METIS;
- superior nationwide compactness or partisan outcomes;
- VRA or constitutional safe-harbor status; or
- global optimality among unrestricted districting plans.

## Evidence

- `crates/bisect-ilp/src/certified_split.rs`
- `crates/bisect-ilp/src/certified_tree.rs`
- `crates/bisect-ilp/src/proof_backend.rs`
- `docs/examples/certified-recursive/`
- `docs/examples/certified-proof-backend/`
- `docs/experiments/certified-recursive/`
- exact and hostile verifier test suites

## Required Comparisons

| Comparison | Current status |
|---|---|
| Certified bounded cut vs METIS on same synthetic instances | Complete for eight precommitted fixtures and 40 fixed-seed rows; 30/40 full-objective and 20/40 canonical-assignment agreements |
| Exact objective agreement across implementations | Available for E0; generalized independent verifier future work |
| Runtime scaling | Bounded timings only; no State-scale proof timing |
| National map-quality superiority | Not available |
| Proof strength vs audit/solver metadata | Available |

## Recommended Structure

1. Introduction and Huntington--Hill motivation.
2. Fixed recursive bisection model.
3. Per-cut and whole-tree certificate contracts.
4. Discovery-to-proof architecture.
5. Implemented evidence and hostile validation.
6. What certification establishes—and does not.
7. Scalability frontier and Rhode Island.
8. Conclusion.
