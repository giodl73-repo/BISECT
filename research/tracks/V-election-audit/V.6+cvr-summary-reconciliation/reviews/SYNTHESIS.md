# V.6 CVR-To-Summary Reconciliation: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| TALLY | 3 / 4 |
| BENCHMARK | 3 / 4 |
| CANVASS | 3 / 4 |
| VAULT | 2 / 4 |
| LEDGER | 3 / 4 |

Average: 2.8 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Clarify CVR row cardinality

The paper should say that the implemented row is one contest interpretation and
that duplicate `(cvr_id, contest_id)`, residual-plus-selection rows, and
over-vote-for rows are verifier failures.

### P1.2 Add fixture traceability

The paper should cite exact package paths, transcript paths, expected check ids,
and the Candidate A mismatch in the negative fixture.

### P1.3 Add CVR privacy risk table

V.6 must make the publication risk of ballot-level CVR evidence as visible as
the arithmetic benefit.

### P1.4 State adapter/version boundaries

The normalized row is an implemented first slice. V.9 still owns vendor export
schemas, raw evidence preservation, parser diagnostics, and schema evolution.

## P2 Important Improvements

- Add an implemented-check table.
- Tie `cvr_summary_total`, `contest_selection_sum`, and
  `jurisdiction_contest_total` together.
- Keep V.7 and V.9 handoffs visible.

## Recommended Next Action

Revise V.6 to ready before drafting V.7. The implementation is strong, but the
paper should make privacy and row-shape constraints impossible to miss.
