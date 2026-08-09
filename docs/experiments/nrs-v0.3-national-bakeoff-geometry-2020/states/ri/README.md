# NRS v0.3 RI 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.013950260 | 0.019738826 | +0.005788566 |
| Exact Reock | 0.296212240 | 0.228464276 | -0.067747964 |
| Convex-hull ratio | 0.530027828 | 0.568569953 | +0.038542125 |
| Schwartzberg | 8.498963187 | 7.458617276 | -1.040345911 |

Both plans are dissolved from the same 24,831 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state RI `
  --state-fips 44 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_44_tabblock20/tl_2020_44_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_44_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ri
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ri
```

## Claim Boundary

Descriptive compactness measurements of Rhode Island block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
