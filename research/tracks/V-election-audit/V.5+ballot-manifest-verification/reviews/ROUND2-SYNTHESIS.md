# V.5 Round 2 Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Status

Ready.

## Resolved P1 Items

- Expanded the accounting semantics so batch manifest rows are explicitly
  separate from voters, ballots, ballot cards, CVR rows, contests, selections,
  and votes.
- Added the normalized package contract for `normalized/batches.ndjson`,
  `batch_id`, `source_refs`, and source-index linkage.
- Added fixture path anchors for batches, summaries, status events, and
  verifier transcripts.
- Added lifecycle and lawful-change language for late mail, provisional review,
  cure, duplicated ballots, recounts, court orders, and amendments.
- Added a publication/privacy risk table for small batches and rare metadata.

## Remaining Risk

V.5 verifies batch accounting controls, not voter intent, tabulator correctness,
full CVR semantics, certification, or risk-limiting audit correctness. Those
boundaries are now named and passed to V.6 and V.7.
