---
wave: cli-weighted-cut-scoring-cleanup
date_open: 2026-05-14
status: archived
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# CLI Weighted Cut Scoring Cleanup

## Mission

Consolidate repeated weighted cut scoring inside `bisect-cli` while keeping the
weighted scoring policy out of `rgraph-core`.

## Claim Boundary

This wave may add a local helper for summing crossing-edge weights over a
`HashMap<(usize, usize), f64>`. It must not add a shared weighted-cut API until
there is reuse outside this CLI runner and a clear weight-validation boundary.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Local weighted cut helper | DONE | Replaced repeated bisection-runner weighted cut scoring loops with a local helper |
