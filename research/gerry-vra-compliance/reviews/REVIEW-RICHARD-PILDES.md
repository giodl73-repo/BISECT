# Review: Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning

**Reviewer**: Richard Pildes (NYU School of Law)
**Expertise**: Election law, constitutional doctrine, Voting Rights Act
**Date**: February 7, 2026

---

## Overall Assessment

This paper tackles a central tension in redistricting law: achieving VRA compliance while maintaining neutral, principled districting criteria. The edge-weighting methodology shows promise in creating demographically viable districts (80% success rate across 5 states), and the systematic empirical approach is commendable. However, the paper fundamentally misunderstands the legal framework governing VRA compliance, conflates constitutional requirements with statutory obligations, and ignores the critical interplay between VRA Section 2 and Equal Protection Clause constraints on racial redistricting.

Most critically, the paper operates as if VRA compliance is solely a matter of achieving demographic thresholds, when constitutional doctrine requires careful balancing between VRA obligations (statutory) and racial gerrymandering prohibitions (constitutional). The edge-weighting approach explicitly uses race to construct districts—this raises serious constitutional questions the paper never addresses.

**Score**: 2/4 (Major revisions required)

---

## Major Issues

### M1: Constitutional Constraints on Racial Redistricting Are Absent

The paper discusses VRA compliance without engaging with the **Equal Protection Clause limits** on race-conscious redistricting. This is a fundamental oversight.

**Key doctrine the paper ignores**:

**Shaw v. Reno (1993)**: Redistricting that is "unexplainable on grounds other than race" triggers strict scrutiny. Districts cannot be "so bizarre" that they constitute racial gerrymandering.

**Miller v. Johnson (1995)**: Race cannot be the "predominant factor" in redistricting unless the plan is narrowly tailored to a compelling government interest (VRA compliance).

**Cooper v. Harris (2017)**: Even when pursuing VRA compliance, race can only predominate if there is "strong basis in evidence" that VRA compliance requires race-conscious districting.

**Critical question the paper never asks**: Does edge-weighting make race the "predominant factor"?

**Edge-weighting explicitly considers race**:
- Identifies minority tracts (race-conscious classification)
- Assigns high weights to edges between minority tracts (race-based decision)
- Optimizes to keep minority tracts together (race-conscious objective)

This is facially race-based redistricting. Under *Miller* and *Cooper*, this triggers strict scrutiny:
1. **Compelling interest**: Is there strong basis in evidence that VRA requires this level of race-consciousness?
2. **Narrow tailoring**: Is edge-weighting the least race-conscious means of achieving VRA compliance?

The paper addresses neither question.

**Example**: Alabama edge-weighted plan achieves 2 MM districts at 50.8% minority using weight factor 5x-10x (p.8). But:
- Is there evidence that 2 MM districts are required by VRA? (*Allen v. Milligan* says yes, but paper doesn't cite it)
- Could the same result be achieved with less race-consciousness (lower weight factors, or race-neutral criteria like community preservation)?
- Are the district shapes "bizarre" under *Shaw*?

**Recommendation**: Add Section 2.X "Constitutional Constraints" discussing:
- *Shaw*/*Miller*/*Cooper* strict scrutiny framework
- Whether edge-weighting makes race "predominant factor"
- "Strong basis in evidence" requirement for race-conscious districting
- Narrow tailoring analysis—is edge-weighting minimally race-conscious?
- Compliance with both VRA (statutory) and Equal Protection (constitutional)

### M2: VRA Section 2 vs. Section 5 Confusion

The paper discusses "VRA-covered jurisdictions" (p.2) and "covered states" (p.4), suggesting Section 5 preclearance framework, but Section 5 was effectively ended by *Shelby County v. Holder* (2013). All current VRA redistricting cases are Section 2 "results test" cases, not Section 5 preclearance.

**This matters because**:
- Section 5: Required preclearance for any voting change, prevented retrogression
- Section 2: Requires litigation showing vote dilution under totality of circumstances

The legal framework is completely different. Post-*Shelby*, jurisdictions don't need to prove VRA compliance in advance—plaintiffs must prove VRA violations after enactment.

**Implication**: The paper's framework (design algorithmic plans that achieve VRA compliance) is solving the wrong problem. The legal question is not "does this plan achieve VRA compliance?" but rather "can plaintiffs prove this plan violates Section 2?"

These are different standards:
- VRA compliance (paper's framing): Affirmative obligation to maximize minority representation
- Section 2 defense (actual legal standard): Plan doesn't dilute minority voting strength compared to reasonable alternative

**Recommendation**: Reframe entire paper:
- Drop "covered states" language (implies Section 5)
- Clarify that post-*Shelby*, VRA analysis is Section 2 vote dilution claims
- Discuss "reasonably available alternative" standard from *Thornburg* footnote 15
- Frame edge-weighting as demonstrating "no Section 2 violation" rather than "achieving VRA compliance"

### M3: *Allen v. Milligan* (2023) is Directly Relevant but Not Cited

The paper analyzes Alabama redistricting but ignores the most important recent case on VRA redistricting—decided **about Alabama specifically**:

**Allen v. Milligan (2023)**:
- Alabama's 2021 enacted plan: 1 MM district (District 7)
- Section 2 challenge: Plaintiffs argued 2 MM districts required
- District Court: Found Section 2 violation, ordered remedial plan with 2 MM districts
- SCOTUS (6-3): Affirmed, Alabama must create 2 MM districts

This is **directly what your paper claims to do**—create 2 MM Alabama districts algorithmically.

**Questions paper must address**:
1. How does your edge-weighted plan compare to the *Allen* remedial plan?
2. Does your plan use the same geographic areas (e.g., Birmingham + Mobile) or different configuration?
3. What were the minority percentages in the *Allen* remedial plan vs. your 50.8%/50.1%?
4. Did *Allen* litigants use algorithmic methods? If so, which ones? How do they compare to edge-weighting?

**Legal significance**: *Allen* establishes that Alabama **must** create 2 MM districts under Section 2. Your algorithmic approach could provide the neutral, principled mechanism for doing so—but only if you engage with the actual litigation.

**Recommendation**: Add "Alabama Case Study" section:
- Summarize *Allen v. Milligan* litigation history
- Compare edge-weighted plan to enacted and remedial plans
- Discuss whether algorithmic approach would have avoided litigation
- Analyze compactness, traditional criteria compliance of algorithmic vs. court-ordered plans

### M4: Racial Gerrymandering vs. VRA Compliance Line is Unclear

The paper presents edge-weighting as "principled" VRA compliance, but constitutional doctrine recognizes a thin line between permissible VRA compliance and impermissible racial gerrymandering.

**The constitutional tension**:
- **Too few minority districts**: Section 2 violation (vote dilution)
- **Too many or too concentrated**: Equal Protection violation (racial gerrymandering)

**Cases illustrating the line**:
- *Bush v. Vera* (1996): Texas plan using race as predominant factor for MM districts struck down, despite state's VRA compliance defense
- *Bethune-Hill v. Virginia State Bd. of Elections* (2017): 55% racial target unconstitutional absent sufficient justification
- *Cooper v. Harris* (2017): North Carolina MM district struck down for unnecessarily high minority concentration (packing)

**Problem with edge-weighting**: The weight factor α is arbitrary (5x, 10x, 50x, 100x). Higher factors create higher minority percentages:
- Alabama 5x @ 40%: 50.8% minority (2 MM districts)
- Alabama 50x @ 40%: 51.8% minority (2 MM districts)
- Alabama 100x @ 45%: 51.4% minority (2 MM districts)

Which weight factor is constitutionally permissible? At what point does weight factor become "unnecessary" concentration that constitutes racial packing?

**Legal standard from *Cooper***: Race-conscious districting must be "narrowly tailored"—enough to achieve VRA compliance, but not more. Using α=100x to achieve 51.8% when α=5x achieves 50.8% might be unconstitutional over-concentration.

**Recommendation**: Analyze narrow tailoring:
- What is minimum weight factor achieving VRA compliance per state?
- Are higher weight factors constitutionally suspect (unnecessary concentration)?
- How to choose α that satisfies VRA without violating Equal Protection?
- Compare minority percentages across weight factors—is there constitutional "sweet spot"?

---

## Minor Issues

### m1: "One Person, One Vote" is Assumed but Not Verified

The paper assumes METIS population balancing satisfies one-person-one-vote (OPOV), but:

**Constitutional standard**: Total population equality within ±0.5% (technically 0, but courts allow small deviations for state legislative districts; zero deviation required for congressional districts).

**Questions**:
- What is actual population deviation in edge-weighted plans?
- Does METIS guarantee exact equality or merely approximate balance?
- Are congressional districts (zero deviation) treated differently from state legislative?

This is a threshold constitutional requirement that paper should verify explicitly.

### m2: Traditional Redistricting Principles Ranking Unclear

State constitutions and case law establish hierarchies of redistricting principles:
1. OPOV (constitutional requirement)
2. VRA compliance (statutory requirement)
3. Contiguity (universal requirement)
4. Compactness (strong preference)
5. County/city preservation (varies by state)
6. Communities of interest (emerging principle)
7. Core preservation (continuity with prior plans)

The paper optimizes edge-cut (compactness proxy) subject to population balance, but ignores #5-7. In litigation, courts weigh all principles.

**Example**: If edge-weighted plan splits counties unnecessarily compared to enacted plan, court might prefer enacted plan even if edge-weighted has better compactness.

**Recommendation**: Analyze full hierarchy:
- County/city split counts for algorithmic vs enacted plans
- Communities of interest (requires identifying COIs—use MGGG CoI platform data?)
- Core retention from prior districts

### m3: Minority Definition Varies by Jurisdiction

The paper treats "minority" as monolithic, but VRA analysis differs by minority group:

**Black populations**: Generally concentrated (urban), enabling compact MM districts
**Hispanic populations**: More dispersed, sometimes requiring coalition districts
**Asian populations**: Highly concentrated (specific neighborhoods) but small percentages
**Native American populations**: Reservation-based geography

The paper's 5 test states are primarily Black-minority, limiting generalizability. Does edge-weighting work for Hispanic-majority districts (Texas, California, New Mexico)? Coalition districts (Black + Hispanic)?

**Recommendation**: Caveat that findings apply to Black-majority districts; test edge-weighting on Hispanic-majority and coalition district scenarios.

### m4: Retrogression Analysis Missing

Even though Section 5 is defunct post-*Shelby*, courts still consider retrogression as part of Section 2 totality analysis: does new plan reduce minority electoral opportunity compared to prior plan?

**Example**: Alabama 2021 plan had 1 MM district. Your plan has 2 MM districts—clear non-retrogression. But:
- What were minority percentages in prior (2010) Alabama plan?
- Does edge-weighting maintain or increase opportunity compared to benchmark?

This is part of "totality of circumstances" analysis courts conduct.

### m5: Statewide vs. Jurisdiction-Specific Analysis

VRA Section 2 applies to any "voting qualification or prerequisite to voting or standard, practice, or procedure"—this includes **local jurisdictions** (county commissions, city councils, school boards), not just congressional districts.

The paper analyzes only congressional redistricting. Does edge-weighting work for:
- State legislative districts (100+ districts per state)
- County commission districts (5-7 districts)
- City council districts (varies widely)

Different district counts may affect edge-weighting performance.

---

## Methodological Concerns

### MC1: No Legal Expert Validation

The paper creates redistricting plans but never seeks validation from:
- Election law attorneys (would these plans survive litigation?)
- VRA expert witnesses (typical threshold, are these plans defensible?)
- Special masters (judicial redistricting experts)

For a paper claiming to achieve "VRA compliance," lack of legal expert validation is concerning.

**Recommendation**: Seek expert review from practitioners who litigate VRA cases, or special masters who draw remedial plans.

### MC2: Partisan Effects Could Invalidate VRA Defense

*LULAC v. Perry* (2006) established that VRA cannot be used as pretext for partisan gerrymandering. If edge-weighting creates MM districts that also pack Democratic voters to benefit Republicans, courts may reject VRA defense.

The paper provides zero partisan analysis—yet partisan effects could make legally compliant plans politically infeasible or constitutionally suspect.

**Recommendation**: Add partisan analysis showing edge-weighted plans don't constitute partisan gerrymandering disguised as VRA compliance.

### MC3: Feasibility Thresholds May Not Generalize

The paper identifies "~36% threshold" for edge-weighting success (p.15), but this is based on:
- 5 Southern states (specific geography, specific minority distribution patterns)
- Black-minority populations (different from Hispanic/Asian/Native)
- Congressional districts (6-14 districts per state)

Would the 36% threshold hold for:
- Western states (different geography)?
- Hispanic-majority districts (different settlement patterns)?
- State legislative redistricting (different district counts)?

The threshold may be specific to tested conditions, not universal.

---

## Recommendations for Revision

**Critical (required for publication)**:
1. Add constitutional analysis: *Shaw*/*Miller*/*Cooper* strict scrutiny for race-conscious redistricting
2. Engage with *Allen v. Milligan*—directly relevant Alabama case
3. Clarify Section 2 "results test" framework (not Section 5 preclearance)
4. Analyze whether edge-weighting makes race "predominant factor" and narrow tailoring

**Strongly recommended**:
5. Compare edge-weighted plans to enacted plans and litigation remedies
6. Analyze partisan effects (pretext concern from *LULAC*)
7. Verify OPOV compliance explicitly
8. Discuss full set of traditional principles (county splits, COI, core retention)

**Minor improvements**:
9. Caveat findings to Black-majority districts in Southern states
10. Add retrogression analysis (comparison to prior plans)
11. Discuss generalizability to state legislative and local redistricting

---

## Significance

From a constitutional law perspective, this paper's primary contribution is demonstrating **algorithmic feasibility** of creating demographically viable districts that could satisfy Gingles prong 1. This is useful for:

1. **Refuting infeasibility defenses**: Jurisdictions can no longer claim "it's impossible to draw compact MM districts" when algorithms can do so

2. **Providing neutral alternatives**: Algorithmic plans offer transparent baseline for evaluating politically-drawn maps

3. **Narrowing doctrine**: If edge-weighting reduces feasibility threshold from 42% to 36%, this narrows the set of jurisdictions where VRA-compactness tradeoffs are genuinely unavoidable

However, these contributions require establishing that edge-weighted plans are not just demographically plausible but **legally defensible**:
- Satisfy constitutional constraints (race not predominant factor, narrowly tailored)
- Meet statutory requirements (full Gingles test, totality of circumstances)
- Respect traditional principles (county preservation, COI, non-partisan)

Without this legal analysis, the paper is a technical exercise with uncertain legal relevance.

**Recommended venue** (after revisions): *Election Law Journal*, *Yale Law Journal* (symposium), or *Harvard Law Review Forum* (shorter format for quick-turnaround policy piece)

For premier law review publication (*HLR*, *YLJ*, *CLR* main issues), would need:
- Deeper doctrinal analysis (constitutional vs. statutory requirements)
- Comparison to actual litigation (enacted plans, remedial plans, expert testimony)
- Expert validation from VRA practitioners
- Broader implications for redistricting reform and commission design

---

**Bottom Line**: Valuable empirical demonstration that algorithmic methods can create demographically viable districts, but fundamentally incomplete legal analysis. Paper treats VRA compliance as technical optimization problem when it requires balancing statutory obligations (Section 2), constitutional constraints (Equal Protection), and traditional principles (compactness, county preservation). Major revisions needed to engage with constitutional doctrine, recent case law (*Allen v. Milligan*, *Cooper*), and partisan fairness concerns before suitable for law review publication. With revisions, could make important contribution to redistricting law and practice—particularly in demonstrating feasibility of neutral, race-conscious districting that satisfies both VRA and Equal Protection.
