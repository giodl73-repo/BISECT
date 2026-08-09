# NRS v0.3 MD 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.016814690 | 0.051720160 | +0.034905469 |
| Exact Reock | 0.225537138 | 0.303443836 | +0.077906697 |
| Convex-hull ratio | 0.453132985 | 0.655357959 | +0.202224975 |
| Schwartzberg | 10.465998114 | 7.173804266 | -3.292193848 |

Both plans are dissolved from the same 79,629 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MD `
  --state-fips 24 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/md/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_24_tabblock20/tl_2020_24_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_24_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/md
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/md
```

## Claim Boundary

Descriptive compactness measurements of MD block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
