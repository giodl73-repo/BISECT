# NRS v0.3 MI 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.012896629 | 0.065540137 | +0.052643508 |
| Exact Reock | 0.147038428 | 0.400549191 | +0.253510763 |
| Convex-hull ratio | 0.475137278 | 0.749104362 | +0.273967084 |
| Schwartzberg | 9.485360297 | 5.901392895 | -3.583967402 |

Both plans are dissolved from the same 244,398 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MI `
  --state-fips 26 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/mi/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_26_tabblock20/tl_2020_26_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_26_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mi
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mi
```

## Claim Boundary

Descriptive compactness measurements of MI block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
