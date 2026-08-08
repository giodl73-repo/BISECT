# NRS v0.3 Complete-Tree DFS Census

**Status:** pass

| Measure | Result |
|---|---:|
| Multi-district States | 44 |
| Governed split nodes | 385 |
| Assignment-preserving States | 44 |
| Assignment-preserving nodes | 385 |
| Objective-preserving nodes | 385 |
| Nodes with multiple physical initial cuts | 0 |
| Nodes activating v0.2 fallback | 0 |
| Nodes activating v0.3 fallback | 0 |

The complete State and node ledgers are in `state-results.csv` and
`node-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_tree_dfs_census.py `
  --output-dir docs/experiments/nrs-v0.3-complete-tree-dfs-census-2020
python scripts/research/verify_nrs_tree_dfs_census.py `
  docs/experiments/nrs-v0.3-complete-tree-dfs-census-2020
```

## Claim Boundary

Governed 2020 complete-tree initial DFS partition and fallback activation diagnostics only; no seed-invariant label or plan, cross-census, national robustness, optimality, partisan, or legal-quality claim.
