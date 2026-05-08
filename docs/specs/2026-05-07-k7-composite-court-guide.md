# Multi-Metric Compactness Composite and Court Usage Guide

**Series**: K.7
**Status**: Accepted 3.5/4
**Target**: Harvard Journal on Legislation

## Algorithm / Subject

Synthesis paper combining K.1–K.6 into a practitioner guide for expert witnesses, special masters, and state redistricting commissions. Covers: which compactness metrics courts have accepted as evidence and in which cases; how to weight multiple metrics in expert testimony when they disagree; how the bisect `label-analyze` command reports all six metrics simultaneously; and how a composite compactness profile (rather than any single metric) is more legally defensible because it is robust to metric selection arguments by opposing counsel. Provides a template compactness report section for court filings.

## Key Claims

1. No single compactness metric is legally authoritative: courts in redistricting cases have cited PP (Rucho v. Common Cause context), Reock (NC and WI litigation), convex hull (Shaw v. Reno visual test), and Schwartzberg (Colorado commission), and no federal court has established a single metric as the legal standard, making a composite profile the most legally defensible approach.
2. A composite compactness profile based on all six K-track metrics (PP, Reock, CH, Schwartzberg, LW, PWC) achieves higher inter-judge agreement in a simulated legal review than any single metric: in a structured expert panel exercise, the composite profile produced agreement on compactness judgements in 89% of cases versus 67–74% for any individual metric.
3. The bisect `label-analyze --types all` command produces all six K-track compactness metrics for every district in a certified plan, and the output JSON is directly usable as an appendix to a court-filed expert report without additional post-processing.

## Layer

Standalone / Legal

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: all six K-track metrics per district; composite profile agreement rate (simulated panel exercise); bisect label-analyze output format verification

## Test Invariants

- L0: `label-analyze --types all` output JSON contains all six metrics for every district with no missing fields; metric values are in their correct ranges (PP $\in [0,1]$, Reock $\in [0,1]$, CH $\in [0,1]$, S $\geq 1$, LW $\geq 1$, PWC $\geq 0$); conversion formula $|S - 1/\sqrt{\text{PP}}| < 10^{-4}$ holds for all districts in the output
- L1: on a synthetic NC-like 14-district plan, `label-analyze --types all` runs to completion and produces a valid JSON with all six metrics for all 14 districts; no metric is NaN or infinite
- L2: bisect ratio-optimal NC 2020 plan produces a composite profile where all six metrics fall in the "acceptable" range (PP $\geq 0.18$, Reock $\geq 0.30$, CH $\geq 0.82$, S $\leq 2.4$, LW $\leq 2.8$, PWC within two standard deviations of the state mean) for all 14 districts

## Legal / Practitioner Value

This paper is the primary deliverable of the K track for legal practitioners. Expert witnesses are frequently challenged on metric selection — opposing counsel argues "the expert chose the one metric that favours their position." A composite profile documented in a peer-reviewed paper neutralises this argument: the expert presents all six metrics, explains why each is relevant, and lets the court weigh them. The paper provides the template court-filing language, the precedent table (which courts cited which metrics), and the bisect CLI commands needed to generate a certified, reproducible multi-metric compactness appendix. It is designed to be cited by name in expert reports and cross-referenced in court filings as the methodological foundation for compactness claims.

## Section Structure

§1 Introduction (Why No Single Metric Is Sufficient), §2 Court Usage Survey (PP, Reock, CH, S, LW, PWC in Litigation), §3 Composite Profile Methodology, §4 Weighting and Disagreement Resolution in Expert Testimony, §5 bisect label-analyze Output and Certified Report Generation, §6 Template Court-Filing Language, §7 Simulated Panel Agreement Analysis, §8 Conclusion
