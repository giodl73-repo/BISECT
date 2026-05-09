# POST-WRITE CHECK — M.2 Land Use Edge Weights via NLCD

**Date**: 2026-05-08
**Status**: READY FOR PANEL

## Validation Summary

| Phase | Result |
|-------|--------|
| Consistency | PASS (1 P1 fixed) |
| Contract | PASS (7/7 promises) |
| Referee sim | Major Revision → Minor Revision after fix |
| Abstract | ~175 words, daggered primary result |

## P1 Fixed

- **homer2020nlcd citation mismatch**: The bib entry cites the 2020 ISPRS journal paper on NLCD 2016 change analysis. This paper is used for the MRLC validation methodology description (85% accuracy claim) but is not the primary 2021 NLCD product citation. Fixed by adding `dewitz2023nlcd2021` (USGS data release doi:10.5066/P9OGBGM6) and co-citing in §1.

## P2 Items

- All-water tract degenerate case (N(t)=0) not handled in algorithm section

## Key Checks

- Hard-cut rule: consistently stated as `min(pct_water_u, pct_water_v) > 0.5 → w=0` in abstract, §3.3 eq.(3), §5 legal, and §6 conclusion. CONSISTENT.
- All quantitative predictions daggered throughout.
- L0 invariants: all 3 correctly specified.
- CLI flag `--weights-override land-use` stated once in §3.4.
- LODES URL: N/A (NLCD). NLCD URL correct (www.mrlc.gov/data/nlcd-2021-land-cover-conus).
