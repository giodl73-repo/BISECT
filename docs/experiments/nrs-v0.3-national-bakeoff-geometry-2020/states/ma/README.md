# NRS v0.3 MA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.011483604 | 0.021064485 | +0.009580881 |
| Exact Reock | 0.194204624 | 0.322967344 | +0.128762720 |
| Convex-hull ratio | 0.459773258 | 0.599277299 | +0.139504041 |
| Schwartzberg | 9.752377107 | 7.539873671 | -2.212503436 |

Both plans are dissolved from the same 102,527 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MA `
  --state-fips 25 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ma/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_25_tabblock20/tl_2020_25_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_25_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ma
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ma
```

## Claim Boundary

Descriptive compactness measurements of MA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
