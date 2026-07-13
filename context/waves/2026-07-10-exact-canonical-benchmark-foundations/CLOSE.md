# Exact Canonical Benchmark Foundations - Close Record

**Opened:** 2026-07-10  
**Closed:** 2026-07-10  
**Disposition:** `complete_bounded_e0_foundation`  
**Posture:** `internal_engineering_baseline_only`

## Achieved

1. Versioned exact instance, certificate, proof, and package schemas.
2. Four-level lexicographic objective with canonical tie-breaking.
3. Bounded exhaustive `k=2`, `n <= 24` optimality/infeasibility oracle.
4. `bisect exact --method canonical-exhaustive` package integration.
5. Hash-bound optimal and infeasible fixtures.
6. Ordered proof transcript committing to every candidate.
7. Five-case hostile certificate corpus.
8. Rust reference verifier and independent Python verifier.
9. Deterministic two-verifier acceptance/rejection report.
10. Rhode Island block-level frontier report with complete source custody.

## Important Negative Result

Rhode Island contains 25,649 statutory 2020 Census blocks. The E0 exhaustive
oracle supports 24 units, and its symmetry-reduced search would contain
`2^25648-1` candidates. No exact State certificate was issued. Tract-level,
inhabited-only, heuristic, or bounded-gap substitutions remain disallowed.

## Claim Disposition

| Claim class | Disposition |
|---|---|
| Bounded E0 exact objective | Implemented |
| Optimal/infeasible certificate contract | Implemented |
| Proof transcript and tamper rejection | Implemented |
| Two independent verifier implementations | Implemented for E0 |
| Real Rhode Island source custody | Complete for block geometry/population |
| Real-State exact certificate | Blocked by model, compute, and adjacency custody |
| National exact readiness | Not implemented |
| Legal certification | Not claimed |

## Lessons

- Exhaustive enumeration is a trustworthy oracle, not a scalable solver.
- Proof and discovery should be separate systems.
- A real-data blocker is more valuable than a mislabeled coarse-resolution
  success.
- Zero-population blocks remain statutory units and cannot be silently removed.
- The next exact objective must certify each enacted recursive bisection cut,
  not replace BISECT with unrestricted statewide optimization.

## Carry-Forward

The next wave is **Certified Recursive Bisection**:

1. freeze the exact per-node split objective;
2. generalize population targets to `k_left:k_right`;
3. chain split certificates through the canonical bisection tree;
4. preserve California's `52 -> 26/26 -> 13/13 -> 6/7` structure;
5. separate fast solver discovery from proof-producing certification; and
6. return to Rhode Island blocks with a scalable proof backend.

No git commit or public release was created by wave closure. The working tree
remains ready for maintainer review and an explicitly requested commit.
