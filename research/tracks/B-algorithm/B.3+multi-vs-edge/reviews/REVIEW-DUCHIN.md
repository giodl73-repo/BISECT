> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Moon Duchin (Rutgers University)
**Expertise**: Redistricting, gerrymandering detection, geometric probability, VRA compliance, GerryChain
**Date**: 2026-02-08

## Overall Assessment

This paper makes an important contribution to computational redistricting by systematically comparing two algorithmic approaches for creating majority-minority districts under the Voting Rights Act. The core finding—that edge-weighted optimization substantially outperforms multi-constraint methods—is policy-relevant and will influence how redistricting practitioners approach VRA compliance. The experimental scope (160 runs across five states) is impressive, and the constraint conflict explanation is intuitive.

However, as someone who specializes in redistricting and VRA compliance, I have significant concerns about the paper's legal and geographic realism. The paper treats VRA compliance as a binary metric (≥50% minority = success) without engaging with the legal complexity of Sections 2 and 5, coalition districts, or functional analysis of minority electoral opportunity. The choice to use "all non-white" as the minority population definition is legally questionable—VRA analysis requires examining specific protected groups, not aggregate minorities. The compactness analysis relies solely on edge cut, ignoring geographic realism, irregular boundaries, and community integrity. Finally, all experiments use a single census year (2020) and focus on southern states, limiting generalizability across demographic and temporal contexts.

These issues don't invalidate the algorithmic findings, but they limit the paper's applicability to real redistricting. With revisions addressing legal accuracy, geographic realism, and demographic specificity, this could be an important contribution to the redistricting literature.

## Score: 3/4

**My score**: 3/4 - Minor revision required. Valuable algorithmic insights, but needs better engagement with VRA legal standards, geographic realism, and demographic specificity.

## Major Strengths

1. **First Systematic Algorithmic Comparison for VRA Compliance**: Previous redistricting research uses MCMC for plan evaluation or heuristics for plan generation, but this is the first careful comparison of graph partitioning formulations specifically for VRA compliance. This is a genuine contribution.

2. **Policy-Relevant Findings**: The 12.9 percentage point success rate gap has immediate practical implications. Redistricting commissions and courts could use this guidance to improve algorithmic map generation.

3. **Constraint Conflict Hypothesis**: The observation that 100× tolerance differences cause constraint conflict is not just algorithmically interesting—it explains why well-intentioned redistricting algorithms fail to achieve VRA compliance. This is valuable mechanistic insight.

## Major Issues (Must Address)

### Issue 1: Oversimplified VRA Legal Standard
**Severity**: High
**Description**: The paper treats VRA compliance as a simple threshold test (≥50% minority = success) without engaging with the actual legal standards. This is problematic for several reasons:

**Section 2 vs Section 5**: The VRA has two main provisions:
- **Section 5** (preclearance, partially struck down in *Shelby County*): Required maintaining existing minority districts
- **Section 2** (vote dilution): Requires creating minority opportunity districts where possible, per *Gingles* three-prong test

Your paper doesn't specify which provision you're addressing. Alabama is mentioned, which became relevant after *Allen v. Milligan* (2023) regarding Section 2, but you never cite this case or explain the legal framework.

**Gingles three-prong test** (Section 2):
1. Minority group is sufficiently large and geographically compact to constitute a majority in a district
2. Minority group is politically cohesive
3. White majority votes as a bloc to usually defeat minority-preferred candidates

Your paper tests only Prong 1 (numerical majority), ignoring Prongs 2-3 (political behavior). But Prong 1 requires "geographically compact" which is assessed via compactness metrics (Polsby-Popper, Reock, convex hull)—you use only edge cut.

**Functional analysis**: Courts assess whether minority voters have the *opportunity* to elect candidates of choice. This requires analyzing election results, not just demographics. A 50.1% minority district might not be sufficient if minority turnout is lower than white turnout or if minority voters are not politically cohesive.

**Coalition districts**: Recent court decisions recognize coalition districts where multiple minority groups combine to form >50%. Your "all non-white" definition might approximate this, but it's legally imprecise without analyzing specific coalitions.

**Recommendation**:
1. Clarify which VRA provision (Section 2 or 5) you're addressing
2. Explain *Gingles* test and acknowledge you're testing only Prong 1 (numerical threshold)
3. Cite relevant case law: *Allen v. Milligan* (Alabama), *Cooper v. Harris* (racial gerrymandering), *Bartlett v. Strickland* (crossover districts)
4. Discuss functional analysis: 50% is necessary but not sufficient for electoral opportunity
5. Acknowledge limitation: your approach doesn't assess compactness per *Gingles* Prong 1 or political cohesion per Prong 2
6. Consider raising threshold to 55% or 60% to ensure functional electoral opportunity (this is common practice)

### Issue 2: Legally Questionable Minority Population Definition
**Severity**: High
**Description**: The paper defines "minority" as "all non-white racial groups" (Section 4.1). This is legally problematic for VRA analysis:

**Specific protected classes**: The VRA protects specific racial and language minorities:
- Black/African American
- Hispanic/Latino
- Asian American
- Native American/Alaska Native
- Native Hawaiian/Pacific Islander

Courts analyze these groups *separately*, not in aggregate. A district with 30% Black, 15% Hispanic, 10% Asian is not automatically a 55% minority opportunity district—you need to analyze each group's political cohesion and electoral opportunity separately.

**Coalition districts**: While courts have recognized coalition districts (e.g., Black + Hispanic coalitions in some jurisdictions), this requires demonstrating that the groups share common political interests and vote cohesively. You cannot simply add percentages without political analysis.

**Your states' demographics**:
- **Alabama**: 36.9% "minority" but this is primarily Black (26.8%) + Hispanic (5.3%). Are you creating Black opportunity districts or coalition districts?
- **Georgia**: 49.9% "minority" with diverse demographics (Black 32%, Hispanic 10%, Asian 5%). Which groups are you targeting?

**Supreme Court precedent**: *Bartlett v. Strickland* (2009) held that Section 2 does not require creating crossover districts where minority voters rely on white crossover votes. This suggests VRA analysis should focus on single protected groups reaching >50%, not aggregate minorities.

**Recommendation**:
1. Rerun experiments with race-specific minority populations (Black VAP specifically for Alabama, Georgia, Louisiana, Mississippi, South Carolina)
2. Separately analyze Hispanic, Asian, and Native American communities
3. If testing coalition districts, justify which coalitions are politically meaningful based on analysis of election results
4. Cite *Bartlett v. Strickland* and explain whether you're testing majority-minority or coalition districts
5. Add results appendix showing outcomes for each racial group separately
6. Discuss policy implications: should algorithms target single groups or coalitions?

### Issue 3: Inadequate Compactness Analysis for Legal Standards
**Severity**: Medium-High
**Description**: The paper uses edge cut as the sole compactness metric, but courts and redistricting criteria use different metrics:

**Legal compactness standards**:
- **Polsby-Popper**: $\text{PP} = \frac{4\pi \times \text{Area}}{\text{Perimeter}^2}$ (measures shape regularity, 1 = circle)
- **Reock**: $\text{Reock} = \frac{\text{Area}}{\text{Area of minimum bounding circle}}$ (measures dispersion)
- **Convex Hull**: Ratio of district area to convex hull area
- **Population Polygon**: Distance-based measures

Edge cut is a graph metric (number of boundary edges), but courts evaluate *geographic* compactness (shape regularity). Two districts with same edge cut can have very different Polsby-Popper scores.

**Your findings**: Edge-weighted pays 13% edge cut penalty on average (Table 2). But we don't know if this translates to meaningful geographic compactness reduction. In Georgia, the penalty is only 0.8%, suggesting minimal geographic impact. In Louisiana, it's 48%, which might be legally problematic.

***Gingles* Prong 1**: Requires minority group be "sufficiently... geographically compact to constitute a majority in a single-member district." This has been interpreted to require reasonable compactness, not just contiguity. Courts have struck down districts that are geographically spread across large distances even if numerically balanced.

**Comparison to baselines**: How compact are your districts compared to actual enacted plans? Compared to MCMC ensembles? Without context, we can't assess whether 13% penalty is acceptable.

**Recommendation**:
1. Compute Polsby-Popper and Reock scores for all districts (not just edge cut)
2. Compare compactness to enacted plans for these states (available from Redistricting Data Hub)
3. Compare to MCMC ensemble distributions (use GerryChain or similar)
4. Discuss which compactness differences are legally meaningful (cite case law)
5. Add geographic visualizations showing district shapes (maps!) to assess visual compactness
6. Acknowledge that edge cut is a proxy and may not reflect legal compactness standards

### Issue 4: Insufficient Geographic and Temporal Validation
**Severity**: Medium
**Description**: The paper tests only Census 2020 data in five southern states. This limits generalizability:

**Census year variation**: Demographics change substantially across decades. Alabama's Black population was 25.8% in 2000, 26.4% in 2010, 26.8% in 2020. Your findings for 2020 may not apply to earlier decades. Courts often use historical analysis spanning multiple cycles.

**Geographic concentration changes**: Minority populations urbanize, suburbanize, and migrate between census cycles. An algorithm that works for 2020 Alabama may fail for 2010 Alabama if geographic clustering differs.

**Southern vs other regions**: Your five states are all southern with historically high Black populations. Do findings generalize to:
- **Southwestern states**: Arizona, New Mexico (Hispanic-majority populations, different geographic clustering)
- **Large diverse states**: California, Texas, New York (multiple minority groups, urban concentration)
- **States with Native populations**: Montana, North Dakota, Alaska (reservation geography, different clustering patterns)

**Partisan geography interaction**: Southern Black populations tend to cluster in Democratic urban areas and rural Black Belt counties. Does your algorithm work in states where minority populations are more geographically dispersed (e.g., Hispanic populations in California) or differently aligned with partisan geography?

**Recommendation**:
1. Test on at least one additional census year (2010 recommended) to show temporal robustness
2. Test on geographically diverse states: Arizona or New Mexico (Hispanic populations), California (diverse demographics), Montana (Native populations)
3. Analyze geographic clustering of minority populations (Moran's I, local autocorrelation) and test whether algorithm success correlates with clustering
4. Compare your districts' minority populations to actual enacted plans (are you creating realistic or artificial concentrations?)
5. Discuss generalizability limitations: "for southern states with clustered Black populations" vs "for all VRA-relevant states"

## Minor Issues

- **Title**: "Asymmetric redistricting goals" is jargon. Consider "Voting Rights Act Compliance" for clarity to policy audience.

- **Abstract**: "majority-minority districts where minority voters comprise ≥50% of voting-age population" - legally precise is Citizen Voting Age Population (CVAP), not VAP. VAP includes non-citizens who cannot vote. For Hispanic populations especially, CVAP differs substantially from VAP.

- **Section 1**: "*Reynolds v. Sims*" is cited for population balance, but the specific standard (±0.5%) comes from *Karcher v. Daggett* (1983) for congressional districts. Cite both.

- **Section 2.1**: "VRA requires creation of MM districts where feasible" - too strong. VRA prohibits vote dilution; it doesn't affirmatively require creation of districts. The requirement emerges from Section 2 enforcement via *Gingles* test. Rephrase for legal accuracy.

- **Table 1**: "Target MM" column lists proportional targets (e.g., Alabama 36.9% minority → 2/7 MM districts). But *Gingles* doesn't require proportionality. It requires creating opportunity districts where possible. Clarify this is your heuristic, not legal requirement.

- **Section 4.1**: "Data: Census 2020 tract-level" - specify exact data source. Census Bureau redistricting files (P.L. 94-171)? Citizen Voting Age Population (CVAP) from ACS? This matters for replication.

- **Section 4.2.1**: "Minority VAP" - should be CVAP for legal analysis. VAP includes non-citizens. For Alabama this may not matter (low non-citizen population), but for Georgia and other states it does.

- **Missing**: No maps! Redistricting papers should show geographic visualizations. Include at least one map comparing edge-weighted vs multi-constraint districts for Alabama (the critical case). Referees and readers need to see whether districts are geographically realistic.

- **Section 5.2**: Alabama best edge-weighted achieves 53.6% minority. Is this sufficient for functional electoral opportunity? Literature suggests 55-60% is often needed to account for turnout and age differences. Discuss whether 53.6% is robust.

- **Section 6.3**: "Courts have consistently held that some compactness reduction is acceptable" - citation needed. Cite specific cases (*Shaw v. Reno*, *Bush v. Vera*, *Miller v. Johnson*) which establish that race cannot be the predominant factor to the exclusion of traditional criteria.

- **Section 7**: Missing references to MGGG Redistricting Lab work (Duchin et al., DeFord et al.) on algorithmic redistricting, GerryChain, and ensemble analysis. This is highly relevant related work.

- **Conclusion**: "Ensure fair political representation for minority voters" - note that VRA aims to ensure opportunity to elect candidates of choice, which is related to but distinct from proportional representation. Be precise about legal goals.

## Detailed Comments

### Section-by-Section

**Introduction**: Strong motivation with clear problem statement. However, the VRA background is oversimplified (50% threshold without nuance). Needs engagement with *Gingles* test and functional analysis.

**Background**: Section 2.1 lists redistricting criteria but omits several important ones: respecting communities of interest, preserving political subdivisions (counties, cities), protecting incumbents (in some states), respecting cores of prior districts. These matter because they compete with VRA compliance.

**Theory**: The constraint conflict explanation is intuitive. However, Section 3.1.2's calculation about "impossibility" is confused (other reviewers will likely note this). The constraint conflict argument stands without claiming geometric impossibility—geographic dispersion is sufficient explanation.

**Experiments**: Systematic design but lacks geographic validation. You test whether algorithms create ≥50% districts, but not whether those districts are geographically realistic or legally compliant (*Gingles* compactness requirement). Need maps and comparison to enacted plans.

**Results**: Well-presented. Table 4 (constraint conflict test) is excellent. However, all results are demographic outcomes. Where are the maps? The geographic visualizations? Redistricting is fundamentally a spatial problem—show us the districts.

**Discussion**: Section 6.2's algorithm selection guidance is useful but presented as general graph partitioning advice when it's really redistricting-specific. Section 6.3 discusses compactness tradeoff but should include actual compactness scores (Polsby-Popper), not just edge cut.

### Figures and Tables

**Table 2**: Excellent head-to-head comparison. This is your strongest evidence. Consider adding column for Polsby-Popper scores to assess true compactness.

**Missing Figure**: Need at least one map comparing edge-weighted vs multi-constraint districts. Show Alabama (the critical case where edge-weighted succeeds and multi-constraint fails). Visual comparison is essential for redistricting paper.

**Figure 3 & Table 4**: The constraint conflict test is excellent empirical work. This is the paper's most important contribution.

**Figure 5**: Parameter sensitivity is thorough. The non-monotonic behavior suggests complex optimization landscape—this deserves deeper investigation.

## Questions for Authors

1. **Legal framework**: Which VRA provision (Section 2 or 5) are you addressing? Can you discuss *Gingles* three-prong test and acknowledge you're testing only Prong 1?

2. **Minority definition**: Why aggregate all non-white populations? Would results differ if you analyzed Black VAP specifically (the primary protected class in your five states)?

3. **Functional opportunity**: Is 50% sufficient for minority electoral opportunity, or should threshold be 55-60% to account for turnout and age differences?

4. **Geographic realism**: Can you show maps of your districts? Are they geographically compact per legal standards, or are they gerrymandered to concentrate minority populations?

5. **Comparison to enacted plans**: How do your algorithmic districts compare to actual enacted plans for these states? Are you creating more or fewer MM districts than current law?

6. **MCMC ensembles**: Have you compared your results to GerryChain-generated ensembles? Where do your districts fall in the ensemble distribution for minority percentage and compactness?

7. **Other census years**: Would your findings hold for Census 2010 or 2000? Have you tested temporal robustness?

8. **Other regions**: Would results generalize to southwestern states (Hispanic populations) or states with Native American populations?

## Recommendation

**Minor revision required, with emphasis on legal accuracy and geographic realism**. The algorithmic contribution is solid, but the paper needs better engagement with redistricting practice and legal standards.

Required changes:

1. **Engage with VRA legal standards** (Issue 1): Discuss *Gingles* test, cite relevant case law, acknowledge limitations of 50% threshold
2. **Use legally accurate minority definitions** (Issue 2): Analyze specific racial groups, justify coalition districts if using aggregate minorities
3. **Add geographic compactness analysis** (Issue 3): Compute Polsby-Popper and Reock scores, compare to enacted plans
4. **Add maps**: Show at least one state (Alabama) with geographic visualization of districts

Optional but strongly recommended:

5. Test on additional census year (2010) to show temporal robustness
6. Test on geographically diverse states (Arizona, California)
7. Compare to enacted plans and MCMC ensembles
8. Use CVAP instead of VAP for legal accuracy

This paper makes a valuable contribution to computational redistricting by demonstrating that edge-weighted optimization outperforms multi-constraint for VRA compliance. The constraint conflict explanation is insightful and will influence how practitioners approach algorithmic redistricting. However, the paper needs stronger engagement with legal standards, geographic realism, and demographic specificity to be fully applicable to real-world redistricting.

With revisions addressing these concerns, I recommend publication in a venue that reaches both the algorithms community (for the constraint conflict contribution) and the redistricting community (for the VRA compliance guidance). Suitable venues: *Political Analysis*, *Election Law Journal*, SIAM conferences (SDM, ALENEX), or interdisciplinary venues like *Science* or *PNAS* if framed as applied algorithm work with policy impact.

The redistricting community needs better algorithmic tools for VRA compliance, and this paper provides important guidance. Make the legal and geographic analysis stronger, and this will be an important contribution.
