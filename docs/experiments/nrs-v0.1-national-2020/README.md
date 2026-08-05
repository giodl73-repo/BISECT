# NRS v0.1 National 2020 Reference Baseline

This tracked summary package records the completed 50-State NRS v0.1
reference-baseline run without committing the 896 MB State package set.

## Result

- 50 States and 435 congressional districts;
- 8,126,956 Census blocks assigned exactly once;
- 385 connected recursive split nodes meeting the frozen population tolerance;
- 0 failed States and 0 disconnected districts; and
- independent `--require-complete` verification passed.

The exact-proof boundary remains explicit: 4 nodes attain their arithmetic
population floors, while 381 population optima, all 385 weighted-boundary
optima, and all 385 canonical ties remain unproved.

## Replay

With the ignored State packages present at the default run path:

```powershell
target\release\bisect-ops.exe verify-nrs-batch --require-complete
target\debug\bisect-ops.exe summarize-nrs-batch
```

`manifest.json` binds this summary and proof matrix to the source ledger hash.
The ledger itself binds the canonical standard profile and exact BISECT
executable. See the adjacent national diagnostic for runtime history and the
algorithm revisions that closed the original ten-State failure set.

## Claim boundary

This is a deterministic geographic reference baseline, not an exact canonical
national plan, enacted law, VRA determination, partisan-fairness finding, or
official adoption record.
