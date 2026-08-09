# NRS v0.3 AZ 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.056307972 | 0.163049851 | +0.106741879 |
| Exact Reock | 0.313760614 | 0.414257511 | +0.100496897 |
| Convex-hull ratio | 0.608506522 | 0.739240513 | +0.130733991 |
| Schwartzberg | 4.648533285 | 2.661063892 | -1.987469393 |

Both plans are dissolved from the same 154,209 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state AZ `
  --state-fips 04 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/az/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_04_tabblock20/tl_2020_04_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_04_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/az
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/az
```

## Claim Boundary

Descriptive compactness measurements of AZ block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
