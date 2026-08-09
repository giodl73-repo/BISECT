# NRS v0.3 ME 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.006521846 | 0.005567535 | -0.000954311 |
| Exact Reock | 0.301550020 | 0.358009950 | +0.056459930 |
| Convex-hull ratio | 0.695539238 | 0.621860511 | -0.073678728 |
| Schwartzberg | 12.748655646 | 13.404701090 | +0.656045444 |

Both plans are dissolved from the same 43,179 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state ME `
  --state-fips 23 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/me/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_23_tabblock20/tl_2020_23_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_23_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/me
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/me
```

## Claim Boundary

Descriptive compactness measurements of ME block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
