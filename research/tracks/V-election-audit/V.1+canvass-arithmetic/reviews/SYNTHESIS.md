# V.1 Canvass Arithmetic: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| CANVASS | 3 / 4 |
| TALLY | 3 / 4 |
| BENCHMARK | 2 / 4 |
| LEDGER | 2 / 4 |
| VAULT | 3 / 4 |

Average: 2.6 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Define the status-event schema

V.1 needs a compact event schema showing event id, event type, from status, to
status, authority, date/source timestamp, source references, explanation, and
affected reporting units or batches. This is the bridge between CANVASS's legal
workflow and LEDGER's stable interchange vocabulary.

### P1.2 Add reproducible fixture commands and expected results

BENCHMARK cannot validate the paper until the examples map to commands and
expected transcripts. Add a table for `canvass-correction`,
`mail-batch-added`, and `missing-batch`, including pass/fail expectations and
check ids.

### P1.3 Separate declared ballot decisions from arithmetic checks

TALLY and CANVASS agree that RCOUNT may verify accepted/count/rejected
arithmetic, but it must not imply that software decides ballot eligibility.
State this near the batch equations and in the boundaries.

### P1.4 Add privacy-safe event metadata language

VAULT needs one paragraph explaining that event metadata and source references
must avoid small-cell leakage and that source hashes do not make private data
publishable.

## P2 Important Improvements

- Add a small lifecycle diagram or transition table.
- Add one worked correction delta with residual counts.
- Add a sample transcript excerpt.
- Mention V.6 for CVR-to-summary reconciliation and V.9 for source adapters.

## Recommended Next Action

Revise V.1 before drafting V.2. The paper has the right argument, but it needs
two more contract tables and one concrete verifier transcript to become a
usable anchor for the rest of the V track.
