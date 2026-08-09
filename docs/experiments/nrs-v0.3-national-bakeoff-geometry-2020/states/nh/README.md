# NRS v0.3 NH 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.021040990 | 0.017924333 | -0.003116657 |
| Exact Reock | 0.333697394 | 0.244544268 | -0.089153127 |
| Convex-hull ratio | 0.698928429 | 0.641993247 | -0.056935182 |
| Schwartzberg | 7.364439660 | 7.504925976 | +0.140486317 |

Both plans are dissolved from the same 30,498 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state NH `
  --state-fips 33 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/nh/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_33_tabblock20/tl_2020_33_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_33_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nh
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/nh
```

## Claim Boundary

Descriptive compactness measurements of NH block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
