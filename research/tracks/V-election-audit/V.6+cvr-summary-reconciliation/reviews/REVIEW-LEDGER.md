# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The normalized path and row contract are a good start. The paper should make
versioning and adapter boundaries more explicit.

## Major Issues

1. **State the canonical path.** `normalized/cvr.ndjson` is now a package table
   and should be named as such.

2. **Version the row contract.** The paper should say this is the first
   normalized RCOUNT CVR surface, not a final vendor-neutral standard.

3. **Name adapter obligations.** V.9 adapters must preserve raw source evidence
   and parser diagnostics, not just normalized totals.

## Minor Issues

- Mention `source_refs`.
- Mention future schema evolution for multi-card/multi-contest systems.
- Keep source hash checks separate from CVR arithmetic.

## Strengths

- Good package-local field names.
- Clear source-reference field exists.
- The contract is simple enough to audit.
