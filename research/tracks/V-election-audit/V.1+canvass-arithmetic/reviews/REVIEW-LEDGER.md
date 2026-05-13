# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The paper has the right public-record instincts, but the event and source
terminology is still too prose-heavy for an interchange paper. V.1 should
define the minimal names that future adapters, vendor exports, and statements of
votes must map into.

## Major Issues

1. **Define event field names.** A status-event sketch should include stable
   names for event type, source references, affected units, status transition,
   authority, and explanation.

2. **Separate source hash from normalized event hash.** The paper says source
   files match hashes, but not whether the correction event is hashed as a
   normalized record, linked to raw source hashes, or both.

3. **Name compatibility boundaries.** V.1 should say that NIST CDF, vendor
   exports, canvass statements, and state statement-of-vote formats are source
   adapters, not the base RCOUNT schema.

## Minor Issues

- Use stable status names exactly as in the crate.
- Mention that event ids must be stable within a package.
- Avoid implying that every jurisdiction has an artifact named "canvass report."

## Strengths

- Status-specific summary keys are a strong format rule.
- The paper correctly avoids overwriting previous snapshots.
- The source hash check belongs in this paper and is well motivated.
