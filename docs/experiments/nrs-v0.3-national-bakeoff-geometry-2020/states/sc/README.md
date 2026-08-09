# NRS v0.3 SC 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.015410506 | 0.027238872 | +0.011828365 |
| Exact Reock | 0.258304029 | 0.375097986 | +0.116793957 |
| Convex-hull ratio | 0.605074150 | 0.725856952 | +0.120782802 |
| Schwartzberg | 8.741553287 | 7.321850740 | -1.419702547 |

Both plans are dissolved from the same 142,433 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state SC `
  --state-fips 45 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/sc/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_45_tabblock20/tl_2020_45_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_45_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/sc
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/sc
```

## Claim Boundary

Descriptive compactness measurements of SC block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
