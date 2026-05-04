> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Percy Liang (Stanford University, Center for Research on Foundation Models)
**Expertise**: Empirical methods, evaluation methodology, benchmark design, statistical rigor, machine learning systems evaluation
**Date**: 2026-05-02

## Overall Assessment

StabilitySection is an evaluation paper more than an algorithm paper: it proposes a metric (CSS) for assessing whether a redistricting algorithm's output is invariant to the choice of input data. Evaluated as an evaluation paper, it has real methodological strengths --- the CSS framework is well-motivated, the Type I/Type II decomposition is the right way to isolate confounders, and the 67% stability finding is reported with appropriate caveats. But the paper has significant weaknesses from an empirical methods standpoint: the primary metric (CSS) cannot be computed from the current data, the stability threshold (5%) is unjustified and not subjected to sensitivity analysis, and the sample size (30 comparable states in the 2000--2010 analysis) is marginal for the claims being made.

The paper would be improved substantially if it were reframed as presenting what it actually delivers: a ratio-stability framework for the 2000--2010 cross-decade comparison, with CSS as the target metric for when three-census data is available. The current draft presents CSS prominently but computes only a subset of its components.

## Score: 2/4

**My score**: 2/4 --- Methodologically interesting framework; current empirical findings are preliminary and the primary metric is underspecified relative to what the data supports. The paper needs more data before it can support its main claims.

## Major Strengths

1. **The CSS decomposition is well-designed**: Splitting the stability measurement into $s_{\text{seat}}$, $s_{\text{ratio}}$, and $s_{\text{gap}}$ is the right design. Each component measures a distinct dimension of stability, and the weights (0.5, 0.3, 0.2) reflect an explicit ordering of importance. For an evaluation framework, this kind of structured decomposition is preferable to a single holistic score.

2. **The Type I / Type II decomposition is methodologically rigorous**: Population-shift effects and tract-boundary-redesign effects are confounded in a naive cross-year comparison. The proposed decomposition --- running GeoSection on the 2020 graph with 2000 populations --- is the right experiment to isolate these effects. This would be a genuine methodological contribution if executed.

3. **The Lorenz drift proxy is a testable prediction**: The claim that $\Delta p^* > 0.05$ between census years predicts ratio instability is a falsifiable empirical claim. For states where 2000 data is complete, this can be tested now. The paper should report this validation rather than presenting the proxy as merely theoretical.

4. **Appropriate use of [TBD] markers**: The paper is honest about what is complete versus in progress. This is good scientific practice for a draft paper.

## Major Issues (Must Address)

### Issue 1: 30 States Is a Marginal Sample for the Core Claim
**Severity**: High
**Description**: The 67% stability result (20/30 states) is the paper's headline empirical finding. But 30 is a small sample for a claim about the general behavior of the algorithm. The confidence interval on 20/30 = 0.67 is approximately [0.48, 0.82] at 95% confidence (Wilson interval). The paper presents "67% of states are stable" as a strong positive result, but the confidence interval reaches as low as 48% --- not much better than chance for a binary stability classification.

More importantly, the 30 states are not a random sample. The paper reports 47/50 states completed the 2000 sweep and 48/50 completed the 2010 sweep, but the cross-year comparison covers only 30 states. The selection of which 30 states are comparable is not described in sufficient detail. If the 30 comparable states are systematically different from the 20 non-comparable states (e.g., they are the smaller, simpler states), the 67% finding may not generalize.

**Recommendation**: (1) Report the Wilson confidence interval for the 20/30 finding. (2) Characterize the 30 comparable states: how were they selected? Are they representative of all 50 states in terms of size, population growth rate, and geographic complexity? (3) Discuss what the finding would mean if only 10 of the remaining 20 states are stable --- would the overall rate be 30/50 = 60% or 25/50 = 50%?

### Issue 2: The 5% Threshold for Ratio Stability Is Arbitrary
**Severity**: High
**Description**: The paper classifies a state as stable if $|f_{2000} - f_{2010}| \leq 0.05$ and unstable otherwise. This threshold is stated without justification. The choice of 5% is consequential: a state at $\Delta f = 0.06$ is classified as unstable while a state at $\Delta f = 0.04$ is classified as stable, even though these are nearly identical.

For the 20/30 finding to be robust, the paper needs to show that the classification is not threshold-sensitive. A simple robustness check: how many states change classification at 3% and 10% thresholds? If the 67% result is robust across these thresholds, the finding is credible. If the classification changes substantially, the threshold needs theoretical justification.

The threshold also needs a connection to the CSS framework. The binary $s_{\text{ratio}}$ in the CSS formula is 1 if ratios are identical and 0 otherwise, which is different from the 5% threshold used in the cross-decade comparison. These two stability measures should be unified or the discrepancy should be explained.

**Recommendation**: Report robustness to 3% and 10% thresholds. Provide a theoretical justification for 5% (e.g., it corresponds to approximately 1/2 of one district's worth of tracts for a typical 8-district state). Connect the continuous threshold measurement to the binary $s_{\text{ratio}}$ in the CSS formula.

### Issue 3: The CSS Formula Has Not Been Validated
**Severity**: High
**Description**: The CSS weights (0.5 for seat stability, 0.3 for ratio stability, 0.2 for gap similarity) are justified by appeal to "legal importance ordering" but are otherwise arbitrary. Any weighted composite metric requires at least one of: (a) a theoretical derivation of the weights, (b) an empirical fit to outcome data, or (c) a sensitivity analysis showing the conclusions are robust to weight variation.

The paper provides none of these. The weights are stated once ("The weights reflect the legal importance ordering") without any further justification. Given that $s_{\text{seat}}$ cannot currently be computed, the formula is effectively validated only on $s_{\text{ratio}}$ and $s_{\text{gap}}$, which together contribute only 50% of the CSS.

Furthermore, the CSS thresholds ($\geq 0.90$ = "highly stable," $0.70$--$0.90$ = "moderately stable") are also stated without empirical calibration. What does CSS $= 0.90$ mean in practice? If a state has $s_{\text{seat}} = 1$, $s_{\text{ratio}} = 1$, $s_{\text{gap}} = 0.50$, its CSS is $0.5 + 0.3 + 0.1 = 0.90$ --- but is a state with a proportionality gap that swings by 50 percentage points between censuses really "highly stable"?

**Recommendation**: Add a sensitivity analysis showing how the CSS threshold classifications change if weights are varied by $\pm 0.1$. Add a worked example (e.g., Iowa) showing what each CSS threshold implies in practice. If the formula cannot be validated empirically yet, present it explicitly as a proposed framework pending validation, not as an established metric.

### Issue 4: Iowa's $\Delta f = 0.31$ May Not Reflect a Meaningful Geographic Change
**Severity**: Medium
**Description**: The Iowa instability result ($f_{2000} = 0.18 \to f_{2010} = 0.49$) is striking but its interpretation is not clear. The tract split ratio $f$ measures the fraction of tracts on the left side of the first bisection, not the fraction of population. A shift from 0.18 to 0.49 in tract count terms may correspond to a much smaller shift in population terms if the tracts differ substantially in size.

Iowa's census tract structure changed between 2000 and 2010: the Census Bureau redesigned tracts in many rural counties. If the 2000 tracts in Iowa were predominantly small rural tracts and the 2010 tracts consolidated some of these, the shift from $f = 0.18$ to $f = 0.49$ might partially reflect tract redesign rather than a genuine change in the optimal geographic partition. This is exactly the Type I vs. Type II decomposition the paper proposes but has not yet executed.

For Iowa specifically: is the $\Delta f = 0.31$ consistent with a genuine geographic reorganization, or could it be driven by the 2010 tract boundary redesign? Without the Type I/Type II decomposition, the Iowa result is ambiguous.

**Recommendation**: Run the Type I/Type II decomposition for Iowa as a priority case. If the population-only perturbation (2020 graph with 2000 populations) shows the same $\Delta f$ as the full cross-year comparison, the shift is driven by population dynamics. If the population-only perturbation shows much smaller $\Delta f$, the instability is primarily due to tract redesign, which is a different kind of finding.

## Minor Issues

- **Lorenz drift proxy validation is missing**: Proposition 1 predicts that states with $\Delta p^* > 0.05$ will show ratio instability. For the 47 states with 2000 data complete, this prediction can be tested empirically. The paper should report the validation: what is the precision and recall of the Lorenz proxy for predicting the binary stable/unstable classification?

- **"50 seeds per ratio" may be insufficient for convergence**: The paper reports 50 seeds per ratio for the 2020 baseline. For states with many ratios to evaluate (TX at $k=38$ has 19 candidate ratios), 50 seeds per ratio may produce high variance in the edge-cut estimates. The paper should report the standard deviation of the normalised edge-cut across seeds, at least for a representative state.

- **The North Carolina case is unclear**: The paper reports NC had a ratio shift from 6:8 (2020) to "preliminary 2000 data suggests 5:8." A shift of one unit at the left side of the ratio (6 to 5) out of 14 is a 7% change in the first-level split. Is this within or outside the 5% threshold? If measured in tract-fraction terms rather than ratio-integer terms, the classification may differ.

- **The Lorenz Drift Proposition needs $C_{\text{pop}}$ estimated empirically**: The stability bound involves $C_{\text{pop}}$, described as "the maximum rate of change of normalised edge-cut per unit of population fraction shift." Without a numerical estimate, the proposition cannot be used as a quantitative predictor.

## Questions for Authors

1. How were the 30 comparable states selected from the 47 with complete 2000 data and 48 with complete 2010 data? What made the remaining states not comparable?

2. For the 20 stable states, what is the distribution of $\Delta f$ values? Are they clustered near 0 or spread across the 0--0.05 range?

3. For the 10 unstable states, are any near the stability boundary (0.05--0.10)? Is Alabama at 0.06 classified as unstable, or does it fall in a gray zone?

4. Does the paper have any data on whether the 5% threshold for $f$ corresponds to any meaningful quantity in the algorithm's behavior (e.g., half a district's worth of tracts)?

## Recommendation

The paper has the right methodological framework and reports an interesting preliminary finding. But the CSS formula is not fully computable from the current data, the 5% threshold is unjustified, and the sample of 30 states does not support strong confidence in the 67% headline. The paper should be revised to clearly distinguish what is computed from what is projected, add robustness checks for the threshold, and provide confidence intervals for the stability rate. With those additions, the framework would be publishable as a methods contribution.
