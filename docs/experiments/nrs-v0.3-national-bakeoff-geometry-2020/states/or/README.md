# NRS v0.3 OR 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.030021977 | 0.078694995 | +0.048673018 |
| Exact Reock | 0.215437499 | 0.423298857 | +0.207861358 |
| Convex-hull ratio | 0.577699328 | 0.751358264 | +0.173658935 |
| Schwartzberg | 6.210959218 | 4.067854485 | -2.143104732 |

Both plans are dissolved from the same 128,036 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state OR `
  --state-fips 41 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/or/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_41_tabblock20/tl_2020_41_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_41_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/or
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/or
```

## Claim Boundary

Descriptive compactness measurements of OR block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
