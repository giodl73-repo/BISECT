# NRS v0.3 OK 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.014875304 | 0.047208610 | +0.032333307 |
| Exact Reock | 0.226760303 | 0.418862426 | +0.192102124 |
| Convex-hull ratio | 0.648752315 | 0.739298412 | +0.090546097 |
| Schwartzberg | 9.017883647 | 6.258030705 | -2.759852942 |

Both plans are dissolved from the same 175,160 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state OK `
  --state-fips 40 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ok/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_40_tabblock20/tl_2020_40_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_40_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ok
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ok
```

## Claim Boundary

Descriptive compactness measurements of OK block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
