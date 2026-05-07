---
reviewer: Christopher Liang
round: 1
score: 3
date: 2026-05-06
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting

### Strengths

The empirical design is well-chosen for the research question. Using $T = 101$ with $p \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ produces exact quantile ranks with no interpolation, which is the correct choice for a transparent, reproducible analysis. The decision to restrict the sample to states with G.1 ensemble baselines is methodologically sound: it allows the author to situate each percentile within the broader ReCom distribution, which is what makes the insensitivity finding legally actionable rather than merely statistical. Reporting both $\ECnorm$ and Polsby-Popper in Table 2 confirms that compactness is genuinely varying across percentiles even when partisan outcomes are not, which rules out the trivial explanation that all plans in the bisection family are identical.

The mechanistic explanation is correct and internally consistent. If the coefficient of variation of $\ECnorm$ across seeds is below 2% (B.7), and if partisan outcomes are a monotone function of compact geographic bundling (B.0), then the 101-plan bisection distribution should be tightly clustered in both compactness and partisan outcome space. The zero-seat variation finding follows logically from these priors. The paper earns its conclusion rather than asserting it.

The NC case (§4.3) is handled correctly. The $\Delta\ECnorm = 0.0039$ being the smallest of the six states is consistent with geographic convergence compressing the bisection distribution, and the observation that both compactness doctrine and representativeness doctrine converge on the same plan for NC is a genuinely interesting special case worth highlighting.

### P1 Items (must resolve before acceptance)

**P1-A: $T = 101$ is insufficient to establish the insensitivity claim as a general result.**
The central empirical claim requires that the $p = 1.0$ plan (the compactness worst case — rank 100 out of 101) not differ from the $p = 0.0$ plan in partisan outcomes. With $T = 101$ draws, the maximum of 101 draws from a distribution with true CV = 2% will typically sit at approximately $\mu + 2.5\sigma$ (expected maximum of 101 draws from a normal). This is not particularly far from the mean. However, the tails of the seed-outcome distribution for METIS are not Gaussian and are not characterized in the paper. For states with higher CV (Georgia CV = 4.3%, NC CV = 3.8% per B.7), the maximum of 101 draws could be a meaningful outlier relative to T = 600 draws. The paper should run at minimum a sensitivity check: what is the partisan outcome of the $p = 1.0$ plan for GA and NC at $T = 601$? If the outcome is still identical to $p = 0.0$, the insensitivity claim is robust to $T$. If it differs, the claim is conditional on $T = 101$ and should be stated as such.

**P1-B: The 0.5-seat bound in the abstract is inconsistently stated relative to the results.**
The abstract states "partisan seat counts vary by at most 0.5 seats across $p$" and the introduction repeats this bound (§1.3). But §4.4 reports "the maximum variation in projected D seats for any individual state is 0 seats." The 0.5-seat bound appears to have been written before the experiments were run and then not updated in the abstract/introduction. If the actual result is zero seats, the abstract should say so (with the caveat that TX and CA are interpolated). If 0.5 seats is a theoretical bound intended to allow for the interpolated states, that must be explained. The current inconsistency reads as if the headline was not updated after the data came in, which damages credibility.

**P1-C: The experimental evidence does not cover the full percentile range for large-delegation states.**
Texas ($k = 38$) and California ($k = 52$) together account for 90 seats — more than 20% of the House. These are also the states where bisection complexity is highest (large prime-factorisation trees with many levels) and where leaf-level METIS variance is most likely to produce partisan surprises at extreme percentiles. Yet these are precisely the two states where actual PS sweeps were not run. The interpolation from B.11 is from a different experiment (a single plan at $p = 0.0$, not a sweep across $p$). The paper is making a claim about percentile insensitivity for TX and CA based on zero data about the variation across percentiles for those states. This is not a minor gap; it is a gap in the primary empirical contribution for the two largest states. These sweeps must be run before the insensitivity claim can be stated as established for the six-state sample.

### P2 Items (recommended improvements)

**P2-A:** The paper should report the realized seed values $s_0$ through $s_{100}$ (or a hash thereof) for at least one state in an appendix. This would allow independent verification of the seed derivation and is in keeping with the paper's emphasis on auditability and reproducibility.

**P2-B:** Table 2 reports Mean PP compactness only at $p = 0.0$. For completeness, the Polsby-Popper at $p = 1.0$ should also be reported to characterise the compactness loss under the worst-case percentile. The current presentation understates the compactness variation by anchoring only on the best case.

**P2-C:** The paper's future work item on $T$ sensitivity (§6.3) should be elevated to a more prominent position. Given that $T$ determines the resolution of the percentile estimate and that P1-A above identifies this as a material gap, the $T$-sensitivity analysis should arguably precede the present paper rather than follow it.

**P2-D:** The projected Democratic seat count methodology — "applying the 2020 presidential vote margins at the census-tract level to the district boundaries" — is the ecological inference approach. This should be noted as a standard but imperfect measure; presidential vote is a reasonable proxy for partisan lean but district-level seat assignments under actual candidate competition may differ.

### Score

**3 / 4** — Accept with revisions. The insensitivity claim is empirically plausible and mechanistically well-grounded, but the abstract/body inconsistency (P1-B) and the missing TX/CA sweeps (P1-C) are material gaps that prevent acceptance in the current form. Running the actual TX and CA sweeps (approximately 7 minutes of computation by the paper's own runtime estimate) would resolve P1-C; fixing the abstract would resolve P1-B.
