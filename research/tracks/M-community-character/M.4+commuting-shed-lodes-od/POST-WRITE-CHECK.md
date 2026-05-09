# POST-WRITE CHECK — M.4 Commuting Shed Similarity via LODES OD

**Date**: 2026-05-08
**Status**: READY FOR PANEL

## Validation Summary

| Phase | Result |
|-------|--------|
| Consistency | PASS |
| Contract | PASS (8/8 promises) |
| Referee sim | Accept |
| Abstract | ~165 words, daggered primary results |

## P1 Issues

None.

## P2 Items

- Normalization sentence in §3.1 ("normalization performed at compute time") is confusing since the Jaccard formula uses unnormalized counts and normalization is actually not needed. Clarification recommended.
- Conclusion does not state cross-state commuter omission as a limitation.
- Implementation should use HashMap<tract_id, u32> for sparse union iteration — not specified in paper.

## Key Checks

- Weighted Jaccard formula: `Σmin(D(h1,w), D(h2,w)) / Σmax(D(h1,w), D(h2,w))` — consistent in abstract, §3.2 eq.(2), §5 legal, and §6 conclusion.
- Worked example: min(100,80)+min(20,60)=100; max(100,80)+max(20,60)=160; 100/160=0.625. Arithmetic CORRECT.
- LODES URL: `https://lehd.ces.census.gov/data/lodes/LODES8/{state}/od/{state}_od_main_JT00_2020.csv.gz` — consistent with spec.
- All WI (≥70%) and NC (<5%) predictions are daggered.
- L0 invariants: all 3 correctly specified and provable.
- CLI flag: `--weights-override commuting-shed` stated in §3.4.
