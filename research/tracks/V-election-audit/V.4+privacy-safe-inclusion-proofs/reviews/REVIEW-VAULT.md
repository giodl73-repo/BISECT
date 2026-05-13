# Simulated Review: VAULT

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The draft has the right central rule: public inclusion proofs must not become
candidate-choice receipts. It is still too thin as a privacy paper because it
names coercion channels without modeling them.

## Major Issues

1. **Add a coercion threat model.** The paper should show what a buyer,
   employer, campaign worker, family member, or state actor can demand, and why
   the public proof cannot satisfy that demand.

2. **Separate hashing from privacy.** `token_hash` is not automatically private.
   The paper should state what salts, nonces, custody, and token issuance
   questions are outside the current baseline.

3. **Define forbidden linkability fields.** Candidate selections are the
   obvious danger, but timestamps, ballot style, small precincts, rare contests,
   and deterministic token issuance can also leak.

## Minor Issues

- Say whether proof ids are package-local only.
- Add an explicit small-cell review rule.
- Make clear that passing `proof_privacy_gate` is necessary, not sufficient, for
  a deployable cryptographic voter-verification protocol.

## Strengths

- Good direct rejection of choice-bearing proof artifacts.
- Correctly avoids claiming end-to-end verifiability.
- The positive and negative fixtures are exactly the right first pair.
