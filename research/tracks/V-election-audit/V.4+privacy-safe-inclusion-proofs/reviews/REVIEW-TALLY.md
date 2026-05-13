# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The draft protects ballot secrecy, but it is light on the accounting semantics
behind accepted and counted tokens. That needs tightening before the paper can
anchor later CVR and manifest work.

## Major Issues

1. **Define what a token is not.** It is not necessarily a ballot, ballot card,
   CVR row, voter, or contest row.

2. **State the relationship to manifests and CVRs.** V.5 and V.6 can handle the
   full accounting, but V.4 should say this proof only references an anonymized
   token in that larger evidence model.

3. **Warn about reporting units.** Vote centers, central-count mail batches,
   and split precincts can make reporting-unit references less intuitive than a
   voter expects.

## Minor Issues

- Add one sentence that undervotes, overvotes, write-ins, and adjudication are
  deliberately absent from the public proof.
- Do not let `reporting_unit_id` look like a complete ballot-location proof.
- Mention batch boundaries as metadata that can become sensitive.

## Strengths

- Correctly rejects candidate selections.
- The sketch is small enough to audit.
- The paper does not normalize away later accounting work.
