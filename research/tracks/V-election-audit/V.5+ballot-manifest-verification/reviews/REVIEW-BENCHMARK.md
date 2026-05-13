# Simulated Review: BENCHMARK

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The test story is already strong. It needs exact anchors so a reader can tie
each claim to package files, transcripts, and crate tests.

## Major Issues

1. **Add file path anchors.** Name `normalized/batches.ndjson`,
   `normalized/summaries.ndjson`, `status/events.ndjson`, and
   `transcripts/verify-transcript.json`.

2. **Name exact checks.** The draft should show `accepted_ballots`,
   `batch_summary_total`, `jurisdiction_contest_total`, and
   `source_hash_match` where they apply.

3. **State the regression caught.** The missing-batch fixture should catch a
   verifier that silently treats absent batch evidence as zero or as optional.

## Minor Issues

- Cite the targeted test filters used for acceptance.
- Include the exact negative error.
- Add a future negative for duplicate batch ids.

## Strengths

- Positive and negative fixtures are paired well.
- The command block is reproducible.
- The negative failure is specific and useful.
