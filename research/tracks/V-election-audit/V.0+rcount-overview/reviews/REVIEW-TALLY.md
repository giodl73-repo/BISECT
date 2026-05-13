# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The draft gives a clean package overview, but it currently compresses several
election-accounting distinctions that must be explicit before this can anchor
the V track. The verifier table is good; the data semantics table needs more
precision.

## Major Issues

1. **Ballots, ballot cards, CVR rows, contests, selections, and summaries are
   not distinguished.** The paper says RCOUNT is not a CVR schema, but V.0
   should define these terms and explain what is in scope for the current
   summary-level substrate.

2. **Batch accounting needs a worked example.** `mail-batch-added` is listed,
   but the paper never shows how accepted, counted, and rejected ballots relate
   to the contest summary. A small equation or table would make the contract
   much more concrete.

3. **Residual counts need semantic guardrails.** Undervotes, overvotes, blank
   contests, and write-in buckets appear in the model, but the paper should
   warn that jurisdictions and vendors may encode these differently.

## Minor Issues

- "Source evidence" should include parser diagnostics or warnings as a future
  field.
- The table should mention `batch_id` as optional and explain when it appears.
- "DistrictTotal" is a reporting-unit kind, but district totals should not be
  folded into jurisdiction totals without care.

## Strengths

- The package layout is readable.
- The fixture matrix is concrete and tied to code.
- The verifier contract separates missing batch evidence from bad arithmetic.

