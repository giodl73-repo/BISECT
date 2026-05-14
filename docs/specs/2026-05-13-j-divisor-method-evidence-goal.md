# J Divisor Method Evidence Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Raise J.2--J.5 from source/test-only apportionment-method claims to
package-backed evidence by adding a minimal, hash-bound divisor-method fixture
that replays Webster, Adams, Jefferson, and shared divisor paradox immunity.

## Acceptance

- [x] Add a wave and pulse context for the J divisor-method evidence slice.
- [x] Add a portable manifest schema and verifier for divisor-method smoke
  fixtures.
- [x] Add a positive fixture covering Webster, Adams, Jefferson allocations and
  the shared no-Alabama-paradox surface for all four divisor rules.
- [x] Add negative verifier coverage that rejects a tampered allocation.
- [x] Update J.2--J.5 papers, public paper index, scorecard, and manifest docs.
- [x] Rebuild J.2--J.5 PDFs, run focused tests, run formatting, and commit.

## Closeout

The wave delivered
`docs/examples/j-apportionment-evidence-packages/divisor-method-smoke/`, a
hash-bound synthetic divisor-method fixture with verifier coverage in
`bisect-apportion::divisor_evidence`. J.2, J.3, J.4, and J.5 now cite the
package and distinguish this smoke evidence from future Census-year historical
replay.

## Non-Goals

- Do not claim full historical Webster/Adams/Jefferson Census replay until those
  source tables are packaged.
- Do not add a public CLI verifier in this slice; verification remains Rust test
  coverage.
