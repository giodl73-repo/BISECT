# V.17 Plan

## Thesis

Real contests may span jurisdictions with different evidence surfaces. RCOUNT
needs stratified/hybrid audit transcripts that combine ballot-polling,
ballot-comparison, and batch-comparison strata without pretending the sample was
homogeneous.

## Atlas

- `docs/algorithm-atlas/v17-stratified-suite-hybrid.md`

## Implementation Tasks

- [x] Define stratum records and stratum method ids.
- [x] Add two-stratum synthetic fixture.
- [x] Add combining-rule transcript field.
- [x] Verify stratum component run references and reject missing components.
- [x] Expose `stratified-hybrid-rla-v1` through
  `replay-audit-algorithms` as an explicit combined-risk boundary.
- [x] Add nuisance-parameter and allocation transcript fields.
- [x] Add negative fixture for flattened-stratum misuse.
- [ ] Add combined-risk replay once the combining rule math is selected and
  validated.

## Claim Boundary

The combined risk claim depends on correct stratum definitions and sampling
frames. RCOUNT now preserves strata and component run references, but still
does not claim combined-risk replay.
