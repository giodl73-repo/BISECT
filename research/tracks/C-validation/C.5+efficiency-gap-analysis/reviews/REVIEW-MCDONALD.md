> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Michael D. McDonald (Binghamton University, Political Science)
**Expertise**: Electoral bias measurement, partisan asymmetry, seats-votes analysis
**Date**: 2026-02-08
**Venue**: American Political Science Review

---

## Overall Assessment

This paper provides the first national-scale efficiency gap analysis of algorithmic redistricting, establishing -3.2% as an empirical baseline for neutral algorithms. The proportionality analysis (Section 4.6) is particularly strong, connecting efficiency gap to seats-votes curves and mean-median differences. However, the paper relies too heavily on efficiency gap as the sole metric when multiple partisan fairness measures exist, and the comparative methodology section needs substantial development.

**Verdict**: Accept with moderate revisions
**Score**: **3.5/4**

---

## Major Issues

### 1. Over-Reliance on Single Metric (P1)

You use efficiency gap as if it's the definitive partisan fairness metric, but scholarly consensus is that NO single metric captures all forms of bias. Different metrics detect different pathologies:

**Metrics and what they detect:**
- **Efficiency gap**: Detects asymmetric wasted votes (packing/cracking)
- **Mean-median difference**: Detects vote distribution asymmetry
- **Partisan bias (at 50%)**: Detects seat advantage at vote parity
- **Declination**: Detects whether party wins more districts by similar margins
- **Seats-votes elasticity**: Detects responsiveness to vote shifts

**Problem:** Your paper reports efficiency gap as primary finding, mentions mean-median in passing (Section 4.6.1), and includes seats-votes curves (Section 4.6.2) but doesn't systematically compare all metrics.

**Required revision:**
1. Add Table 5: "Multiple Partisan Fairness Metrics Comparison"
   - Rows: Algorithmic plans, Enacted plans, Difference
   - Columns: Efficiency Gap, Mean-Median Diff, Partisan Bias @50%, Declination, Elasticity
2. Add paragraph discussing whether all metrics agree (if yes: robust finding; if no: metric-dependent)
3. Cite literature on metric comparison (Goedert, Warshaw, etc.)

### 2. Seats-Votes Analysis Needs Full Treatment (P1)

Section 4.6.2 introduces seats-votes curves in a single page, but this is one of the most important analyses in the paper. Seats-votes curves have several advantages over efficiency gap:
- Show full range of electoral scenarios (not just observed outcomes)
- Reveal responsiveness (elasticity) vs bias (symmetry)
- More intuitive for legal audiences ("what if vote share changes?")

**Missing components:**
- Formal specification of how you estimated seats-votes curves (uniform swing? Simulation?)
- Standard errors on curve estimates
- Comparison to historical seats-votes curves for these states
- Discussion of whether algorithmic vs enacted differences are larger in bias (intercept) or responsiveness (slope)

**Required revision:** Expand Section 4.6.2 to 2-3 pages with full methodological detail and graphical presentation. This should be a co-equal finding with efficiency gap, not an afterthought.

### 3. Proportionality vs. Fairness Distinction Unclear (P2)

You conflate "partisan fairness" (treating parties symmetrically) with "proportional representation" (matching seats to votes). These are conceptually distinct:

**Partisan fairness**: If Democrats get 52% votes and 48% seats, Republicans should get 48% seats with 52% votes (symmetry)
**Proportional representation**: 52% votes should yield 52% seats (proportionality)

**Example:** A plan with perfect partisan symmetry (no bias) can still have high disproportionality (winner's bonus). Your -3.2% EG shows asymmetry (Democrats advantaged) but you don't clearly separate this from disproportionality.

**Required clarification:** Add paragraph explicitly defining and distinguishing partisan fairness (symmetry) from proportional representation (proportionality). Explain that your paper primarily measures fairness, not proportionality.

---

## Minor Issues

### 4. Temporal Analysis Could Be Deeper (P2)

Figure 3 shows EG stability across 2016-2020, but you don't connect this to the seats-votes literature. Temporal stability has two interpretations:

**Interpretation 1 (your implicit view)**: Stability shows EG measures durable geography, not transient electoral swings
**Interpretation 2 (alternative)**: Stability could reflect biased baseline + uniform swing, not true geographic determinism

**Test:** Compare actual year-to-year EG changes to uniform swing prediction. If observed changes match uniform swing, your baseline is all that matters. If observed changes differ, there's non-uniform swing that EG misses.

**Suggested addition:** Add analysis testing whether temporal EG changes follow uniform swing assumption.

### 5. Competitive Districts Entirely Missing (P3)

You focus on partisan bias but say nothing about competitiveness. A map with zero bias but zero competitive districts is problematic for democratic accountability. Your algorithmic plans might reduce bias while eliminating swing districts.

**Suggested analysis:**
- Count competitive districts (won by <55%) in algorithmic vs enacted plans
- Report tradeoff: do enacted plans have more competitive districts? (They might, through intentional district balancing)
- Discuss normative question: is competition a separate redistricting value?

### 6. Comparing to Other Bias Metrics Missing (P3)

The classic partisan bias measure is "bias at 50%": extrapolate seats-votes curve to 50% vote share and measure seat share deviation from 50%. You mention this briefly but don't report it systematically.

**Suggested addition:** Report "partisan bias at 50%" alongside efficiency gap for all 15 states. Discuss whether these metrics correlate (they should) or diverge (which would be interesting).

---

## Positive Aspects

1. **National scope**: 50 states × 3 years is comprehensive
2. **Multiple metrics**: Including mean-median and seats-votes alongside EG shows methodological sophistication
3. **Temporal consistency**: Demonstrating EG stability is important
4. **Honest about limits**: Acknowledging geographic determinism is refreshing
5. **Regional variation**: Rust Belt vs Sunbelt analysis reveals important patterns
6. **Proportionality section**: Section 4.6 is strongest part of paper—shows you understand the literature

---

## Specific Recommendations

### Section 3 (Methodology)
- Add "3.6 Partisan Fairness Metrics" defining EG, mean-median, partisan bias, declination, elasticity
- Explain why you chose EG as primary metric (but acknowledge others exist)

### Section 4 (Results)
- Add Table 5 comparing multiple partisan fairness metrics
- Expand seats-votes subsection to 2-3 pages with full methodology
- Add competitive districts analysis (count districts won by <55%)

### Section 5 (Discussion)
- Add paragraph distinguishing partisan fairness (symmetry) from proportional representation (proportionality)
- Discuss whether metrics agree: is -3.2% EG consistent with mean-median and partisan bias findings?
- Connect temporal stability to uniform swing assumption

### Section 6 (Conclusion)
- Acknowledge that partisan fairness is only one redistricting value (others: competitiveness, communities of interest, minority representation)
- Clarify that algorithmic plans improve fairness but may not maximize ALL redistricting values simultaneously

---

## Questions for Authors

1. Why did you choose efficiency gap as primary metric rather than partisan bias at 50%?
2. Do all partisan fairness metrics (EG, mean-median, bias, declination) agree on which states are most biased?
3. Have you computed competitive district counts for algorithmic vs enacted plans?
4. How sensitive are seats-votes curves to uniform swing assumption? Did you test alternative swing models?

---

## Methodological Comparison to My Work

In my research on partisan asymmetry with Brunell and Best, we emphasize:

1. **Multiple metrics**: Report EG, partisan bias, mean-median simultaneously
2. **Seats-votes curves**: Primary tool, with EG as supplementary
3. **Symmetry testing**: Formal tests of whether bias(D)=bias(R)
4. **Temporal dynamics**: Tracking bias evolution across redistricting cycles

**Strengths of your approach:**
- Larger scale (you have 50 states; we typically analyze 5-10)
- Algorithmic baseline (clever counterfactual)
- Regional variation (we typically don't disaggregate)

**Suggestions from my methods:**
- Add formal symmetry test: is algorithmic bias(D) = -enacted bias(R)?
- Report seats-votes curves graphically (not just elasticity numbers)
- Show confidence bands on all estimates

---

## Verdict Justification

This paper makes a strong empirical contribution by establishing -3.2% efficiency gap as the baseline for algorithmic redistricting. The national scope is impressive, and the proportionality analysis (especially seats-votes curves) adds important depth beyond efficiency gap alone.

However, the paper needs:
1. Systematic comparison across multiple partisan fairness metrics (not just EG)
2. Fuller treatment of seats-votes analysis (this deserves co-equal status with EG)
3. Clearer distinction between partisan fairness (symmetry) and proportional representation

With these revisions, this will be a landmark paper in the redistricting metrics literature. The empirics are strong, and the algorithmic baseline approach is innovative. The main limitation is over-reliance on single metric when multiple measures exist.

**Recommendation**: Accept with moderate revisions, primarily expanding multi-metric comparison and seats-votes analysis. The core contribution is solid and publication-ready.
