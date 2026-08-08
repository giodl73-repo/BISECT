# NRS v0.3 KY 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 125,853 | 125,853 | 0 |
| County split units | 69 | 6 | -63 |
| Tract split units | 328 | 23 | -305 |

After maximum-overlap district-label matching, 62,951 of
125,853 blocks match (50.019467%); the
remaining 62,902 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ky/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_21_tabblock20/tl_2020_21_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_21_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ky
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ky
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
