# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

V.2 generally stays within the privacy boundary because it works at the
reporting-unit level and does not publish voter movement records. The paper
should still strengthen the small-unit warning because lineage can expose
patterns for very small precincts, rare ballot styles, or named facilities.

## Major Issues

1. **Small-unit lineage needs a disclosure rule.** A split of a very small unit
   may reveal more than a normal precinct summary. The paper should require
   adapter-level small-cell review before publishing granular lineage evidence.

2. **Source hashes do not sanitize sensitive source files.** If voter-assignment
   or registration evidence supports lineage, hashing it does not make it safe
   to publish.

3. **Do not imply voter movement tracking.** The user-facing story should be
   explicit: RCOUNT verifies reporting-unit lineage, not individual voter
   movement.

## Minor Issues

- Repeat "prove inclusion, never candidate choices" only if future voter-file
  adapters are mentioned.
- Add one sentence that private support evidence can be hashed without being
  publicly released.
- Avoid exact timestamps where a date is enough.

## Strengths

- Good privacy boundary in the current draft.
- Clear non-goal around voter-level movement.
- No choice-bearing receipt risk appears.
