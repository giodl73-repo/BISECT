# V.12 Plan

## Thesis

BRAVO is the classic ballot-polling RLA. RCOUNT should be able to replay its
sequential likelihood-ratio transcript from public contest totals, sample order,
and ballot observations.

## Atlas

- `docs/algorithm-atlas/v12-bravo-ballot-polling.md`

## Implementation Tasks

- [x] Define BRAVO method id and transcript record.
- [x] Add tiny two-candidate positive fixture.
- [x] Add observation-drift negative fixture.
- [x] Add boundary fixture that does not stop.
- [x] Add missing-field boundary fixture for unreplayable public evidence.
- [x] Expose BRAVO replay through `rcount replay-audit-algorithms`.
- [ ] Decide whether BRAVO replay should become part of baseline `rcount verify`
  or remain an explicit algorithm-replay command.

## Claim Boundary

BRAVO verifies a ballot-polling outcome assertion under sampling assumptions. It
does not verify ballot custody, eligibility, or CVR comparison claims.
