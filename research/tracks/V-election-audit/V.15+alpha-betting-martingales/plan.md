# V.15 Plan

## Thesis

ALPHA and betting-martingale tests are the best candidate for RCOUNT's shared
modern RLA test core. They can support ballot polling, comparison, stratified,
and SHANGRLA assertion workflows.

## Atlas

- `docs/algorithm-atlas/v15-alpha-betting-martingale.md`

## Implementation Tasks

- [x] Define explicit assorter-value input fixture.
- [x] Create `rcount-stats` as the shared deterministic numerical layer.
- [x] Add exact rational arithmetic and ppm risk/probability helpers.
- [x] Implement fixed-bet martingale replay.
- [x] Expose fixed-bet replay through `rcount replay-audit-algorithms`.
- [x] Add package-level CLI fixture for fixed-bet pass.
- [x] Add package-level CLI fixture for declared P-value drift.
- [x] Add package-level CLI boundary fixture for missing bets.
- [ ] Add deterministic adaptive-bet transcript fixture.
- [ ] Add without-replacement fixture.
- [x] Decide whether shared math stays in `rcount-audit` or moves to
  `rcount-stats`.

## Claim Boundary

The transcript must record the betting rule. A martingale value without the
assorter, sampling mode, and bet sequence is not independently replayable.
