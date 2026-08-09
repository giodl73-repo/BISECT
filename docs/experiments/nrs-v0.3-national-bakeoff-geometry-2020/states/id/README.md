# NRS v0.3 ID 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.013351476 | 0.020389429 | +0.007037954 |
| Exact Reock | 0.195510291 | 0.391212240 | +0.195701949 |
| Convex-hull ratio | 0.544448466 | 0.769246458 | +0.224797992 |
| Schwartzberg | 8.683452607 | 7.037367730 | -1.646084877 |

Both plans are dissolved from the same 78,652 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state ID `
  --state-fips 16 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/id/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_16_tabblock20/tl_2020_16_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_16_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/id
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/id
```

## Claim Boundary

Descriptive compactness measurements of ID block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
