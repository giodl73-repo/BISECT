# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection

**Reviewer**: Moon Duchin (Rutgers University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, VRA compliance, Shaw v. Reno analysis, GerryChain, geometric probability
**Date**: 2026-05-02

## Overall Assessment

VRASection is a genuinely clever design: it reframes the VRA compliance problem as a geographic signal extraction problem rather than an optimization constraint problem, and in doing so sidesteps the numerical instability that killed MetisVra. The alignment score $A(\text{split}) = |\text{MVAP}_\text{frac}(L) - \text{MVAP}_\text{frac}(R)|$ is simple, interpretable, and directly operationalizes Gingles Prong 1's geographic compactness predicate. The Alabama result---shifting from 1:6 (Birmingham isolation) to 2:5 (Black Belt concentration) at a 4.3% EC premium---is the right empirical case study for demonstrating the concept.

However, the paper has a significant structural gap that must be addressed before publication in a law review or election law journal: **the legal analysis of the Shaw/Miller predominance threshold is thin and, in several respects, incorrect**. The claim that $w_\text{vra} < 0.5$ satisfies Shaw's predominance test rests on a mathematical inequality that has no grounding in how the Supreme Court actually applies the predominance doctrine. Courts do not look at objective function weights; they look at whether racial considerations explain departures from traditional redistricting criteria that could not be explained otherwise. The paper's quantitative Shaw analysis is internally consistent but legally disconnected.

The empirical findings are also incomplete in a way that matters for the core claim. VRASection concentrates minority population on one side of the first bisection, but the paper does not demonstrate that this concentration produces actual majority-minority districts in the final 7-way partition. Without showing MM district counts under VRASection versus GeoSection versus MetisVra, the claim that VRASection "satisfies" Section 2 is unsubstantiated.

## Score: 3/4

**My score**: 3/4 --- Important algorithmic contribution with flawed legal analysis; revise to connect Shaw argument to actual doctrine and add end-to-end MM district counts.

## Major Strengths

1. **Correct problem diagnosis**: The MetisVra failure analysis (constraint scale mismatch at 60--800x) is an important contribution from B.3 that this paper correctly builds on. The architectural decision to move the minority signal out of the METIS constraint layer and into the ratio selection layer is a genuine insight.

2. **Legal design defensibility**: The decision to apply alignment only at level 1 of the bisection tree, bounded to 40% of the selection score, is well-motivated from both algorithmic and legal perspectives. The "race observed once, at the highest geographic scale" framing is compelling for expert testimony.

3. **Gingles Prong 1 operationalization**: The alignment score directly encodes what Prong 1 requires --- geographic concentration. A community that satisfies Prong 1 is one whose tracts cluster in the adjacency graph, and high $A$ measures exactly that clustering. This is a principled connection between the legal standard and the algorithm.

4. **Graceful degradation**: VRASection reducing to GeoSection when minority population is geographically dispersed ($A \approx 0$ for all ratios) is the correct failure mode. The algorithm does not force a minority-concentrated partition where none exists in the geography.

## Major Issues (Must Address)

### Issue 1: The Shaw Predominance Argument Does Not Work As Written
**Severity**: High
**Description**: Section 3.3 and Section 5.2 argue that $w_\text{vra} < 0.5$ satisfies the Shaw/Miller racial predominance test because the compactness component of the objective exceeds the alignment component. This argument has a fundamental flaw: the Supreme Court's predominance test is not an inequality on objective function coefficients.

*Shaw v. Reno* (1993) and *Miller v. Johnson* (1995) ask whether a legislature "subordinated" traditional redistricting criteria to racial considerations. The predominance inquiry is holistic and counterfactual: would the district have substantially the same shape if race had not been considered? *Hunt v. Cromartie* (2001) clarified that when a racial classification correlates with a political one, plaintiffs must show race was the actual explanation for the boundary, not partisanship or geography.

None of this maps onto a weight-in-objective-function argument. A redistricting plan could assign race a 1% weight in a formal objective and still be found to exhibit racial predominance if the 1% caused the key boundary shift. Conversely, a plan could assign race a 49% weight and survive if the boundary choice is explicable on geographic grounds.

What the paper *can* legitimately claim is weaker but still valuable: (1) compactness is the primary criterion by construction; (2) the alignment bonus is bounded and acts only as a tie-breaker; (3) the algorithm produces boundaries that are explicable on geographic grounds (the Black Belt boundary in Alabama is geographically real, not manufactured). This is a plausible basis for arguing the plan survives racial predominance scrutiny, but it requires engagement with the actual legal standard, not a substitution of mathematical inequality for constitutional doctrine.

**Recommendation**: Rewrite Section 3.3 and Section 5.2 to (1) accurately describe the Shaw/Miller/Hunt test, (2) acknowledge that the $w_\text{vra} < 0.5$ inequality is a necessary but not sufficient condition for surviving scrutiny, and (3) argue that VRASection's outputs survive the counterfactual test because the geographic boundaries it selects are explicable on non-racial grounds. Cite *Hunt v. Cromartie* (2001) on the racial/political correlation issue.

### Issue 2: No End-to-End MM District Counts
**Severity**: High
**Description**: The paper's central claim is that VRASection produces better conditions for minority opportunity district formation than GeoSection. But the paper reports only first-level bisection ratios, not actual MM district counts in the final partition. For Alabama's 2:5 split, the paper projects "35--40% Black VAP" in the southern sub-region, which "may" produce one or two Black-opportunity districts --- but this is speculative.

The difference between 1 and 2 Black-majority districts in Alabama is the precise question that *Allen v. Milligan* (2023) resolved: Alabama was required to draw 2, not 1. A paper motivated by *Allen v. Milligan* that cannot report whether VRASection produces 2 MM districts in Alabama is incomplete as an empirical contribution.

**Recommendation**: Run the full pipeline for Alabama and report: (a) the number of districts with >50% Black VAP under VRASection versus GeoSection; (b) whether VRASection produces 2 MM districts as required by *Allen v. Milligan*; (c) the compactness scores (Polsby-Popper or equivalent) for those MM districts. For the other five states, "TBD" is acceptable as a draft, but Alabama must be resolved.

### Issue 3: The Alignment Score Conflates Two Different Quantities
**Severity**: Medium
**Description**: Definition 3 defines MVAP_frac(S) as $\sum_{v \in S} m_v / \sum_{v \in V} m_v$ --- the fraction of the *subgraph's total minority population* that lands on side $S$, not the minority fraction *of side $S$*'s population. This means $A = 1$ means "all minority population is on one side" not "one side is 100% minority." The paper is internally consistent about this, but it creates a gap in the interpretation.

High alignment score ($A \approx 0.8$) means a large fraction of the state's minority population is concentrated on one side, but that side could still have a minority VAP of only 35% if the side contains a large share of the state's total population. In Alabama's 2:5 case, the 2-district side containing the Black Belt has ~35--40% Black VAP even with high alignment---well below 50%, which means Gingles Prong 1 is satisfied at the sub-region level but additional concentration work is needed in the downstream recursion.

The paper acknowledges this in Section 5.4 (the "limitation" paragraph) but frames it as a limitation rather than a definitional clarification. This should be clarified upfront in Section 3.1: the alignment score measures *distributional concentration* of minority population across the bisection, not the *minority percentage* of any district.

**Recommendation**: Add a clarifying paragraph in Section 3.1 distinguishing MVAP_frac (what fraction of the state's minority population is on this side) from minority percentage (what fraction of this side's population is minority). Add a table for Alabama showing both quantities for the 2:5 and 1:6 splits.

## Minor Issues

- **CVAP vs VAP**: The paper uses VAP (voting-age population) throughout. For legal analysis of minority districts, CVAP (citizen VAP) is the legally appropriate denominator because non-citizens cannot vote. For Alabama this matters less (low non-citizen population) but for future applications to Georgia, North Carolina, and especially any southwestern state, the distinction is important. Flag this as a limitation.

- **The "50/50" Shaw boundary**: The claim that $w_\text{vra} = 0.5$ is "borderline" (Table 3) suggests a bright-line rule that does not exist in the case law. The 50% threshold is the paper's own construct. Remove this column from the table or reframe it as "within the proposed safety margin" rather than an externally defined legal threshold.

- **The NC 7:7 to 1:13 shift**: The paper notes this result (VRASection shifting from GeoSection's near-equal 7:7 to 1:13) but does not fully discuss its implications. A 1:13 ratio in a 14-district state means Charlotte and Raleigh are isolated as a single unit, which is the same caterpillar pattern that GeoSection was designed to avoid in the B.8 work. Is VRASection re-introducing a form of caterpillar peeling by a different mechanism? This deserves discussion.

- **Missing maps**: Any redistricting paper discussed at an election law journal will be expected to show maps. Even a schematic showing the Black Belt boundary overlaid on Alabama's 2:5 vs 1:6 first-level split would substantially improve the paper's accessibility.

- **Callais section is underdeveloped**: Section 5.5 on the Callais evidence layer is three paragraphs and reads as a pointer to other work. Either develop it into a substantive integration --- showing how the two-stage workflow (VRASection map + bloc-voting analysis) maps onto the Callais evidentiary standard --- or remove it and save it for the companion Callais paper.

## Questions for Authors

1. Does VRASection produce 2 Black-majority districts in Alabama? What are their Black VAP percentages and Polsby-Popper compactness scores?

2. The Shaw argument relies on $w_\text{vra} < 0.5$. What is the paper's response to a plaintiff who argues that the 2:5 split in Alabama would not have been chosen but-for the minority VAP input, and that this but-for causation is sufficient for racial predominance regardless of the 40/60 weighting?

3. The NC shift from 7:7 to 1:13 is described as "Charlotte/Raleigh isolated." Is this an improvement over GeoSection's 7:7? Does it produce more MM districts or fewer?

4. For states where MetisVra succeeded (MS, GA), does VRASection produce the same MM count? If VRASection produces fewer MM districts than MetisVra in states where MetisVra works, that is a meaningful limitation.

## Recommendation

Revise with emphasis on legal analysis and empirical completeness. The algorithm is sound; the paper needs to connect it to actual VRA doctrine rather than a mathematical proxy for racial predominance, and must demonstrate end-to-end MM district production for the lead case study (Alabama).
