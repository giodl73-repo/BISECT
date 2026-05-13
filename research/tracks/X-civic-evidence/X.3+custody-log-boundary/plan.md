# X.3 RCHAIN/RLOG Boundary

Goal: keep custody and event-log modeling available without letting RCOUNT
claim physical process facts it cannot prove.

## Status

- [x] Define RLOG as normalized event/status logs.
- [x] Define RCHAIN as physical chain-of-custody evidence.
- [x] Record that both are deferred until real source artifacts demand schemas.
- [ ] Collect public examples of EMS logs, tabulator logs, seal logs, custody
  forms, and observer records.
- [ ] Write L0 source-pressure fixtures before any crate work.

## Specification

- `docs/specs/2026-05-13-downstream-evidence-boundaries.md`
- `docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md`

## Boundary Rule

RLOG can say what a source log records. RCHAIN can say what a custody source
records. Neither can infer custody or correctness from count reconciliation
alone.
