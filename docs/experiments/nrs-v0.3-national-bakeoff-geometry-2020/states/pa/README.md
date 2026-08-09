# NRS v0.3 PA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.021281417 | 0.053447093 | +0.032165676 |
| Exact Reock | 0.233279287 | 0.453153640 | +0.219874353 |
| Convex-hull ratio | 0.567071698 | 0.770040297 | +0.202968599 |
| Schwartzberg | 7.588978410 | 4.663733202 | -2.925245208 |

Both plans are dissolved from the same 329,888 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state PA `
  --state-fips 42 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/pa/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_42_tabblock20/tl_2020_42_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_42_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/pa
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/pa
```

## Claim Boundary

Descriptive compactness measurements of PA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
