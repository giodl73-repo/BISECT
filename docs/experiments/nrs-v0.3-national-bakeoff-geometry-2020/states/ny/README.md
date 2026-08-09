# NRS v0.3 NY 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.036289438 | 0.088026773 | +0.051737335 |
| Exact Reock | 0.243773518 | 0.368194852 | +0.124421334 |
| Convex-hull ratio | 0.540290932 | 0.693241012 | +0.152950080 |
| Schwartzberg | 6.412447578 | 4.400745414 | -2.011702164 |

Both plans are dissolved from the same 281,159 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state NY `
  --state-fips 36 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ny/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_36_tabblock20/tl_2020_36_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_36_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ny
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ny
```

## Claim Boundary

Descriptive compactness measurements of NY block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
