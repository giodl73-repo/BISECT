# NRS v0.3 NM 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.068111477 | 0.124633492 | +0.056522015 |
| Exact Reock | 0.358555547 | 0.400583757 | +0.042028209 |
| Convex-hull ratio | 0.736938835 | 0.728266446 | -0.008672388 |
| Schwartzberg | 4.082517905 | 2.928845919 | -1.153671986 |

Both plans are dissolved from the same 105,847 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state NM `
  --state-fips 35 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/nm/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_35_tabblock20/tl_2020_35_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_35_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nm
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nm
```

## Claim Boundary

Descriptive compactness measurements of NM block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
