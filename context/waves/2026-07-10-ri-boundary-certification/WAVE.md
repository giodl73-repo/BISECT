---
wave: ri-boundary-certification
date_open: 2026-07-10
status: archived-partial
source_goal: prove Rhode Island boundary and canonical optimality at population floor
vtrace_posture: internal_engineering_baseline_only
---

# Rhode Island Boundary Certification

## Autonomous Execution

This wave executes under
[`../CERTIFIED_NATIONAL_ROADMAP.md`](../CERTIFIED_NATIONAL_ROADMAP.md).
Routine pulses proceed automatically toward the nationwide 2020/2010/2000
wall-to-wall goal. User input is required only for normative, destructive, or
externally authorized decisions.

## Mission

Reduce and then prove Rhode Island's weighted boundary cut while preserving the
already certified population deviation of 1.

## Fixed Invariants

- block universe and island bridges unchanged;
- district populations remain 548,689 / 548,690;
- both children remain connected;
- weighted boundary is the next lexicographic objective;
- canonical proof waits for boundary optimality.

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - Population-floor boundary cleanup | DONE | Two zero-pop moves reduce cut by 36,496 with population proof preserved |
| 02 - Balanced swap neighborhoods | DONE | Local search reduced the seed-1 cut to 97,994,953 |
| 02A - METIS ensemble discovery | DONE | Best-of-32 seed 4 plus full refinement reduces cut to 64,132,468 |
| 02B - Reduced-band MILP discovery | DONE | Top-two consensus band plus HiGHS reduces cut to 49,081,395 |
| 02C - Pairwise-band discovery | DONE | Seed-4/28 band reduces cut to 43,806,724; heuristic search frozen |
| 02D - Nested shell expansion | DONE | Four-hop shell reaches 43,047,238; discovery frozen |
| 03 - Boundary model strengthening | DONE | Parent-depth v3 removes depth and root-choice symmetry |
| 04 - Boundary proof search | DEFERRED | Exact decompositions and four solver architectures exhausted; unrestricted proof remains open |
| 05 - Canonical proof | DEFERRED | Requires unrestricted boundary optimality |
| 06 - Rhode Island root certificate | PARTIAL | Population proof, branch proof, incumbent, failures, and residual blocker published |

## Claim Boundary

Population optimality is fixed and proved. No boundary or canonical claim
advances without an independently checked proof.
