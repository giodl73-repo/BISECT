# RI connected boundary-move diagnostic

Date: 2026-09-10. Result: bounded single-block moves escape the fixed-tree
candidate restriction, reduce split counties from four to three at every
tested alpha, and produce two distinct final assignments.

| Additive alpha | Moves | Raw cut | Within-county cut | Deviation numerator | Split counties |
|---|---:|---:|---:|---:|---:|
| Initial seed (all arms) | 0 | 303313349 | 297954300 | 853 | 4 |
| 0, 0.5 | 50 each | 230786999 | 220398621 | 981 | 3 |
| 1, 2, 4, 8 | 50 each | 231699372 | 214068666 | 969 | 3 |

All final assignments satisfy the fixed 1000-ppm relative child-population
band and are connected in the original graph, including its declared bridges.
The deviation numerator is `abs(2*p-P)`, not a percentage. Each arm terminates
at the predeclared 50-move budget, **not** at convergence or a proven optimum.

## Interpretation

This is the first diagnostic in this sequence to show actual assignment
response to county weights. Higher-alpha results have less within-county
boundary cost but slightly more raw boundary cost than the alpha-zero result.
The decline from four split counties to three also occurs at alpha zero:
this experiment does not attribute that county-count improvement to weighting.
It demonstrates a benefit of expanding search beyond the fixed-tree cuts.

All six arms start from the same unique alpha-zero winner of the preceding
1000-ppm five-tree diagnostic. The move generator considers single boundary
blocks, rejects moves outside the population band, orders strictly improving
moves by exact scaled weighted-cost delta then ascending GEOID index, and
accepts the first move that leaves both labels connected. Each arm has 50
accepted moves and at most 5000 candidate connectivity checks available.
Integer scaling implements rational alpha without rounding. Zero-population
blocks remain eligible. Empty labels are rejected.

These are two-label RI assignments, not new production plan packages or
certified solutions. The common input includes 25,649 retained blocks with
population 1,097,379. Subdivision counts include every retained block, including
zero-population blocks; no new geometry/common-universe comparison was run.
No claim about other States, legal compliance, fairness, or optimality follows.

## Artifacts and checks

- [Manifest](manifest.json): pre-run seed identity, input/source hashes, bands,
  alphas, move/check budgets and tie ordering.
- [Analysis](analysis.json): complete assignments, hashes, move traces and metrics.
- [Verification](verification.json): replay of all 300 published moves.

The replay checker does not run candidate search: it applies each recorded
move and recomputes full-graph cost, population and connectivity at every step.
It shares metric/connectivity helpers with the generator, so this is not an
independent implementation or operator audit. A separate search replay is
byte-identical. Five new unit tests cover invalid/empty/disconnected seeds,
strict improvement, cost reconciliation and budget termination.

```powershell
python -m unittest discover -s scripts/research -p test_ri_boundary_moves.py
python scripts/research/run_ri_boundary_moves.py run --manifest docs/experiments/ri-boundary-moves-2020/manifest.json --out target/ri-boundary-moves-new.json
python scripts/research/verify_ri_boundary_moves.py --manifest docs/experiments/ri-boundary-moves-2020/manifest.json --analysis target/ri-boundary-moves-new.json --out target/ri-boundary-moves-verification-new.json
```

## Next gate

Freeze a longer-budget experiment and measure improvement/stalling, rather
than treating 50 moves as sufficient. Then compare direct fragmentation scoring
and richer connected moves under separately declared objectives. Complete-plan
packaging, independent checking, and multi-district descendant feasibility
remain outstanding. Production rules and previous baseline artifacts are unchanged.
These deterministic operation budgets do not enforce wall-time or memory caps.
