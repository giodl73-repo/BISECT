# NRS v0.3 TX 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.022294564 | 0.055989412 | +0.033694848 |
| Exact Reock | 0.217496144 | 0.326912557 | +0.109416413 |
| Convex-hull ratio | 0.530182793 | 0.645869588 | +0.115686795 |
| Schwartzberg | 7.668768694 | 5.170219510 | -2.498549185 |

Both plans are dissolved from the same 655,121 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state TX `
  --state-fips 48 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/tx/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_48_tabblock20/tl_2020_48_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_48_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/tx
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/tx
```

## Claim Boundary

Descriptive compactness measurements of TX block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
