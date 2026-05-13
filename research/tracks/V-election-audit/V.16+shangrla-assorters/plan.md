# V.16 Plan

## Thesis

SHANGRLA gives RCOUNT a common assertion language: reduce outcome claims to
assorter mean tests, then run a valid sequential test such as ALPHA.

## Atlas

- `docs/algorithm-atlas/v16-shangrla-assorters.md`

## Implementation Tasks

- [x] Add assertion ids and registered assorter ids to `rcount-core`.
- [x] Add optional package-level audit algorithm transcript records.
- [x] Add verifier checks for duplicate assertions, missing assertion links,
  invalid risk limits, invalid p-values, and out-of-bound assorter values.
- [ ] Add plurality winner-loser assorter fixture.
- [ ] Add multi-assertion fixture where one assertion fails.
- [ ] Wire V.15 ALPHA to consume explicit assorter values.
- [ ] Use the assertion model as the bridge to V.19 RCV audits.

## Claim Boundary

SHANGRLA defines the assertion layer. It does not replace source hashing,
sampler replay, ballot manifests, or CVR linkage.
