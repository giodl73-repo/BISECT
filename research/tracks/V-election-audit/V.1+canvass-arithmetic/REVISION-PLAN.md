# Revision Plan

Round 1 simulated review is complete.

## P1 Items

1. [x] Define the minimal status-event schema and transition evidence contract:
   event id, event type, from status, to status, authority, timestamp/source
   date, source references, explanation, and affected reporting units or
   batches.
2. [x] Add fixture-to-command traceability for `canvass-correction`,
   `mail-batch-added`, and `missing-batch`, including the expected passing or
   failing check ids.
3. [x] Separate batch accounting, contest residual arithmetic, and
   ballot-acceptance decisions more visibly so readers do not infer that RCOUNT
   decides voter eligibility.
4. [x] Name jurisdiction-variation fields/adapters without implying one national
   canvass workflow.

## P2 Items

- [ ] Add a compact visual lifecycle diagram.
- [x] Add one worked correction delta as a before/after equation.
- [x] Add a status-event JSON sketch after the correction table.
- [x] Add a short public-reader interpretation paragraph for each negative
  fixture.
