# NRS v0.3 MS 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.018447400 | 0.041803057 | +0.023355657 |
| Exact Reock | 0.279317574 | 0.406170548 | +0.126852974 |
| Convex-hull ratio | 0.599168207 | 0.771697272 | +0.172529065 |
| Schwartzberg | 7.710681157 | 6.039786851 | -1.670894306 |

Both plans are dissolved from the same 109,351 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state MS `
  --state-fips 28 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ms/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_28_tabblock20/tl_2020_28_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_28_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ms
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ms
```

## Claim Boundary

Descriptive compactness measurements of MS block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
