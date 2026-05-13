# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

CVR rows are useful but dangerous. The paper names privacy risk, but V.6 should
carry V.4's receipt-safety discipline more visibly.

## Major Issues

1. **Add a CVR privacy risk table.** Rare write-ins, unique selection patterns,
   ballot style, tiny batches, and timestamps can become identifying evidence.

2. **Separate public and restricted CVRs.** The verifier can consume richer
   evidence than a jurisdiction should publish.

3. **Warn against joining CVRs to inclusion proofs.** Combining CVR rows with
   voter-facing tokens can narrow a voter to a choice pattern.

## Minor Issues

- Say hashes are not privacy controls.
- Mention aggregation, redaction, or suppression.
- Keep candidate-choice receipts forbidden.

## Strengths

- No voter identity is present in the normalized row.
- The row uses package-local ids.
- The boundary section is already cautious.
