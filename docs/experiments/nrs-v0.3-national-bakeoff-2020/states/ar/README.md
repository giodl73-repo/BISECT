# NRS v0.3 AR 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 130,244 | 130,244 | 0 |
| County split units | 39 | 2 | -37 |
| Tract split units | 158 | 15 | -143 |

After maximum-overlap district-label matching, 80,715 of
130,244 blocks match (61.972145%); the
remaining 49,529 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ar/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_05_tabblock20/tl_2020_05_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_05_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ar
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ar
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
