# NRS v0.3 NE 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.055809131 | 0.107682949 | +0.051873817 |
| Exact Reock | 0.279864590 | 0.415762138 | +0.135897548 |
| Convex-hull ratio | 0.596544878 | 0.805593492 | +0.209048615 |
| Schwartzberg | 4.781059272 | 3.356367505 | -1.424691767 |

Both plans are dissolved from the same 118,223 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state NE `
  --state-fips 31 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ne/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_31_tabblock20/tl_2020_31_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_31_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ne
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ne
```

## Claim Boundary

Descriptive compactness measurements of NE block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
