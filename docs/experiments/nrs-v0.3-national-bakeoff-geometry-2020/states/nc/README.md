# NRS v0.3 NC 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.019735170 | 0.045246376 | +0.025511206 |
| Exact Reock | 0.205797889 | 0.447219127 | +0.241421238 |
| Convex-hull ratio | 0.553758812 | 0.771499230 | +0.217740418 |
| Schwartzberg | 8.466738526 | 5.778505193 | -2.688233333 |

Both plans are dissolved from the same 229,625 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state NC `
  --state-fips 37 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/nc/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_37_tabblock20/tl_2020_37_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_37_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nc
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nc
```

## Claim Boundary

Descriptive compactness measurements of NC block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
