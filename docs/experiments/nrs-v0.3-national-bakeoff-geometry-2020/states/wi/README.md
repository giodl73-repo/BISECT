# NRS v0.3 WI 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.014682790 | 0.034343355 | +0.019660566 |
| Exact Reock | 0.244120609 | 0.452862714 | +0.208742104 |
| Convex-hull ratio | 0.571964696 | 0.749135624 | +0.177170928 |
| Schwartzberg | 10.141965523 | 7.808716191 | -2.333249333 |

Both plans are dissolved from the same 193,379 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state WI `
  --state-fips 55 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/wi/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_55_tabblock20/tl_2020_55_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_55_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wi
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wi
```

## Claim Boundary

Descriptive compactness measurements of WI block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
