# RI county-aware tree diagnostic

Date: 2026-09-10. Result: neither paired tree generator provides an eligible
cut under the unchanged 10, 100 and 1000 ppm population bands.

| Generator | Minimum population-deviation numerator over all cuts | Eligible cuts at 10 / 100 / 1000 ppm |
|---|---:|---|
| Geographic maximum spanning tree | 82307 | 0 / 0 / 0 |
| County-first maximum spanning tree | 214727 | 0 / 0 / 0 |

The numerator is `abs(2*p-P)` for this two-seat RI case, not a percentage.
The maximum allowed numerator at 1000 ppm is 1097.379 for P=1,097,379.
Empty results are infeasibility **within each single-tree candidate family**,
not global infeasibility or a legal conclusion. No county-weight response or
county-fragment improvement can be measured without feasible candidates.

## Paired design

Both arms use deterministic Kruskal tree construction, descending original
boundary weight and ascending endpoint indices for ties. The county-first arm
adds a leading sort key that processes all within-county edges before
cross-county edges. This favors internal county connectivity during construction;
it is not a direct district-fragment objective or a guarantee of county preservation.

Each arm builds one unrooted tree and tests every tree edge. Rooting a fixed
tree differently does not create additional unoriented edge-cut partitions.
The earlier five-DFS-tree experiment is historical context, not an equal-budget
paired arm here. Every new candidate would be scored on the **original graph**,
not just the selected tree edges. Bands and alpha 0, 0.5, 1, 2, 4, 8 are unchanged;
bridges and retained zero-population units are unchanged. Production rules and
earlier artifacts were not modified.

## Interpretation and next gate

The pre-run success test required fewer split counties among winners at a
common feasible band/alpha. It was not met: there were no feasible comparisons.
Do not tune these bands after observing this failure. The next candidate
generator should incorporate population during construction or support
connectivity-preserving boundary adjustment. County-aware edge ordering alone
is insufficient in these two tested trees. This does not exhaust county-aware
search or establish anything about its broader effectiveness.

## Artifacts and verification

- [Manifest](manifest.json): locally written before execution, hashes and settings.
- [Analysis](analysis.json): both tree hashes, cross-county tree-edge counts,
  all-cut minimum deviations, and every empty band/alpha cell.

```powershell
python -m unittest discover -s scripts/research -p test_ri_county_tree.py
python scripts/research/run_ri_county_tree.py run --manifest docs/experiments/ri-county-tree-2020/manifest.json --out target/ri-county-tree-new.json
```

Four tests check county-first edge preference, scoring against the original
graph, insertion-order independence, and rejection of disconnected graphs.
Execution is serial without an enforced resource cap. Hashes are local provenance,
not external timestamp proofs. No complete-plan package, optimality certificate,
legal-compliance or neutrality claim is produced.
