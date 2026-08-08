# NRS v0.3 MO 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 245,402 | 245,402 | 0 |
| County split units | 54 | 9 | -45 |
| Tract split units | 369 | 76 | -293 |

After maximum-overlap district-label matching, 143,806 of
245,402 blocks match (58.600174%); the
remaining 101,596 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/mo/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_29_tabblock20/tl_2020_29_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_29_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/mo
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/mo
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
