# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection

**Reviewer**: Percy Liang (Stanford University, Center for Research on Foundation Models)
**Expertise**: Machine learning, empirical evaluation methodology, statistical validity, benchmark design, model evaluation
**Date**: 2026-05-02

## Overall Assessment

VRASection is a well-motivated and clearly specified algorithm with a clean implementation. The paper's contribution --- layering a geographic alignment bonus on top of GeoSection's isoperimetric ratio scan --- is well-described, and the theoretical propositions are correct. The Alabama empirical result is interesting and the VRASection vs. GeoSection comparison table is the right framework.

My concern as a reviewer focused on empirical methodology is that the paper's statistical claims substantially outrun its data. The core empirical finding is a comparison of first-level bisection ratios across six states. In three states (AL, NC, SC), VRASection changes the selected ratio; in three states (MS, GA, LA), it does not. The paper presents this as a systematic empirical finding about VRASection's behavior, but the sample is too small, the statistical analysis is absent, and the relationship between first-level ratio change and VRA compliance improvement is not established.

Put concretely: the paper's headline result is that VRASection shifts the ratio in 3 of 6 states. This is interesting as a description, but it is not a statistically characterized finding. Are there states where VRASection would shift the ratio but should not? Are there states where VRASection fails to shift the ratio but should? What is the sensitivity of the 3/6 result to the specific parameter choices ($N=50$ seeds, $w_\text{vra}=0.40$, 2020 Census)?

The paper projects three hypotheses about full-run results (Section 4.3) but does not test them. For a paper submitting to a quantitative venue, this is the central gap.

## Score: 2/4

**My score**: 2/4 --- Good algorithm design but insufficient empirical evidence for the claims made. The paper should be revised to provide statistical characterization of the 3/6 state result and end-to-end MM district validation before resubmission.

## Major Strengths

1. **Mechanistic clarity**: The alignment score definition is precise, the objective function is well-specified, and the algorithm pseudocode is complete. The paper can be implemented directly from the description.

2. **The right comparison baseline**: Comparing VRASection to GeoSection (the immediate baseline) and MetisVra (the previous approach) is the correct experimental design. The table structure --- showing both first-level ratio change and MetisVra success status --- sets up a meaningful comparison.

3. **Alabama result is concrete**: The specific CLI output showing score comparisons ($EC_\text{norm} = 448.7$ for 2:5, score $= 322.5$; $EC_\text{norm} = 430.1$ for 1:6, score $= 350.2$) is the right kind of evidence. Showing the actual numbers makes the result verifiable.

4. **Sensitivity analysis framework**: Table 4 ($w_\text{vra}$ regimes) is a useful characterization of the parameter space, even though the empirical sensitivity analysis is described as pending.

## Major Issues (Must Address)

### Issue 1: The 3/6 State Finding Is Not Statistically Characterized
**Severity**: High
**Description**: The paper's primary empirical finding is that VRASection changes the first-level bisection ratio in 3 of 6 states (AL, NC, SC) and does not change it in 3 states (MS, GA, LA). The paper interprets this as supporting VRASection's design: it responds to geographic minority concentration (high $A$) where it exists and reduces to GeoSection where it does not.

But this 3/6 finding is not statistically characterized in any way. The paper does not report:
- The alignment scores at the winning ratio for each state (only Alabama's 0.42 is given)
- Whether the ratio change is stable across different seed draws (all runs use 50 seeds)
- The margin by which VRASection's winning ratio beats the GeoSection winner (the score gap)
- Whether the 3-state ratio change correlates with any measurable geographic property (Moran's I, Black VAP concentration, geographic clustering index)

Without this characterization, "3 of 6 states change ratio" is descriptive but not analytically useful. A reviewer at an empirical venue cannot assess whether this result is robust, whether it generalizes, or whether it is sensitive to parameter choices.

**Recommendation**: Report, for all six states: (a) the alignment score $A$ at the VRASection winning ratio; (b) the score gap between the VRASection winner and the GeoSection winner; (c) the alignment scores at all candidate ratios (not just the winner), as a bar chart or table. This would allow readers to understand how close the decision was in each state and whether the outcome is stable.

### Issue 2: The Relationship Between First-Level Ratio Change and MM District Production Is Not Established
**Severity**: High
**Description**: The paper presents first-level bisection ratio changes as proxies for VRA improvement, but the connection between ratio change and MM district count is asserted without evidence. The Alabama section projects that the 2:5 split will produce 1--2 MM districts in the southern sub-region, but this is conditional on the downstream GeoSection recursion behaving in a specific way.

The paper's hypothesis (Section 4.3, Hypothesis 1) states that VRASection should produce positive MM district counts in states where MetisVra failed. This is the key testable claim, and it is labeled as "pending." For a paper submitting to an election law or political science venue, this is the primary empirical question. An algorithms paper can get away with leaving this open, but a paper positioned as a VRA compliance contribution cannot.

**Recommendation**: Run the full pipeline for at least Alabama and one MetisVra-failure state (Louisiana or South Carolina) and report: (a) MM district count under GeoSection; (b) MM district count under VRASection; (c) MM district count under MetisVra (for comparison). For Alabama, the target is 2 MM districts per *Allen v. Milligan*.

### Issue 3: The Empirical Claim About the Abstract's MetisVra Comparison Is Overstated
**Severity**: Medium
**Description**: The abstract states that MetisVra "fails in 3 of 5 majority-minority states," which is a result from B.3. VRASection is then presented as resolving this failure. But the paper does not demonstrate that VRASection succeeds in the three states where MetisVra failed. It shows that VRASection changes the first-level bisection ratio in two of those three states (AL, SC), and that the ratio changes are algorithmically motivated. It does not show that the changed ratios produce MM districts.

The abstract's implicit comparison ("we fix what MetisVra broke") is not supported by the current empirical data. If VRASection produces 0 MM districts in Alabama and MetisVra also produces 0 MM districts in Alabama, the paper's central contribution claim is false.

**Recommendation**: Either (a) run the full pipeline and verify that VRASection achieves positive MM district counts where MetisVra produced zero, or (b) reframe the abstract to make clear that the paper's empirical contribution is the first-level ratio analysis and that end-to-end MM district validation is future work.

### Issue 4: The Sensitivity Analysis Is Described But Not Executed
**Severity**: Medium
**Description**: Section 5.1 describes three qualitative $w_\text{vra}$ regimes (low/default/high) and Table 4 lists five specific weight values. But the sensitivity analysis is entirely qualitative. The paper does not report what happens to the ratio selection in Alabama (or any other state) as $w_\text{vra}$ varies from 0 to 1.

For Alabama, the paper implies (from the remark about empirical sensitivity) that $w_\text{vra} = 0.20$ would not shift the ratio from 1:6 to 2:5, while $w_\text{vra} = 0.40$ does. But the critical threshold --- the minimum $w_\text{vra}$ at which the ratio shifts in Alabama --- is not reported. This is a basic sensitivity characterization that should be in the paper.

**Recommendation**: Report, for Alabama, the score of each candidate ratio as a function of $w_\text{vra} \in \{0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6\}$. Show at what weight the 2:5 ratio becomes the winner. This would provide an empirical calibration of the default parameter choice and support the claim that $w_\text{vra} = 0.40$ is appropriate.

## Minor Issues

- **N = 50 seeds vs. N = 20 seeds**: The abstract and Section 4.1 report using $N = 50$ seeds per ratio for the main experiments, but the verbatim CLI output in Section 4.2 says "20 seeds per ratio, 2020 Census." Which is correct?

- **Single run, not replicated**: The comparison table (Table 2) shows single results, not averages across multiple runs. Are these results stable across different runs? For METIS (a randomized algorithm), results can vary across invocations even with the same seeds if the seeds are machine-dependent. Clarify that these results are from a specific run and discuss stability.

- **The "projected results" section**: Section 4.3 describes three hypotheses that "will be tested" in future runs. This section reads as a placeholder. For a submitted paper, hypotheses should be tested, not projected. Either remove this section or move it to future work.

- **Missing error bars and confidence intervals**: The score values reported (EC_norm = 448.7 for ratio 2:5, score = 322.5) are single-point estimates from a specific run. There are no confidence intervals. How much would these values change with different seeds?

- **The 4.3% EC premium**: The paper describes this as "modest" and "within any legally defensible range." This is a qualitative judgment without empirical comparison. What is the EC premium distribution for random perturbations of the partition? A 4.3% premium that is in the 1st percentile of random perturbations is different from one in the 50th percentile.

## Questions for Authors

1. What is the minimum $w_\text{vra}$ at which the ratio shifts from 1:6 to 2:5 in Alabama?

2. Do the score values in Section 4.2 vary substantially across multiple independent runs with the same seeds? What is the standard deviation of EC_norm and the selection score across 10 independent runs?

3. The abstract says the full six-state comparison is "pending." When does the paper expect to complete this? Is it blocked on pipeline runtime, or on the analysis?

4. For states where VRASection and GeoSection agree (MS, GA, LA): what are the alignment scores at the winning ratio? If $A$ is near 0 for all ratios in those states, that would confirm the graceful degradation property empirically.

5. Are there states outside the six tested where VRASection would make the same ratio change as in AL/NC/SC but where the paper does not expect VRA benefits? This would test the false-positive rate of the alignment score as a predictor of VRA opportunity.

## Recommendation

Major revision required. The algorithm design is sound but the empirical evidence is insufficient for the claims made. The paper needs: (1) statistical characterization of the 3/6 ratio-change finding; (2) end-to-end MM district counts for at least Alabama; (3) sensitivity analysis showing how the ratio selection varies with $w_\text{vra}$; (4) clarification of the N=20 vs. N=50 discrepancy. The paper should be resubmitted as a full empirical paper once these results are available, not as a draft pending future runs.
