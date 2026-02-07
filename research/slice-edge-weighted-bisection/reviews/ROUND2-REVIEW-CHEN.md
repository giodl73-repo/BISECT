# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Jowei Chen (University of Michigan)
**Expertise**: Automated redistricting, compactness, neutrality
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made exemplary improvements addressing all major empirical concerns from Round 1. The addition of partisan outcome analysis (P1.1), VRA compliance evaluation (P1.2), geographic sorting quantification (P2.5), and county preservation analysis (P2.4) transforms this from a computational paper to a comprehensive redistricting study that meets empirical social science standards.

As someone who has conducted similar automated redistricting research, I particularly appreciate three aspects of the revisions:

1. **Honest partisan analysis**: The paper shows that compactness produces mixed results—not uniformly "better" outcomes. This intellectual honesty prevents overselling and builds credibility.

2. **VRA confrontation**: Many automated redistricting papers ignore VRA compliance. This paper directly addresses the 68% reduction in majority-minority districts and proposes concrete solutions.

3. **Geographic sorting quantification**: The methodology for separating geographic baseline from gerrymandering premium is novel and provides objective metrics for evaluating redistricting plans.

The paper now demonstrates that edge-weighted bisection is not just computationally efficient (2-3 hours for 50 states) but also produces substantively interesting results that advance our understanding of redistricting tradeoffs.

## Updated Score

**Score**: 4/4 — **Accept**

*Upgrade from 3/4 in Round 1*

## P1 Items Addressed (My Areas)

### P1.1: Partisan Outcome Analysis ✓ COMPREHENSIVE

Section 3.2 (Partisan Outcome Analysis) provides exactly what I requested:

**Metrics computed**:
- Efficiency gap (wasted votes asymmetry)
- Mean-median difference (median district lean vs statewide vote)
- Partisan bias (expected seat share at 50% vote share)

**Results**:
- **Efficiency gap**: 36% improved, 52% worsened, 12% neutral
- **Mean-median difference**: 54% improved, 38% worsened, 8% neutral
- **Partisan bias**: 14% improved, 72% worsened, 14% neutral

**Key finding**: Mixed results demonstrate that compactness optimization doesn't uniformly improve partisan fairness. States with strong geographic sorting (Democrats in cities, Republicans in suburbs/rural) show pro-Republican bias in compact plans.

This is exactly the analysis I requested. The paper now empirically validates that:
1. Compactness ≠ partisan fairness
2. Geographic sorting dominates outcomes in most states
3. Algorithmic objectivity (no partisan data) ≠ partisan neutrality (equal outcomes)

**Minor suggestion**: Seats-votes curves would strengthen this further, but the three metrics provided are sufficient and standard in redistricting literature.

### P1.2: VRA Compliance Evaluation ✓ THOROUGH

Section 3.3 (Voting Rights Act Compliance) provides comprehensive VRA analysis:

**Findings**:
- 68% reduction in majority-minority districts (65 enacted → 21 algorithmic)
- 9 states non-compliant: AL, AZ, CA, GA, LA, MD, NC, SC, TX
- State-by-state comparison table shows where minority representation is lost

**Key insight**: "Geographic optimization conflicts with minority representation when populations are concentrated."

**Proposed solutions**:
1. Hybrid objectives (compactness + demographic representation)
2. Protected communities (pre-designate majority-minority districts)
3. Post-hoc adjustment (manual tweaking to meet VRA)

This is excellent analysis. The paper doesn't minimize the problem or pretend it doesn't exist. Instead, it directly confronts the tension and proposes reasonable (if not experimentally validated) solutions.

**From my experience**: This mirrors findings from my own automated redistricting work—purely geometric objectives often conflict with VRA requirements. The protected communities approach is most practical and has been used successfully by some state commissions.

### P1.6: Soften Neutrality Claims ✓ EXCELLENT

Language updated throughout:
- "Partisan neutrality" → "Political blindness"
- "Neutral baseline" → "Geometric baseline"
- Three-part taxonomy: (1) algorithmic neutrality, (2) political blindness, (3) partisan neutrality
- Removed "implicitly promotes fairness" claims

This is exactly what I requested. The new language accurately describes what the algorithm does (uses no political data, cannot intentionally favor parties) without overclaiming about outcomes (equal partisan results).

This is critical for redistricting reform—overselling algorithmic methods as "neutral" or "fair" when they have partisan effects can undermine reform efforts.

### P2.4: County Preservation Analysis ✓ STRONG

Section 3.5 (County Preservation Analysis) addresses my Round 1 concern about county splits:

**Findings**:
- Modest increase in splits: 28.4% algorithmic vs 27.4% enacted (+1.0 percentage points)
- Strong negative correlation (-0.68) between compactness gains and county splits
- Only 4 states show significant tradeoff (>3 splits for >5% compactness)
- National statistics: 2,636 counties analyzed

**Key insight**: Compactness and county preservation are largely compatible. The tradeoff exists but is modest—only a few states face substantial tension between objectives.

**Indiana case study**: Indiana Commission achieved both high compactness (0.478) and low county splits (32%), demonstrating that human expertise can sometimes optimize multiple objectives simultaneously.

This addresses my concern about whether compactness optimization destroys other redistricting criteria. The answer: it doesn't, at least for county preservation.

### P2.5: Geographic Sorting Quantification ✓ NOVEL

Section 3.6 (Geographic Sorting Quantification) provides sophisticated analysis I didn't explicitly request but is highly valuable:

**Methodology**:
- Compare partisan bias in algorithmic plans (geographic baseline) vs enacted plans
- Calculate "geographic fraction": percentage of bias attributable to unavoidable geography
- Calculate "gerrymandering premium": additional bias from intentional manipulation

**Findings**:
- 60% of states are geography-dominated (>60% of bias is geographic)
- 26% are gerrymandering-dominated (<30% geographic)
- Average: 63% geographic, 37% gerrymandering

**Key insight**: Two-thirds of partisan bias would persist even with perfectly compact districts. This quantifies the "geography is destiny" problem.

**Policy relevance**: Provides objective metrics for courts and legislatures evaluating gerrymandering claims. If bias is mostly geographic (Ohio: 89%), claims are weak. If mostly non-geographic (Maryland: 31%), claims are strong.

This is novel and valuable analysis that advances redistricting research. I'm not aware of other papers that quantify this separation so systematically.

## Observations on Other P1 Items

While not my primary expertise:

- **P1.3 (Partitioning Quality)**: Topological vs geometric tradeoff is well-explained
- **P1.4 (Alternative Partitioners)**: Validates generalization to KaHIP, Scotch
- **P1.5 (Recursive Bisection)**: 100% contiguity justifies choice

These strengthen algorithmic credibility.

## Remaining P2 Items

Six P2 items remain incomplete:

### P2.3: MCMC Ensemble Comparison (My Area—Important but Not Blocking)

The paper doesn't compare to MCMC ensemble distributions. This is the gold standard for gerrymandering detection—comparing enacted plans to distributions of neutral plans.

**Why I requested this**: MCMC ensembles show the range of "reasonable" compactness values. Without this, we can't assess whether algorithmic plans or enacted plans are outliers.

**Why not blocking**: MCMC sampling is computationally expensive (hours per state) and would require substantial additional work. The partisan and geographic sorting analysis provides sufficient context about whether plans fall within reasonable ranges.

**For future work**: Compare algorithmic plans to MCMC ensemble medians for 3-5 representative states. This would position the work within the ensemble redistricting literature.

### P2.2: Multi-Objective Formulation (Important but Not Blocking)

The paper acknowledges multiple objectives (compactness, VRA, counties) but doesn't implement multi-objective optimization. The VRA section *proposes* hybrid objectives but doesn't *demonstrate* them.

**Why I care**: Real redistricting requires balancing competing criteria. Single-objective optimization is useful as baseline but insufficient as practical system.

**Why not blocking**: The paper honestly acknowledges this as future work. Implementing full multi-objective optimization with Pareto frontiers would be a separate paper.

**Recommendation**: Future work should implement weighted objectives with tunable parameters.

### P2.6: Indiana Case Study (My Concern—Interesting but Not Critical)

Indiana's commission plan (0.478 vs algorithmic 0.353, +35%) remains unexplored. What did they do to achieve exceptional compactness? The paper mentions this as interesting but doesn't investigate.

**Why interesting**: Indiana demonstrates that human expertise can sometimes exceed algorithmic performance. Reverse-engineering their approach could improve the algorithm.

**Why not blocking**: This would require substantial additional work (interviewing commissioners, analyzing their GIS process, understanding their constraints). Acknowledging the outlier is sufficient.

### Others (Outside My Expertise)

- P2.1: Approximation analysis (optimization concern)
- P2.7: Census tract limitations (GIS concern)
- P2.8: Hypergraph formulation (partitioning concern)

## Minor Issues from Round 1

Most Round 1 minor issues are now addressed or acceptable:

- **m1 (Indiana)**: ✓ Acknowledged, not deeply investigated—acceptable
- **m2 (County preservation)**: ✓ Fully addressed in P2.4
- **m3 (Competitive districts)**: Not addressed—acceptable, outside scope
- **m4 (Temporal stability)**: Not tested on 2010 data—acceptable, 2020 validation is sufficient
- **m5 (Communities of interest)**: Not addressed—acceptable, difficult to quantify
- **m6 (Comparison baseline)**: Clarified in methodology

None are blocking.

## Comparison to My Own Work

As someone who published automated redistricting research (Chen & Rodden 2013), I can assess this relative to existing literature:

**Strengths relative to existing work**:
1. **Computational efficiency**: 2-3 hours for 50 states vs my work (days per state with simulated annealing)
2. **National scope**: Full 50 states vs most papers (3-5 states)
3. **Honest partisan analysis**: Shows mixed results vs papers that oversell neutrality
4. **VRA engagement**: Many papers ignore VRA entirely
5. **Geographic sorting quantification**: Novel metric not present in existing literature

**How this advances the field**:
1. Demonstrates that graph partitioning with domain-specific weights can match/exceed simulated annealing quality at fraction of computational cost
2. Provides empirical evidence that compactness ≠ fairness (counters naive reform proposals)
3. Quantifies geographic vs gerrymandering effects systematically
4. Shows VRA-compactness tension explicitly

This is a strong contribution to automated redistricting literature.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Comprehensive empirical evaluation**: Partisan metrics, VRA compliance, geographic sorting, county preservation—covers all major redistricting criteria

2. **Intellectual honesty**: Shows mixed partisan results, acknowledges VRA reduction, doesn't oversell

3. **Novel metrics**: Geographic sorting quantification is original contribution

4. **Policy relevance**: Provides objective metrics for courts/legislatures evaluating redistricting

5. **Computational efficiency**: Enables rapid scenario exploration (2-3 hours) vs MCMC (hours per state)

6. **Proper positioning**: Engages with existing redistricting literature (Rodden, partisan metrics, VRA)

## Final Recommendations

**For publication acceptance**: None. The paper is ready.

**For future research** (not blocking):
1. MCMC ensemble comparison for 3-5 states
2. Multi-objective optimization with Pareto frontiers
3. Protected communities validation for VRA compliance
4. Temporal stability testing on 2010 Census
5. Competitive district analysis
6. Indiana commission reverse-engineering

---

## Verdict

**Accept** — Ready for publication

**Changes from Round 1**: Upgraded from "Accept with Major Revisions" (3/4) to "Accept" (4/4)

**Rationale**: All required empirical analysis is now complete:
1. **Partisan analysis** (P1.1): Shows compactness ≠ fairness with standard metrics
2. **VRA compliance** (P1.2): Confronts 68% reduction, proposes solutions
3. **Geographic sorting** (P2.5): Novel quantification of geographic vs gerrymandering effects
4. **County preservation** (P2.4): Shows modest tradeoff, largely compatible objectives

The paper now meets empirical social science standards for redistricting research. It demonstrates:
- Computational innovation (edge weighting improves compactness)
- Substantive findings (compactness ≠ fairness, VRA tension, geographic sorting dominance)
- Policy relevance (objective metrics for evaluating redistricting)

**Contribution to field**:
- **Computational**: Fast, scalable algorithm (2-3 hours for 50 states)
- **Empirical**: Comprehensive evaluation of redistricting criteria
- **Theoretical**: Quantifies geographic baseline vs gerrymandering premium

This is publication-quality work suitable for top computational venues (KDD, AAAI) and would be strong submission to interdisciplinary venues (Science Advances, PNAS) or political science journals with minor framing adjustments.

**Confidence**: High — I have extensive experience with automated redistricting research and empirical evaluation. This paper now provides comprehensive analysis that meets field standards. The additions from Round 1 are substantial and address all major empirical gaps. This is excellent work.
