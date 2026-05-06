# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 2
**Score**: 3/4

## Summary

The revision addresses the main issues from Round 1. The theorem is now an honest $O(n^3 \log n)$ claim, the adversarial-misuse paragraph is present, and the $< 1\%$ outlier frequency claim now has a derivation. The hybrid protocol section has been updated. From a political science standpoint, the paper is now more useful as a practical guide to when ensembles are needed and when CS is sufficient.

## Changes Evaluated

**$< 1\%$ outlier frequency (my main Round 1 concern):** Resolved. The derivation from the B.7 50-state sweep — 0 outliers out of 150 runs, 95% upper bound of $1/150 \approx 0.7\%$ — is shown. This is the right approach: the claim is now backed by a specific empirical calculation rather than asserted.

**Theorem 1 downgrade:** The $O(n^3 \log n)$ bound is correct given the proof sketch. The Remark about the conjectured tighter bound is appropriately hedged.

**Adversarial misuse paragraph (H3):** Present. The statement that "G.4 diagnostic standards provide the appropriate practical standard regardless of the relationship to the worst-case theoretical bound" is exactly what courts need to understand. The paper can now be cited in response to an expert witness who misuses Theorem 1 as evidence that no ensemble is ever long enough.

**Concentration explanation:** The revised two-component explanation (worst-case initial state doesn't arise; empirical criterion is weaker than total-variation) is correct from a political geography standpoint. The original "concentration near compact plans" framing was misleading — what matters is that practice starts from typical, not worst-case, initial conditions.

## Residual Concern

My Round 1 concern about start-from-compact-plan bias in ensemble audits has been addressed in Section 6.4: "The ensemble audit in the hybrid protocol should start from random initial plans, not from the CS plan." This is the right protocol specification. However, the paper should also note that most published redistricting ensembles (Herschlag 2020, DeFord 2021) are NOT started from compact plans — they typically use random spanning-tree plans — so the concern applies to the hybrid protocol specifically, not to the existing literature. This clarification would prevent misreading.

## Recommendation

Accept with minor revisions. The revision resolves all my Round 1 concerns. Add a clarification that the start-from-random-plan requirement applies specifically to the hybrid protocol, not as a critique of existing published ensembles.
