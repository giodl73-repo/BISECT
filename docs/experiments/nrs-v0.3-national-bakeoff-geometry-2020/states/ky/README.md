# NRS v0.3 KY 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.006908949 | 0.019877982 | +0.012969033 |
| Exact Reock | 0.258676615 | 0.379536072 | +0.120859457 |
| Convex-hull ratio | 0.533739335 | 0.685780635 | +0.152041300 |
| Schwartzberg | 12.724086578 | 9.672976972 | -3.051109607 |

Both plans are dissolved from the same 125,853 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state KY `
  --state-fips 21 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ky/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_21_tabblock20/tl_2020_21_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_21_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ky
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ky
```

## Claim Boundary

Descriptive compactness measurements of KY block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
