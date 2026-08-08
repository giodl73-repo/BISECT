---
wave: nrs-2010-national-baseline
date_open: 2026-08-05
status: active
source_goal: extend the verified NRS v0.1 reference baseline to 2010
vtrace_posture: internal_engineering_baseline_only
---

# NRS 2010 National Baseline

## Mission

Freeze complete 2010 block/source custody, construct the 2010 national context
inventory, and execute the same governed NRS v0.1 procedure used for the
verified 2020 national baseline.

## Fixed invariants

- Census tabulation blocks remain the atomic units;
- the standard and legal profiles are unchanged unless a versioned,
  independently reviewed revision is required by a documented 2010 source
  incompatibility;
- State apportionment counts and recursive seat schedules are year-correct;
- islands, bridges, population tolerance, seed derivation, and executable
  custody retain the 2020 contracts; and
- tolerance conformance, exact population proof, boundary proof, and canonical
  proof remain separate claims.

## Pulses

| Pulse | Status | Outcome |
|---|---|---|
| 01 - 2010 source and context inventory | COMPLETE | 50/50 PL packages hash-bound; 0/50 TIGER and RCTX; 11,071,790 blocks inventoried |
| 02 - 2010 block RCTX generation | COMPLETE | 50/50 archive-backed contexts independently verified; 11,071,790 blocks |
| 03 - 2010 NRS v0.1 batch execution | COMPLETE | 49 State packages verified; California retained as a deterministic tolerance-failure witness at node `00110` |
| 04 - 2010 NRS v0.2 amendment and verification | COMPLETE | Independent `--require-complete` verification passed: 50 States, 435 districts, 385 nodes, 11,071,790 blocks |
| 05 - decade comparison and publication | COMPLETE | Three-cycle NRS v0.3 summaries, node/tree/cut stability, certified-versus-METIS evidence, and county/tract split audit published |
| 06 - independent replication and closeout | ACTIVE | Author-machine fresh-clone Level 1 rehearsal passed at `d0053ff3`; independent external v0.3 record remains |
| 07 - NRS v0.3 national bakeoff | ACTIVE | National 2020 Tier 1 enacted-plan package passed for 50 States and 435 districts; later comparator and metric tiers remain |
| 08 - Rhode Island NRS v0.3 sensitivity | COMPLETE | All 100 frozen seeds reproduced one benchmark assignment/objective; result is mechanism-specific, not national robustness |
| 09 - Multi-State NRS v0.3 root sensitivity | COMPLETE | All 300 pairs reproduced State benchmarks; candidate-tie instrumentation now gates broader seed expansion |
| 10 - Initial DFS tie census | ACTIVE | Behavior-preserving counters and 44-State root census protocol frozen before instrumented replay |

## Versioned amendment

The complete v0.1 batch did not conform nationally. California's canonical
candidate at node `00110` achieved scaled population deviation `14,735`
against the unchanged allowed bound `6,991`. Sixteen adjacent node seeds and
one independently derived full-State seed reproduced the same miss, showing
that retrying the seed was not a remedy. The retained witness explicitly is
not an infeasibility proof.

NRS v0.2 preserves the v0.1 candidate as the primary candidate and activates
a fixed 16-root canonical DFS fallback only after that candidate misses the
0.5 percent tolerance. Candidates use the same deterministic population
repair and are ordered by population deviation, weighted cut, moved
population, and canonical assignment. The exact California witness replay
achieved `207`, and the complete 53-district California v0.2 package passed
independent verification.

That national gate subsequently passed. The final v0.2 ledger contains 50
verified States and zero failures; a separate
`verify-nrs-batch --require-complete` invocation passed all packages. The
verified aggregate covers 11,071,790 blocks, population 308,143,815, all 435
districts, and all 385 recursive split nodes. All nodes meet the population
tolerance. Arithmetic population-floor equality is proved at 7 nodes;
weighted-boundary and canonical global optimality remain explicitly unproved
at all 385 nodes.

The Census 2000 extension is also nationally complete. Its v0.2 run retained
Hawaii as a root tolerance-failure witness (`802,861` achieved versus `6,058`
allowed). NRS v0.3 preserves the v0.2 seed stream and candidate stages, then
uses a frozen bridge-aware land-component fallback only after a v0.2 miss.
The complete v0.3 batch and independent `--require-complete` verifier passed
50 States, 435 districts, 385 nodes, and 8,199,908 blocks. All nodes satisfy
population tolerance; arithmetic-floor equality is proved at 2 nodes, while
boundary and canonical proof coverage remain 0/385.

## Closeout status

The three-cycle stability matrix, national geographic split audit, verifier
bundle, technical schedule, and adoption materials are published. On
2026-08-07, the Level 1 artifact verifier passed from a clean clone of commit
`d0053ff3`, including recomputation of the 120 all-cycle common node signatures
and all 231,765 State/level geographic rows by the separate verifier
implementations.

That rehearsal was executed on the author machine. It confirms that the
fresh-clone artifact path works, but it is not the required independent
external replication record. The remaining wave gate requires a non-author
machine and reviewer record under the published independence protocol.

## Claim boundary

The wave may report complete reference-baseline conformance only after the
independent complete verifier passes. It may not infer 2010 exact objective,
legal, VRA, fairness, or adoption claims from the 2020 result.
