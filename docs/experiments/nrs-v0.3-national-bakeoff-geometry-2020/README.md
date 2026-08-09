# NRS v0.3 National 2020 Tier 2 Geometry Bakeoff

**Status:** pass

| District-weighted mean | NRS v0.3 | Official CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby Popper | 0.025380339 | 0.061087819 | +0.035707480 |
| Reock | 0.245552262 | 0.377238283 | +0.131686021 |
| Convex Hull Ratio | 0.557042985 | 0.702220801 | +0.145177815 |
| Schwartzberg | 7.847347095 | 5.656288737 | -2.191058358 |

The package covers 50 States, 435 NRS districts,
435 comparator districts, and
7,889,194 retained land-containing Census blocks.
State-weighted results, all district rows, all State rows, and failures are in
the machine-readable package.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_bakeoff_geometry_national.py `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020
python scripts/research/verify_nrs_bakeoff_geometry_national.py `
  docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020
```

## Claim Boundary

Descriptive national compactness summaries for 2020 NRS v0.3 and official CD118 assignments projected to identical retained Census-block geometry; no original-linework, compactness-superiority, fairness, intent, VRA, legal, community, robustness, optimality, causal, or adoption claim.
