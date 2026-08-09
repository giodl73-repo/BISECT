# NRS v0.3 IN 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.025750923 | 0.083789451 | +0.058038528 |
| Exact Reock | 0.241156469 | 0.464475962 | +0.223319493 |
| Convex-hull ratio | 0.591302249 | 0.827453962 | +0.236151713 |
| Schwartzberg | 6.462495977 | 4.178130389 | -2.284365589 |

Both plans are dissolved from the same 200,959 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state IN `
  --state-fips 18 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/in/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_18_tabblock20/tl_2020_18_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_18_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/in
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/in
```

## Claim Boundary

Descriptive compactness measurements of IN block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
