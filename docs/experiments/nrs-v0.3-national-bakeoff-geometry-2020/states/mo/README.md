# NRS v0.3 MO 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.014153138 | 0.053742224 | +0.039589086 |
| Exact Reock | 0.250341315 | 0.449352423 | +0.199011108 |
| Convex-hull ratio | 0.566120979 | 0.777669182 | +0.211548204 |
| Schwartzberg | 9.734000602 | 6.548157065 | -3.185843537 |

Both plans are dissolved from the same 245,402 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MO `
  --state-fips 29 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/mo/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_29_tabblock20/tl_2020_29_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_29_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mo
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/mo
```

## Claim Boundary

Descriptive compactness measurements of MO block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
