# U Search Evidence Packages Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Add auditable package evidence for U-track search papers whose scorecard gap is
package backing rather than mathematical exposition.

## First Target

The first slice is a deterministic synthetic package covering:

- U.2 parameter-sensitivity sweep shape, baseline binding, output hashes, and
  metric ranges.
- U.4 parallel-tempering audit shape, geometric ladder, swap counts, selected
  rank, and cold-chain record count.

## Acceptance

- [x] Add an active wave and pulse context for U search evidence.
- [x] Add a hash-bound U.2/U.4 synthetic fixture package.
- [x] Add `bisect-ensemble` verifier coverage.
- [x] Update U.2/U.4 papers and ledgers to cite the package without claiming
  full 50-state sweep or production CLI support.
- [x] Close the first slice with validation commands and commit evidence.

## Non-Goals

- Do not claim a full 50-state parameter sweep until real sweep packages exist.
- Do not claim a production parallel-tempering CLI until a checked-in CLI path,
  invocation test, and run package exist.
