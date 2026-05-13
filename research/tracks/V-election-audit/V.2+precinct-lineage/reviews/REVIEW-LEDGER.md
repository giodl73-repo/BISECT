# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The paper introduces the right interchange concepts but needs more stable
vocabulary. Cycle ids, lineage ids, reporting-unit ids, source refs, package
hashes, plan hashes, and unit-universe hashes need to be visibly part of the
contract.

## Major Issues

1. **Define cycle id syntax and scope.** The examples use `SYN-2024-general`;
   say that cycle ids are package-level labels, not national identifiers.

2. **Define the unit-universe hash.** It should be clear whether the hash covers
   sorted unit ids, plan assignments, reporting-unit records, or the aggregation
   transcript. Without this, interop adapters cannot reproduce it.

3. **Name future lineage kinds carefully.** Rename and boundary-adjustment are
   likely needed, but should be marked future if the crate currently supports
   only unchanged/split/merge.

## Minor Issues

- Distinguish source hashes from normalized lineage record hashes.
- Mention RPLAN as optional, not required for base lineage verification.
- State that stale external crosswalks are adapter failures.

## Strengths

- Good use of explicit ids.
- The hash triple is an important reproducibility contract.
- Strong distinction between RCOUNT and RPLAN responsibilities.
