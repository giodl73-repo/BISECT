# NRS v0.3 MT 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.035446203 | 0.042938066 | +0.007491862 |
| Exact Reock | 0.409155403 | 0.458518517 | +0.049363113 |
| Convex-hull ratio | 0.871550835 | 0.821415303 | -0.050135533 |
| Schwartzberg | 5.658224196 | 5.391222563 | -0.267001632 |

Both plans are dissolved from the same 85,584 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MT `
  --state-fips 30 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/mt/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_30_tabblock20/tl_2020_30_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_30_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mt
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mt
```

## Claim Boundary

Descriptive compactness measurements of MT block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
