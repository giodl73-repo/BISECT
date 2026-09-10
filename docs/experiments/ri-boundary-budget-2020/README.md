# RI longer-budget boundary search

Date: 2026-09-10. All six runs stop naturally before their fixed 500-move
budget. Each reproduces its earlier 50-move control exactly, then improves
boundary costs further. All final assignments remain connected and within
the unchanged 1000-ppm relative child-population band.

| Alpha | Accepted moves | Raw cut | Within-county cut | Deviation numerator | Split counties |
|---|---:|---:|---:|---:|---:|
| 0 | 275 | 197825676 | 186306787 | 1059 | 3 |
| 0.5 | 276 | 197215906 | 177795336 | 1049 | 3 |
| 1 | 272 | 197276138 | 177855568 | 1045 | 3 |
| 2 | 271 | 197387261 | 177747697 | 1045 | 3 |
| 4 | 278 | 196816946 | 176112210 | 1035 | 3 |
| 8 | 283 | 196421408 | 175716672 | 1051 | 3 |

Population numerator means `abs(2*p-P)` for two seats, not a percentage.
The band permits a numerator at most 1097.379 for this input. No legal threshold
is asserted. All results use retained blocks, including zero-population units,
and connectivity through declared source-graph bridges.

## What stopped, and what did not improve

The engine exhausted strictly cost-reducing **single-block** moves satisfying
both population and connectivity constraints. It did not exhaust swaps,
connected groups, temporary score increases, alternative seeds, or other
population/objective policies. These are local stopping states, not global
optima or complete algorithm convergence claims.

County splits remain at three in all arms. The earlier improvement from four
to three occurred within the first 50 moves, including at alpha zero. Longer
search improves boundaries but does not further reduce split counties here.
Different weights follow different greedy paths; cross-alpha cost orderings
are not monotonicity guarantees or a basis for declaring one alpha universally
best. No alpha is selected for production from this small RI experiment.

## Frozen design and verification

The original seed, input, move engine, population band, alpha schedule, exact
arithmetic and tie ordering are unchanged. Only accepted-move budget rises
from 50 to 500; candidate connectivity-check budget stays at 5000 per arm.
Checkpoints are fixed at 50, 100, 250 and 500 moves; the last is absent because
every run stops earlier. Final records retain actual stopping states.

- [Manifest](manifest.json): pre-run settings, source/input and baseline hashes.
- [Analysis](analysis.json): complete assignments, 1655 move records, checkpoints,
  and exact agreement with all six original 50-move trajectories.
- [Trace verification](verification.json): rechecks every recorded move's
  population, connectivity and full-graph score change.
- [Stopping check](stopping-check.json): restarting the unchanged engine from
  each final assignment reproduces zero accepted improving moves.

Both checkers share implementation helpers with search; neither is an
independent implementation/operator audit. Full long-search byte replay is
not claimed. Trace replay and final-state rechecks verify different aspects.
Operation budgets are enforced; wall time and memory are not hard-capped.
The pre-run manifest is locally hashed, not externally witnessed.

```powershell
python scripts/research/run_ri_boundary_budget.py run --manifest docs/experiments/ri-boundary-budget-2020/manifest.json --out target/ri-boundary-budget-new.json
python scripts/research/verify_ri_boundary_moves.py --manifest docs/experiments/ri-boundary-budget-2020/manifest.json --analysis target/ri-boundary-budget-new.json --out target/ri-boundary-budget-verification-new.json
python scripts/research/check_ri_boundary_stopping.py --manifest docs/experiments/ri-boundary-budget-2020/manifest.json --analysis target/ri-boundary-budget-new.json --out target/ri-boundary-budget-stopping-new.json
```

## Next gate

Test direct county-fragment objectives and connected multi-block moves as
separate, predeclared changes, keeping these local stopping states as controls.
Simply extending this same move budget will not escape them. New production
plan packages, independent checking, geometry comparison, and multi-district
recursive feasibility remain outstanding. No production rule was changed.
