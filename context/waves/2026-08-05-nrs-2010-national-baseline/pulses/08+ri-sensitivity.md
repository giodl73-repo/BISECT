---
pulse: 08
title: Rhode Island NRS v0.3 sensitivity
status: complete
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
- [x] Execute all 100 diagnostic indices.
- [x] Preserve compact assignment traces and every failure.
- [x] Independently regenerate and verify the package.
- [x] Record the result and remaining national sensitivity gate.

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

## Result

All 100 diagnostic indices completed, passed the scaled population bound, and
reproduced one unique canonical assignment: the governed benchmark. Assignment
agreement was 100 percent for every seed, and all three objective components
were invariant. The benchmark and all diagnostics therefore tie across the
full rank interval `1-101 / 101`.

This does not prove general seed robustness. In the NRS v0.3 execution path,
the seeded METIS assignment informs the moved-population tie-break among
equal-deviation deterministic DFS cut candidates. The Rhode Island root
selected the same DFS candidate under every frozen seed. The remaining gate is
a separately precommitted multi-State or national node sample capable of
exercising seed-sensitive ties and fallback stages.

The accepted package is
`docs/experiments/nrs-v0.3-ri-sensitivity-2020/`.

## Validation

```powershell
python -m pytest tests/unit/test_nrs_ri_sensitivity.py -q
python scripts/research/verify_nrs_ri_sensitivity.py `
  docs/experiments/nrs-v0.3-ri-sensitivity-2020
git --no-pager diff --check
```
