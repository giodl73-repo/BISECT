# Track S — Statistical Inference for Redistricting

**Theme**: The program uses confidence intervals (C.7), ESS-based uncertainty (G.4), and bootstrap (C.1), but has no unified statistical inference framework. Track S provides the foundation: how do you formally test whether a plan is a statistical outlier, how do you compute valid confidence intervals on redistricting metrics, and how do you handle multiple testing when evaluating dozens of metrics across dozens of states?

**Core question per paper**: How do you draw valid statistical inferences from redistricting data?

## Track Chain

S.0 (overview: statistical challenges in redistricting)
  → S.1 (hypothesis testing: is this plan an outlier?)
  → S.2 (Bayesian redistricting)
  → S.3 (power analysis for gerrymandering detection)
  → S.4 (multiple testing and correction)

## Papers

| Paper | Title | Method | Application |
|-------|-------|--------|-------------|
| S.0 | Statistical Inference in Redistricting: Challenges and Framework | Overview | Cross-track |
| S.1 | Hypothesis Testing for Partisan Gerrymandering: Permutation Tests and Calibrated p-Values | Permutation test; calibrated ensemble | G.1 percentile claims |
| S.2 | Bayesian Redistricting: Prior Specification and Posterior Interpretation | Bayesian inference | Ensemble uncertainty |
| S.3 | Power Analysis for Gerrymandering Detection: How Many Plans Do You Need? | Power analysis | G-track ensemble size |
| S.4 | Multiple Testing in Redistricting Analysis: FDR Control Across Metrics and States | Benjamini-Hochberg; Bonferroni | L-track metrics |

## Contracts

Every S.1–S.4 paper must:
- Provide a worked example on NC 2020 data using the proposed method
- Compare to the naive/uncorrected approach and quantify the difference
- Specify assumptions required for validity
- Provide code or pseudocode for the method

## Key Problems to Solve

**S.1**: The G.1 percentile claim ("bisect plan at 0.4th percentile") is a point estimate with ESS≈42 for TX. What is the valid p-value under a properly calibrated permutation test using the GerryChain ensemble? This is what courts need — a p-value, not a percentile estimate.

**S.2**: If we treat the ensemble as a prior over plans, what is the posterior distribution of "fairness" metrics given the observed plan? Bayesian framing converts the ensemble from a frequentist description to a probabilistic one.

**S.3**: How large does a GerryChain ensemble need to be to reliably detect a 10pp partisan advantage at 80% power? This determines the minimum steps for litigation-grade ensembles.

**S.4**: The L-track reports 7 partisan fairness metrics across 50 states × 3 cycles = 1050 tests. Without multiple testing correction, a 5% significance threshold expects 52 false positives. S.4 shows how to apply FDR control to the full test battery.

## Relationship to Existing Program

S.1 extends G.1 (ensemble percentiles → calibrated p-values).
S.2 extends G.7 (SMC ensemble → Bayesian posterior).
S.3 extends G.4 (ESS → minimum ensemble size for power).
S.4 extends L.0 (partisan metrics → corrected inference).

Track S is the statistical foundations track that makes all ensemble-based claims litigation-grade rather than merely descriptive.
