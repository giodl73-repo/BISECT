# NRS v0.3 MN 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.018534096 | 0.042885277 | +0.024351181 |
| Exact Reock | 0.287475509 | 0.402815896 | +0.115340388 |
| Convex-hull ratio | 0.651693502 | 0.740177260 | +0.088483758 |
| Schwartzberg | 8.490951108 | 6.678237546 | -1.812713562 |

Both plans are dissolved from the same 190,280 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MN `
  --state-fips 27 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/mn/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_27_tabblock20/tl_2020_27_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_27_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mn
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mn
```

## Claim Boundary

Descriptive compactness measurements of MN block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
