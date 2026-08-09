# NRS v0.3 LA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.005021293 | 0.006797854 | +0.001776561 |
| Exact Reock | 0.188332949 | 0.313933228 | +0.125600279 |
| Convex-hull ratio | 0.426385346 | 0.544352480 | +0.117967134 |
| Schwartzberg | 15.707369082 | 13.133653282 | -2.573715800 |

Both plans are dissolved from the same 134,965 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state LA `
  --state-fips 22 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/la/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_22_tabblock20/tl_2020_22_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_22_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/la
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/la
```

## Claim Boundary

Descriptive compactness measurements of LA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
