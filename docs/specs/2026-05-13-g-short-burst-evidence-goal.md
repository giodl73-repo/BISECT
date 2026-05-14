# G Short-Burst Evidence Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Promote G.6/G.12 short-burst claims from paper-only mechanics to a hash-bound
smoke package that verifies deterministic seed streams, endpoint retention,
selected-endpoint ordering, and acceptance-rate diagnostics.

## Acceptance

- [x] Add a package manifest, positive fixture replay, and tamper-rejection tests.
- [x] Add consumer coverage in `bisect-ensemble` for short-burst package reads.
- [x] Update G.6/G.12 papers, public index, scorecard, and manifest docs.
- [x] Rebuild the G.6/G.12 PDFs, run focused tests, run formatting, and commit.

## Closeout

The wave delivered
`docs/examples/g-ensemble-evidence-packages/G.6-G.12+short-burst-smoke/` and
`bisect-ensemble::short_burst_evidence`. G.6/G.12 now have package-backed
mechanics evidence, while production CLI modes and state empirical archives
remain future work.
