# NRS v0.3 WA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.018224921 | 0.041189241 | +0.022964320 |
| Exact Reock | 0.274075899 | 0.369835837 | +0.095759938 |
| Convex-hull ratio | 0.532413648 | 0.710282272 | +0.177868625 |
| Schwartzberg | 7.733231695 | 5.394802702 | -2.338428993 |

Both plans are dissolved from the same 149,816 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state WA `
  --state-fips 53 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/wa/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_53_tabblock20/tl_2020_53_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_53_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wa
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/wa
```

## Claim Boundary

Descriptive compactness measurements of WA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
