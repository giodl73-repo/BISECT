# Paper Rubric Uplift Goal

**Opened:** 2026-05-13
**Status:** active - pulse 01 complete

## Goal

Raise the highest-priority below-rubric papers by fixing concrete correctness,
claim-boundary, evidence-path, and reader-experience gaps identified in the
algorithm paper scorecard.

## First Target

K.2 Reock is first because the current scorecard identifies a concrete
Welzl/implementation mismatch. The pass should distinguish:

1. canonical Reock as an area divided by a minimum enclosing circle area;
2. `bisect-analysis` production compactness as a centroid-plus-max-boundary
   radius approximation retained for Python parity;
3. moving-knife orientation search as a point-cloud Reock proxy over tract
   centroids, not as polygon exact-MBC Reock.

## Acceptance

- [x] K.2 source no longer implies that production compactness uses exact
  Welzl MBC where it does not.
- [x] K.0 and K.7 inherit the corrected Reock caveat.
- [x] The paper quality ledger records the review/fix/build loop.
- [x] The algorithm paper scorecard updates K.2/K.0/K.7 scores or status notes.
- [x] The wave pulse is closed with validation commands and commit evidence.
