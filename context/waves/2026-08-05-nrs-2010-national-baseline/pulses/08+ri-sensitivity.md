---
pulse: 08
title: Rhode Island NRS v0.3 sensitivity
status: active
wave: nrs-2010-national-baseline
validation_level: L2 governed empirical evidence
---

# Pulse 08 - Rhode Island NRS v0.3 Sensitivity

## Purpose

Execute the evaluation schedule's first 100-seed diagnostic slice without
changing or replacing the governed benchmark.

## Deliverables

- [x] Replay the published benchmark seed exactly.
- [x] Precommit seed derivation, execution, metrics, and failure posture.
- [ ] Execute all 100 diagnostic indices.
- [ ] Preserve compact assignment traces and every failure.
- [ ] Independently regenerate and verify the package.
- [ ] Record the result and remaining national sensitivity gate.

## Pre-Execution Evidence

The committed release executable replayed Rhode Island's published benchmark
seed `1983447153` against the hash-bound 2020 block context. The regenerated
`certified-discovery.json` SHA-256 was
`1fa5775fda7b9370f4341e81268df8a24fa256eb2a32e013544b20e67edb265f`,
matching the governed package exactly.

## Governing Boundary

The benchmark assignment remains authoritative. Diagnostic seeds measure
sensitivity only and are not an ensemble, a replacement-selection process, or
evidence of national, partisan, legal, or optimal behavior.
