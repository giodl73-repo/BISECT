> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Jonathan Rodden (Stanford University, Political Science)
**Expertise**: Geographic sorting, urban-rural divide, electoral geography
**Date**: 2026-02-08
**Venue**: American Political Science Review

---

## Overall Assessment

This paper tackles the central tension in algorithmic redistricting: geographic sorting creates partisan asymmetry even without intentional manipulation. The finding that neutral algorithms produce -3.2% Democratic efficiency gap is exactly what my work on urban concentration predicts, and the 8.3 percentage point gap relative to enacted plans quantifies manipulation vs. geography nicely. However, the paper needs to engage much more deeply with *why* geography produces these patterns and what this means for democratic representation.

**Verdict**: Accept with moderate revisions
**Score**: **3.5/4**

---

## Major Issues

### 1. Insufficient Geographic Mechanism Analysis (P1)

Section 5.1 explains that "Democratic urban concentration creates packing," but this is far too cursory. The geographic sorting story is the paper's theoretical foundation—you can't just wave at it. Your readers need to understand:

- **Scale**: Urban cores are 5-10x more Democratic than rural areas are Republican
- **Compactness interaction**: Round urban districts necessarily pack Democrats; elongated districts could crack them but violate compactness
- **Why asymmetry**: Republicans' suburban dispersion allows natural cracking without gerrymandering

**Required additions:**
1. Quantify urban concentration using actual district-level data (what % of Democratic votes come from districts >70% Democratic? Compare to Republican distribution)
2. Show compactness-partisan tradeoff explicitly: if you relax compactness (allow Polsby-Popper 0.15 instead of 0.25), does algorithmic EG approach zero?
3. Explain why suburbs matter: suburban swing districts are geometrically feasible, urban swing districts are not

### 2. Regional Variation Undertheorized (P1)

You show Rust Belt exhibits largest enacted-algorithmic gap (10.0 pp), but don't explain *why*. My hypothesis: Rust Belt has sharp urban-rural divides (Pittsburgh, Milwaukee, Detroit vs. rural PA/WI/MI) that make geographic packing severe. Sunbelt cities are more sprawled, creating weaker concentration.

**Missing analysis:**
- Urban density by region (people per square mile in core districts)
- City structure (monocentric vs polycentric) and its effect on geographic sorting
- Why Texas shows 9.8 pp gap despite sprawl (answer: intentional cracking of San Antonio, Houston, Dallas)

**Required revision:** Add Section 5.2 "Regional Variation in Geographic Sorting" explaining why Rust Belt, South, and Sunbelt show different patterns. Use census data on urban density, city structure, and commuting zones.

### 3. Proportional Representation Discussion Too Shallow (P2)

Section 5.4 briefly mentions that algorithmic redistricting doesn't achieve proportionality, but this deserves extended treatment. Your -3.2% EG means Democrats win 56.5% seats with 52% votes—is this acceptable? From different normative frameworks:

- **Competitive elections view**: Modest bias okay if districts competitive
- **Proportional representation view**: Any systematic deviation from vote-seat proportionality is problematic
- **Geographic representation view**: Bias reflects legitimate geographic communities

**Required addition:** Dedicated subsection discussing three normative frameworks for evaluating partisan fairness. Your readers need to understand that "reducing bias" doesn't mean "eliminating bias," and whether -3.2% is acceptable depends on one's theory of fair representation.

---

## Minor Issues

### 4. Missing Counterfactual: Optimizing for Proportionality (P2)

You show what happens when algorithms optimize for compactness (they produce -3.2% EG). But what if algorithms explicitly optimized for proportional representation? How much would compactness suffer?

**Suggested analysis:** Generate alternative algorithmic plans that minimize efficiency gap (target EG = 0%) and report their compactness scores. This quantifies the compactness-proportionality tradeoff.

### 5. Rural Over-Representation Unexplored (P3)

Your analysis focuses on partisan efficiency gaps, but there's also a geographic efficiency gap: rural voters are geometrically over-represented (their votes are less "wasted" because rural districts are closer to 50-50). This connects to Senate apportionment debates.

**Suggested addition:** Brief note in Discussion about how geographic sorting creates *geographic* bias (urban votes wasted) alongside *partisan* bias (Democratic votes wasted). These aren't entirely separate phenomena.

### 6. Temporal Stability Needs Deeper Interpretation (P3)

Figure 3 shows EG is stable across 2016-2020, but you don't discuss what this stability means. In my work, stable partisan patterns indicate durable geography rather than transient electoral swings. This has important implications:

- **For reformers**: Geographic patterns won't change with new districts unless you change residential patterns
- **For courts**: Stability suggests manipulation (in enacted plans) is durable, not election-specific
- **For scholars**: Reinforces that partisan bias is structural, not contingent

**Suggested expansion:** Add 2-3 paragraphs interpreting temporal stability through the lens of geographic determinism.

---

## Positive Aspects

1. **Empirical rigor**: National scale with 3 election years is exactly what the field needs
2. **Clean experimental design**: Algorithmic plans truly can't access partisan data, unlike human "simulations"
3. **Honest about limits**: Acknowledging that algorithms produce Democratic bias due to geography is refreshing
4. **Regional analysis**: Rust Belt vs Sunbelt breakdown reveals important variation
5. **Policy relevance**: Clear implications for reform efforts

---

## Specific Recommendations

### Section 3 (Methodology)
- Add paragraph on why METIS recursive bisection is appropriate (optimizes compactness, respects contiguity, avoids partisan data)
- Discuss alternative algorithms (would k-means, Voronoi diagrams, or other methods produce similar EG?)

### Section 4 (Results)
- Add subsection quantifying urban concentration by region (avg Democratic % in top-5 urban districts per state)
- Show scatter plot: urban density (y-axis) vs enacted-algorithmic EG gap (x-axis) to demonstrate correlation

### Section 5 (Discussion)
- Expand Section 5.1 to 2-3 pages with detailed geographic sorting mechanisms
- Add Section 5.2 on regional variation in urban-rural divides
- Add Section 5.3 on normative frameworks for evaluating partisan fairness
- Connect proportionality analysis to geographic sorting explicitly

### Section 6 (Conclusion)
- Add discussion of residential sorting as upstream problem (if Democrats move to suburbs, geography becomes less asymmetric)
- Mention that algorithmic redistricting can't solve problems created by housing policy, zoning, and economic geography

---

## Questions for Authors

1. Have you tested whether relaxing compactness (allow less compact districts) reduces algorithmic EG toward zero?
2. Could you generate "urban district" vs "suburban district" typology and compute EG separately for each type?
3. What happens if you allow population deviations (e.g., ±5% instead of ±0.5%)? Does geographic sorting become less severe?

---

## Verdict Justification

This paper makes an important empirical contribution by quantifying the efficiency gap for algorithmic redistricting at national scale. The finding that neutral algorithms produce -3.2% Democratic EG is exactly what theory predicts, and the 62% bias reduction relative to enacted plans demonstrates reform value.

However, the paper needs to engage more deeply with *why* geography produces partisan asymmetry. As the leading scholar on urban-rural divides, I need to see:
1. Quantitative analysis of urban concentration by region
2. Explicit modeling of compactness-proportionality tradeoff
3. Normative discussion of whether -3.2% bias is acceptable

With these additions, this will be a landmark paper demonstrating the limits of algorithmic objectivity. The empirics are strong; the theory needs development.

**Recommendation**: Accept with moderate revisions (primarily expanding geographic mechanisms and normative discussion).
