# Review R2: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Percy Liang (Stanford University, Center for Research on Foundation Models)
**Expertise**: Empirical methods, evaluation methodology, benchmark design, statistical rigor, machine learning systems evaluation
**Round**: 2
**Date**: 2026-05-02

## Overall Assessment

The revision substantially addresses the two highest-severity issues I raised in Round 1. The threshold sensitivity table (Table 5) directly answers the "5% threshold is unjustified" concern: it reports stable state counts at $\tau = 0.02/0.05/0.10/0.15$ (15/20/23/26), and the accompanying explanation correctly identifies which part of the sensitivity is signal versus artifact. The Iowa Type I/Type II decomposition framing is a genuine methodological contribution even in its current "two hypotheses" form — it shows the authors understand the confounding structure and have designed the right experiment.

My primary remaining concern is sample size. I gave a 2/4 in Round 1 specifically because 30 states is marginal for the 67% headline claim. The revision does not add states to the comparison — the comparable set is still 30 of 50 — and does not report the Wilson confidence interval I requested. The threshold sensitivity table does not address the sample size concern; it shows the finding is robust to threshold variation within the same 30-state sample, but the 95% CI on 20/30 = 0.67 is still approximately [0.48, 0.82]. The paper still presents "67% of states are stable" without acknowledging this uncertainty.

I am raising my score to 3.0/4, reflecting meaningful progress on threshold sensitivity and Iowa framing, but maintaining that the confidence interval omission is a gap for an evaluation paper.

## Score: 3.0/4

**My score**: 3.0/4 — Threshold sensitivity and Iowa framing are genuine improvements; sample size confidence interval still missing; CSS formula validation concerns from R1 partially addressed but not resolved.

## Changes Since Round 1: What Was Addressed

### Threshold Sensitivity (Issue 2 from R1 — Fully Addressed)

Table 5 is exactly what I requested. The four threshold levels cover the reasonable range, and the explanation that the 67% finding is "not a threshold artifact" because Iowa ($\Delta f = 0.31$) and the large-magnitude unstable states remain unstable at all thresholds above $\tau = 0.02$ is methodologically sound. The calibration argument — that 5% corresponds approximately to the sampling uncertainty in $f$ at 50 seeds for a 1,000-tract state — is the kind of principled justification that was missing in Round 1.

One observation about Table 5: the jump from 15 to 20 stable states between $\tau = 0.02$ and $\tau = 0.05$ means that five states have $\Delta f$ in the 0.02--0.05 band. These are exactly the threshold-sensitive cases. The paper presents this as confirming robustness, but for an evaluation paper, the interesting follow-up is: are these five borderline states systematically different from the clearly stable states? If they are smaller, slower-growing, or more geometrically simple, the borderline classification may reflect measurement noise rather than genuine geographic borderline status.

### Iowa Case Study (Issue 4 from R1 — Substantially Addressed)

The two-hypothesis framing is a methodological improvement over Round 1. Hypothesis A (Lorenz drift from suburban sprawl) and Hypothesis B (Type II tract redesign) are the correct decomposition of what could cause the observed $\Delta f = 0.31$. The authors correctly predict what data would distinguish them: a Type I/Type II experiment running GeoSection on the 2020 graph with 2000 population weights.

The remaining gap: the $p^*_{2000}$ value is projected as "$\approx 0.25$" from the ratio geometry but not computed from the sweep output. For an evaluation paper, a projected value derived from the thing being explained (the ratio outcome) is circular. The value should be computed from the Lorenz curve independently and reported as supporting or contradicting the hypothesis.

### CSS Formula Validation (Issue 3 from R1 — Partially Addressed)

The revision correctly limits the empirical claims to ratio stability ($f$ proxy) and clearly defers the $s_{\text{seat}}$ and $s_{\text{gap}}$ components. The s_seat fix — treating seat-count-changing states (MT, TX, NY) as "structurally changed" rather than "unstable" — is a definitional clarification that removes a category error from Round 1. However, the CSS weights (0.5/0.3/0.2) remain unvalidated, and the threshold classifications ($\geq 0.90$ = "highly stable") are still not empirically calibrated because no full CSS values exist yet.

## Remaining Issues

### Issue 1: Wilson Confidence Interval Still Missing
**Severity**: High
**Description**: The 67% stability finding (20/30 states) is the paper's headline empirical result. The 95% Wilson confidence interval for 20/30 is approximately [0.48, 0.82]. The paper reports "67%" as if it were a population parameter rather than an estimate from a 30-state sample. This is the central statistical concern for an evaluation paper, and it was not addressed in the revision.

The threshold sensitivity table is not a substitute for the confidence interval. The table shows the finding is robust to threshold variation; it does not show the finding is robust to sampling variation. These are different sources of uncertainty.

**Recommendation**: Report "20/30 comparable states (67\%, 95\% CI: [0.48, 0.82]) are stable." This is a single number that does not require additional computation. Add one sentence noting that if the remaining 20 states were comparable and showed the same rate, the 95% CI would narrow to approximately [0.52, 0.79] — still a majority but not an overwhelming one.

### Issue 2: Selection of 30 Comparable States Is Still Unexplained
**Severity**: Medium-High
**Description**: The paper reports 47/50 states completed the 2000 sweep and 48/50 completed the 2010 sweep. The cross-decade comparison covers 30 states. Why 30 rather than all 45--47 that have complete data in both years? The selection of the 30 comparable states is not described. If the 30 include the states that are easiest to compare (same seat count, stable geography), the 67% finding may be upward-biased relative to the true 50-state rate.

Round 1 flagged this; Round 2 does not address it.

**Recommendation**: Either (a) explain why only 30 states have paired 2000--2010 data when both sweeps have 47--48 states complete, and characterise how the 30 compare to the full set in terms of size, growth rate, and seat-count change; or (b) expand the comparison to all states with complete data in both years.

### Issue 3: The Lorenz Proxy Has Not Been Validated Against the Stability Outcome
**Severity**: Medium
**Description**: The paper proposes the Lorenz drift proxy ($\Delta p^*$) as a leading indicator of ratio instability. For the 30 states with 2000 and 2010 complete data, this prediction can be validated now: compute the correlation between $\Delta p^*$ and $|f_{2000} - f_{2010}|$. If the proxy has good precision and recall for the binary stable/unstable classification, it is a genuine predictive tool. If the correlation is weak, the proxy's practical value is limited.

This validation was requested in Round 1 and was not executed in Round 2.

**Recommendation**: Report Spearman correlation between $\Delta p^*$ and $\Delta f$ across the 30 comparable states. Even a figure with $\Delta p^*$ on the x-axis and $\Delta f$ on the y-axis would constitute adequate validation.

## Minor Issues

- The Jaccard proxy table (Table 4) has NC in the Medium section but with $\Delta f = 0.00$ and label "High." This appears to be a table placement error.

- The paper now provides a methodological justification for the 5% threshold (corresponds to sampling uncertainty at 50 seeds), but this argument is presented only in the threshold sensitivity paragraph and not in the methodology section where the threshold is first defined. A forward reference would help.

- The $p^*_{2000} \approx 0.25$ projection for Iowa is described as "expected direction" — this language suggests a prediction, not a computation. If this projection is derived from the ratio geometry (a 1:3 split at $k=4$ implies $p^* = 0.25$ by construction), the paper should say so explicitly. The value is not informative as an independent check on the hypothesis.

## Questions for Authors

1. What is the exact selection criterion for the 30 comparable states? Which 17--18 states with complete 2000 or 2010 sweeps are excluded from the comparison, and why?

2. Has the Lorenz proxy been validated against the 30-state $\Delta f$ outcomes? Is there a positive correlation between $\Delta p^*$ and $\Delta f$?

3. For the five states that change classification between $\tau = 0.02$ and $\tau = 0.05$ in Table 5: what are their $\Delta f$ values? Are they the same states that appear near the stability boundary in Table 4 (AL at 0.06, SC at 0.07, MS at 0.05, UT at 0.07)?

4. Is Iowa's 2000 pipeline sweep output available? If so, what does the pipeline report for $p^*_{2000}$ independently of the ratio arithmetic?

## Recommendation

The revision shows real methodological improvement. The threshold sensitivity table and the Iowa Hypothesis A/B decomposition address the most important concerns from Round 1. My score rises from 2/4 to 3/4. The paper can reach acceptance if it adds the Wilson confidence interval, characterises the 30-state selection, and validates the Lorenz proxy empirically. These additions do not require new data collection — they can be computed from the current sweep outputs. With those additions, the evaluation methodology would meet journal standards.
