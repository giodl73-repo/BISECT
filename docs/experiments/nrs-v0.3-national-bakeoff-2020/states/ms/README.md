# NRS v0.3 MS 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 109,351 | 109,351 | 0 |
| County split units | 46 | 4 | -42 |
| Tract split units | 154 | 18 | -136 |

After maximum-overlap district-label matching, 62,099 of
109,351 blocks match (56.788690%); the
remaining 47,252 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/ms/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_28_tabblock20/tl_2020_28_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_28_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ms
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/ms
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
