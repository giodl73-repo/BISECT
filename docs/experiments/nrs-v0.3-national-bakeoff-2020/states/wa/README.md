# NRS v0.3 WA 2020 Tier 1 Bakeoff

**Status:** governed accepted evidence slice

| Measure | NRS v0.3 | Enacted CD118 comparator | Comparator minus NRS |
|---|---:|---:|---:|
| Blocks | 149,816 | 149,816 | 0 |
| County split units | 33 | 7 | -26 |
| Tract split units | 367 | 129 | -238 |

After maximum-overlap district-label matching, 89,207 of
149,816 blocks match (59.544374%); the
remaining 60,609 blocks are assigned to different
districts.

## Rebuild And Verify

```powershell
python scripts/research/analyze_nrs_bakeoff_slice.py --state RI --state-fips 44 `
  --year 2020 --projection EPSG:32130 `
  --nrs-assignment runs/nrs-v0.3/neutral-analysis/national-2020/states/wa/package/baseline_assignments.json `
  --block-shapefile data/2020/tiger/blocks/tl_2020_53_tabblock20/tl_2020_53_tabblock20.shp `
  --comparator-source data/enacted_districts/tl_2020_53_cd118.zip `
  --comparator-state-column STATEFP20 `
  --comparator-district-column CD118FP `
  --comparator-session-column CDSESSN `
  --expected-session 118 `
  --output-dir docs/experiments/nrs-v0.3-national-bakeoff-2020/states/wa
python scripts/research/verify_nrs_bakeoff_slice.py docs/experiments/nrs-v0.3-national-bakeoff-2020/states/wa
```

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
