# Scalable Certified Split Frontier Review

## Outcome

Rhode Island has a **partial exact root certificate**.

### Proved

- The connected block instance contains 25,649 units and 66,161 edges.
- The selected district populations are 548,689 and 548,690.
- Scaled population deviation is 1.
- No integer split can achieve deviation below 1.
- RoundingSat generated the population proof.
- VeriPB independently accepted it.

### Not Proved

- The weighted boundary cut 102,659,356 is optimal.
- The submitted assignment is the canonical optimum among boundary ties.
- The complete Rhode Island root split is certified.

## Boundary Search Evidence

| Attempt | Proof logging | Limit | Result | Decisions | Conflicts | Propagations |
|---|---|---:|---|---:|---:|---:|
| A | Yes | 300 s | TIMELIMIT | 418,460 | 6,064 | 74,994,532 |
| B | No | 300 s | TIMELIMIT | 606,392 | 9,387 | 116,612,433 |

Attempt A produced a 7.1 GB incomplete proof log, which was deleted.
Attempt B shows that search remains unresolved without proof-log I/O.

## What Has Been Demonstrated

1. Full block-level source and island-link custody.
2. Deterministic connected discovery.
3. Exact population-floor local improvement.
4. Polynomial compact connectivity encoding.
5. State-scale model compilation.
6. External proof generation and checking.
7. One completed exact objective stage.

## Residual Technical Frontier

The next work should focus on boundary optimization:

1. improve the cut incumbent while preserving deviation 1;
2. add boundary-specific symmetry breaking and implied constraints;
3. test decomposition and cut-generation variants;
4. tune RoundingSat without weakening proof output;
5. evaluate a dedicated exact discovery solver; and
6. run canonical certification only after boundary optimality closes.

## Public Claim

Publishable:

> Rhode Island's certified BISECT root split has exact population optimality.

Not publishable:

> Rhode Island's full root cut or final two-district plan is exactly certified.

## Superseding Boundary Incumbent

The following boundary-certification wave subsequently reduced the cut to
102,622,860 using two zero-population connectivity-safe moves, then to
102,193,710 using three equal-population swaps, then to 98,348,913 using 25
one-to-two exchanges, and finally to 97,994,953 using one bounded two-to-two
population-preserving exchange. The timeout table above applies to
prior incumbents and remains historical evidence.

A subsequent deterministic 32-seed METIS ensemble accepted 13 connected,
population-floor candidates and rejected 19 candidates during exact
post-validation. Seed 4 was best; full deterministic refinement reduced its
weighted cut to 64,132,468. All earlier proof probes are therefore superseded.

An elite-two consensus fixed 24,217 blocks into two connected cores and left a
1,432-block disagreement band. After true fixed-variable elimination, SciPy
HiGHS found a connected candidate at weighted cut 49,081,395. The Rust
certified split validator independently accepted its population, connectivity,
and exact objective. This remains an incumbent, not a proof.

Further pairwise branches found 43,806,724 for the seed-4/28 band and
43,885,450 for seed-4/11. The former is the current connected incumbent.
Additional heuristic pair search is frozen because returns are diminishing.

A nested shell ladder around the seed-4/28 disagreement band then produced:

- one hop: 43,628,645;
- two hops: 43,156,153; and
- four hops: 43,047,238 after two connectivity-cut rounds.

The four-hop plan is the final frozen heuristic incumbent.
