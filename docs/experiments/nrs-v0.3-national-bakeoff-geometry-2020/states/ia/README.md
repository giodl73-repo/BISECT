# NRS v0.3 IA 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.026895852 | 0.046474500 | +0.019578649 |
| Exact Reock | 0.286479576 | 0.448766025 | +0.162286449 |
| Convex-hull ratio | 0.671764922 | 0.739932734 | +0.068167812 |
| Schwartzberg | 6.166670454 | 4.815585468 | -1.351084986 |

Both plans are dissolved from the same 172,136 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state IA `
  --state-fips 19 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ia/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_19_tabblock20/tl_2020_19_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_19_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ia
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/ia
```

## Claim Boundary

Descriptive compactness measurements of IA block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
