# NRS v0.3 Fallback Candidate Census

**Status:** pass

| Measure | Result |
|---|---:|
| Replayed State packages | 6 |
| Activated stage/node pairs | 8 |
| Assignment-preserving States | 6 |
| Assignment-preserving stage/nodes | 8 |
| Objective-preserving stage/nodes | 8 |
| Stage/nodes with multiple tied physical partitions | 0 |

The exact stage and State ledgers are in `stage-results.csv` and
`state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_fallback_candidate_census.py `
  --output-dir docs/experiments/nrs-v0.3-fallback-candidate-census
python scripts/research/verify_nrs_fallback_candidate_census.py `
  docs/experiments/nrs-v0.3-fallback-candidate-census
```

## Claim Boundary

Candidates evaluated by the current v0.2 and v0.3 fallback algorithms at eight activated governed stage/node pairs only; no seed-invariant candidate generation, label, plan, robustness, optimality, partisan, or legal-quality claim.
