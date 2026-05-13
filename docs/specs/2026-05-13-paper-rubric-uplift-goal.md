# Paper Rubric Uplift Goal

**Opened:** 2026-05-13
**Status:** active - pulse 04 complete

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

## Pulse 02 Result

- [x] J.6 no longer claims a five-method public API, standalone CLI, exact
  integer priority comparisons, or checked-in SHA/Census verification fixtures.
- [x] J.0-J.5 bridge sections inherit the current implementation boundary.
- [x] The scorecard and paper review ledger record J-track uplift status.
- [x] Changed J-track PDFs are rebuilt and copied to `docs/papers/`.

## Pulse 03 Result

- [x] U.0-U.7, U.11, and U.13-U.15 now state current implementation and
  evidence boundaries.
- [x] Older search papers no longer rely on unqualified convergence,
  representativeness, legal-sufficiency, production-CLI, or global-optimality
  language where only a method/protocol/substrate is present.
- [x] The scorecard, paper review ledger, and public paper index record U-track
  uplift status and remaining package-evidence ceilings.

## Pulse 04 Result

- [x] G.1-G.3, G.5, and G.12 now carry current data, validation, diagnostic, or
  implementation-boundary sections.
- [x] Ensemble percentile, partisan-distribution, compactness-position, mixing,
  and short-burst claims are scoped to archived traces, diagnostics, package
  manifests, and verifier paths.
- [x] The scorecard, paper review ledger, and public paper index record G-track
  uplift status and remaining external-trace ceilings.
