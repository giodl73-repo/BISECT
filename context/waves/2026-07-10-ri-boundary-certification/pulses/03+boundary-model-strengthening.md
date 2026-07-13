---
pulse: 03
title: Boundary model strengthening
status: done
depends_on: 01, 02
wave: ri-boundary-certification
validation_level: proof model
---

# Pulse 03 - Boundary Model Strengthening

Add implied constraints, fixed assignments, and symmetry reductions justified
by the improved incumbent and certified model.

## Deliverables

- [x] Upgrade proof requests through schema v5.
- [x] Add exact right-child population branch identity.
- [x] Compile the 548,689 population branch.
- [x] Compile the 548,690 population branch.
- [x] Prove the two branches cover the population-optimal feasible set.
- [x] Hash and verify both branch models.
- [x] Run bounded solver probes on both branches.

## Result

Both strengthened branches reached `TIMELIMIT` after 120-second probes on the
then-current incumbent. The decomposition is exact. New branch models have
been regenerated for cut 64,132,468. Parent-depth v2 fixes the equal-seat left
root to canonical unit 0 and forces unassigned depth bits to zero. Parent-depth
v3 deterministically selects the minimum-index assigned root for both children.
The low-population v3 branch also reached `TIMELIMIT` after 300 seconds.

## Validation

```powershell
cargo test -p bisect-ilp proof_backend -- --test-threads=1
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
python scripts/research/verify_ri_proof_frontier.py --check-local
cargo fmt --all -- --check
git --no-pager diff --check
```
