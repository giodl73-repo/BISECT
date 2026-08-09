# NRS v0.3 ND 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.025174089 | 0.025174089 | +0.000000000 |
| Exact Reock | 0.527939615 | 0.527939615 | +0.000000000 |
| Convex-hull ratio | 0.972532183 | 0.972532183 | +0.000000000 |
| Schwartzberg | 6.302648903 | 6.302648903 | +0.000000000 |

Both plans are dissolved from the same 82,443 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state ND `
  --state-fips 38 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/nd/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_38_tabblock20/tl_2020_38_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_38_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nd
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nd
```

## Claim Boundary

Descriptive compactness measurements of ND block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
