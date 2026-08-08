# NRS v0.3 Cross-Census Complete-Tree DFS Census

**Status:** pass

| Measure | Result |
|---|---:|
| Census years | 2000, 2010 |
| Multi-district State packages | 86 |
| Governed split nodes | 770 |
| Assignment-preserving States | 86 |
| Assignment-preserving nodes | 770 |
| Objective-preserving nodes | 770 |
| Nodes with multiple physical initial cuts | 0 |
| Nodes activating v0.2 fallback | 7 |
| Nodes activating v0.3 fallback | 1 |

The complete State and node ledgers are in `state-results.csv` and
`node-results.csv`.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_cross_census_dfs.py `
  --output-dir docs/experiments/nrs-v0.3-cross-census-complete-tree-dfs
python scripts/research/verify_nrs_cross_census_dfs.py `
  docs/experiments/nrs-v0.3-cross-census-complete-tree-dfs
```

## Claim Boundary

Governed 2000/2010 complete-tree initial DFS partition and fallback activation diagnostics only; no seed-invariant label or plan, national robustness, optimality, partisan, legal-quality, or assignment-overlap claim.
