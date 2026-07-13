---
wave: scalable-certified-split-solver
date_open: 2026-07-10
status: complete
date_close: 2026-07-10
source_goal: produce the first scalable proof-checked certified State split
vtrace_posture: internal_engineering_baseline_only
---

# Scalable Certified Split Solver

## Mission

Replace the bounded exhaustive/static-no-good prototype with a production
discovery and proof pipeline capable of certifying Rhode Island's connected
25,649-block `1:1` root split.

## Fixed Contract

The recursive BISECT schedule, objective order, island bridges, certificate
schemas, tree semantics, and three-stage decision sequence remain unchanged.
This wave may change solver and proof implementation only.

## Success Metrics

| Metric | Baseline | Target |
|---|---|---|
| Proof generator | Missing | Pinned RoundingSat smoke proof |
| Proof checker | Missing | Pinned VeriPB verification |
| Connectivity encoding | Exponential no-goods | Compact proof-loggable formulation |
| Discovery solver | Missing | Deterministic exact candidate workflow |
| RI root input | Connected RCTX | Solver-ready model package |
| RI root result | Blocked | Verified three-stage certificate or precise solver blocker |

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - Proof toolchain acquisition | DONE | Pinned RoundingSat proof generated and accepted by pinned VeriPB |
| 02 - Compact connectivity encoding | DONE | Parent/depth polynomial OPB with external RoundingSat/VeriPB proof |
| 03 - Discovery solver integration | DONE | METIS plus nine articulation-safe moves reaches RI population floor |
| 04 - RI root model package | DONE | Three 163-168 MB compact OPBs with independently checked identities |
| 05 - RI proof generation | DONE (PARTIAL) | Population proof verified; boundary timeout; canonical not run |
| 06 - Scalable frontier review | DONE | Population stage certified; boundary/canonical unresolved with documented solver frontier |

## Claim Boundary

No production or State-level exactness claim advances until an external proof
checker accepts every required decision proof.

## Validation

Every pulse runs:

```powershell
cargo fmt --all -- --check
git --no-pager diff --check
```
