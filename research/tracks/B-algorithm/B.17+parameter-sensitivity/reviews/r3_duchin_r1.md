# Review 3 — Reviewer: Moon Duchin (Metric Geometry / GerryChain)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 1
**Score:** 2/4

## Summary

This paper makes a reasonable empirical contribution, but it has a significant methodological gap that undermines the strength of the main claim. The OAT design is inadequate for ruling out parameter manipulation, and the paper's conclusion that parameter combinations cannot produce meaningful partisan effects is not empirically established — it is argued by an additive upper bound that assumes away the most dangerous interactions. The framing is also overly reassuring in ways that could be exploited in litigation.

## Strengths

The basic finding — that individual parameters produce at most ±0.2 seat national effects — is well-documented and plausible. The comparison to algorithm-structure effects (60x larger) is the right frame for contextualizing the result. The compactness-vs-partisanship separability finding is genuinely interesting and well-explained.

The statistical framing of the binomial test (Section 3.3) is appropriate for the adversarial concern: can a hostile actor find a parameter combination that systematically favors one party? This is the right question, even if the answer isn't fully established.

## Weaknesses and Concerns

The core problem is the OAT design and the additive bound. The paper acknowledges (Section 7.1) that "a full factorial design (5^5 = 3,125 runs) would be more complete but computationally expensive." But the relevant threat model is not the full parameter space — it is the adversarial parameter combination. An adversary with knowledge of the algorithm structure and the geographic distribution of partisan voters would not choose parameters randomly; they would search for the combination that maximizes their partisan advantage.

The additive bound of 0.5 seats is derived by assuming that all five parameters move together in the most unfavorable direction AND that their effects are additive. But the paper's own data shows that `ufactor` and `aswing` both have monotone partisan effects in the same direction (lower ufactor and lower aswing both produce slightly more Democratic outcomes). If these effects reinforce each other, the combined effect could be larger than the additive bound suggests.

The paper needs a targeted adversarial experiment: fix the "most Republican-favorable" values for all five parameters simultaneously (ufactor=5%, acounty=10, T=1000, aswing=1.20, wvra=0.6) and measure the actual joint effect. This is one run, not 3,125. The paper claims the joint effect is ≤0.3 seats "empirically," but this crucial run is not documented in the results section — only the additive calculation is shown. If this run has been done, report it. If it hasn't, do it.

There is also a conceptual problem with the framing. The paper argues that parameter insensitivity makes it safe to "fix parameters in statute" because no parameter change "within the studied range" produces a partisan effect exceeding 0.3 seats. But the studied range is itself a choice — the ranges for ufactor (0.1%-5%) and acounty (1.0-10.0) were presumably chosen based on what seems reasonable for redistricting purposes. A hostile actor might argue for parameters outside these ranges. The paper should explicitly justify the range endpoints as the appropriate statutory boundaries, not just as the ranges studied.

The multiple testing correction for the binomial test is not reported. With five parameters and five grid points each, there are 20 non-baseline comparisons. If the paper is testing whether each of these produces a systematically partisan result, it should apply a correction for the 20 tests. Without this, the absence of a significant binomial test result is less convincing.

## Minor Issues

- Section 5.2 claims the parameter effect is "approximately 60x smaller" than the algorithm-structure effect. This calculation (from B.0's 12.8pp gap, scaled to 435 districts, giving ~18 seats vs. 0.3 seats) is correct in order of magnitude but uses a very high estimate for the B.0 algorithm-structure effect. A more careful presentation would give the range (e.g., 40x to 80x depending on the baseline state).
- The ConvergenceSweep convergence claim ("For T >= 200, outcomes are identical across all 50 states") is stated as fact but needs supporting evidence beyond mean D_nat values.
- The paper cites B.16 extensively but there is no B.16 in the reference list. This needs to be resolved.

## Recommendation

Major revisions required. The adversarial joint run (all five parameters at their most partisan-favorable values simultaneously) must be included as a primary result, not deferred as "future work." The additive bound argument is insufficiently rigorous for the policy claims being made.
