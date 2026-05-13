# Revision Plan

Round 1 simulated review is complete.

## P1 Items

1. [x] Clarify CVR row cardinality and the one-row-per-contest interpretation
   contract, including duplicate `(cvr_id, contest_id)` rejection.
2. [x] Add exact fixture path and transcript anchors for `cvr-summary` and
   `bad-cvr-summary`.
3. [x] Add a privacy/publication risk table for ballot-level CVR patterns.
4. [x] State adapter/version boundaries and future negative fixture
   expectations for duplicate rows and invalid cardinality.

## P2 Items

- [x] Add a short implemented-check table.
- [x] Tie `cvr_summary_total` to `contest_selection_sum` and
  `jurisdiction_contest_total` more explicitly.
- [x] Keep V.7 and V.9 handoffs visible.
