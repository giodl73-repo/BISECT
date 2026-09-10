# Five-root county-response diagnostic

Date: 2026-09-10. Status: root-only diagnostic complete; full-plan W1/W3
bakeoffs remain outstanding. The source/input manifest was written before
execution, locally, without an external timestamp witness.

## Results

| State | Single-root population deviation numerator | Five-root numerator | Distinct per-tree minima in union | Surviving physical cuts | County-weight winner changed |
|---|---:|---:|---:|---:|---|
| RI | 1303 | 27 | 6 | 1 | No |
| IA | 1550 | 390 | 5 | 1 | No |
| NC | 243222 | 98 | 5 | 1 | No |
| TX | 1539 | 1539 | 4 | 1 | No |

The numerator is `abs(k_parent * left_population - k_left * parent_population)`;
it is not a population percentage or final-district deviation. Comparisons are
within State, with the same seat schedule. Five fixed roots improve this
first-cut objective in three States. They leave a single physical choice after
the pooled population filter in all four States.

Better population balance is not automatically better boundary quality:
RI's selected raw cut rises from 243,087,025 to 304,157,226; IA's rises from
1,462,896,021 to 1,786,892,349. NC's falls from 2,819,128,337 to 2,300,365,217;
TX's is unchanged. These are graph boundary-weight sums, not compactness scores.
The JSON also records counties divided by each candidate first cut. These are
not counts of counties divided by completed district plans.

## Method and limits

The roots are indices `floor(j*(n-1)/4)` for `j=0..4` in sorted GEOID order,
fixed before the run and not chosen from outcomes. Each root uses the current
mark-on-push DFS construction, with minimum population deviation retained per
tree. The expanded arm takes the union of those candidates and applies the
same strict population-first rule before boundary scoring. Keeping only each
tree's minimum is sufficient for this pooled strict-minimum objective, but
would **not** be sufficient for a later population-band experiment.

The JSON's `physical_candidates_before_population_filter` counts this union
of per-tree minima, not every tree-edge cut before any population filtering.
Physical cuts are deduplicated by SHA-256 of their canonical byte labels,
with unit zero always labeled zero. Zero-population units remain in identity
and subdivision counts. Graph bridges retain their source weights.

County scoring uses exact rational `raw_cut + alpha * within_county_cut`,
with additive alpha 0, 0.5, 1, 2, 4, 8. No RNG, METIS rerun, repair,
recursive descendant generation, or production-profile changes occur.
Connectivity follows from cutting an edge of the constructed spanning tree;
there is no separate production assignment/certificate replay here.

All four single-root controls replay their previous deviation, candidate
costs and alpha responses. Synthetic tests additionally check reversed-root
physical deduplication, strict population priority, zero-population identity,
and agreement with the control on a branched graph.

## Next gate

The next distinct objective arm should declare a population feasibility band
and score **all eligible tree-edge cuts**, not just each tree's population
minimum. Choose and freeze bands before execution; evaluate final leaf balance
and descendant feasibility before any complete-plan claim. A research band is
not a statement of legal compliance. Retain this strict-minimum search arm as
the control. More expressive search under the strict objective remains possible;
five roots do not establish that search has been exhausted.

## Artifacts and replay

- [Manifest](manifest.json): source/input hashes, root schedule, population rule,
  alpha schedule and execution policy.
- [Analysis](analysis.json): control checks, every root, pooled results and costs.

```powershell
python -m unittest discover -s scripts/research -p 'test_*county*.py'
python scripts/research/run_multiroot_county_diagnostic.py run --manifest docs/experiments/multiroot-county-response-2020/manifest.json --out target/multiroot-county-new.json
```

The runner rejects changed source/input hashes and existing outputs. Execution
is serial with a cooperative 1800-second deadline checked between roots; no
hard process timeout or memory cap is enforced. This diagnostic does not satisfy
the full-plan resource-isolation gate. A timeout preserves completed States but
not a partially processed State. Paths in the manifest reference the local M
vault. No neutrality, global optimality, legal-quality, or final-plan claim.
