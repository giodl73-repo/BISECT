# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Moon Duchin (Rutgers)
**Expertise**: Gerrymandering, metric geometry, fairness
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made transformative improvements addressing all major fairness and representation concerns from Round 1. Most critically, the addition of partisan outcome analysis (P1.1), VRA compliance evaluation (P1.2), and geographic sorting quantification (P2.5) elevates this from a purely algorithmic paper to a substantive redistricting contribution that engages with political and social realities.

The partisan analysis (Section 3.2) is exactly what I requested—showing that compact districts produce mixed results (54% states improved on mean-median, only 36% on efficiency gap) demonstrates empirically that compactness ≠ fairness. This intellectual honesty is commendable and crucial for redistricting research.

The VRA compliance analysis (Section 3.3) confronts the elephant in the room: purely algorithmic compactness optimization reduces majority-minority districts by 68% (65 enacted → 21 algorithmic). This demonstrates the fundamental tension between geometric optimization and minority representation. The proposed solutions (hybrid objectives, protected communities, post-hoc adjustment) are reasonable, though not experimentally validated.

The language changes (P1.6) are excellent—replacing "partisan neutrality" with "political blindness" and distinguishing three types of neutrality (algorithmic, political blindness, partisan neutrality) clarifies what the method can and cannot achieve. This prevents misinterpretation that could harm redistricting reform efforts.

The geographic sorting quantification (P2.5) is sophisticated analysis showing that 60% of states are geography-dominated (>60% of bias is unavoidable from geographic sorting alone). This provides objective metrics for courts and legislatures to separate geographic baseline from gerrymandering premium.

With these additions, the paper now honestly engages with redistricting as a political process with political outcomes, not just a geometric optimization problem.

## Updated Score

**Score**: 4/4 — **Accept**

*Upgrade from 3/4 in Round 1*

## P1 Items Addressed (My Areas)

### P1.1: Partisan Outcome Analysis ✓ EXCELLENT

Section 3.2 (Partisan Outcome Analysis) provides comprehensive partisan metrics:
- **Efficiency gap**: 36% of states improved, 52% worsened, 12% neutral
- **Mean-median difference**: 54% improved, 38% worsened, 8% neutral
- **Partisan bias**: Only 14% improved, 72% worsened, 14% neutral

**Key insight**: Compactness optimization produces *mixed* results, not uniformly "fairer" outcomes. States where Democrats cluster in cities (geographic sorting) show pro-Republican bias in compact plans, confirming Rodden's "geography is destiny" hypothesis.

This is exactly what I requested and is intellectually honest. The paper no longer oversells fairness claims and acknowledges that geometric optimization cannot eliminate geographic sorting effects.

**Minor observation**: The paper could strengthen this by showing seats-votes curves for representative states, but the current metrics are sufficient.

### P1.2: VRA Compliance Evaluation ✓ COMPREHENSIVE

Section 3.3 (Voting Rights Act Compliance) provides thorough analysis:
- 68% reduction in majority-minority districts (65 enacted → 21 algorithmic)
- 9 states would be VRA non-compliant (AL, AZ, CA, GA, LA, MD, NC, SC, TX)
- Clear documentation of states where minority representation is lost

**Key insight**: Compactness and VRA compliance are fundamentally in tension when minority populations are geographically concentrated. The paper explicitly states: "Geographic optimization conflicts with minority representation when populations are concentrated."

The proposed solutions are reasonable:
1. **Hybrid objectives**: Weight compactness + demographic representation
2. **Protected communities**: Designate majority-minority districts before optimization
3. **Post-hoc adjustment**: Manual adjustment to meet VRA requirements

I would have liked to see experimental validation of at least one approach (e.g., protected communities), but acknowledging the problem and proposing solutions is sufficient for this paper.

**Critical strength**: The paper doesn't minimize or hide this problem. Many algorithmic redistricting papers ignore VRA entirely. This paper confronts it directly.

### P1.6: Soften Neutrality Claims ✓ EXCELLENT

Language updated throughout the paper:
- **Abstract**: Added "compactness ≠ fairness" caveat
- **Introduction**: "Algorithmic objectivity" and "political blindness" replace "partisan neutrality"
- **Discussion**: Three-part taxonomy distinguishing (1) algorithmic neutrality, (2) political blindness, (3) partisan neutrality
- **Conclusion**: "Geometric baseline" replaces "neutral baseline"

**Key improvement**: The paper now clearly distinguishes what the algorithm guarantees (uses no political data, cannot favor parties *intentionally*) from what it doesn't guarantee (equal partisan outcomes).

Removed problematic claims:
- "Partisan neutrality" → "Political blindness"
- "Implicitly promotes fairness" → Removed
- "Neutral baseline" → "Geometric baseline"

This is exactly what I requested. The new language prevents misinterpretation while accurately describing the method's properties.

### P2.5: Geographic Sorting Quantification ✓ SOPHISTICATED

Section 3.6 (Geographic Sorting Quantification) provides novel analysis:
- **Geographic fraction**: Percentage of partisan bias attributable to unavoidable geographic sorting
- **Gerrymandering premium**: Additional bias from intentional manipulation
- **Key finding**: 60% of states are geography-dominated (>60% of bias is geographic)
- **National average**: 63% geographic, 37% gerrymandering

**Methodology**: Compares partisan bias in algorithmic plans (geographic baseline) vs enacted plans (geographic + gerrymandering) to separate unavoidable from intentional effects.

This is sophisticated and valuable analysis. It provides objective metrics for courts evaluating gerrymandering claims: if enacted plan's bias is mostly geographic (>60%), gerrymandering claims are weak; if mostly non-geographic (<30%), claims are strong.

**Example**: Ohio shows 89% geographic fraction—most partisan bias would persist even with perfectly compact districts. This clarifies that Ohio's partisan tilt is geography, not primarily gerrymandering.

**Minor suggestion**: The paper could discuss implications for redistricting reform—if 60% of states are geography-dominated, reforms focused solely on process (independent commissions) may not achieve partisan balance without explicit fairness objectives.

## Observations on Other P1 Items

While not my primary expertise:

- **P1.3 (Partitioning Quality)**: Topological vs geometric tradeoff analysis is valuable—explains why edge cuts increase but perimeter decreases

- **P1.4 (Alternative Partitioners)**: METIS, KaHIP, Scotch within 0.3% validates generalization

- **P1.5 (Recursive Bisection)**: 100% contiguity vs k-way's 20% failure rate justifies choice

These strengthen the algorithmic contribution.

## P2 Items Completed

### P2.4: County Preservation Analysis ✓ GOOD

Section 3.5 (County Preservation Analysis):
- Modest increase in splits (28.4% vs 27.4%, +1.0pp)
- Strong negative correlation (-0.68) between splits and compactness gains
- Only 4 states show clear tradeoff (>3 splits for >5% compactness)

**Conclusion**: Compactness and county preservation are largely compatible objectives. The tradeoff is real but modest.

This is outside my primary expertise but appears thorough. It addresses the common concern that compactness optimization destroys county integrity.

## Remaining P2 Items

Six P2 items remain incomplete:
- P2.1: Approximation analysis (optimization concern)
- P2.2: Multi-objective formulation (MY AREA—see below)
- P2.3: MCMC ensemble comparison (MY AREA—see below)
- P2.6: Indiana case study (multiple reviewers)
- P2.7: Census tract limitations (GIS concern)
- P2.8: Hypergraph formulation (partitioning concern)

### P2.2: Multi-Objective Formulation (Important but Not Blocking)

The paper now acknowledges multiple objectives (compactness, VRA, county preservation) but doesn't implement multi-objective optimization. The VRA section proposes hybrid objectives but doesn't demonstrate them.

**Why this matters**: Real redistricting requires balancing competing criteria. Single-objective optimization is a toy problem.

**Why not blocking**: The paper honestly acknowledges this limitation. Proposing solutions (hybrid objectives, protected communities) without implementing them is acceptable given the scope of revisions already completed.

**Recommendation for future work**: Implement Pareto frontier analysis with weighted objectives.

### P2.3: MCMC Ensemble Comparison (Important but Not Blocking)

The paper doesn't compare to MCMC ensemble distributions (gold standard for outlier detection in gerrymandering analysis).

**Why this matters**: Without ensemble comparison, we can't assess whether algorithmic plans fall within "reasonable" range of compact redistricting or are themselves outliers.

**Why not blocking**: This is deep analysis that would require substantial additional work (MCMC sampling is computationally expensive). The partisan and VRA analysis provides sufficient context.

**Recommendation for future work**: Compare to MCMC ensembles for 3-5 representative states.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Honest partisan analysis**: Shows compactness ≠ fairness empirically, doesn't oversell results

2. **VRA confrontation**: Directly addresses 68% reduction in minority districts, proposes solutions

3. **Geographic sorting quantification**: Novel metric separating geographic baseline from gerrymandering premium

4. **Clear language**: Three-part neutrality taxonomy prevents misinterpretation

5. **Policy relevance**: Geographic sorting analysis provides objective metrics for courts/legislatures

6. **Intellectual integrity**: Paper acknowledges limitations (VRA tradeoff, partisan effects) rather than hiding them

## Minor Issues Remaining

From my Round 1 review, most issues are now addressed or acceptable:

- **m1 (Gerrymandering resistance)**: Geographic sorting analysis provides quantification
- **m2 (Geographic sorting)**: ✓ Fully addressed in P2.5
- **m3 (Indiana outlier)**: Still not deeply investigated—remains interesting question
- **m4 (Communities of interest)**: Not addressed—acceptable, difficult to quantify
- **m5 (Competitive districts)**: Not addressed—acceptable, outside paper's scope

None of these are blocking.

## Comparison to Existing Redistricting Literature

The paper now properly positions itself within redistricting research:
- Acknowledges Rodden's geographic sorting work
- Cites partisan metrics literature (efficiency gap, mean-median, etc.)
- Engages with VRA requirements
- References MCMC ensemble methods (though doesn't implement)

This is substantially improved positioning compared to Round 1.

## Final Recommendations

**For publication acceptance**: None. The paper is ready for publication.

**For future research directions** (not blocking):
1. Multi-objective optimization with Pareto frontiers
2. MCMC ensemble comparison for representative states
3. Experimental validation of protected communities approach for VRA
4. Competitive district analysis
5. Communities of interest quantification

---

## Verdict

**Accept** — Ready for publication

**Changes from Round 1**: Upgraded from "Accept with Major Revisions" (3/4) to "Accept" (4/4)

**Rationale**: The paper now provides comprehensive analysis of:
1. **Partisan outcomes**: Shows compactness ≠ fairness with three standard metrics
2. **VRA compliance**: Confronts 68% reduction in minority districts, proposes solutions
3. **Geographic sorting**: Quantifies unavoidable geographic effects vs gerrymandering
4. **Honest language**: Distinguishes political blindness from partisan neutrality

These additions transform the paper from a pure algorithmic contribution to a substantive redistricting paper that engages with political and representational realities.

**Key insight for redistricting field**: The paper demonstrates empirically that:
- Compactness optimization resists *intentional* gerrymandering but cannot eliminate *geographic* sorting effects
- 60% of states have partisan tilt dominated by geography (>60% unavoidable)
- VRA compliance and compactness are fundamentally in tension for geographically concentrated minorities

These are important findings that advance redistricting research beyond the naive "compact districts = fair districts" assumption.

**Confidence**: High — I have extensive experience with redistricting mathematics, gerrymandering detection, and electoral fairness. This paper now makes a strong contribution to redistricting research by honestly engaging with fairness, representation, and political outcomes. The algorithmic method is sound, the empirical evaluation is comprehensive, and the intellectual honesty about limitations is commendable. This is publication-quality work.
