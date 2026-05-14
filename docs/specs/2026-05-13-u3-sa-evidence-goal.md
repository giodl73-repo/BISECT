# U.3 Simulated Annealing Evidence Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Promote U.3 simulated annealing from paper-only heuristic framing to a
hash-bound mechanics package that verifies deterministic seed derivation and a
small synthetic annealing split.

## Acceptance

- [x] Add a package manifest, positive fixture replay, and tamper-rejection test.
- [x] Add RCOUNT/RCTX-style consumer coverage for the package reader/verifier boundary in `bisect-cli`.
- [x] Update U.3 paper, public index, scorecard, and manifest docs.
- [x] Rebuild the U.3 PDF, run focused tests, run formatting, and commit.

## Closeout

The wave delivered
`docs/examples/u-search-evidence-packages/U.3+sa-smoke/` and
`bisect-cli::sa_evidence`. U.3 now has package-backed mechanics evidence for
seed derivation, deterministic replay, partition invariants, and a synthetic
edge-cut bound, while state-level empirical claims remain future work.
