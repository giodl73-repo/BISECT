# NRS v0.3 FL 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.014772908 | 0.032450085 | +0.017677177 |
| Exact Reock | 0.225603775 | 0.390221433 | +0.164617658 |
| Convex-hull ratio | 0.500540029 | 0.723690062 | +0.223150034 |
| Schwartzberg | 9.928632607 | 7.326395974 | -2.602236633 |

Both plans are dissolved from the same 375,513 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state FL `
  --state-fips 12 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/fl/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_12_tabblock20/tl_2020_12_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_12_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/fl
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/fl
```

## Claim Boundary

Descriptive compactness measurements of FL block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
