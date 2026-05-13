# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The paper is now backed by a real normalized CVR slice. It correctly separates
CVR contest interpretations from public summaries and keeps residuals explicit.

## Major Issues

1. **Clarify row cardinality.** The paper should say the implemented row is one
   CVR contest interpretation and that `(cvr_id, contest_id)` must be unique.

2. **Name invalid mark shapes.** Multiple residual flags, residual plus
   selections, and too many selections for `vote_for` should be described as
   verifier failures.

3. **Explain the current ballot-card limit.** The fixture is one contest, so it
   does not yet exercise multi-card ballots or multiple contests per CVR id.

## Minor Issues

- Add a compact implemented-check table.
- Mention future duplicate-row and invalid-cardinality negative fixtures.
- Keep V.5 manifest card counts in view.

## Strengths

- Real CVR rows now exist in `normalized/cvr.ndjson`.
- Positive and negative fixtures test the meaningful mismatch.
- Residual counts are not hidden.
