---
pulse: 04
title: RI root model package
status: done
depends_on: 02, 03
wave: scalable-certified-split-solver
validation_level: L2 real model
---

# Pulse 04 - RI Root Model Package

Compile Rhode Island's connected block RCTX into hash-bound discovery and proof
models.

## Deliverables

- [x] Compile population lower-bound OPB.
- [x] Compile boundary lower-bound OPB.
- [x] Compile canonical predecessor OPB.
- [x] Replace large lexicographic coefficients with a prefix automaton.
- [x] Hash every OPB and request.
- [x] Independently verify request IDs and model dimensions.
- [x] Publish committed model metadata without committing 495 MB of generated inputs.
- [x] Run a bounded RoundingSat parse/solve probe.

## Result

All three State-scale models compile. The 30-second population probe returns
`TIMELIMIT`; no SAT, UNSAT, or optimality conclusion is claimed.

## Validation

```powershell
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
python -m pytest -q tests/unit/test_ri_model_package.py
cargo test -p bisect-ilp proof_backend -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```
