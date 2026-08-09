# NRS v0.3 CA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.036149764 | 0.089980519 | +0.053830755 |
| Exact Reock | 0.232918362 | 0.337909174 | +0.104990812 |
| Convex-hull ratio | 0.534360498 | 0.656616470 | +0.122255973 |
| Schwartzberg | 5.709218554 | 3.854184149 | -1.855034404 |

Both plans are dissolved from the same 511,819 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state CA `
  --state-fips 06 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ca/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_06_tabblock20/tl_2020_06_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_06_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ca
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ca
```

## Claim Boundary

Descriptive compactness measurements of CA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
