# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.5 provides the theoretical grounding for why redistricting ensemble evidence requires convergence certification. The mixing time analysis supports the legal argument that uncertified ensembles — those run for an arbitrary number of steps without Rhat or ESS reporting — are insufficient for evidentiary use. The paper's "when ensemble is irreplaceable" section is the most practically useful contribution for legal practitioners.

## Strengths

The paper correctly frames the theoretical result as establishing a worst-case lower bound on convergence requirements, not an exact mixing time. This is the right framing for legal purposes: the paper is not claiming that practitioners need $10^8$ steps, but rather that the worst case is that bad, which motivates the need for empirical convergence certification (as developed in G.4).

The finding that the practical rate-limiter is district count $k$ (not tract count $n$) is legally significant. It means that Texas and California — with large $k$ — require more steps than small states, not because of their geographic complexity per se, but because they have more districts to mix. This is a non-obvious result with practical implications for which states require more careful ensemble certification.

The division of labor table (Table 1 in Section 5) contrasting CS and certified ReCom is well-constructed and directly usable by courts and special masters.

## Weaknesses

**The paper's framing of Theorem 1 may be used adversarially in litigation.** The theorem establishes that the worst-case mixing time is $O(n^2 \log n)$ steps — which for California ($n = 8{,}057$) is approximately $2 \times 10^9$ steps. An expert witness for a defendant in a redistricting challenge could cite Theorem 1 as evidence that "even 50,000 steps is infinitesimally small compared to the theoretical mixing time, so the ensemble cannot be trusted." The paper should explicitly address this potential misuse: the worst-case bound applies to adversarially chosen initial conditions that do not arise in practice; empirical certification (G.4) demonstrates adequacy for typical starting conditions.

**The "expected frequency $< 1\%$" claim for CS outlier status (Section 6.4) is quantitative and should be supported by a derivation or citation.** If this is an inference from the B.7 50-state sweep (which showed Georgia's 511-seed tail as the maximum), then the claim requires explanation: how does "maximum observed tail of 511 seeds" translate into "fewer than 1% of states will produce an outlier plan under CS"? This calculation must be shown.

**The hybrid protocol for special masters (Section 6.4) has a gap.** The protocol says: generate with CS, audit with ensemble, adjust if outlier. But the adjustment mechanism is described only as "constrained bisection with modified objective." If the CS plan is an outlier in partisan outcomes (say, 3rd percentile of Democratic seats), what objective modification would bring it into a non-outlier range without introducing partisan considerations? The paper does not specify the adjustment, which means the protocol is incomplete for implementation.

## Minor Issues

- The paper notes that "5,000 steps is a minimum for NC" (from G.4). But if the theoretical mixing time is $10^8$ steps for NC, the 5,000-step minimum is justified only by empirical observation, not theory. The paper should clarify that the minimum step counts in G.4 are empirically motivated, not theoretically derived.
- The Lipton-Tarjan separator theorem is correctly cited for planar graphs. But census-tract adjacency graphs are not exactly planar — they may have crossing adjacencies due to bodies of water or unusual geographic features. The paper should note this assumption and whether the separator bound still applies approximately.

## Recommendation

Accept with minor revisions. Address the potential adversarial misuse of Theorem 1, source the $< 1\%$ outlier frequency claim, and specify the adjustment mechanism in the hybrid protocol.
