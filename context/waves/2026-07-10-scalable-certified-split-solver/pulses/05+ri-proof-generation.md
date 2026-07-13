---
pulse: 05
title: RI proof generation
status: done
depends_on: 01, 02, 03, 04
wave: scalable-certified-split-solver
validation_level: external proof
---

# Pulse 05 - RI Proof Generation

Generate and independently verify population, boundary, and canonical decision
proofs for Rhode Island's root split.

## Result

- Population: `UNSATISFIABLE`, proof generated, VeriPB accepted.
- Boundary: `TIMELIMIT`; incomplete 7.1 GB proof log deleted.
- Canonical: not run because boundary optimality is unresolved.

The population stage is certified at scaled deviation 1. The root cut is not
fully certified.

## Validation

```powershell
python scripts/research/verify_ri_proof_frontier.py --check-local
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
python scripts/research/verify_proof_toolchain_smoke.py
cargo fmt --all -- --check
git --no-pager diff --check
```
