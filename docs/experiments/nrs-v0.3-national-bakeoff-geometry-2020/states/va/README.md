# NRS v0.3 VA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.009945892 | 0.023201964 | +0.013256071 |
| Exact Reock | 0.215278444 | 0.354150670 | +0.138872226 |
| Convex-hull ratio | 0.519037258 | 0.696974384 | +0.177937126 |
| Schwartzberg | 11.228109507 | 8.527692980 | -2.700416528 |

Both plans are dissolved from the same 155,892 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state VA `
  --state-fips 51 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/va/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_51_tabblock20/tl_2020_51_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_51_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/va
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/va
```

## Claim Boundary

Descriptive compactness measurements of VA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
