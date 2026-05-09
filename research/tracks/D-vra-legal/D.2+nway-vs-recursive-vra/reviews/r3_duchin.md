# Review — D.2: N-Way vs. Recursive VRA
**Reviewer**: Moon Duchin (Gerrymandering, metric geometry, redistricting)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

This is useful work that fills a genuine gap: no prior study has systematically compared n-way and recursive bisection for minority opportunity district formation at this scale. The paired t-test approach is correct for this dataset, and the equivalence conclusion (p=0.634, Cohen's d=-0.018) is well-supported. The speed difference is real and operationally important. My main concerns are about the statistical power calculation (or its absence) and the interpretation of state-level "winners."

## Strengths
- The paired within-state design is statistically correct: it accounts for the fact that states differ in minority populations, geographic clustering, and baseline feasibility.
- The 20 configurations per state per method (5 weight factors × 4 thresholds) are sufficiently varied to detect differential parameter sensitivity.
- The threshold sensitivity analysis (n-way prefers lower thresholds, recursive prefers higher thresholds) is a novel and useful finding that has practical implications for parameter selection.
- The runtime comparison is credibly documented.

## Concerns
- **Statistical power**: The paper concludes equivalence based on p=0.634, but does not report the statistical power of the test to detect a meaningful difference. If the test has 30% power to detect a 5pp difference (which it might, given the high within-state variance), then failing to reject equivalence is weak evidence for equivalence. The paper needs a power analysis or minimum detectable effect size (MDES) calculation.
- **State-level winner interpretation**: The paper flags Virginia (+35% n-way) and Connecticut (+45% recursive) as "state-specific advantages" — but these are very large differences (35pp, 45pp). At the state level, method choice clearly matters. The national equivalence finding should not be presented as "practitioners can choose either method" without acknowledging that for specific states, the choice is consequential.
- **Multiple comparisons**: Table 3 reports success rates for 5 weight factors and 4 thresholds (20 cells). Testing for differences in each cell without multiple comparison correction (Bonferroni or FDR) may produce spurious significant findings in individual cells. The paper should note whether any individual cell findings survive correction.

## Required Changes (P1/P2)
- **P1**: Add a power analysis or MDES calculation. Report the minimum detectable effect size at 80% power for this dataset. If the MDES is 5pp (reasonable for 44-state paired test), then the equivalence conclusion is meaningful; if it is 15pp, the conclusion is weak.
- **P2**: Add confidence intervals to the overall success rate comparison (not just p-value and Cohen's d). "47.5% ± X% for n-way, 48.3% ± Y% for recursive" gives readers a clearer sense of the precision of the estimate.
- **P2**: Add a caveat that state-level results should take precedence over national aggregate when choosing methods for specific states.
