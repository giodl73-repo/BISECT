---
pulse: 06
title: Scalable frontier review
status: done
depends_on: 05
wave: scalable-certified-split-solver
validation_level: panel
---

# Pulse 06 - Scalable Frontier Review

Publish the first exact State root certificate or the precise remaining
solver/proof blocker.

## Disposition

**Partial exact certificate.**

- Population stage: proved and independently verified.
- Boundary stage: two 300-second timeouts, with and without proof logging.
- Canonical stage: blocked behind boundary optimality.

The precise residual blocker is State-scale boundary optimization/proof search.

## Validation

```powershell
python scripts/research/verify_ri_proof_frontier.py --check-local
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
python scripts/research/verify_nrs_challenge_bundle.py
cargo fmt --all -- --check
git --no-pager diff --check
```
