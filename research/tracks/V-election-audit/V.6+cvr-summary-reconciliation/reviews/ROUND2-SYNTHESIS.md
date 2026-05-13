# V.6 Round 2 Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Status

Ready.

## Resolved P1 Items

- Clarified that the implemented row is one CVR contest interpretation and that
  duplicate `(cvr_id, contest_id)`, invalid residual combinations, unknown
  selections, and over-`vote_for` rows are verifier failures.
- Added exact fixture path anchors for `normalized/cvr.ndjson`,
  `normalized/summaries.ndjson`, `normalized/contests.ndjson`,
  `sources/source-index.json`, and verifier transcripts.
- Added an implemented-check table connecting `cvr_summary_total`,
  `contest_selection_sum`, `jurisdiction_contest_total`, and
  `source_hash_match`.
- Added a CVR privacy/publication risk table covering rare write-ins, unique
  patterns, ballot style and batch joins, provisional/cure batches, and
  inclusion-token joins.

## Remaining Risk

The normalized CVR row is the first implemented RCOUNT surface, not a full
vendor-export standard. V.9 still owns external adapters, raw source evidence,
parser diagnostics, and schema evolution. V.7 owns risk-limiting audit replay.
