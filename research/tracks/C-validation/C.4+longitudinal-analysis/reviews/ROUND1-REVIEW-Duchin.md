# Round 1 Review: Twenty Years of Congressional Redistricting

**Reviewer**: Moon Duchin (Tufts University / MGGG Redistricting Lab)
**Date**: 2026-02-08
**Round**: 1
**Previous Score**: 3.5/4.0
**Current Score**: 3.5/4.0

---

## Summary Assessment

The authors have addressed my primary statistical concerns with impressive thoroughness. The addition of effect sizes, confidence intervals, and multiple robustness checks elevates the empirical rigor to publication standard. The VRA discussion appropriately contextualizes the algorithmic approach as a pre-legal-constraint baseline. I maintain my score of 3.5, as the paper was already strong and these improvements solidify its contributions without fundamentally changing the scope.

## Assessment of P1 Revisions

### P1.2: Statistical Rigor (Addressed ✓)

**Original concern**: Commission effectiveness needed effect size quantification and robustness testing.

**Author response**: Added Cohen's d = 1.64, 95% CI [2.1pp, 12.5pp], plus comprehensive robustness subsection.

**Assessment**: **Exemplary**. The robustness checks (Section 5.4.1) demonstrate methodological sophistication:

- **Alternative metrics**: Reock compactness (+6.8pp) confirms results aren't PP-specific. Good choice of alternative—Reock is less perimeter-sensitive.
- **Outlier analysis**: Excluding CA still yields +2.7pp (p=0.041), showing effect isn't driven by one large state. Excluding NY (advisory commission) strengthens to +4.1pp—this hints at commission type mattering.
- **Confound controls**: Multivariate regression controlling for state size, baseline compactness, and swing state status yields β=6.9pp (p=0.009). This addresses selection bias concerns.
- **Pre-trend analysis**: 2000-2010 change for commission states was -0.8%, ruling out pre-existing positive trajectory.

This is textbook robustness testing. The interpretation ("strong correlational evidence... observational data cannot establish causation") is appropriately cautious.

### P1.3: VRA Discussion (Addressed ✓)

**Original concern**: Algorithmic approach ignored Voting Rights Act compliance requirements.

**Author response**: Added "Voting Rights Act Considerations" subsection (3.2.1) explaining Section 2 requirements and framing as "pre-VRA baseline."

**Assessment**: **Well-handled**. The new subsection (~200 words) covers:
- Gingles preconditions (geographic compactness + racially polarized voting → majority-minority districts)
- Retrogression (Beer v. US: can't diminish minority voting strength)
- Proportionality considerations (Johnson 1997 citation)
- Clear framing as "pre-VRA baseline... starting point demonstrating what purely geometric optimization produces before legal mandates"

This contextualizes the algorithmic districts correctly. In practice, VRA post-processing would reduce compactness in favor of representation—the paper now acknowledges this trade-off explicitly.

### P1.5: Causality Language (Addressed ✓)

**Original concern**: Some causal language ("improved outcomes") overstated observational findings.

**Author response**: Systematic replacement of causal language with correlational framing + selection bias caveats.

**Assessment**: **Thoroughly revised**. Spot checks confirm appropriate language throughout. The Causality limitation (7.4.3) now explicitly notes "commission adoption is non-random (all commission states are blue or purple states)" and lists potential confounds (political shifts, civic culture). This is honest and appropriate.

## Remaining Observations

### Strengths to Highlight

1. **Longitudinal consistency**: Using identical METIS parameters across 20 years remains the paper's core methodological strength. This isolation of temporal effects is rare in redistricting literature.

2. **Geographic stability analysis**: 61% districts with IoU > 0.7 despite population shifts—this demonstrates algorithmic adaptability, a key practical concern.

3. **Commission type hint**: The NY exclusion strengthening effect (+4.1pp when excluding advisory commission) suggests independent commissions may outperform advisory ones. This deserves a sentence in Discussion.

### Minor Methodological Notes

1. **METIS version stability**: The paper uses "METIS 5.1.0" consistently but doesn't note whether this version was available in 2000. If you're retroactively applying METIS 5.1.0 to 2000 data, clarify this is retrospective analysis, not contemporaneous.

2. **Ensemble comparison**: My original review suggested comparing to ensemble methods (DeFord et al. 2019). The authors chose not to pursue this—acceptable for scope, but acknowledge as future work.

3. **Population balance**: ±0.5% tolerance is standard but strict. A sentence justifying this (vs ±1% or equipopulous) would strengthen methodology.

## Verdict

**Accept**. The paper makes novel longitudinal contributions with sound methodology and appropriate claims. The P1 revisions address all substantive concerns. The remaining notes are suggestions for strengthening, not blocking issues.

**Publication readiness**: This is ready for **Political Analysis** or **American Political Science Review**. For Science, you'd likely need to add the partisan fairness analysis that McGhee suggested in Round 0—but that's a different paper.

Well done on a rigorous revision.
