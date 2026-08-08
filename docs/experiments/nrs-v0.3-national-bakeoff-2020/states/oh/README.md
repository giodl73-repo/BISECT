# NRS v0.3 OH 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 270,293 | 270,293 | 0 |
| County split units | 65 | 14 | -51 |
| Tract split units | 784 | 113 | -671 |

After maximum-overlap district-label matching, 114,110 of
270,293 blocks match (42.217150%); the
remaining 156,183 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/oh/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_39_tabblock20/tl_2020_39_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_39_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/oh
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/oh
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
