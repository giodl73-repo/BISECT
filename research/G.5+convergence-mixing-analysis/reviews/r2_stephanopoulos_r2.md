# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 2
**Score**: 3/4

## Summary

The revision resolves my main concerns from Round 1. The adversarial misuse paragraph is present and well-stated. The theorem is now $O(n^3 \log n)$ rather than the unproven $O(n^2 \log n)$, which makes it legally safer to cite. The $< 1\%$ outlier claim now has a derivation. The paper is ready for use as legal foundation material.

## Changes Evaluated

**Adversarial misuse of Theorem 1 (my main Round 1 concern):** The added paragraph is exactly what I asked for: "Theorem 1's bounds are worst-case guarantees. The G.4 empirical certification framework provides the appropriate standard for practical use: a chain certified under G.4 diagnostic standards (Rhat, ESS, Hamming) is adequate for redistricting inference regardless of its relationship to the theoretical worst-case bound." This language is directly usable in legal proceedings to rebut an expert who cites the theoretical bound as evidence of ensemble inadequacy.

**Theorem 1 downgrade:** The correction from $O(n^2 \log n)$ to $O(n^3 \log n)$ actually makes the paper safer for litigation. The original claim was untrue and an opposing expert could have exposed it; the revised claim is proven from the spectral gap argument and is unassailable. That the theoretical bound is orders of magnitude larger than needed makes the point that empirical certification (G.4) is both necessary and sufficient.

**$< 1\%$ outlier derivation:** The 50-state sweep derivation is now present. For legal purposes, "0/150 observed, bounded at 0.7% with 95% confidence" is a specific quantitative claim that can be reproduced. Good.

**Hybrid protocol gap:** Section 6.4 now notes that ensemble audits should use random initial plans. The protocol is more complete than in Round 1. One remaining issue: the paper still does not specify what "constrained bisection with modified objective" means when the CS plan is determined to be an outlier. In litigation, a special master would need to know what to do if the CS plan fails the ensemble audit. This is a practical gap that should be addressed, but it is not a blocking issue for research publication.

## Remaining Issue

The hybrid protocol's adjustment mechanism is still described only at a high level. For a paper aimed at practical legal use, this needs one more paragraph specifying: if the ensemble audit shows the CS plan is an outlier on $S_D$ (say, below the 10th percentile of Democratic seats), what specific adjustment does the protocol recommend? The B.0 bakeoff paper would presumably address this, but G.5 should at minimum state that the adjustment is algorithm-selection (choose a compactness-weighted variant rather than edge-cut minimization) rather than manual partisan targeting.

## Recommendation

Accept with minor revisions. All my blocking and high-priority concerns from Round 1 are addressed. The adjustment mechanism in the hybrid protocol should be clarified in one additional paragraph.
