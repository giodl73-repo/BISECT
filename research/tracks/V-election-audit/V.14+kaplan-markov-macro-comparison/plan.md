# V.14 Plan

## Thesis

Comparison audits turn CVR-vs-hand discrepancies into overstatement errors and
then into risk evidence. RCOUNT needs this family for ballot-level comparison
and batch comparison audits.

## Atlas

- `docs/algorithm-atlas/v14-kaplan-markov-macro-comparison.md`

## Implementation Tasks

- [x] Add comparison observation records for CVR selection, human selection, and
  discrepancy kind.
- [x] Add overstatement-error calculation for simple plurality assertions.
- [x] Add exact overstatement taint normalization by reported margin.
- [x] Add boundary replay surface for `kaplan-markov-comparison-v1`.
- [x] Add initial Kaplan-Markov taint-product running P-value transcript.
- [x] Add exact published MACRO Kaplan-Markov product primitive with
  no-error/overstatement/error-input fixtures.
- [x] Add package schema fields for MACRO design inputs `N`, `V`, and gamma.
- [x] Validate MACRO design fields as an all-or-none package schema bundle.
- [x] Replay package-level `kaplan-markov-comparison-v1` with MACRO when those
  design inputs are present.
- [x] Add reusable synthetic MACRO package fixture with core verification, IO
  round-trip, and CLI replay coverage.
- [x] Record external-public validation boundary for MACRO package replay.
- [ ] Validate the MACRO package path against external public audit fixtures
  when a source exposes `N`, `V`, gamma, and ordered overstatement categories.
- [x] Add no-error and overstatement negative stats fixtures.
- [ ] Reuse this math from V.18 batch comparison where possible.

## Claim Boundary

Comparison math requires trustworthy ballot/CVR or batch/reported-total linkage.
RCOUNT can verify the math and source hashes, not physical custody by itself.
