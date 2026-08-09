# NRS v0.3 OH 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.023501078 | 0.063649818 | +0.040148740 |
| Exact Reock | 0.252435685 | 0.386666908 | +0.134231224 |
| Convex-hull ratio | 0.568956558 | 0.737190893 | +0.168234334 |
| Schwartzberg | 7.069680294 | 4.542753012 | -2.526927282 |

Both plans are dissolved from the same 270,293 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state OH `
  --state-fips 39 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/oh/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_39_tabblock20/tl_2020_39_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_39_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/oh
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/oh
```

## Claim Boundary

Descriptive compactness measurements of OH block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
