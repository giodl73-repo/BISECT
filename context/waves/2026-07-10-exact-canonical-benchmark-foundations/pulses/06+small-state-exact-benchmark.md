---
pulse: 06
title: Small-State exact benchmark
status: done
depends_on: 02, 03, 04, 05
wave: exact-canonical-benchmark-foundations
validation_level: L2 real data
---

# Pulse 06 - Small-State Exact Benchmark

Produce an exact real-State certificate at the statutory unit level or record
the precise data/compute blocker without substituting heuristic evidence.

## Deliverables

- [x] Select Rhode Island 2020 as the first real `k=2` target.
- [x] Hash the complete TIGER block and PL 94-171 source custody.
- [x] Confirm one-to-one coverage of 25,649 block GEOIDs.
- [x] Confirm population and geometry completeness.
- [x] Quantify the E0 model-limit and exhaustive-search gap.
- [x] Audit block adjacency and RCTX custody.
- [x] Publish a hash-bound blocker report and package manifest.
- [x] Add source-backed and source-independent verification paths.
- [x] Document disallowed tract, inhabited-only, and heuristic substitutions.

## Result

No exact State certificate is issued. Rhode Island exceeds the E0 unit limit by
25,625 blocks, and the required search contains `2^25648-1` candidates. The
current data custody also lacks a block adjacency/RCTX artifact.

## Claim Boundary

This pulse proves the blocker, not impossibility for every future exact method.
It does not build block adjacency, invoke a production exact solver, remove
zero-population blocks, or promote tract/heuristic evidence to exact status.

## Validation

```powershell
python scripts/research/analyze_small_state_exact_frontier.py analyze --output docs/experiments/exact-canonical/ri-2020-block-frontier.json --manifest docs/experiments/exact-canonical/manifest.json
python scripts/research/analyze_small_state_exact_frontier.py verify-package docs/experiments/exact-canonical/manifest.json --check-sources
python -m pytest -q tests/unit/test_small_state_exact_frontier.py
python scripts/research/verify_nrs_challenge_bundle.py
cargo fmt --all -- --check
git --no-pager diff --check
```
