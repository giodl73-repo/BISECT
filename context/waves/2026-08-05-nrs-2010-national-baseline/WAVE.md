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
| 07 - NRS v0.3 national bakeoff | ACTIVE | National 2020 Tiers 1 and 2 passed for 50 States and 435 districts; elections, demographics, ensembles, and non-enacted comparators remain |
| 08 - Rhode Island NRS v0.3 sensitivity | COMPLETE | All 100 frozen seeds reproduced one benchmark assignment/objective; result is mechanism-specific, not national robustness |
| 09 - Multi-State NRS v0.3 root sensitivity | COMPLETE | All 300 pairs reproduced State benchmarks; candidate-tie instrumentation now gates broader seed expansion |
| 10 - Initial DFS tie census | COMPLETE | All 44 roots preserved governed assignments/objectives; 29 expose two minimum-deviation/minimum-cut initial candidates |
| 11 - Initial DFS partition census | COMPLETE | All 44 roots have one physical minimum-deviation/minimum-cut partition; 29 oriented ties are equal-seat label symmetry |
| 12 - Complete-tree DFS and fallback census | COMPLETE | All 385 nodes have one physical initial cut; 266 orientation-only ties; zero v0.2/v0.3 fallback activations |
| 13 - Cross-census complete-tree DFS census | COMPLETE | All 770 nodes have one physical initial cut; seven v0.2 activations and one v0.3 activation form the bounded fallback universe |
| 14 - Fallback candidate census | COMPLETE | All eight activated stages preserved assignments/objectives; candidate ties at 2000 HI/root collapse to one physical partition, closing fallback seed expansion |
| 15 - Tier 2 geometry bakeoff | COMPLETE | Rhode Island common-block geometry passed; compactness metric direction is mixed, and national expansion requires a separate sequential protocol |
| 16 - National Tier 2 geometry | COMPLETE | Sequential execution and exact regeneration passed 50 States, 435 districts per plan, and 7,889,194 retained blocks |
| 17 - National bakeoff paper integration | COMPLETE | Paper-facing matrix published; U.21/A.0/A.5 integrated; B.0/K.0 evidence boundaries corrected; five PDFs rebuilt |
| 18 - Neutral algorithm-family proof slice | COMPLETE | Wisconsin four-structure package exactly regenerated; 2/4 plans passed native audits, so national expansion is blocked pending AreaSection and ApportionRegions remediation |
| 19 - Structure validity remediation | COMPLETE | Hidden prime-factor tolerance and contiguity paths corrected; archived 2-pass/2-fail witness retained; post-remediation Wisconsin package passed 4/4 and regenerated exactly |
| 20 - National algorithm-family pilot | COMPLETE | Frozen eight-State/32-cell matrix passed and regenerated exactly after retained failure/remediation history; 44-State phase remains compute-budget gated |
| 21 - National algorithm-family full phase | COMPLETE | Frozen 44-State/176-cell matrix passed without seed retries and an independent 176-cell execution regenerated the normalized evidence exactly |
| 22 - Ensemble evidence ledger reconciliation | COMPLETE | Public G.0/G.1-G.3 routing now points to the real RI/IA/NC package; stale synthetic/missing-evidence posture removed and G.0 rebuilt |
| 23 - Block-level ensemble gate | COMPLETE | RI eight-chain Stage 1 converged under frozen scalar rules and both kernels regenerated normalized metrics/snapshots exactly; material kernel sensitivity retained; expansion remains separately gated |
| 24 - Block-ensemble resource audit | COMPLETE | Both exact resource replays passed; author-machine peak 175,448,064 bytes; frozen formulas authorize 21 hours, 2.25 GiB/process, and 3 GiB retained/scratch |
| 25 - NH/NM/GA block-ensemble expansion | COMPLETE | Gate failed and closed without retry: five primaries passed, then GA Kruskal hit host disk-full; no governed replay ran |
| 26 - Ensemble host-capacity admission | COMPLETE | Future protocols must reserve scratch, remaining retained custody, and 2 GiB host headroom on the actual evidence volume before every runner launch |
| 27 - Capacity-admitted launch boundary | COMPLETE | Reusable fail-closed adapter records fresh actual-volume admission before process creation; no governed experiment was launched |
| 28 - G.0 expansion consistency | COMPLETE | Stale pre-resource-audit language replaced by the terminal Pulse 25 result and Pulse 26/27 successor-protocol boundary; G.0 rebuilt |
| 29 - Block-ensemble successor protocol | COMPLETE | Expansion v2 frozen with fresh seeds, fresh custody, per-process capacity admission, and no reuse of v1 completions; implementation gate remains |
| 30 - Block-ensemble v2 implementation | COMPLETE | Dedicated admitted runner, strict ledger, verifier, and adversarial contract audit passed on an empty package; no process launched |

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
