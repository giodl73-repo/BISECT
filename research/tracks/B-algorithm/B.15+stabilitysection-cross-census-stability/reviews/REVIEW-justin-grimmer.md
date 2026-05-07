> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Justin Grimmer (Stanford University, Department of Political Science)
**Expertise**: Quantitative political methodology, computational text analysis, redistricting measurement, causal inference, electoral politics
**Date**: 2026-05-02

## Overall Assessment

This paper proposes the Census Stability Score (CSS) as a measure of cross-census algorithmic stability in redistricting and reports preliminary empirical findings showing 67% of states exhibit ratio stability across the 2000--2010 decade. The research design is appropriate for the question being asked, and the finding --- that most states' geographic structures are durable enough to produce the same algorithmic partition across a decade of population change --- is scientifically interesting.

My primary concern is a methodological one about the cross-census comparison design: the paper treats ratio stability as the outcome but does not adequately control for confounders introduced by the cross-census comparison setup. The comparison is between algorithms run on different graphs (different tract structures), different population distributions, and in many cases different numbers of districts. Any one of these differences could explain ratio changes independently of genuine geographic reorganization. The paper's Type I / Type II decomposition is designed to address the first two confounders, but it has not been executed. Until it is, the 67% finding cannot be cleanly attributed to geographic stability versus data artifact.

My secondary concern is that the paper's causal framing --- "the partition is determined by geography, not by partisan composition" --- makes a causal claim that the empirical design does not fully support. Geographic stability across census years is consistent with (but does not prove) the claim that the outcome is geographically determined rather than partisan-driven. There is an alternative explanation for cross-census stability that the paper does not consider: if the partisan geography and the physical geography are correlated across all three census years (which they are, by the geographic sorting documented in Rodden 2019), then both geography-driven and partisan-driven algorithms could produce the same cross-census stable outcome.

## Score: 3/4

**My score**: 3/4 --- Good research design for the stability question; causal inference from stability to geographic determinism needs additional support; Type I/Type II decomposition must be executed before the main claim is fully warranted.

## Major Strengths

1. **The correct unit of analysis**: The paper measures stability at the first-level bisection, which is the right level of analysis for the legal argument. The first-level split is the most consequential algorithmic decision (it determines which sub-regions will produce how many districts), and it is the decision most susceptible to partisan manipulation. Showing that this decision is stable across census years is more legally relevant than showing stability at lower levels of the bisection tree.

2. **Interesting state heterogeneity**: The paper's predictions about which states will be stable versus volatile are substantively interesting. The Midwest row (Iowa, Kansas, Nebraska) versus the Sun Belt growth corridor (Arizona, Nevada, Colorado) is a testable and well-motivated contrast. The metropolitan-fixity hypothesis (Milwaukee's boundary is fixed; Phoenix's is not) provides a mechanism, not just a correlation.

3. **The 2D stability matrix is a useful synthesis**: Combining seed stability (B.7) and census stability (B.15) into a 2x2 matrix is a good organizational framework. The prediction that the bottom-right cell (low seed stability, high census stability) should be empty is a testable cross-paper claim.

4. **The proportionality gap analysis**: Documenting that 22 of 37 states with complete analysis show Republican over-representation, and that this pattern is consistent with Rodden's geographic sorting thesis, contextualizes the stability analysis within the broader literature.

## Major Issues (Must Address)

### Issue 1: The Cross-Census Comparison Has Three Uncontrolled Confounders
**Severity**: High
**Description**: The paper compares GeoSection outputs across census years and attributes ratio changes to "geographic reorganization." But three things change between census years simultaneously: (1) the population weights on nodes, (2) the graph topology (tract boundary redesign), and (3) the number of districts in some states. Any of these changes could explain ratio shifts independently.

The paper acknowledges (1) and (2) as the Type I/Type II decomposition and proposes to address them, but has not done so. The paper does not acknowledge that changes in $k$ (the seat count) introduce a third source of variation: for states where $k$ changes, the normalised ratio comparison requires additional assumptions. For example, Texas going from $k=30$ to $k=38$ means the normalised ratio 1:37 (2020) is compared to 1:29 (2000 equivalent). Whether these normalised ratios represent the same geographic structure is an assumption the paper makes without testing.

For the causal claim about geographic determinism to hold, the paper needs to show that the ratio changes observed in the 10 unstable states are driven by genuine geographic reorganization and not by data artifacts (tract redesign) or apportionment changes. The Type I / Type II decomposition is essential for this.

**Recommendation**: Execute the Type I/Type II decomposition for the 10 unstable states before claiming that instability reflects "genuine geographic reorganization." For at least one unstable state (Iowa is the most dramatic), show the normalised edge-cut curves under both the full cross-year comparison and the population-only perturbation.

### Issue 2: Cross-Census Stability Does Not Rule Out Partisan Determination
**Severity**: High
**Description**: The paper's core legal argument is: if GeoSection produces the same partisan outcome across three censuses, the outcome cannot be a product of any particular census year's partisan distribution. This argument has a gap.

Partisan geography and physical geography are correlated. The geographic sorting of Democratic voters into dense urban areas has been stable across all three census cycles; the urban-rural political divide in Wisconsin is similar in 2000, 2010, and 2020. If an algorithm is sensitive to partisan geography --- even if it takes only geographic inputs --- it could produce stable cross-census outcomes *because the partisan geography is stable*, not because the physical geography independently determines the outcome.

The paper's claim is that GeoSection uses only physical geography (tract adjacency and population counts), not partisan data. This is true by construction. But the legal argument requires a stronger claim: that no partisan-sensitive algorithm could have produced the same stable outcomes by tracking the stable partisan geography. This claim is not addressed.

The distinction matters for courts: if the physical geography and the partisan geography are perfectly correlated across all three census cycles (which is approximately true in Wisconsin), then cross-census stability of a geographic algorithm does not distinguish it from a partisan algorithm that happened to select the same outcome.

**Recommendation**: Add a discussion in Section 5.1 that addresses the partisan-physical geography correlation directly. The argument should be: (1) GeoSection uses no partisan data by construction; (2) partisan data is not available for the 2000 and 2010 inputs, so partisan optimization was not possible even if attempted; (3) the stability claim is about the algorithm's invariance to the specific partisan distribution of each census year, not about whether the physical and partisan geographies are correlated. Framing (3) clearly would substantially strengthen the legal section.

### Issue 3: The Iowa Result Needs a Political Geography Explanation
**Severity**: Medium
**Description**: Iowa's shift from $f = 0.18$ to $f = 0.49$ is described as resulting from "rural decline and urban growth." This is plausible but needs verification and a more specific causal story.

Iowa politics changed significantly between 2000 and 2010. In 2000, Iowa was a presidential swing state (Gore won Iowa by 0.3%). By 2010, the rural-urban divide had deepened considerably. The state's congressional delegation flipped from 2D/3R (2000) to 1D/4R (2010) as suburban Des Moines became more Republican. If GeoSection's output shifted from an asymmetric "rural peel" to a near-equal east/west split between these years, was this driven by population dynamics or by the changed population-geography relationship that itself reflects demographic-political sorting?

The paper should not use election results to explain the Iowa shift (since the algorithm takes no partisan input), but it should provide a geographic mechanism: which cities grew, by how much, and why that growth changed the edge-cut comparison at the 2:2 vs 1:3 threshold. A map showing population change by county between 2000 and 2010 in Iowa would make this explanation concrete.

**Recommendation**: Add a geographic analysis of the Iowa shift: population change by county, growth of the Des Moines and Cedar Rapids metros, and the implication for the isoperimetric gap between the 1:3 and 2:2 ratios.

## Minor Issues

- **The 67% finding needs a null hypothesis**: What would we expect the stability rate to be if ratios were assigned randomly? For a typical state, GeoSection selects from a small number of candidate ratios (e.g., 4 candidate ratios for an 8-district state). If ratios were assigned independently and uniformly, the probability of matching across two census years is 1 / (number of candidate ratios). For an average of ~5 candidate ratios, random matching would give 20% stability. The 67% finding is substantially above this baseline but the paper does not compute it.

- **The Lorenz drift proxy should be validated now**: For the 47 states with 2000 data and 48 states with 2010 data, the paper can test whether $\Delta p^* > 0.05$ predicts ratio instability. This validation would strengthen the paper's predictive claim substantially, and the data to do it appears to exist. The paper should run this analysis.

- **Figure 1 (2D stability matrix) is a table, not a figure**: The 2x2 stability matrix is typeset as a table with representative states in each cell. This is correct but should be supplemented with a scatter plot of seed-stability (from B.7) versus census-stability (from this paper) for the states where both are available.

- **The "Structurally volatile" category is undersold**: The Sun Belt states (TX, AZ, FL) are described as "not algorithm failures" but as genuine geographic reorganizations. This framing is correct but could be developed further: are there states where cross-census instability is diagnostic of an ongoing population transition that will eventually reach a new stable equilibrium? This would connect the paper to the broader political geography literature on urban-rural sorting dynamics.

- **The 20-year span argument needs a precedent check**: The claim that "this partition has existed before living memory of the current partisan alignment" is rhetorically powerful but requires that the 2000 partisan alignment was genuinely different from 2020's. The paper should cite evidence for the claim that the urban-rural partisan divide was less pronounced in 2000 than in 2020 (Rodden 2019 documents this, but the paper should be explicit about what specifically differed).

- **Table 3 shows only predictions, not actuals**: Table 3 ("Natural ratio stability: 2000 vs. 2020 prediction") is entirely prediction-based for all states except those where the 2000 sweep is marked "Same" or "Shifted." The paper should clarify which entries are confirmed from data versus predicted from the Lorenz proxy.

## Questions for Authors

1. For the 10 unstable states in the 2000--2010 comparison, what is the distribution of seat count changes? Are the unstable states disproportionately among states that changed seat count (suggesting the apportionment confounder)?

2. For Iowa specifically: does the shift from $f = 0.18$ to $f = 0.49$ correspond to a ratio change from 1:3 to 2:2, or to something else? What are the actual ratios in 2000 and 2010?

3. Is there variation in stability rates across census year pairs? That is, is the 2000--2010 stability rate (67%) different from the 2010--2020 rate (expected once 2010 sweep completes)? If the 2010--2020 rate is substantially different, that would be interesting: the 2010 redistricting cycle was heavily partisan (post-REDMAP), while 2020 had more independent commissions.

4. For the Lorenz drift proxy: in the 47 states with 2000 data complete, does the proxy's prediction of stability (high versus low) match the observed ratio stability at the 5% threshold?

## Recommendation

This paper makes a meaningful contribution to the quantitative redistricting literature by introducing cross-census stability as an evaluation dimension and reporting the first empirical estimates of it. The main issues --- the unexecuted Type I/Type II decomposition, the causal inference gap, and the Iowa case study --- are addressable in a revision. The paper should also add a null hypothesis comparison (random stability rate), run the Lorenz proxy validation now rather than listing it as future work, and be more explicit about the scope of the causal claim regarding geographic determinism. Revise and resubmit.
