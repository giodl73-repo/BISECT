# NRS v0.3 VT 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.010515922 | 0.010515922 | +0.000000000 |
| Exact Reock | 0.365687063 | 0.365687063 | +0.000000000 |
| Convex-hull ratio | 0.811038415 | 0.811038415 | +0.000000000 |
| Schwartzberg | 9.751609776 | 9.751609776 | +0.000000000 |

Both plans are dissolved from the same 22,856 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state VT `
  --state-fips 50 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/vt/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_50_tabblock20/tl_2020_50_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_50_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/vt
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/vt
```

## Claim Boundary

Descriptive compactness measurements of VT block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
