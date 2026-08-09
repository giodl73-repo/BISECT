# NRS v0.3 WY 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.078277263 | 0.078277263 | +0.000000000 |
| Exact Reock | 0.616469281 | 0.616469281 | +0.000000000 |
| Convex-hull ratio | 0.989963197 | 0.989963197 | +0.000000000 |
| Schwartzberg | 3.574227430 | 3.574227430 | +0.000000000 |

Both plans are dissolved from the same 52,881 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state WY `
  --state-fips 56 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/wy/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_56_tabblock20/tl_2020_56_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_56_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wy
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wy
```

## Claim Boundary

Descriptive compactness measurements of WY block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
