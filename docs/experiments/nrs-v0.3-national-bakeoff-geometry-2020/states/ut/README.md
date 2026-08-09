# NRS v0.3 UT 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.050627390 | 0.175644806 | +0.125017416 |
| Exact Reock | 0.327622374 | 0.430384405 | +0.102762031 |
| Convex-hull ratio | 0.583493580 | 0.769623471 | +0.186129891 |
| Schwartzberg | 4.587970192 | 2.411097431 | -2.176872761 |

Both plans are dissolved from the same 70,883 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state UT `
  --state-fips 49 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ut/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_49_tabblock20/tl_2020_49_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_49_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ut
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ut
```

## Claim Boundary

Descriptive compactness measurements of UT block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
