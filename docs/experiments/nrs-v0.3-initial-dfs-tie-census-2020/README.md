# NRS v0.3 Initial DFS Tie Census

**Status:** pass

| Measure | Result |
|---|---:|
| Multi-district State roots | 44 |
| Accepted replays | 44 |
| Assignment-preserving replays | 44 |
| Objective-preserving replays | 44 |
| Roots with initial seed-sensitive tie opportunity | 29 |

Opportunity States: AR, CA, CO, FL, GA, HI, IA, ID, KS, KY, LA, MD, ME, MN, MO, MS, MT, NC, NH, NJ, NV, NY, OR, RI, TX, UT, WA, WI, WV

The full State ledger and both candidate counts are in `state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_dfs_tie_census.py `
  --output-dir docs/experiments/nrs-v0.3-initial-dfs-tie-census-2020
python scripts/research/verify_nrs_dfs_tie_census.py `
  docs/experiments/nrs-v0.3-initial-dfs-tie-census-2020
```

## Claim Boundary

Initial root-0 DFS candidate multiplicity for governed 2020 State roots only; no alternate-root fallback, child-node, final seed-sensitivity, national robustness, optimality, or legal-quality claim.
