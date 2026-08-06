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
| 04 - 2010 NRS v0.2 amendment and verification | ACTIVE | Govern the canonical multi-root fallback and require complete wall-to-wall coverage |
| 05 - decade comparison and publication | PENDING | Publish stability, runtime, and exact-proof matrices |

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
independent verification. National verification remains open until the v0.2
batch passes with all 50 States.

## Claim boundary

The wave may report complete reference-baseline conformance only after the
independent complete verifier passes. It may not infer 2010 exact objective,
legal, VRA, fairness, or adoption claims from the 2020 result.
