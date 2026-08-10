---
pulse: 23
title: Block-level ensemble gate
status: complete
wave: nrs-2010-national-baseline
validation_level: exact normalized governed replay
---

# Pulse 23 - Block-Level Ensemble Gate

## Purpose

Freeze and implement the input boundary for the first governed block-level
ensemble without promoting the completed tract package to national evidence.

## Current State

- [x] Freeze `nrs-v0.3-block-ensemble-gate-v1` before governed chains.
- [x] Freeze the RI Stage 1 schedule, two separate kernels, chains, seeds,
  burn-in, snapshots, diagnostics, and stopping rules.
- [x] Implement direct block RCTX plus NRS assignment loading.
- [x] Reject universe, label, graph-symmetry, weight, contiguity, and population
  violations before ReCom can start.
- [x] Execute the excluded 25-step per-kernel engineering preflight.
- [x] Implement governed trace analysis and Stage 0 package verification.
- [x] Enable the trace runner's separately labeled governed execution mode.
- [x] Execute and exactly regenerate the governed Stage 1 slice.

## Evidence

- `docs/specs/2026-08-10-nrs-v0.3-block-ensemble-gate.md`
- `crates/bisect-ensemble/src/block_input.rs`
- `crates/bisect-ensemble/examples/validate_block_input.rs`
- `crates/bisect-ensemble/examples/block_trace.rs`
- `scripts/research/analyze_block_ensemble.py`
- `scripts/research/verify_block_ensemble_gate.py`
- `tests/unit/test_block_ensemble_analysis.py`
- `docs/experiments/nrs-v0.3-block-ensemble-gate/`

## Excluded Preflight Result

Both kernels completed and accepted 25 of 25 steps. Wilson averaged 177.758
ms/step and Kruskal 61.259 ms/step on the author machine. Maximum observed
population deviations were 0.004914 and 0.004626 respectively. Normalized
replay, excluding diagnostic runtime, reproduced both traces exactly.

## Governed Stage 1 Result

All eight frozen chains completed. Wilson and Kruskal accepted `0.94575` and
`0.953` of proposals; both stayed within the `0.005` population bound. Each
kernel passed the registered split-R-hat and pooled-ESS rules for both scalar
metrics after burn-in. A fresh sequential execution regenerated normalized
metrics and all canonical snapshots exactly for Wilson and then Kruskal.

The result shows material kernel sensitivity. Mean cut fraction was
`0.0103331` under Wilson and `0.00778270` under Kruskal, with descriptive KS
`0.405833`; weighted-cut means were `193,294,034` and `152,599,928`, with KS
`0.31`. Passing diagnostics does not prove mixing or independence.

## Claim Boundary

Stage 1 supports only a Rhode Island block-graph ReCom feasibility and
kernel-sensitivity result under the frozen start, tolerance, metrics, and
seeds. It is not a national, neutrality, legal, mixing-proof,
sampler-equivalence, partisan, demographic, VRA, or polygon-compactness result.
The multi-State expansion gate remains closed pending instrumented peak memory
and a separately frozen compute and storage budget.
