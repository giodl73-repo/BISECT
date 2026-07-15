---
wave: nationwide-2020-operational-certification
date_open: 2026-07-12
status: active
source_goal: nationwide wall-to-wall 2020 operational certification and exact proof coverage
vtrace_posture: internal_engineering_baseline_only
---

# Nationwide 2020 Operational Certification

## Mission

Generate deterministic operational recursive trees for all 50 States, produce
all 435 one-seat leaves, independently verify national wall-to-wall coverage,
and publish exact objective proof coverage separately.

## Success Metrics

| Metric | Baseline | Target |
|---|---:|---:|
| Connected State block contexts | 10 | 50 |
| Operational State packages | 10 | 50 |
| One-seat leaves | 17 | 435 |
| National block coverage | partial sample | 100% |
| Duplicate/omitted blocks | unreported nationally | 0 |
| Population proof status | sample | every recursive node classified |
| Boundary/canonical proof status | sample | every recursive node classified |

## Invariants

- 2020 Census blocks are normative;
- each block appears exactly once in one leaf;
- State graphs use the published deterministic bridge rule;
- recursive seat splits follow floor/ceiling canonical structure;
- every leaf is connected;
- population lower bounds are independently checked;
- heuristic boundary/canonical stages remain explicitly unproved; and
- national completion is operational coverage, not universal exact boundary
  optimality.

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - National data and context inventory | DONE | 50/50 sources, 8,126,956 blocks, 2.02 GiB estimate |
| 02 - Batch block RCTX generation | DONE | 50 contexts, 8,126,956 blocks, 9,657 bridges |
| 03 - Batch operational tree generation | IN PROGRESS | 37 verified packages; New York remains deviation 2/floor 0 after 32 seeds |
| 04 - National wall-to-wall verifier | BLOCKED BY 03 | Zero omissions, duplicates, or disconnected leaves |
| 05 - National proof coverage matrix | BLOCKED BY 04 | Population/boundary/canonical status per node |
| 06 - Nationwide 2020 publication | BLOCKED BY 05 | Reports, maps, custody, and Stage 3 closure |

## Claim Boundary

This wave certifies nationwide operational execution and publishes exact proof
coverage. It does not claim every nontrivial boundary objective is globally
optimal.
