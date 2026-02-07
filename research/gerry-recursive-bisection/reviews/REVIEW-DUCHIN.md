# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Moon Duchin (Rutgers University)
**Expertise**: Gerrymandering, metric geometry, mathematical fairness
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

This paper presents an intellectually honest approach to algorithmic redistricting with sophisticated philosophical grounding in the Huntington-Hill precedent. I appreciate the paper's recognition that no redistricting method achieves perfect fairness and that process legitimacy often matters more than outcome optimality—this maturity is rare in algorithmic redistricting proposals.

The "impossibility defense"—that algorithms cannot gerrymander because they cannot see partisan data—is clever framing, though it requires more careful mathematical analysis than currently provided. The claim that recursive bisection produces "structural immunity" to manipulation needs rigorous justification: are there hidden mathematical properties of edge-cut minimization that correlate with partisan outcomes?

My main concerns are: (1) insufficient mathematical analysis of the compactness objective and its partisan implications, (2) missing engagement with the metric geometry literature on fairness criteria, and (3) inadequate treatment of the Voting Rights Act tension with compactness.

## Score: 2.5/4.0

**Major Revisions Required**

The paper has significant merit but needs substantial strengthening of its mathematical foundations before publication in a top venue. The philosophical framing is excellent, but the mathematical claims need more rigorous justification. With revisions addressing the issues below, this could be an important contribution.

## Major Issues (Must Address)

### M1. Compactness Objective Inadequately Analyzed

**Issue**: The paper uses edge-cut minimization as a proxy for compactness (Section 3.3) but doesn't rigorously analyze what this objective actually optimizes or how it relates to traditional compactness metrics.

**Mathematical concerns**:

1. **Edge cuts ≠ Compactness directly**: Minimizing edge cuts minimizes the number of census tract boundaries crossed by district boundaries. But this is not the same as minimizing perimeter or maximizing Polsby-Popper scores.

   - **Example**: A district could have few edge cuts (crossing few tract boundaries) but still be highly non-compact if those tracts are themselves elongated or irregularly shaped.

   - **Missing**: Formal analysis of relationship between edge cuts and Polsby-Popper/Reock scores. Under what conditions does minimizing edge cuts maximize compactness?

2. **Tract geometry influences outcomes**: Census tracts vary in shape. Urban tracts are small and roughly circular; rural tracts are large and often elongated along roads or geographic features.

   - **Implication**: Edge-cut minimization behaves differently in urban vs. rural areas, which correlates with partisanship.

   - **Missing**: Analysis of whether tract shape variation introduces hidden partisan bias into "neutral" geometric optimization.

3. **Polsby-Popper vs. other metrics**: You report Polsby-Popper (PP) and Reock but don't discuss:
   - Why PP? (It's perimeter-focused, but many redistricting criteria emphasize dispersion or convexity)
   - How results differ across metrics (convex hull ratios, moment of inertia, path-based metrics)
   - Whether your algorithm optimizes PP specifically or just correlates with it

**Recommendation**: Add Section 3.7 "Mathematical Properties of Edge-Cut Minimization" with:

1. **Formal proposition**: Under what graph properties does edge-cut minimization approximate perimeter minimization?
2. **Tract geometry analysis**: Measure tract compactness (Polsby-Popper of individual tracts), stratify by urban/rural, show whether tract shape differences introduce systematic bias
3. **Multi-metric comparison**: Report 3-4 different compactness metrics (PP, Reock, convex hull ratio, moment of inertia) and show whether your algorithm performs consistently across all or only specific metrics
4. **Theoretical discussion**: Cite metric geometry literature on relationship between discrete (graph) and continuous (geometric) optimization

This mathematical rigor would strengthen the impossibility defense by showing you understand exactly what geometric properties drive outcomes.

### M2. Missing Engagement with Fairness Criteria Literature

**Issue**: The paper discusses "fairness" extensively but doesn't engage with the mathematical literature on fairness criteria in apportionment and redistricting (Balinski-Young paradoxes, Arrow impossibility, social choice theory).

**Specific gaps**:

1. **Balinski-Young**: You mention their work on apportionment impossibility (Section 2.2) but don't connect to redistricting impossibility results.

   - **Key insight**: Just as no apportionment method satisfies all fairness axioms simultaneously (monotonicity, quota, population pair consistency), no redistricting method satisfies all criteria (compactness, proportional representation, competitive districts, communities of interest).

   - **Missing**: Formal statement of which fairness axioms your method satisfies and which it violates, with proofs or citations.

2. **Efficiency gap**: You mention Stephanopoulos-McGhee efficiency gap (Sections 5.2, 5.4) but don't:
   - Define it formally
   - Calculate it for your plans
   - Discuss its limitations (asymmetry, winner-take-all assumptions)
   - Engage with critiques (Bernstein-Duchin 2017 showing EG fails basic tests)

3. **Partisan symmetry**: You mention it (Section 5.5) but don't:
   - Define it formally (Grofman-King seats-votes curves)
   - Test whether your algorithm produces symmetric outcomes
   - Discuss its limitations (requires strong assumptions about vote shift uniformity)

**Recommendation**: Add Section 5.7 "Mathematical Fairness Criteria Analysis" that:

1. **Axiomatizes fairness**: List 6-8 fairness criteria (population equality, contiguity, compactness, proportional representation, partisan symmetry, competitive districts, communities of interest, VRA compliance)

2. **Formal analysis**: For each criterion:
   - Mathematical definition
   - Does your algorithm satisfy it? (Yes/No/Partially)
   - Proof or counterexample

3. **Impossibility theorem**: State clearly that NO method satisfies all criteria simultaneously (cite Balinski-Young for apportionment, your contribution is recognizing this extends to redistricting)

4. **Trade-off analysis**: When criteria conflict (compactness vs. communities of interest, proportional representation vs. VRA compliance), what trade-offs does your algorithm make and why?

This would demonstrate mathematical sophistication and show you understand the geometric optimization has unavoidable normative consequences.

### M3. VRA Tension with Compactness Insufficiently Treated

**Issue**: Section 5.6 acknowledges your algorithm produces fewer majority-minority districts (81) than enacted plans (~100-110) but treats this as a solvable engineering problem ("technically feasible modification"). This understates the fundamental mathematical tension.

**Why this is a major issue**:

1. **Compactness disperses minorities**: Black voters in many Southern states live in non-compact geographic distributions (rural areas + urban centers separated by white suburbs). Creating compact majority-Black districts is often mathematically impossible.

   - **Example**: Alabama's Black Belt—Black voters distributed in arc across central Alabama. Compact districts split this community; non-compact districts unite it.

   - **Your algorithm**: Optimizes compactness, inherently disadvantages dispersed populations.

2. **Impossibility defense weakens**: If you incorporate racial data to create majority-minority districts, you CAN see sensitive demographic information. This creates opening for partisan manipulation disguised as VRA compliance (see Shaw v. Reno concerns).

3. **Partisan asymmetry of VRA**: Black voters overwhelmingly Democratic. Creating majority-Black districts affects partisan balance. Your "neutral" algorithm that produces fewer such districts systematically helps Republicans—not through partisan intent but through geometric optimization with partisan consequences.

**Current treatment inadequate**: Three paragraphs (Section 5.6) with hand-wave that "constrained optimization" solves this. No mathematical analysis of:
- How many majority-minority districts are geometrically feasible under compactness constraint?
- What compactness must be sacrificed to create legally-required numbers?
- How does VRA constraint change partisan outcomes?

**Recommendation**: Expand Section 5.6 to full subsection (2-3 pages) that:

1. **Mathematical analysis**: For 3-5 VRA-covered states (AL, MS, LA, GA, SC):
   - Calculate maximum feasible number of majority-minority districts under strict compactness
   - Compare to legally-required numbers
   - Quantify compactness sacrifice needed to meet VRA requirements

2. **Constrained optimization formulation**:
   - Precise mathematical formulation: minimize edge cuts subject to k majority-minority districts
   - Discuss computational complexity (likely NP-hard with additional constraint)
   - Show example solution for one state

3. **Partisan implications**: Show how VRA-constrained vs. unconstrained optimization changes partisan outcomes (efficiency gap, seat-vote curves)

4. **Philosophical discussion**: Address tension between impossibility defense (can't see demographics) and VRA compliance (must see race). Your position seems to be: seeing race for VRA ≠ seeing partisanship, but race correlates with partisanship, so...?

This is a CRITICAL issue that deserves serious mathematical and philosophical treatment, not brief acknowledgment.

## Minor Issues (Should Address)

### m1. Huntington-Hill Analogy Overextended

**Strong framing but**: You draw parallel between apportionment (allocating discrete seats) and redistricting (drawing continuous boundaries). But these problems are mathematically distinct:

- **Apportionment**: Finite discrete optimization (~10^40 possible allocations for 435 seats to 50 states)
- **Redistricting**: Infinite continuous optimization (uncountably infinite possible boundary configurations)

**Issue**: This difference matters for verification. Huntington-Hill is exactly reproducible (same inputs → exactly same outputs). Your algorithm is approximately reproducible (minor variations from random seeds, METIS heuristics).

**Recommendation**: Add paragraph in Section 2.5 acknowledging this mathematical distinction and explaining why approximate reproducibility suffices for procedural legitimacy. (I think your argument works—outcomes cluster tightly—but needs explicit statement.)

### m2. Water-Based Adjacency Needs Geometric Analysis

**Section 3.2 county bridging**: Clever solution to island connectivity but raises questions:

- **Distance metric**: You use Euclidean centroid distance. But:
  - Centroids of irregularly-shaped tracts may be unintuitive
  - Euclidean distance ignores coastline geography (shortest water crossing may not be centroid-to-centroid)

- **Alternative metrics**: Closest point-to-point distance, shortest coastline crossing, ferry route distance

- **Impact on outcomes**: Does bridge placement affect final districts? Are bridges arbitrary or robust?

**Recommendation**: Sensitivity analysis: For 2-3 island states (HI, AK, MI), vary bridging rule (Euclidean centroid, closest point, 5km threshold) and show results are robust.

### m3. Recursive Bisection vs. k-way Partitioning Underanalyzed

**Section 3.8 mentions** you chose recursive bisection over direct k-way partitioning for "hierarchical interpretability." But no mathematical comparison of:

- **Optimality gap**: How much worse is recursive bisection than optimal k-way partitioning?
- **Empirical comparison**: For a few states, run both methods and compare compactness
- **Theoretical analysis**: Under what conditions are they equivalent vs. divergent?

**Why this matters**: If k-way partitioning produces significantly better compactness, readers will ask why you didn't use it. "Interpretability" alone may not justify 10-20% compactness loss.

**Recommendation**: Either (1) empirical comparison showing gap is small (<5%), or (2) stronger argument for hierarchical interpretability value (connects to regional geography, matches nested communities).

### m4. Block-Level Implementation Feasibility Overstated

**You claim**: "Block-level refinement is feasible for operational deployment" (Sections 4.1, 6.1)

**Reality**: 8 million blocks = 100× larger graphs. METIS runtime is O((|V|+|E|)log k). Even with parallelization, computational cost increases substantially.

**Missing**: Any empirical evidence you've actually tried block-level or estimation of feasibility:
- Runtime projections
- Memory requirements
- Whether METIS can handle 100K-node graphs (California blocks)

**Recommendation**: Either (1) implement block-level for 2-3 small states to demonstrate feasibility, or (2) be more cautious about claiming "feasible" without evidence.

## Strengths (Preserve These)

1. **Philosophical sophistication**: Recognition that perfect fairness is impossible and process legitimacy matters more than outcome optimality—this intellectual maturity is rare and valuable.

2. **Impossibility defense**: Clever legal/philosophical framing that sidesteps Rucho's intent-based problems.

3. **Honesty about limitations**: Acknowledging geographic sorting, efficiency gaps, and compactness gaps strengthens credibility rather than weakening it.

4. **Clean methodology**: Recursive bisection is understandable and reproducible, important for public acceptance.

5. **Computational efficiency**: 2-3 hours enables iteration and sensitivity analysis.

## Recommendation

**Major Revisions Required**

This paper has significant potential but needs mathematical strengthening before publication in a top venue. Three critical additions:

1. **M1**: Rigorous analysis of edge-cut minimization and its relationship to compactness metrics
2. **M2**: Engagement with fairness criteria literature and formal axiomatization
3. **M3**: Serious treatment of VRA tension with mathematical analysis of trade-offs

These are not minor edits—they require substantive new mathematical analysis (possibly 3-4 pages of additional content). But they're essential for the paper's central claims:

- **Impossibility defense requires**: Showing edge-cut minimization is truly "neutral" geometric optimization without hidden partisan properties
- **Process fairness argument requires**: Formal statement of which fairness criteria you satisfy/violate and why your trade-offs are defensible
- **VRA feasibility requires**: Mathematical demonstration that constrained optimization can meet legal requirements without excessive compactness sacrifice

With these additions, this would be an important contribution bridging political science, mathematics, and law. Without them, the philosophical framing is compelling but the mathematical foundations are too thin for top-tier publication.

---

**Final note**: I've worked extensively on the mathematics of gerrymandering and know how hard it is to bridge mathematical rigor with policy relevance. Your paper does better than most at philosophical sophistication. But don't sacrifice mathematical precision for accessibility—top venues expect both. The impossibility defense is strong enough to carry rigorous mathematical analysis; give readers that foundation and the contribution will be truly compelling.
