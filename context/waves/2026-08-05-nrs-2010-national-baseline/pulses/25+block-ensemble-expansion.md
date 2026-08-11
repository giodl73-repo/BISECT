---
pulse: 25
title: NH/NM/GA block-ensemble expansion
status: active
wave: nrs-2010-national-baseline
validation_level: Stage 0 excluded preflight and exact replay verified
---

# Pulse 25 - NH/NM/GA Block-Ensemble Expansion

## Purpose

Execute the precommitted three-State expansion under the measured resource
ceilings without promoting three States to a national claim.

## Current State

- [x] Freeze `nrs-v0.3-block-ensemble-expansion-v1` before governed chains.
- [x] Bind the validated NH/NM/GA inputs and the measured resource budgets.
- [x] Extend and test the governed runner and enforcing resource monitor.
- [x] Execute excluded 25-step preflight and exact replay in the frozen order.
- [ ] Execute, analyze, and exactly regenerate all 24 governed chains.

Stage 0 verification passed after preserving and remediating an NM Wilson
fresh-process determinism failure. The canonical six-State/sampler preflight
sweep and all six replays now match exactly; no governed expansion chain ran
before this gate passed.

Governed primary checkpoint: NH Wilson, NH Kruskal, NM Wilson, and NM Kruskal
passed in frozen order with deterministic gzip custody. NM Wilson measured
12,960.48 seconds and 655,495,168 peak RSS bytes; NM Kruskal measured 2,023.14
seconds and 710,582,272 peak RSS bytes. Both remain below the frozen ceilings.

## Claim Boundary

The expansion can support only State-specific block-graph feasibility,
diagnostics, and kernel sensitivity for NH, NM, and GA. It is not national
ensemble, fairness, legal, or sampler-equivalence evidence.
