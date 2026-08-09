# NRS v0.3 AR 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.008422625 | 0.016305382 | +0.007882757 |
| Exact Reock | 0.280041433 | 0.431968133 | +0.151926700 |
| Convex-hull ratio | 0.620916375 | 0.761559740 | +0.140643365 |
| Schwartzberg | 11.015486485 | 8.736412663 | -2.279073822 |

Both plans are dissolved from the same 130,244 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state AR `
  --state-fips 05 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ar/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_05_tabblock20/tl_2020_05_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_05_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ar
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ar
```

## Claim Boundary

Descriptive compactness measurements of AR block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
