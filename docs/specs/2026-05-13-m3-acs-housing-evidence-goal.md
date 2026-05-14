# M.3 ACS Housing Evidence Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Promote M.3 from deferred empirical design to an implementation-backed housing
character slice by wiring the `housing-character` weight mode and adding a
hash-bound ACS housing formula/edge-weight smoke package.

## Acceptance

- [x] Add a reusable ACS housing-character helper and loader.
- [x] Wire `--weights-override housing-character` through the CLI weight layer.
- [x] Correct ACS B25024 denominator and large-multifamily variable handling.
- [x] Add a package manifest, positive fixture replay, and tamper-rejection test.
- [x] Update M.3 paper, public index, scorecard, and manifest docs.
- [x] Rebuild the M.3 PDF, run focused tests, run formatting, and commit.

## Closeout

The wave delivered
`docs/examples/m-community-character-evidence-packages/M.3+acs-housing-smoke/`,
`bisect-cli::housing`, and `bisect-cli::housing_evidence`. M.3 now has an
implemented ACS fetch/formula and edge-weight mode boundary, while full NC-14
empirical outcome claims remain future work.
