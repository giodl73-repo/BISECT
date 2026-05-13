# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The paper has the right privacy instinct: prove inclusion, never candidate
choice. However, V.0 should be more explicit about the threat model and about
what the current "proof sketch" does not yet prove cryptographically.

## Major Issues

1. **Threat model is underspecified.** The paper should separate source
   tampering, parser substitution, package truncation, equivocation, small-cell
   reidentification, coercion receipts, and malware resistance.

2. **Inclusion proofs are described too confidently.** The current fixture is a
   privacy gate, not a complete voter-verifiable protocol. The paper should say
   "proof sketch" consistently and identify what future cryptographic machinery
   would be needed.

3. **Small-cell disclosure needs a visible warning.** Rare write-ins, tiny
   precincts, ballot style combinations, and timestamps can leak identity even
   without explicit voter ids. This should be in the main privacy boundary, not
   only implied.

## Minor Issues

- Domain-separated hash prefixes are mentioned but not listed.
- A table of "RCOUNT proves / does not prove" would help.
- "Tamper evidence" should not sound like malware resistance.

## Strengths

- Strong rejection of choice-bearing proofs.
- Good distinction between hashing and certification.
- Optional RPLAN boundary avoids leaking district machinery into base packages.

