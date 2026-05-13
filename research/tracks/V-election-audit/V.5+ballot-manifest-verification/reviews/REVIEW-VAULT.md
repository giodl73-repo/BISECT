# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The draft recognizes that batch granularity can leak sensitive information. It
should turn that boundary into a small publication-risk table.

## Major Issues

1. **Add small-batch privacy cases.** Single-voter provisional batches,
   small cure batches, rare write-in batches, and precise scanner times can
   become linkability evidence.

2. **Separate public and restricted manifests.** The paper should say RCOUNT can
   verify a richer internal package than a jurisdiction may publish.

3. **Avoid accidental receipt paths.** Batch metadata should not be combined
   with V.4 inclusion tokens in a way that narrows a voter to one record.

## Minor Issues

- Mention aggregation or redaction as publication choices.
- Keep source hashes from being mistaken for privacy protections.
- Carry V.4's receipt-safety rule forward explicitly.

## Strengths

- No candidate-choice proofs are introduced.
- Good small-cell warning already exists.
- The paper keeps privacy separate from arithmetic.
