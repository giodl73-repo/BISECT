# NRS v0.3 AL 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.011138387 | 0.017270972 | +0.006132585 |
| Exact Reock | 0.306196613 | 0.375032696 | +0.068836083 |
| Convex-hull ratio | 0.623542765 | 0.698798804 | +0.075256038 |
| Schwartzberg | 10.000851767 | 8.215364948 | -1.785486819 |

Both plans are dissolved from the same 179,979 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state AL `
  --state-fips 01 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/al/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_01_tabblock20/tl_2020_01_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_01_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/al
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/al
```

## Claim Boundary

Descriptive compactness measurements of AL block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
