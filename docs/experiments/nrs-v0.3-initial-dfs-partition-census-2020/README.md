# NRS v0.3 Initial DFS Partition Census

**Status:** pass

| Measure | Result |
|---|---:|
| Multi-district State roots | 44 |
| Accepted replays | 44 |
| Assignment-preserving replays | 44 |
| Objective-preserving replays | 44 |
| Roots with orientation-only ties | 29 |
| Roots with multiple physical cuts | 0 |

Physical-cut opportunity States: none

The complete oriented and unlabeled count ledger is in `state-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_dfs_partition_census.py `
  --output-dir docs/experiments/nrs-v0.3-initial-dfs-partition-census-2020
python scripts/research/verify_nrs_dfs_partition_census.py `
  docs/experiments/nrs-v0.3-initial-dfs-partition-census-2020
```

## Claim Boundary

Unlabeled initial root-0 DFS tree-edge bipartitions for governed 2020 State roots only; no seed-invariant label, child-node, fallback, full-plan, national robustness, optimality, or legal-quality claim.
