# NRS v0.3 WV 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.009197637 | 0.011798198 | +0.002600561 |
| Exact Reock | 0.290383947 | 0.369987004 | +0.079603057 |
| Convex-hull ratio | 0.608086319 | 0.644993076 | +0.036906757 |
| Schwartzberg | 10.433306029 | 9.412978159 | -1.020327871 |

Both plans are dissolved from the same 69,008 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state WV `
  --state-fips 54 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/wv/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_54_tabblock20/tl_2020_54_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_54_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wv
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wv
```

## Claim Boundary

Descriptive compactness measurements of WV block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
