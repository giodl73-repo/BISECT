# NRS v0.3 CO 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.042861425 | 0.155834363 | +0.112972938 |
| Exact Reock | 0.342243166 | 0.399055154 | +0.056811988 |
| Convex-hull ratio | 0.613000993 | 0.758450574 | +0.145449581 |
| Schwartzberg | 5.444884385 | 2.925911508 | -2.518972876 |

Both plans are dissolved from the same 139,052 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state CO `
  --state-fips 08 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/co/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_08_tabblock20/tl_2020_08_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_08_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/co
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/co
```

## Claim Boundary

Descriptive compactness measurements of CO block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
