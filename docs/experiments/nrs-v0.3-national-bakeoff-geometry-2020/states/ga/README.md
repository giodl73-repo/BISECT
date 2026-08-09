# NRS v0.3 GA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.022916115 | 0.051539308 | +0.028623193 |
| Exact Reock | 0.276585140 | 0.435132871 | +0.158547731 |
| Convex-hull ratio | 0.557942990 | 0.759218354 | +0.201275364 |
| Schwartzberg | 7.766496878 | 5.377718677 | -2.388778201 |

Both plans are dissolved from the same 226,723 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state GA `
  --state-fips 13 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ga/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_13_tabblock20/tl_2020_13_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_13_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ga
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ga
```

## Claim Boundary

Descriptive compactness measurements of GA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
