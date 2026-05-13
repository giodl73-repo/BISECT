# V.5 Ballot Manifest Verification: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| TALLY | 2 / 4 |
| CANVASS | 3 / 4 |
| BENCHMARK | 3 / 4 |
| LEDGER | 2 / 4 |
| VAULT | 3 / 4 |

Average: 2.6 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Strengthen accounting semantics

The paper must make batch rows into accounting controls and explicitly separate
batches from voters, ballots, ballot cards, CVRs, contests, selections, and
votes.

### P1.2 Add batch lifecycle and lawful-change model

The paper should cover late mail, provisional review, cure, duplication,
adjudication, recount, and amended/certified statuses.

### P1.3 Add fixture traceability

The positive and negative fixtures should cite exact package paths, transcript
paths, check ids, expected statuses, and the regression caught by the negative
case.

### P1.4 Define the package format contract

V.5 should name `normalized/batches.ndjson`, `batch_id`, `source_refs`,
identifier scope, and vendor/export boundaries more precisely.

## P2 Important Improvements

- Add a manifest-to-summary-to-jurisdiction diagram.
- Add a publication/privacy risk table for small batches.
- Point forward to V.6 for CVR reconciliation, V.7 for RLAs, and V.9 for vendor
  export adapters.

## Recommended Next Action

Revise V.5 before starting V.6. The fixture spine is correct, but the manifest
contract needs to become precise enough for implementers and reviewers.
