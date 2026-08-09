# NRS v0.3 RI 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.013973094 | 0.019782611 | +0.005809518 |
| Exact Reock | 0.298088463 | 0.231058299 | -0.067030164 |
| Convex-hull ratio | 0.530032258 | 0.568561609 | +0.038529351 |
| Schwartzberg | 8.491570723 | 7.449400105 | -1.042170618 |

Both plans are dissolved from the same 24,831 retained Census block
polygons in `EPSG:32130`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state RI `
  --state-fips 44 --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_44_tabblock20/tl_2020_44_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_44_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-bakeoff-geometry-ri-2020
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-bakeoff-geometry-ri-2020
```

## Claim Boundary

Descriptive compactness measurements of Rhode Island block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
