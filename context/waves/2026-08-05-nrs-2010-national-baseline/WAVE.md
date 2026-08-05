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
| 01 - 2010 source and context inventory | ACTIVE | Audit all 50 State inputs and identify custody gaps |
| 02 - 2010 block RCTX generation | PENDING | Build and independently verify contexts |
| 03 - 2010 NRS batch execution | PENDING | Generate resumable, identity-bound State packages |
| 04 - 2010 national verification | PENDING | Require complete wall-to-wall coverage |
| 05 - decade comparison and publication | PENDING | Publish stability, runtime, and exact-proof matrices |

## Claim boundary

The wave may report complete reference-baseline conformance only after the
independent complete verifier passes. It may not infer 2010 exact objective,
legal, VRA, fairness, or adoption claims from the 2020 result.
