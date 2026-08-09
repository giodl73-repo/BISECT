# NRS v0.3 TN 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.019100740 | 0.042866418 | +0.023765678 |
| Exact Reock | 0.171134934 | 0.360864590 | +0.189729656 |
| Convex-hull ratio | 0.576478603 | 0.703423287 | +0.126944684 |
| Schwartzberg | 8.064333578 | 5.459574746 | -2.604758832 |

Both plans are dissolved from the same 176,552 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state TN `
  --state-fips 47 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/tn/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_47_tabblock20/tl_2020_47_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_47_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/tn
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/tn
```

## Claim Boundary

Descriptive compactness measurements of TN block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
