# NRS v0.3 IL 2020 Tier 2 Geometry Bakeoff

**Status:** governed accepted evidence slice

| Unweighted district mean | NRS v0.3 | Enacted CD118 block projection | Comparator minus NRS |
|---|---:|---:|---:|
| Polsby--Popper | 0.040500708 | 0.044353593 | +0.003852885 |
| Exact Reock | 0.268208941 | 0.287696877 | +0.019487935 |
| Convex-hull ratio | 0.589167583 | 0.563139873 | -0.026027710 |
| Schwartzberg | 5.908368505 | 5.119726925 | -0.788641579 |

Both plans are dissolved from the same 363,324 retained Census block
polygons in `EPSG:5070`. The comparator geometry is its Tier 1 block
projection, not the original enacted polygon linework.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_geometry_slice.py --state IL `
  --state-fips 17 --year 2020 --projection EPSG:5070 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/il/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_17_tabblock20/tl_2020_17_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_17_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 --output-dir docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/il
python scripts/research/verify_nrs_bakeoff_geometry_slice.py docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/states/il
```

## Claim Boundary

Descriptive compactness measurements of IL block-projected NRS v0.3 and enacted CD118 assignments under one frozen geometry contract; no compactness superiority, fairness, intent, VRA, legal-validity, community, robustness, optimality, or adoption claim.
