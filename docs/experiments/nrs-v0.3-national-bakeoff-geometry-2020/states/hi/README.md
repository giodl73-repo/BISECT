# NRS v0.3 HI 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.059784646 | 0.059838139 | +0.000053492 |
| Exact Reock | 0.058941718 | 0.158632332 | +0.099690614 |
| Convex-hull ratio | 0.198666331 | 0.344642923 | +0.145976592 |
| Schwartzberg | 4.208748212 | 4.200207993 | -0.008540219 |

Both plans are dissolved from the same 13,783 retained Census block
polygons in `EPSG:3759`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state HI `
  --state-fips 15 --year 2020 --projection EPSG:3759 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/hi/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_15_tabblock20/tl_2020_15_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_15_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/hi
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/hi
```

## Claim Boundary

Descriptive compactness measurements of HI block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
