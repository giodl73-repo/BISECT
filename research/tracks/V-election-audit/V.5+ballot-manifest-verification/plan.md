# V.5 Plan

## Question

How can a public RCOUNT package prove that reported batch-level summaries have
corresponding batch manifest evidence?

## Core Claims

1. A ballot manifest layer must account for accepted, counted, and rejected
   ballots before vote totals can be trusted.
2. Batch-level summaries must reference existing manifest rows.
3. A late-arriving batch is legitimate when it is represented by manifest
   evidence and canvass status events.
4. A missing batch is a mechanical verification failure, not a political
   inference by itself.

## Evidence

- `mail-batch-added`
- `missing-batch`
- `accepted_ballots`
- `batch_summary_total`
