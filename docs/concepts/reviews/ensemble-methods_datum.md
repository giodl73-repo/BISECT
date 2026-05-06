# Review: ensemble-methods.md
**Reviewer**: DATUM (peer-reviewer)
**Date**: 2026-05-05

---

## What's accurate

The conceptual framing — `redist` finds the optimum, GerryChain characterises the distribution — is accurately stated and is the correct relationship between these two tools. The guide correctly describes ReCom's stationary distribution as "probability proportional to the number of balanced spanning tree cuts that can generate each plan" and correctly identifies this as "a well-characterised distribution that allows rigorous inference." The description of Wilson's loop-erased random walk is technically accurate for uniform spanning tree sampling. The treatment of ESS — that consecutive ReCom plans are autocorrelated because each step changes only two districts — is an accurate and well-explained motivation. The statement that the Hamming autocorrelation lag-1 "above 0.5 suggests the chain is stuck and needs longer steps or a different initialisation" is appropriate guidance, though conservative. The two legal certificate framings (maximum-compactness and geographic-inevitability) are described as distinct argument structures, which is the correct way to present them — they are complementary, not redundant.

---

## P1 — Required fixes

**Section "redist-ensemble", estimated throughput**: The guide states "~50,000 steps per second for NC-scale graphs (estimated 2,300x speedup over Python GerryChain)." The word "estimated" is doing significant work here: the GerryChain speed figure is given as approximately 21 steps/second in the preceding section (a specific measurement). The 50,000 steps/second is not labeled as measured or theoretical. DATUM requires this distinction. If this is a benchmark result, state the hardware (CPU model, core count, memory), the NC graph size used, and the benchmark date. If it is a theoretical estimate based on algorithmic complexity, label it as such. Citing an "estimated 2,300x speedup" in a legal certification without the basis for the estimate is not rigorous.

**Section "The legal argument", maximum-compactness certificate**: The guide states "the ApportionRegions plan sits at the 0.1-0.2nd percentile of the ensemble edge-cut distribution" for Wisconsin, Georgia, and Pennsylvania. It then provides a draft certificate stating "fewer than 2 in 1,000 randomly drawn valid plans are more compact." DATUM notes that this certificate is derived from a specific ensemble size (the guide earlier says "a GerryChain ReCom ensemble of 1,000 valid redistricting plans"). A 1,000-plan ensemble gives an empirical percentile with substantial uncertainty — the true percentile could be 0.05% or 0.5% depending on the distribution. The certificate should acknowledge the ensemble size and the resulting uncertainty in the percentile estimate. A 10,000-plan ensemble would give a more defensible estimate.

**Section "The legal argument", geographic-inevitability certificate**: The certificate draft states that the ApportionRegions plan's edge-cut fraction is "within 0.3% of the ensemble mean." This is a relative difference (percentage of the mean), which is a descriptive statistic, not a statistical test. "Less than one-twentieth of one standard deviation above the mean" is more informative but is an additional statistic that should be computed from reported standard deviation figures. DATUM asks: what is the ensemble mean edge-cut fraction, the standard deviation, and the z-score? These three numbers would make the certificate's claim falsifiable and reproducible from the reported data.

**Section "GerryChain / ReCom"**: The guide states GerryChain samples plans "approximately uniformly at random." The qualifier "approximately" is important and should be explained. The stationary distribution of ReCom is not uniform over all valid plans — it is biased toward plans with more balanced spanning tree cuts. This means the ensemble is not a uniform sample of the feasible space, which is a distinction that matters for the inference claims ("certifying the `redist` plan"). The guide's framing of "rigorous inference" should clarify what distribution GerryChain actually samples and what assumptions are needed for the inference to be valid.

**Section "Ensemble diagnostics", ESS thresholds**: The guide states "ESS >= 1000: Reliable inference. 400 <= ESS < 1000: Adequate for descriptive statistics; marginal for tail inference. ESS < 400: Run longer chains." These thresholds are presented without citation. The standard ESS thresholds in the MCMC literature (Gelman et al., 2013) use 100 as the minimum adequate threshold for bulk statistics, with higher thresholds (400+) recommended for tail percentiles. The guide's 1000 threshold is more conservative than the standard literature — DATUM does not object to conservatism, but the source of these thresholds should be cited so they can be evaluated.

---

## P2 — Suggested improvements

The guide would be stronger if it included a concrete example walkthrough: given a 1,000-step ensemble for NC, what does the ESS, R-hat, and Hamming autocorrelation output look like? Concrete example values (even from a brief run) would help practitioners calibrate their own results. Abstract threshold tables without reference distributions are harder to apply correctly.

The failure-handling section for `redist-ensemble` (10 resamples then select new pair) should note that this introduces a deviation from GerryChain's exact stationary distribution: GerryChain's Python implementation may handle pair-reselection differently, which could result in slightly different stationary distributions between the two implementations. If the Rust implementation is intended to replicate GerryChain exactly, this deviation should be formally characterised.

---

## Score: 2/4

The ensemble methods guide contains several claims that are presented as established results but lack the evidentiary scaffolding needed for legal or academic use. The throughput estimate, the percentile uncertainty from a 1,000-plan ensemble, and the "approximately uniform" sampling qualifier are the three most consequential gaps. The legal certificates are well-structured in form but require quantitative grounding before they can be submitted to a court.
