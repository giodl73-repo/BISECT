> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Jowei Chen (University of Michigan)
**Expertise**: Automated redistricting, compactness, neutrality
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper presents a clever enhancement to automated redistricting: using boundary lengths as graph edge weights to optimize compactness directly. The empirical results are impressive—56% improvement over unweighted baseline and 20% over enacted 2020 districts nationally. As someone who has developed automated redistricting methods, I appreciate the practical engineering quality: handling water crossings, islands, and complex geography shows real-world awareness that purely theoretical papers often lack.

The paper makes two valuable contributions: (1) a clear algorithmic method that others can replicate, and (2) empirical evidence that algorithms can outperform typical human redistricting in compactness while maintaining population balance and contiguity. The comparison to enacted districts (37 of 50 states surpassed) is particularly compelling.

However, the paper has significant gaps in empirical validation. Most importantly, it lacks any analysis of *partisan outcomes*. Compactness is important but not sufficient—redistricting affects representation, and compact districts can still produce severe partisan bias (especially given geographic sorting of voters). Without partisan analysis, we can't assess whether edge-weighted bisection produces fairer outcomes than enacted plans or merely different partisan tilts.

Additionally, the paper needs more rigorous comparison to existing automated methods (MCMC ensembles, BARD, my own automated redistricting work) and deeper investigation of anomalous cases like Indiana. The algorithmic contribution is solid, but empirical social science requires more comprehensive evaluation.

## Score

**Score**: 3/4 — **Accept** (with substantial revisions for partisan analysis)

## Major Issues (Blocking)

### M1: No Partisan Outcome Analysis

The paper's most serious omission: zero analysis of partisan effects. This is essential for redistricting evaluation:

**Required metrics:**
- Efficiency gap (wasted votes asymmetry)
- Mean-median difference (median district lean vs statewide vote)
- Partisan symmetry (flipping vote shares → symmetric seat outcomes?)
- Seats-votes curves (relationship between vote share and seat share)
- Partisan bias (expected seat share at 50% vote share)

**Required analysis:**
- Compute these metrics for algorithmic plans using 2020 presidential/congressional election data
- Compare to enacted 2020 districts
- Show which states favor Democrats vs Republicans under algorithmic plans
- Demonstrate whether partisan effects are smaller, equal, or larger than enacted plans

Without this, claims about "fairness," "neutrality," or "gerrymandering resistance" are unsubstantiated. Geographic compactness and partisan fairness are distinct properties that don't necessarily correlate.

### M2: Comparison to Other Automated Methods Missing

The paper compares to unweighted baseline and enacted districts but not to other automated redistricting methods:
- **MCMC ensembles** (Duchin, Fifield): Gold standard for detecting gerrymandering outliers
- **BARD** (Mira): Multi-objective optimization
- **My work** (Chen & Rodden 2013): Automated plans with county preservation
- **Shortest splitline** (Barnett): Simple geometric algorithm

At minimum, compare edge-weighted bisection to MCMC ensemble medians on compactness and partisan metrics for representative states. This would position the work within existing automated redistricting literature.

### M3: Geographic Sorting Not Quantified

The paper acknowledges that compact districts may produce partisan bias due to geographic sorting (Democrats concentrated in cities) but doesn't quantify this. Key question: *How much of enacted districts' partisan advantage is geography vs intentional gerrymandering?*

Methodology:
- Generate algorithmic plans for all 50 states
- Compute partisan metrics (efficiency gap, seats-votes curves)
- Compare to enacted plans' partisan metrics
- Separate geographic effects (algorithmic) from gerrymandering (enacted - algorithmic)

This would validate whether compact districts truly reduce partisan manipulation or merely substitute one bias for another.

## Minor Issues

### m1: Indiana Case Not Deeply Investigated

Indiana's commission plan (0.478 PP) exceeds algorithmic result (0.353 PP) by 35%. This is the paper's biggest failure case and deserves deep investigation:
- What techniques did Indiana's commission use?
- Did they sacrifice other objectives (county splits, communities) for compactness?
- Can their approach be reverse-engineered algorithmically?
- What's their partisan lean vs algorithmic?

Interview commission members or analyze their GIS process. This could improve the algorithm substantially.

### m2: County Preservation Not Measured

Many states constitutionally require minimizing county splits. The paper doesn't measure:
- How many county splits do algorithmic plans create?
- How does this compare to enacted plans?
- What's the tradeoff between compactness and county preservation?

Include county split analysis for all states.

### m3: Competitive Districts Not Evaluated

Some reform advocates prioritize competitive districts (close elections). Do compact districts tend to be competitive or safe?
- Compute average margin of victory for algorithmic vs enacted districts
- Count safe seats (>10 point margin) vs competitive (<5 point margin)
- Analyze whether compactness optimization creates more/fewer swing districts

### m4: Temporal Stability Not Tested

The paper uses 2020 data exclusively. Do results hold across census cycles?
- Apply to 2010 Census data (at least for representative states)
- Measure whether compactness improvements persist
- Test sensitivity to census tract boundary changes

This would strengthen generalizability claims.

### m5: Communities of Interest Not Addressed

Redistricting commissions often preserve "communities of interest"—cities, neighborhoods, cultural regions. Pure geometric optimization may split these arbitrarily.
- Measure city split rates (how often do districts bisect municipalities?)
- Compare to enacted plans
- Discuss normative tension between compactness and community preservation

### m6: Comparison Baseline Unclear for Some States

For states that redistricted before 2020 Census data release (delayed by COVID-19), enacted districts may have used 2010 estimates or older boundaries. Clarify:
- Which states' enacted districts are based on 2020 Census?
- Which used estimates or earlier data?
- Does this affect comparisons?

## Strengths

1. **Strong compactness results**: 56% improvement over baseline, 37/50 states exceed enacted plans is compelling.

2. **Practical engineering**: Water crossings, bridge connections, adaptive weight scaling shows real-world problem-solving.

3. **Clear methodology**: Detailed pipeline (R-tree, Shapely, METIS configuration) enables replication.

4. **Computational efficiency**: 2-3 hours for 50 states vs days for MILP or hours-per-state for MCMC is excellent.

5. **National scope**: Full 50-state evaluation demonstrates generalizability.

6. **Honest limitations**: Acknowledges single-objective optimization, political blindness vs neutrality distinction (though needs expansion).

## Questions for Authors

1. **Partisan outcomes**: What are efficiency gap, mean-median difference, partisan symmetry for all 50 states? Do algorithmic plans favor D or R compared to enacted?

2. **MCMC comparison**: How do edge-weighted plans compare to MCMC ensemble medians on compactness and partisan metrics?

3. **Geographic bias quantification**: How much partisan advantage in enacted plans is geographic (unavoidable) vs gerrymandering (intentional)?

4. **Indiana investigation**: What did their commission do to achieve 0.478 compactness? Can it be algorithmically replicated?

5. **County preservation**: How many county splits? What's the compactness-county tradeoff?

6. **Competitive districts**: Do compact plans create more/fewer swing districts than enacted plans?

7. **Temporal stability**: Do results hold for 2010 Census data?

8. **Sensitivity to election year**: Results use 2020 elections. Would 2016, 2018, 2022 data materially change partisan metrics?

## Recommendations

- **Add partisan analysis (REQUIRED)**: Compute efficiency gap, mean-median, partisan symmetry, seats-votes curves for all 50 states using 2020 election data. Compare to enacted districts. This is non-negotiable for redistricting research.

- **Compare to MCMC**: Run MCMC ensembles for 3-5 representative states; compare algorithmic plans to ensemble distributions.

- **Quantify geographic sorting**: Separate geographic bias from intentional gerrymandering by comparing algorithmic vs enacted partisan metrics.

- **Investigate Indiana deeply**: Conduct case study of their commission process; identify techniques that achieve exceptional compactness.

- **Add county split analysis**: Measure county preservation for all states; show compactness-county tradeoff.

- **Evaluate competitiveness**: Compute margin-of-victory distributions; count safe vs swing districts.

- **Test temporal stability**: Apply to 2010 data for 5-10 states to validate robustness.

- **Discuss normative tradeoffs**: Engage with tension between compactness and other criteria (counties, communities, competitiveness, VRA).

---

**Verdict**: Accept with Major Revisions

**Confidence**: High — I have extensive experience with automated redistricting research and empirical evaluation of redistricting plans. This paper makes a solid algorithmic contribution (edge weighting improves compactness substantially), but it's incomplete as a redistricting paper without partisan analysis. The computational method is sound and the compactness results are impressive, but redistricting is fundamentally a political process with political outcomes. Any serious redistricting paper must evaluate partisan effects, not just geometric properties. For KDD/AAAI (computational venues), the current work may be acceptable with partisan analysis added. For political science venues (APSR, JOP), substantially more empirical analysis would be required: MCMC comparison, county preservation, communities of interest, temporal stability, and normative discussion of fairness criteria.
