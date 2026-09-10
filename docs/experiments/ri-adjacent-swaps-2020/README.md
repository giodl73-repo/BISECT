# RI adjacent-block swap diagnostic

Date: 2026-09-10. Result: zero swaps accepted for every alpha (0, 0.5, 1,
2, 4, 8). All six assignments exactly match their corresponding
[single-block stopping states](../ri-boundary-budget-2020/README.md).
Each still splits three counties. No score, population or connectivity change
occurred, and neither swap nor candidate-check budgets caused termination.

## Predeclared move and control

Each arm starts from its own frozen longer-budget result, with the same
alpha and 1000-ppm population band. An allowed move exchanges the labels of
two endpoints of an original cut edge. Population feasibility is checked for
the simultaneous exchange; both resulting labels must be connected in the
original graph. The edge joining the swapped blocks remains cut, so its cost
is excluded from both single-flip delta terms. Rational weights use integer
scaling without rounding. Strictly improving moves are ordered by cost delta,
then ascending endpoint indices. Budgets: 50 swaps and 5000 candidate checks.

These are paired continuation comparisons against each arm's own control,
not a new same-seed comparison across alpha. Adjacent swaps were tested in
isolation, without alternating single-block moves. Because no swap occurred,
the preceding single-block stopping checks still apply to these exact labels.
This remains a local result for these starts and declared objectives.

## What remains open

No-improvement under adjacent swaps does not imply no improvement under
nonadjacent boundary-block exchanges, connected-group transfers, neutral-cost
moves, alternative seeds, or direct county-fragment scoring. The next move
family must be declared separately; do not relabel this experiment as a test
of all multi-block moves. No wider population band or production rule is adopted.

## Artifacts and verification

- [Manifest](manifest.json): pre-run move definition, budgets, exact source/input
  and baseline hashes.
- [Analysis](analysis.json): all six full assignments, checks, empty traces and metrics.

Four tests cover a beneficial equal-population swap, exact delta/full-score
agreement including the shared-edge correction, budget termination, and an
invalid starting population. Real-data replay is byte-identical and every
final assignment hash and metric matches its control. The checker shares
production-diagnostic helpers; no independent implementation audit is claimed.

```powershell
python -m unittest discover -s scripts/research -p test_ri_adjacent_swaps.py
python scripts/research/run_ri_adjacent_swaps.py run --manifest docs/experiments/ri-adjacent-swaps-2020/manifest.json --out target/ri-adjacent-swaps-new.json
```

Connectivity includes declared bridges; zero-population retained blocks remain
in labels and county counts. Serial execution has no hard time or memory cap.
This is not a legal, neutrality, global-optimality or complete-plan bakeoff claim.
