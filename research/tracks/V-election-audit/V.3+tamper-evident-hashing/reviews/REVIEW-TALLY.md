# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The batch manifest section correctly separates accepted/count/rejected
arithmetic from source evidence. It also avoids treating late mail as suspicious
just because it is late.

## Major Issues

1. **Batch source refs need a clearer role.** They bind manifest rows to source
   evidence, but they do not prove every ballot in the batch was interpreted
   correctly.

2. **Hashing should not hide parser semantics.** The paper should say raw source
   hashes let an independent parser revisit the input, but normalization still
   requires adapter diagnostics.

3. **Future CVR/batch relationships should be deferred.** Ballot manifest and
   CVR reconciliation belong to V.5/V.6.

## Minor Issues

- Mention that batch ids are package-scoped.
- Say rejected-ballot reasons are future schema expansion.
- Add central-count and provisional batches as examples only.

## Strengths

- Strong distinction between arithmetic and hash checks.
- Good use of `mail-batch-added`.
- Clear boundary around late mail.
