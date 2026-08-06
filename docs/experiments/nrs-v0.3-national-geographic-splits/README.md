# NRS v0.3 National Geographic Split Audit

**Status:** governed analyzer output with independent aggregate verification

| Cycle | Counties split | County excess pieces | Tracts split | Tract excess pieces |
|---:|---:|---:|---:|---:|
| 2000 | 1,812 / 3,140 | 3,002 | 17,268 / 65,255 | 18,791 |
| 2010 | 1,819 / 3,142 | 3,140 | 18,800 / 72,878 | 20,682 |
| 2020 | 1,823 / 3,142 | 3,057 | 20,288 / 84,208 | 21,849 |

Raw counts are descriptive within each Census geography vintage. They are not cross-cycle improvement measures because county and tract definitions and district allocations change. Every county and tract district set is in `geography-projection.csv`; every State source assignment hash and committed recursive-structure snapshot is bound in `analysis.json`. Raw tree transport hashes match in 149 of 150 State-cycle packages; Maryland 2010 uses the protocol's metadata-only diagnostic exception.

## Evaluation readiness

| Metric family | Status |
|---|---|
| population-and-contiguity | `complete-in-national-baselines` |
| county-and-tract-splits | `complete-in-this-package` |
| municipality-splits | `not-computed` |
| geometric-compactness | `not-computed` |
| racial-and-language-opportunity | `not-computed` |
| partisan-diagnostics | `not-computed` |
| 100-seed-sensitivity | `not-computed` |
| ensemble-percentiles | `not-computed` |

Unavailable metrics were not replaced with zeros or post-outcome proxies. See `analysis.json` for the exact reason attached to each status.

## Rebuild and verify

```powershell
python scripts/research/analyze_nrs_geographic_splits.py `
  --cycle 2000=<run-dir-2000> --cycle 2010=<run-dir-2010> `
  --cycle 2020=<run-dir-2020> --out-dir <new-output-directory>
python scripts/research/verify_nrs_geographic_splits.py <new-output-directory>
```

## Claim boundary

Complete, hash-bound county and tract intersection counts for the governed NRS v0.3 assignments in 2000, 2010, and 2020; no compactness-superiority, municipality, community, demographic, partisan, VRA, legal-validity, cross-cycle-improvement, optimality, or adoption claim.
