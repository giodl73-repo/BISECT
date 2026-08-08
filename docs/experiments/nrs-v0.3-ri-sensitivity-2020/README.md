# NRS v0.3 Rhode Island 100-Seed Sensitivity

**Status:** complete

| Measure | Result |
|---|---:|
| Diagnostic indices | 100 |
| Accepted | 100 |
| Rejected | 0 |
| Unique assignments | 1 |
| Exact benchmark reproductions | 100 |
| Population-tolerance passes | 100 |
| Benchmark objective rank interval | 1-101 / 101 |
| Minimum benchmark agreement | 100.000000% |
| Median benchmark agreement | 100.000000% |
| Mean benchmark agreement | 100.000000% |
| Maximum benchmark agreement | 100.000000% |

The governed benchmark remains authoritative. Diagnostic assignments are
packed in `assignments.bin`; offsets and metrics are recorded in
`seed-results.csv`.

The NRS v0.3 path uses the seeded METIS assignment only in a moved-population
tie-break among equal-deviation deterministic DFS cut candidates. The complete
invariance observed here is specific to this Rhode Island root and mechanism;
it is not a national robustness claim.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_ri_sensitivity.py `
  --output-dir docs/experiments/nrs-v0.3-ri-sensitivity-2020
python scripts/research/verify_nrs_ri_sensitivity.py `
  docs/experiments/nrs-v0.3-ri-sensitivity-2020
```

## Claim Boundary

Rhode Island 2020 root sensitivity diagnostic only; no national robustness, ensemble convergence, random-sampling, partisan, compactness, demographic, VRA, legal-validity, optimality, or benchmark-replacement claim.
