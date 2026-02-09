# Review Summary - Round 1

**Paper**: Recursive Bisection for Congressional Redistricting
**Date**: 2026-02-07
**Round**: 1
**Reviewers**: 7

---

## Scores

| Reviewer | Affiliation | Expertise | Score |
|----------|-------------|-----------|-------|
| Jonathan Rodden | Stanford | Political Geography | **3.0/4.0** |
| Jowei Chen | Michigan | Automated Redistricting | **3.0/4.0** |
| Moon Duchin | Rutgers | Metric Geometry | **2.5/4.0** |
| George Karypis | Minnesota | METIS/Graph Partitioning | **3.0/4.0** |
| Ümit Çatalyürek | Georgia Tech | Parallel Algorithms | **3.0/4.0** |
| Richard Pildes | NYU Law | Constitutional Law | **2.5/4.0** |
| Michael Goodchild | UCSB | GIS Theory | **3.0/4.0** |

**Average Score**: **2.86/4.0** (Strong Accept with Revisions)

**Consensus**: All reviewers find significant merit in the core contribution (extending Huntington-Hill precedent to redistricting with "impossibility defense"). The paper is publishable with major revisions addressing specific technical, legal, and empirical gaps.

---

## Overall Assessment

### Strengths (Unanimous)

1. **Novel "Impossibility Defense"**: All reviewers praise this framing as creative and legally sophisticated, offering a way around *Rucho*'s justiciability problems
2. **Huntington-Hill Precedent**: The historical analogy is compelling and provides legitimacy
3. **Intellectual Honesty**: Acknowledging limitations (geographic sorting, compactness gaps, VRA tensions) strengthens credibility
4. **Technical Soundness**: Algorithm implementation appears correct and produces reasonable results
5. **Computational Efficiency**: 2.5 hours for 50 states enables iteration and scenario analysis

### Common Concerns (All Reviewers)

1. **Parameter Sensitivity**: Multiple reviewers (Chen, Duchin, Karypis) emphasize need for empirical demonstration that results are robust across reasonable parameter variations
2. **VRA Compliance**: Multiple reviewers (Rodden, Duchin, Pildes) note inadequate treatment of Voting Rights Act tensions
3. **Compactness Gap**: All technical reviewers want better analysis of why enacted districts are more compact (0.305 vs. 0.220 PP)
4. **Missing Empirical Comparisons**: Chen and Karypis want comparison to ensemble methods and alternative graph partitioners

---

## Major Issues by Category

### Political Science Issues

**M1. Geographic Sorting Analysis (Rodden)**
- Need state-by-state efficiency gap comparison (algorithmic vs. enacted)
- Scatter plot: urban density vs. Democratic district advantage
- Case studies showing how specific geographic features drive partisan outcomes
- Sensitivity analysis: edge-weighted vs. unweighted

**M2. Communities of Interest (Rodden)**
- Currently dismissed too quickly without serious engagement
- Need empirical analysis: do algorithmic districts preserve counties, municipalities?
- Discussion of descriptive vs. substantive representation literature
- Tension: COI can be invoked strategically to justify gerrymandering

**M3. Policy Adoption Barriers (Rodden)**
- Current discussion underanalyzes political economy obstacles
- Why would Democrats support algorithms producing efficiency gaps?
- Why would Republicans cede gerrymandering advantages?
- Need case studies: Iowa succeeded (why?), others failed (why?)

### Algorithmic/Technical Issues

**M4. Parameter Sensitivity Analysis (Chen, Karypis)**
- **CRITICAL**: Empirical demonstration that partisan outcomes don't change significantly across reasonable parameter ranges
- Vary ufactor (1, 5, 10, 20), niter (10, 50, 100, 200), objtype (cut vs. vol)
- Random seed ensemble: 100 runs showing <1% variation (with actual data)
- Essential for impossibility defense—without this, critics say "parameter tuning = manipulation"

**M5. Comparison to Ensemble Methods (Chen)**
- Current dismissal of MCMC as "diagnostic not prescriptive" is inadequate
- Need empirical comparison: Is your Alabama map within the distribution of 10,000 simulated maps?
- Propose hybrid: Your algorithm as "default" plan + sensitivity analysis provides uncertainty bounds

**M6. Alternative Graph Partitioners (Karypis)**
- Compare to KaHIP, Scotch (may achieve 5-15% better edge cuts)
- If gap narrows, helps explain compactness difference with enacted plans
- Standard in graph partitioning research

**M7. Underutilized METIS Features (Karypis)**
- Edge-weighted graphs (edge weight = boundary length) → direct perimeter minimization
- Multi-constraint partitioning for VRA compliance (demonstrate it, don't just claim it)
- Direct k-way vs. recursive bisection empirical comparison

### Mathematical/Geometric Issues

**M8. Edge-Cut Minimization Analysis (Duchin)**
- Rigorous analysis: Under what conditions does minimizing edge cuts maximize compactness?
- Tract geometry effects: Do urban vs. rural tract shapes introduce hidden partisan bias?
- Multi-metric comparison: PP, Reock, convex hull, moment of inertia

**M9. Fairness Criteria Axiomatization (Duchin)**
- Formal statement of which fairness criteria your method satisfies/violates
- Impossibility theorem: NO method satisfies all criteria (like Balinski-Young for apportionment)
- Trade-off analysis with mathematical rigor

**M10. Scalability to Block-Level (Çatalyürek)**
- Currently claimed "feasible" without evidence
- Need empirical implementation for 3 small states (VT, DE, WY) with 10K blocks
- Runtime projections to large states based on empirical scaling
- Discuss whether ParMETIS or MT-METIS needed for large states

### Legal/Constitutional Issues

**M11. VRA Comprehensive Analysis (Pildes, Rodden, Duchin)**
- **CRITICAL**: State-by-state Section 2 compliance assessment
- For covered states (AL, MS, LA, GA, SC, NC, TX, FL):
  - How many majority-minority districts does algorithm produce?
  - How many does Section 2 require? (Gingles analysis)
  - What compactness sacrifice needed?
- Actually implement VRA-constrained optimization for 2-3 states (demonstrate feasibility)
- Address philosophical tension: VRA requires seeing race, impossibility defense requires not seeing sensitive data

**M12. *Rucho* Deep Engagement (Pildes)**
- Current treatment is superficial citations
- Need analysis: Do algorithms provide "manageable standards" or just shift value judgments?
- State constitutional variability: Can algorithm satisfy diverse state requirements?
- Political question doctrine: Is process fairness judicially manageable?

**M13. State Constitutional Requirements (Pildes)**
- Federal analysis only; ignores state constitutions
- States require: county preservation (IA, WA), competitiveness (AZ, CO), communities of interest (CA, MI)
- Need flexibility analysis: Can algorithm be configured for state-specific requirements?
- Case studies: Iowa (counties), Arizona (competitiveness), California (COI)

### Geographic/GIS Issues

**M14. Geographic Feature Preservation (Goodchild)**
- No analysis of how boundaries relate to rivers, mountains, highways, cities
- Water boundary alignment: Do districts follow river courses?
- Topographic analysis: Do boundaries respect mountain ranges?
- Urban coherence: Do cities stay intact or split?

**M15. Projection and CRS (Goodchild)**
- One sentence on projections; needs comprehensive discussion
- Which projection per state? Distortion impacts on compactness?
- Multi-state comparison issues if different projections used
- Alaska special handling (extreme projection distortion)

---

## Minor Issues (Consolidated)

### All Reviewers Noted:
- **VRA discussion too brief** (needs 3-4 pages, not 3 paragraphs)
- **Competitive districts incomplete** (normative debate on whether we SHOULD want them)
- **Edge-weighted optimization teased** (show actual results for 1-2 states, don't just mention 50-60% improvement)
- **Random seed variation** (provide actual data, not just "<1% claim")
- **Word count** (currently ~17,600 words; APSR prefers 12,000-15,000)

### Technical Details:
- Hierarchical structure advantage underexplored (Chen)
- Huntington-Hill analogy overextended (discrete vs. continuous) (Duchin)
- Block-level feasibility overstated without evidence (Duchin, Çatalyürek)
- Hypergraph formulation unexplored (Çatalyürek)
- Graph construction details missing (Karypis)
- Contiguity verification missing (Karypis)
- Alternative adjacency models (Çatalyürek, Goodchild)

### Legal Details:
- Equal population standard unclear (Pildes)
- Compactness as constitutional requirement needs clarification (Pildes)
- Citation errors (*Reynolds* vs. *Wesberry*) (Pildes)
- Census data quality issues (Goodchild)

---

## Recommendations

### Immediate Priorities (P1 - Blocking Issues)

1. **Parameter Sensitivity Analysis** (M4): Add entire Section 4.5 with systematic parameter sweeps and random seed ensemble showing <1% variation
2. **VRA Comprehensive Analysis** (M11): Expand Section 5.6 to 3-4 pages with state-by-state Gingles analysis and VRA-constrained optimization demonstrations
3. **Comparison to Ensemble Methods** (M5): Expand Section 6.2 showing your plans fit within simulated ensemble distributions

### High Priority (P2 - Important Issues)

4. **Geographic Sorting Empirical Analysis** (M1): Add Section 5.4 with efficiency gap tables, urban density correlations, case studies
5. **Edge-Weighted Optimization** (M7): Implement for 3-5 states NOW (not future work), show actual 50-60% improvement
6. **Compactness Gap Better Analysis** (M3): Rewrite Section 4.3 with commission vs. gerrymandered states breakdown, trade-off analysis
7. **Communities of Interest** (M2): Expand discussion with empirical analysis of county/municipality preservation
8. ***Rucho* Deep Engagement** (M12): Add subsection analyzing whether algorithms address Court's concerns

### Medium Priority (P3 - Nice to Have)

9. **Alternative Graph Partitioners** (M6): Compare METIS to KaHIP, Scotch empirically
10. **Fairness Criteria Axiomatization** (M9): Formal mathematical framework for which criteria satisfied/violated
11. **State Constitutional Requirements** (M13): Survey 50 states, show algorithm flexibility for state-specific needs
12. **Geographic Feature Preservation** (M14): Analyze river alignment, mountain ranges, urban boundaries
13. **Policy Adoption Barriers** (M3): Political economy analysis of coalition politics

---

## Overall Verdict

**Status**: **Strong Accept with Major Revisions**

**Consensus**: The paper makes a valuable contribution (impossibility defense + Huntington-Hill precedent) that merits publication in top venues. However, significant empirical, legal, and technical gaps must be addressed.

**Estimated Revision Scope**:
- P1 priorities: +3,000-4,000 words
- P2 priorities: +2,500-3,000 words
- P3 priorities: +2,000-2,500 words
- **Total addition**: ~8,000-10,000 words

**Cutting needed**: Currently ~17,600 words. With additions, would reach ~26,000 words. Must cut ~8,000-10,000 words from methodology/results detail to stay within 15,000-word range for APSR.

**Timeline Estimate**:
- P1 revisions: 2-3 weeks (requires significant new empirical work)
- P2 revisions: 1-2 weeks (mostly analysis and writing)
- P3 revisions: 1 week (nice-to-haves)
- **Total**: 4-6 weeks for comprehensive revision

**Next Step**: Create synthesis document consolidating all reviews into prioritized action items (P1/P2/P3) with specific implementation guidance.
