# Peer Review: The 42% Threshold
**Reviewer**: Nicholas Stephanopoulos (Harvard Law School)
**Expertise**: Efficiency Gap, Partisan Symmetry, Quantitative Legal Standards
**Date**: 2026-02-08
**Round**: 1

## Overall Assessment

This paper represents an ambitious and valuable attempt to bring quantitative rigor to VRA compliance assessment. The identification of a 42% state-level minority threshold through systematic algorithmic testing is methodologically sound and fills a genuine gap in redistricting scholarship. The statistical evidence is compelling (r=0.88), the sample design is appropriate for the research question, and the policy implications are clearly articulated.

From a quantitative legal perspective, this work exemplifies how empirical analysis can inform—though not replace—legal judgment. The paper's strength lies in demonstrating that VRA feasibility has measurable, predictable patterns rather than being purely case-specific. However, the transition from empirical findings to legal standards requires more careful calibration. Courts adopt quantitative metrics cautiously (witness the efficiency gap's limited uptake), and this paper must address how its threshold would interact with existing doctrine and judicial preferences.

The methodology is stronger than the legal framing. The edge-weighted optimization approach is well-justified, the ablation study (140 configurations) is comprehensive, and the statistical analysis appropriately identifies state minority percentage as the dominant predictor. Minor methodological concerns exist (single METIS runs, N=5 states) but don't undermine core findings. The paper would benefit from treating its 42% threshold as an empirical regularity requiring legal interpretation rather than a self-executing legal standard.

## Score: 3/4

**Score**: 3/4 (Minor revision needed)

## Major Issues (P1 - Blocking)

1. **Legal standard vs empirical finding distinction**: The paper oscillates between presenting the 42% threshold as an empirical finding and as a proposed legal standard. These require different rhetorical strategies and evidentiary burdens. As an empirical finding, the paper succeeds—it demonstrates clear patterns in 5-state sample. As a legal standard, it needs more work: how would courts apply it? Is it dispositive or one factor among many? What role does it play in the Gingles three-prong test? The paper should explicitly distinguish: "We find a 42% empirical pattern" vs "Courts should adopt a 42% legal threshold," and address the gap between these claims.

## Important Issues (P2 - Should Address)

1. **Confidence intervals and statistical significance**: While r=0.88 is impressive, the paper doesn't provide confidence intervals or hypothesis tests. With N=5, individual state variations significantly affect correlation. What's the 95% CI for the 42% threshold? If Alabama were excluded (outlier with high Moran's I enabling success at 36.9%), how would the threshold change? Bootstrapping or jackknife resampling could strengthen statistical claims.

2. **Success rate interpretation ambiguity**: The paper reports "success rates" (% of 28 configs achieving target) but treats binary achievement (achieves target: yes/no) as primary outcome. This creates confusion: is Louisiana (42.9% success) closer to success (achieved target in best config) or failure (failed 57.1% of configs)? Courts need clarity on whether "42.9% success" means "likely feasible with optimization" or "more likely to fail than succeed." Recommend separating: (1) binary achievement rate (4/5 states achieved target), (2) robustness (success across multiple configs).

3. **Algorithm-specific vs algorithm-independent claims**: The paper shows edge-weighted optimization outperforms multi-constraint (47.9% vs 35% overall success), implying the threshold is algorithm-dependent. Yet the paper presents "42%" as if it's fundamental geographic truth. This tension needs resolution: is 42% the threshold for edge-weighted optimization specifically, or for any algorithmic approach? If the former, how should courts interpret this limitation? If the latter, what evidence supports algorithm-independence?

4. **Proportionality assumption justification**: Like Pildes, I find the proportional representation assumption (state with X% minority should create X% MM districts) legally problematic. However, from a quantitative perspective, it's a reasonable simplifying assumption for detecting feasibility patterns. The paper should: (1) explicitly defend this as modeling choice (not legal requirement), (2) test sensitivity to alternative targets (e.g., targeting 1-2 fewer MM districts), (3) acknowledge this might underestimate feasibility for states where VRA requires fewer MM districts than proportionality suggests.

5. **Borderline cases need probabilistic framing**: The practical guidelines table (Table 7) categorizes states into discrete bins ("Almost Always," "Very Likely," "Borderline," etc.) based on deterministic thresholds. A probabilistic approach would better serve courts: instead of "42-45% = Very Likely," provide "43% minority state has 85% probability of achieving target (95% CI: 65-95%)." This requires extending the sample or using Bayesian methods to generate posterior distributions, but would provide more actionable guidance for legal decision-making.

## Minor Issues (P3 - Nice to Have)

1. **Comparison to existing quantitative standards**: The paper doesn't situate its threshold within the landscape of other quantitative redistricting metrics (efficiency gap, mean-median difference, partisan symmetry). How does judicial treatment of those metrics inform likely reception of a 42% threshold? Are there lessons from the efficiency gap's limited adoption?

2. **Sensitivity analysis for MM definition**: The paper assumes 50% minority defines MM districts but notes courts sometimes accept 45-50%. Showing how the state-level threshold changes with MM definition (e.g., 45% MM threshold → 40% state threshold) would strengthen generalizability.

3. **Multi-year stability**: The paper uses 2020 data exclusively. Adding one historical year (2010) to test threshold stability would significantly strengthen claims about the threshold's robustness. Even if the threshold shifts slightly (e.g., 40% in 2010, 42% in 2020), demonstrating consistency across census years bolsters legal applicability.

4. **Visualization of uncertainty**: The figures beautifully display point estimates but don't show uncertainty. Adding error bars (from METIS randomness) or shaded confidence regions would better communicate the stochastic nature of algorithmic redistricting.

5. **Explicit treatment of contiguity relaxation**: The paper assumes contiguous districts but doesn't explore how relaxing contiguity (allowing island districts connected by water) might lower the threshold. Some jurisdictions allow non-contiguity; knowing whether the 42% threshold depends on contiguity requirements would help courts apply the finding appropriately.

## Strengths

- **Rigorous ablation study**: Testing 140 configurations (7 weights × 4 thresholds × 5 states) far exceeds typical redistricting studies
- **Strong statistical validation**: r=0.88 correlation is compelling evidence for state minority % as dominant predictor
- **Clear policy implications**: Table 7 practical guidelines are exceptionally useful for courts/legislatures
- **Methodological transparency**: Edge-weighted optimization is well-explained, enabling replication
- **Appropriate sample design**: Five states span 35.1%-46.1% minority, bracketing the threshold region
- **Honest limitations section**: Section 6 acknowledges algorithm-dependency, small N, and other constraints
- **Disciplinary integration**: Successfully bridges computer science, political science, and law
- **Novel contribution**: First systematic attempt to quantify Gingles "sufficiently large" requirement

## Detailed Comments by Section

### Introduction
The research question is clearly stated and well-motivated. The preview table (showing threshold pattern) is effective. Consider adding: "This paper identifies an empirical pattern that courts may find useful; it does not propose a binding legal rule."

### Background
Good coverage of Gingles doctrine. Could strengthen by discussing how courts have resisted quantitative thresholds in other contexts (e.g., efficiency gap's rocky reception) and what that suggests for a 42% threshold's prospects.

### Methodology
Excellent. The edge-weighted optimization explanation is accessible to non-technical readers. The ablation study design is appropriate. Minor suggestion: Add paragraph explaining why proportional representation targets are used (modeling choice to detect patterns, not legal mandate).

### Results
The 8 detailed tables are comprehensive and well-organized. Table 6 (correlations) is particularly valuable, showing state minority % dominates combined metrics. The figures effectively visualize threshold patterns. Consider adding: confidence intervals for correlations, sensitivity analysis excluding Alabama.

### Discussion
Section 5.1 ("The 42% Rule of Thumb") is excellent—clear, accessible, policy-relevant. Section 5.2 (geographic vs algorithmic limits) makes an important conceptual distinction. Section 5.3 (legal implications) needs more nuance about how courts would actually use this threshold (as one factor? dispositive? evidentiary presumption?).

### Limitations
Honest and thorough. Good acknowledgment of algorithm-dependency, small N, and binary MM threshold. Could add: discussion of how courts typically adopt quantitative standards (slowly, cautiously, with caveats).

### Conclusion
Strong summary of contributions. Final paragraph effectively distinguishes impossibility from failure. Consider tempering claims slightly: "provides evidence for a 42% threshold" rather than "establishes a 42% threshold."

## Recommendation

**Accept with minor revisions**. This paper makes a genuine contribution to VRA scholarship by systematically quantifying feasibility patterns. The methodology is sound, the findings are robust within the tested domain, and the policy implications are clearly articulated. The revisions needed are primarily framing issues—clarifying the relationship between empirical findings and legal standards—rather than fundamental methodological problems. With modest revisions addressing P1 and key P2 issues, this paper will be ready for publication in a top-tier election law journal.

The paper's quantitative approach represents the future of redistricting scholarship—replacing ad-hoc expert testimony with systematic empirical analysis. While courts may not adopt a rigid 42% threshold, this work provides essential evidence for assessing VRA feasibility in specific cases.
