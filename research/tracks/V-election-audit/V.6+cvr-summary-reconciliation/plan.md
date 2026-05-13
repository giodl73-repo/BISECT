# V.6 Plan

## Question

How should RCOUNT reconcile cast-vote records and public contest summaries
without confusing ballot-card accounting, contest interpretations, residuals,
and certified totals?

## Core Claims

1. CVR row/card counts must reconcile to ballot manifest accounting before
   contest totals can be trusted.
2. Contest selections plus residuals must equal counted ballots for every
   reporting unit, status, contest, and optional batch.
3. Aggregated CVR interpretations must match published summaries when a CVR
   export is available and legally publishable.
4. The current substrate verifies `normalized/cvr.ndjson` rows with
   `cvr_summary_total` and keeps `contest_selection_sum` as the summary-side
   guardrail.

## Evidence

- `cvr-summary`
- `bad-cvr-summary`
- `cvr_summary_total`
- `contest_selection_sum`
- `jurisdiction_contest_total`
