# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The tally arithmetic is mostly sound for a summary-level canvass paper. The
draft separates contest arithmetic, jurisdiction rollups, batch manifests, and
source hashes, which is the right structure. It also identifies the important
negative case where totals can look plausible while a batch manifest is absent.

## Major Issues

1. **Contest residuals should be explicit in the worked example.** The paper
   mentions candidate votes plus residual counts, but the table only shows
   candidates and counted ballots. Add undervote, overvote, write-in, or blank
   residual columns or a short equation.

2. **Batch accounting and ballot acceptance need separate names.** The equation
   accepted = counted + rejected is useful, but readers may infer that RCOUNT
   determines acceptance. Say that accepted/rejected are asserted source facts,
   while RCOUNT verifies arithmetic among declared facts.

3. **Batch-specific summaries need one more constraint.** The paper should say
   whether batch summaries are optional, required for certain fixtures, or
   adapter-dependent. This matters for central-count and vote-center workflows.

## Minor Issues

- Add one line about CVR-to-summary reconciliation being deferred to V.6.
- Say that `missing-batch` fails because a summary references absent manifest
  evidence, not because late mail is disallowed.
- Consider naming ballot cards as out of scope for V.1.

## Strengths

- Good failure attribution table.
- Good distinction between batch manifest evidence and summary arithmetic.
- Strong fixture selection for the first canvass paper.
