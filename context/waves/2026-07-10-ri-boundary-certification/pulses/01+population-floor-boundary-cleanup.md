---
pulse: 01
title: Population-floor boundary cleanup
status: done
depends_on: none
wave: ri-boundary-certification
validation_level: L2 real local search
---

# Pulse 01 - Population-Floor Boundary Cleanup

Improve weighted cut without changing population deviation or disconnecting
either child.

## Deliverables

- [x] Move zero-population boundary units when cut improves.
- [x] Preserve exact child populations.
- [x] Reject articulation removals.
- [x] Require destination adjacency.
- [x] Emit deterministic improved discovery identity.
- [x] Measure cut reduction and regenerate proof models.

## Result

- zero-population moves: 2;
- prior cut: 102,659,356;
- improved cut: 102,622,860;
- reduction: 36,496;
- population deviation: unchanged at 1; and
- population proof: still valid because the population OPB is unchanged.

## Validation

```powershell
cargo test -p bisect-cli exact_cmd::tests::zero_population_boundary_cleanup --lib -- --test-threads=1
python scripts/research/verify_ri_proof_frontier.py --check-local
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
cargo fmt --all -- --check
git --no-pager diff --check
```
