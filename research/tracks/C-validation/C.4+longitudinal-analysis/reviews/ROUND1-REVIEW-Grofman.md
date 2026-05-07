# Round 1 Review: Twenty Years of Congressional Redistricting

**Reviewer**: Bernard Grofman (UC Irvine)
**Date**: 2026-02-08
**Round**: 1
**Previous Score**: 3.0/4.0
**Current Score**: 3.0/4.0

---

## Summary Assessment

The authors have addressed technical concerns (statistical rigor, VRA discussion, causality language) but have not substantially strengthened the theoretical contributions. The paper remains primarily an empirical exercise in longitudinal algorithmic redistricting, which is valuable but not theoretically novel. I maintain my score of 3.0. For a top-tier political science venue, the paper would benefit from deeper engagement with representation theory.

## Assessment of P1 Revisions

### P1.2: Statistical Evidence (Addressed ✓)

**Original concern**: Commission effectiveness claim needed rigorous statistical support.

**Author response**: Added effect size (Cohen's d=1.64), confidence intervals (95% CI [2.1pp, 12.5pp]), and comprehensive robustness checks.

**Assessment**: **Fully addressed**. The robustness subsection (5.4.1) is methodologically sound and provides multiple validity checks. This strengthens the empirical contribution significantly. The correlational interpretation is appropriately cautious.

### P1.1: Scope Framing (Addressed ✓)

**Original concern**: Unclear whether paper claimed to assess full redistricting quality or just geometric properties.

**Author response**: Explicit "geometric fairness only" scope statement with three-part fairness framework in limitations.

**Assessment**: **Clearly addressed**. The scope statement and limitations section make the paper's boundaries explicit. This is honest and appropriate.

However, this framing also reveals the theoretical limitation I noted in Round 0: the paper doesn't explain *why geometric fairness matters for representation quality*. It asserts compactness is "necessary but not sufficient" but doesn't develop the theoretical link between district shape and representational outcomes.

### P1.5: Causality Language (Addressed ✓)

**Original concern**: Overstated causal claims from observational data.

**Author response**: Systematic revision to correlational language with selection bias acknowledgments.

**Assessment**: **Appropriately revised**. The new language is methodologically sound for observational research.

## Theoretical Assessment

### What the Paper Contributes (Empirically)

1. **Longitudinal consistency**: First multi-decade algorithmic redistricting study with identical methodology
2. **Temporal stability demonstration**: Algorithmic compactness (0.45-0.46 PP) stable across 20 years
3. **Enacted comparison**: 12% decline in enacted compactness quantifies geometric quality erosion
4. **Commission effectiveness**: 7.3pp correlational improvement with robust statistical evidence
5. **Geographic stability**: 61% districts maintain IoU > 0.7 despite population shifts

These are solid empirical contributions to the redistricting literature.

### What the Paper Lacks (Theoretically)

The paper claims "theoretical contributions" (Section 7.3) but these are primarily conceptual labels for empirical patterns:

**"Temporal stability in algorithmic redistricting"** (7.3.1): This observes that algorithmic compactness remains stable across demographic change. But it doesn't explain *why* this matters theoretically. What is the normative value of temporal stability? Does it improve representation? Reduce incumbency advantage? Enhance accountability? The paper doesn't develop the theoretical mechanism.

**"Gerrymandering evolution"** (7.3.2): Quantifies that enacted compactness declined 12% as technology improved. This is consistent with the REDMAP narrative but doesn't extend theory—it confirms existing accounts.

**"Reform measurement"** (7.3.3): Quantifies commission effectiveness but doesn't theorize *why* commissions might improve outcomes. Is it reduced partisan bias? Increased transparency? Public participation? The paper treats commissions as a black box intervention.

### Missing Theoretical Engagement

The paper cites representation theory (Grofman appears in bibliography placeholder) but doesn't engage substantively:

1. **Compactness-representation link**: Why does compact geometry improve representation? Is it constituency service (smaller perimeters)? Community coherence? Electoral competition? The paper asserts the link but doesn't theorize it.

2. **Algorithmic neutrality**: The paper frames algorithms as "objective" but doesn't engage with the STS literature on algorithmic politics (Gillespie, Noble, O'Neil). Algorithms embed values through parameter choices—what values does METIS compactness optimization embed?

3. **Reform pathways**: The commission findings are empirical, not theoretical. A theoretical contribution would explain the *mechanism* by which commissions improve outcomes. Is it the removal of legislative conflict of interest? Increased public input? Transparency requirements?

## Empirical Strengths

Despite theoretical limitations, the empirical contributions are strong:

1. **Methodological rigor**: Identical METIS parameters across 20 years is rare and valuable for isolating temporal effects.

2. **Statistical thoroughness**: The revised robustness checks are exemplary. Effect sizes, confidence intervals, alternative metrics, outlier sensitivity—this is high-quality empirical work.

3. **Policy relevance**: The commission findings provide actionable evidence for reform advocates, even if the mechanism remains undertheorized.

## Minor Issues

1. **Literature engagement**: The Background section cites key works (Altman, Henderson, McGhee) but engages shallowly. What specifically does this paper add to each of those research programs? A paragraph per major work would strengthen contextualization.

2. **Commission heterogeneity**: All commissions are treated equally despite varying structures (independent vs advisory, appointed vs elected). Disaggregation could reveal mechanism insights—do independent commissions outperform advisory ones?

3. **Predictive validation**: The paper predicts 2030 trends (Discussion 7.2.3) but doesn't discuss how to validate these predictions. Pre-register the predictions? Compare to actual 2030 outcomes in future work?

4. **International comparison**: The U.S.-only focus is limiting. Australia, UK, Canada, New Zealand all use commissions—comparative analysis could illuminate mechanisms.

## Verdict

**Accept**. This is high-quality empirical work that makes meaningful contributions to redistricting research. The lack of deep theoretical engagement prevents it from being a top-tier APSR paper, but it's well-suited to specialized venues.

**Recommended venues**:
- **Political Analysis**: Methodological sophistication fits well, theoretical expectations lower
- **Political Geography**: Spatial analysis is central, theoretical framing less critical
- **Election Law Journal**: Applied policy focus, empirical emphasis

**Not recommended**:
- **American Political Science Review**: Would need substantial theoretical development
- **Science**: Would need partisan analysis + broader social science framing

### Conditional on Theoretical Development

If the authors wish to aim for APSR, they should:
1. Develop a theory of *why* compactness matters for representation (not just assert it)
2. Theorize commission effectiveness mechanisms (not just measure outcomes)
3. Engage with representation theory literature (Pitkin, Mansbridge, Rehfeld)
4. Explain what this tells us about democratic representation beyond redistricting

This would transform the paper from "high-quality empirical work" to "theoretical contribution with empirical support."

**Bottom line**: Accept as is for specialized venues. Revise with theoretical development for general-interest political science venues.
