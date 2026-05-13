# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The paper does the most important thing right: it does not sell hashes as
certification, secrecy, malware resistance, or end-to-end cryptographic voting.
The boundary language is unusually careful for a hashing paper.

## Major Issues

1. **Threat model should be tabular.** The paper names boundaries in prose, but
   should show what RCOUNT detects, what it may help detect, and what it does
   not detect.

2. **Canonical bytes need more precision.** Domain separation is named, but the
   paper should say that normalized records are hashed after canonical JSON
   projection while source hashes bind raw bytes.

3. **Source hashes are public-evidence commitments, not privacy tools.** This is
   stated, but should be tied to source-index rows and publication decisions.

## Minor Issues

- Name `RCOUNT_SOURCE_V1`, `RCOUNT_RECORD_V1`, and `RCOUNT_PACKAGE_V1` if the
  current crates expose them.
- Mention detached signatures as future work, not current baseline.
- Say explicitly that hash collisions are outside the practical threat model
  for draft SHA-256 fixtures, but algorithm agility is still required.

## Strengths

- Strong distinction between tamper evidence and legal certification.
- Good transcript example where arithmetic passes and source hash fails.
- No voter-choice receipt risk appears.
