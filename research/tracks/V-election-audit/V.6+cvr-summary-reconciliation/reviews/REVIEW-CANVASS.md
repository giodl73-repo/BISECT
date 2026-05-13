# Simulated Review: CANVASS

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The paper keeps certification and voter intent out of scope, which is correct.
It should say more about adjudication and recount timing around CVR publication.

## Major Issues

1. **Name adjudication timing.** CVR rows may change after write-in review,
   duplication, provisional handling, or recount.

2. **Avoid certified-total overclaim.** Reconciled CVRs support public evidence
   but do not certify an election.

3. **Tie status to CVR rows.** The `status` field matters because unofficial,
   canvassed, recounted, amended, and certified CVRs may differ.

## Minor Issues

- Add a sentence about court orders and amended certifications.
- Say jurisdictions vary in CVR publication rules.
- Keep V.7 audit handoff visible.

## Strengths

- Good separation from voter intent.
- Status is part of the implemented row.
- The paper does not treat CVR exports as ground truth.
