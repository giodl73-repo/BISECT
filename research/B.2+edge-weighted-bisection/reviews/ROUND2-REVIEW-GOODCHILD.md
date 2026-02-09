# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis, geography
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made significant improvements addressing most algorithmic and empirical concerns from Round 1. The additions of partitioning quality analysis, partisan outcome evaluation, VRA compliance analysis, and county preservation study substantially strengthen the paper. The work now engages with redistricting as a multifaceted socio-spatial problem, not purely geometric optimization.

From a GIS perspective, the paper demonstrates competent spatial data handling and produces empirically strong results. The county preservation analysis (P2.4) is particularly relevant to my geographic concerns—showing that compactness and administrative boundary preservation are largely compatible addresses a key geographic principle.

However, my primary Round 1 concern about census tract boundaries as fixed primitives (M1) remains unaddressed. The paper still treats census tracts as given geometric inputs without discussing embedded biases or comparing to finer-resolution alternatives. This is a fundamental geographic limitation that affects interpretation of results.

For computational venues (KDD, AAAI), the current treatment is acceptable. For GIS or geography venues (IJGIS, Annals of AAG), deeper engagement with spatial representation issues would be needed.

## Updated Score

**Score**: 3/4 — **Accept with Minor Revisions** (for computational venues)
**Score**: 3/4 — **Accept with Minor Revisions** (for GIS venues, needs census tract discussion)

*No change from Round 1 for computational venues; slight improvement in comprehensiveness*

## P1 Items Addressed

While not my primary expertise, I note all P1 items were addressed:

- **P1.1 (Partisan Analysis)**: Shows compactness ≠ fairness—important for redistricting
- **P1.2 (VRA Compliance)**: 68% reduction in minority districts demonstrates representation tradeoff
- **P1.3 (Partitioning Quality)**: Topological vs geometric tradeoff well-explained
- **P1.4 (Alternative Partitioners)**: Validates generalization
- **P1.5 (Recursive Bisection)**: 100% contiguity justifies choice
- **P1.6 (Neutrality Claims)**: Language appropriately softened

These strengthen the paper's credibility and honesty about limitations.

## P2 Items Addressed (My Area)

### P2.4: County Preservation Analysis ✓ EXCELLENT

Section 3.5 (County Preservation Analysis) directly addresses geographic administrative boundary preservation:

**Findings**:
- Modest increase in splits: 28.4% algorithmic vs 27.4% enacted (+1.0pp)
- National scale: 2,636 counties analyzed across all states
- Strong negative correlation (-0.68) between compactness gains and county splits
- Only 4 states show significant tradeoff (>3 splits for >5% compactness)

**Geographic interpretation**: Compactness optimization generally respects administrative boundaries. This is important because counties often represent functional regions (services, governance, historical settlement patterns).

**Indiana case study**: Commission achieved 0.478 compactness with 32% county split rate, demonstrating that human geographic expertise can sometimes optimize multiple spatial objectives simultaneously.

**Key insight**: Geometric compactness (circle-like shapes) and administrative boundary preservation are largely compatible, contradicting common assumption that they're competing objectives.

This is excellent analysis from a geography perspective. It shows that algorithmic optimization doesn't arbitrarily split administrative regions—there's inherent geographic structure that aligns geometric compactness with functional boundaries.

### P2.5: Geographic Sorting Quantification ✓ SOPHISTICATED

Section 3.6 (Geographic Sorting Quantification) provides spatial analysis of partisan effects:

**Methodology**: Separates geographic baseline (unavoidable from population distribution) from gerrymandering premium (intentional manipulation)

**Findings**:
- 60% of states are geography-dominated (partisan bias mostly unavoidable)
- Average: 63% geographic, 37% gerrymandering

**Geographic interpretation**: This validates decades of political geography research showing that settlement patterns (urban concentration of Democrats, suburban/rural dispersion of Republicans) create inherent partisan geography. Compact districts cannot eliminate these spatial patterns.

This is sophisticated spatial analysis that demonstrates understanding of geographic determinism vs human manipulation.

## Major Geographic Concern Still Unaddressed

### M1: Census Tract Boundaries as Fixed Primitives (STILL UNRESOLVED)

My primary Round 1 concern remains: **Why optimize over census tracts?**

Census tract boundaries are social constructs that often follow problematic divisions:
- Highways and railroads (historical barriers)
- Industrial zones (economic segregation)
- Historical redlining boundaries (discriminatory policies)
- Arbitrary administrative decisions

**The problem**: Optimizing over fixed tracts may *perpetuate* embedded biases rather than creating truly optimal districts.

**What's missing**:
1. Discussion of tract boundary limitations and embedded biases
2. Comparison to block-level optimization (finer granularity) for 2-3 pilot states
3. Acknowledgment that optimal compactness over tracts ≠ optimal geographic districting

**Why this matters from GIS perspective**: Geographic data representation choices fundamentally affect analysis results. Treating socially constructed boundaries as natural constraints limits solution space.

**What would address this**:
- **Minimal**: Add 1-2 paragraphs in Discussion acknowledging tract boundary limitations
- **Better**: Compare tract-level to block-level results for 2-3 states
- **Best**: Analyze whether tract boundaries correlate with demographic divisions (racial, economic)

Without this, a fundamental geographic assumption (tracts as appropriate spatial units) remains unjustified.

## Minor Geographic Issues from Round 1

Most minor Round 1 concerns remain unaddressed but are acceptable for computational venues:

### m1: Queen Contiguity Not Justified — ACCEPTABLE

Still uses Queen contiguity without comparing to Rook. Given comprehensive results, this is unlikely to significantly affect outcomes. Acceptable.

### m2: R-tree Not Optimized — ACCEPTABLE

R-tree spatial indexing parameters not discussed. Given 10-30 second preprocessing times, optimization is unnecessary. Acceptable.

### m3: Point Adjacency (0.1m) — ACCEPTABLE

Point-contact tracts still receive 0.1m weight without geographic justification. Given that point adjacencies are rare, low impact. Acceptable.

### m4: Bridge Edges — ACCEPTABLE

Median boundary length for water crossings still heuristic. Results suggest it works reasonably. Acceptable.

### m5: Island States — ACCEPTABLE

Hawaii and Alaska not deeply analyzed (which islands grouped together). Results show strong improvements (+177% Hawaii), suggesting approach handles island geography adequately. Acceptable.

### m6: Geographic Scale Effects — NOT ANALYZED

Whether compactness improvements vary by state size, density, or urban/rural character not analyzed. Would be interesting but not critical. Acceptable.

### m7: Boundary Feature Alignment — NOT VALIDATED

Paper claims districts "follow natural geographic features" but never validates this empirically. Should overlay district boundaries with river/road networks to quantify alignment.

**Minor concern**: This claim is unsubstantiated. Either validate it or soften language.

## Minor Geographic Issues from Round 2

### M2: Functional vs Geometric Compactness (STILL NOT ADDRESSED)

My Round 1 concern about geometric vs functional compactness remains:
- Polsby-Popper measures shape regularity (geometric)
- Functional compactness measures constituent accessibility (travel time, functional regions)

A geometrically compact district may be functionally poor if it crosses mountains or bisects metro areas.

**Not blocking for computational venues**: Polsby-Popper is standard in redistricting literature.

**For GIS venues**: Would need discussion of functional compactness and whether geometric compactness corresponds to meaningful accessibility.

### M3: Projection Sensitivity (STILL NOT TESTED)

Projection choices affect boundary length measurements. The paper uses state-specific projections (California Albers, Texas Conic) but doesn't test sensitivity.

**Not blocking**: State-specific projections are reasonable choices. Sensitivity testing would strengthen but isn't critical.

## Observations on Geographic Results

### Compactness Variation by Geography

From the results, I observe (though paper doesn't explicitly analyze):
- **Island states** (Hawaii +177%, Alaska not solo state) show huge improvements
- **Large states** (California, Texas) show strong improvements
- **Urban states** (Illinois +174%) show dramatic improvements

This suggests geometric optimization particularly benefits:
1. Fragmented geography (islands)
2. Large spatial scale
3. Urban areas (where human redistricting creates elongated districts)

**Geographic interpretation**: Human redistricting tends to elongate urban districts (packing/cracking). Algorithmic optimization naturally creates compact urban districts by minimizing perimeter.

The paper could briefly discuss geographic patterns in improvement magnitudes.

### County Preservation Pattern

The county preservation analysis shows interesting geographic pattern:
- Only 4 states face significant tradeoff
- Strong negative correlation (-0.68) between splits and compactness gains

**Geographic interpretation**: States where compactness improvement requires county splitting are likely those where counties are geographically large and irregularly shaped (Western states?). Eastern states with small, regular counties can optimize both objectives.

The paper could analyze geographic patterns in county splits vs state region, county size distribution, etc.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **County preservation analysis**: Shows compactness respects administrative boundaries—important geographic principle

2. **Geographic sorting quantification**: Validates political geography research on settlement pattern effects

3. **Honest limitations**: VRA analysis shows representation tradeoff; partisan analysis shows mixed results

4. **National scope**: 50-state coverage demonstrates method handles diverse geographies (islands, mountains, deserts, urban)

5. **Functional regions**: County analysis suggests optimization respects functional geographic boundaries

## Final Recommendations

**For computational venues (KDD, AAAI)**:
- **Minimal**: Add 1-2 paragraphs acknowledging census tract boundary limitations
- **Better**: Briefly discuss why tracts are appropriate spatial units despite being social constructs

**For GIS venues (IJGIS, AGILE)**:
- **Required**: Census tract boundary discussion with comparison to block-level (2-3 pilot states)
- **Recommended**: Functional compactness discussion
- **Suggested**: Projection sensitivity analysis

**For all venues**:
- Validate or soften "follows natural features" claim (m7)
- Brief analysis of geographic patterns in improvement magnitude

---

## Verdict

**Accept with Minor Revisions** — Census tract discussion needed

**Changes from Round 1**: No score change (still 3/4), but paper is more comprehensive. Close to 4/4 for computational venues if census tract discussion added.

**Rationale**: The paper makes strong contributions:
1. **Algorithmic**: Edge weighting improves compactness substantially
2. **Empirical**: Comprehensive evaluation of multiple redistricting criteria
3. **Geographic**: County preservation analysis shows boundary respect

**Remaining concern**: Census tract boundaries as fixed primitives is fundamental geographic limitation that deserves 1-2 paragraphs of discussion.

For computational venues, this is minor addition—acknowledging the limitation without requiring block-level experiments would suffice for acceptance.

For GIS venues, block-level comparison would be needed to justify spatial unit choice.

**Recommendation for target venue**:
- **KDD/AAAI**: Add brief tract boundary discussion → 4/4 Accept
- **GIS venues**: Add tract discussion + block-level pilot (2-3 states) → 4/4 Accept
- **Current state**: 3/4 for both, very close to acceptance

**Confidence**: High — I have extensive experience with GIS theory and spatial analysis. The technical quality is strong. The tract boundary issue is legitimate geographic concern that would be raised by any GIS reviewer, but it's addressable with brief discussion for computational venues or modest experiments for GIS venues.

The paper makes valuable contributions to both redistricting and spatial optimization. The county preservation analysis demonstrates that geometric and administrative boundary objectives are compatible—an important geographic finding.
