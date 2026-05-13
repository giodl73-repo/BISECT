# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The draft names the right files but needs a clearer interchange contract. If
RCOUNT is a package format, V.5 should define the normalized manifest path,
identifier scope, and source-reference expectations precisely.

## Major Issues

1. **Define canonical package paths.** `normalized/batches.ndjson` should be
   named as the manifest table and summaries should reference it by
   `batch_id`.

2. **Define identifier stability.** `batch_id` is package-stable, but the paper
   should say how it relates to external vendor ids, scanner ids, or election
   office batch labels.

3. **Define source references.** Manifest rows should carry `source_refs` that
   bind the normalized row back to hashed source evidence.

## Minor Issues

- Mention versioned package schemas.
- Avoid treating the synthetic field set as the final national standard.
- Say that vendor export adapters belong to V.9.

## Strengths

- Good use of NDJSON paths.
- Correctly separates source hash checks from accounting checks.
- The format is simple enough to be externally audited.
