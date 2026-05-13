# Simulated Review: BENCHMARK

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The fixture matrix is the strongest part of V.0. It gives the paper a real
ground truth rather than hand-wavy claims. The next revision should make every
claim traceable to a command, test, or fixture path.

## Major Issues

1. **Fixture-to-command traceability is incomplete.** The paper names fixtures,
   but it does not list the exact commands that regenerate and verify them.

2. **Pass/fail acceptance criteria should be explicit.** For example,
   `stale-plan` exits through `aggregate-districts` with a missing plan-unit
   summary, while bad lineage fails `rcount verify`. Those are different
   failure surfaces and should be stated.

3. **L0/L1/L2 labels are used in docs but absent from the paper.** V.0 should
   explain why synthetic L2 is the milestone boundary before real-data adapters.

## Minor Issues

- Mention the tested crates and command: `cargo test -p rcount-district -p
  rcount-audit -p rcount-cli`.
- Avoid saying "all verifier layers" unless the table lists every implemented
  one.
- Note that the PDF currently has overfull table boxes.

## Strengths

- Negative fixtures are excellent.
- The paper does not overclaim real-world coverage.
- The package/check/failure mapping is already close to testable.

