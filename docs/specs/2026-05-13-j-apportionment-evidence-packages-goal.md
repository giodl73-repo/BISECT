# J Apportionment Evidence Packages Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Turn J-track apportionment implementation alignment into artifact-backed
evidence by adding Census-source hashes, extracted apportionment rows, and
verifier coverage that replays current `bisect-apportion` results against the
official Census table.

## First Target

The first target is the 2020 Census Table 1 apportionment population and
representative count package. It is small enough for a deterministic fixture and
directly exercises the Huntington-Hill / Method of Equal Proportions path that
anchors J.0-J.6.

## Acceptance

- [x] Add an active wave and pulse context for J evidence packages.
- [x] Add a hash-bound 2020 Census Table 1 extracted fixture.
- [x] Add verifier coverage that recomputes Huntington-Hill seats from the
  extracted populations and matches the official 435-seat result.
- [x] Document source URL, source SHA-256, and verifier path.
- [x] Update J paper ledgers after the package is in place.
- [x] Run validation and commit.

## Closeout

The wave delivered
`docs/examples/j-apportionment-evidence-packages/2020-census-table01/`, a
source-SHA-bound 2020 Census Table 1 extraction, and verifier coverage in
`bisect-apportion::evidence_manifest` that recomputes Huntington-Hill and
matches the official 435-seat result. J.0, J.1, J.6, the public paper index, and
the scorecard now cite that package. Historical 2000/2010 packages and a
standalone `bisect apportion --verify` command remain carry-forwards.

## Non-Goals

- Do not claim full historical 1910-2020 replay until those source tables are
  packaged.
- Do not add Hamilton as a public API in this first slice unless needed for the
  fixture verifier.
