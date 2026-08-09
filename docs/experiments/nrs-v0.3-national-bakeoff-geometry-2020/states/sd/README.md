# NRS v0.3 SD 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.035556245 | 0.035556245 | +0.000000000 |
| Exact Reock | 0.490987873 | 0.490987873 | +0.000000000 |
| Convex-hull ratio | 0.921524198 | 0.921524198 | +0.000000000 |
| Schwartzberg | 5.303249430 | 5.303249430 | +0.000000000 |

Both plans are dissolved from the same 69,859 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state SD `
  --state-fips 46 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/sd/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_46_tabblock20/tl_2020_46_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_46_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/sd
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/sd
```

## Claim Boundary

Descriptive compactness measurements of SD block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
