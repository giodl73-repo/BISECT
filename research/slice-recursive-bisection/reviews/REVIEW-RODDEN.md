# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Jonathan Rodden (Stanford University)
**Expertise**: Political geography, gerrymandering, representation
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

This paper makes an important contribution to the redistricting reform debate by extending the Huntington-Hill apportionment precedent to boundary design. The "impossibility defense"—that algorithms cannot gerrymander because they cannot see partisan data—is a novel and powerful framing that sidesteps the intent-based arguments that failed in *Rucho*. The paper's emphasis on process fairness over outcome fairness is philosophically sophisticated and appropriate given the constraints of political geography.

The integration of my work with Chen on "unintentional gerrymandering" is excellent and accurate. The authors correctly recognize that geographic sorting—not algorithmic bias—explains the Democratic efficiency gap in district counts. This understanding is crucial and often missing from algorithmic redistricting proposals.

However, the paper needs strengthening in several areas before it's ready for APSR publication: the empirical analysis of geographic sorting could be deeper, the discussion of communities of interest is underdeveloped, and the policy implications need more engagement with real-world political constraints.

## Score: 3.0/4.0

**Strong Accept with Revisions**

The paper's core contribution—extending Huntington-Hill principles to redistricting with structural manipulation-resistance—is significant and novel enough for APSR. The philosophical framing is sophisticated. The impossibility defense offers a genuinely new approach to the gerrymandering problem. With revisions addressing the issues below, this would be an excellent contribution to the literature.

## Major Issues (Must Address)

### M1. Insufficient Empirical Analysis of Geographic Sorting Mechanisms

**Issue**: While the paper correctly identifies geographic sorting as the source of Democratic efficiency gaps (Section 5), it doesn't provide sufficient empirical analysis of *how* and *why* this sorting occurs in the algorithmic districts versus enacted districts.

**Why this matters**: For the impossibility defense to be convincing, readers need to see clear evidence that the partisan patterns arise from geography + compactness optimization, not hidden algorithmic properties that correlate with partisanship.

**Specific gaps**:
- No state-by-state comparison of efficiency gaps (algorithmic vs. enacted)
- Missing analysis of urban density thresholds that trigger Democratic concentration
- No examination of which specific geographic features (water barriers, mountain ranges, urban cores) drive district boundaries
- Lack of counterfactual analysis: what if you varied the compactness weight?

**Recommendation**: Add a subsection (5.4 or 5.5) with:
1. State-level efficiency gap table comparing algorithmic vs. enacted plans
2. Scatter plot: urban density (% population in urban areas) vs. Democratic district advantage
3. Case study: Pick 2-3 states showing how specific geographic features (e.g., Chicago's urban core, Miami's coastal concentration) drive partisan outcomes
4. Sensitivity analysis: How do results change if you minimize perimeter (edge-weighted) instead of edge counts?

This would strengthen the claim that outcomes reflect geography, not algorithm design.

### M2. Communities of Interest Underdeveloped

**Issue**: Section 6.1 mentions "communities of interest" as a traditional criterion the algorithm doesn't optimize, but dismisses it too quickly without engaging with the substantial political science literature on what communities of interest actually are and why they matter for representation.

**Why this matters**: APSR readers will be skeptical of purely geometric approaches that ignore social, economic, and political communities. If the paper argues for prioritizing process fairness, it needs to seriously engage with the outcome fairness critique that algorithmic maps might split coherent communities.

**Specific problems**:
- No definition of "communities of interest" beyond generic mention
- No empirical analysis of whether algorithmic districts do or don't preserve counties, municipalities, or other traditional boundaries
- Missing engagement with literature on descriptive vs. substantive representation
- No discussion of whether recursive bisection's hierarchical structure actually *helps* preserve regional communities (north/south, urban/rural splits often align with social geography)

**Recommendation**: Add subsection 6.2.1 "Communities of Interest and Hierarchical Structure" that:
1. Defines communities of interest (county boundaries, municipalities, media markets, economic regions)
2. Empirically analyzes how often algorithmic districts split these boundaries vs. enacted plans
3. Discusses whether recursive bisection's hierarchical nature (state → regions → districts) actually aligns well with nested community structures
4. Engages with the tension: communities of interest can be invoked strategically to justify gerrymandering (see NC 12th district claiming "community" of Black voters across 100+ miles)

This would show you've seriously considered the critique rather than dismissing it.

### M3. Policy Adoption Barriers Underanalyzed

**Issue**: Section 6.3 discusses adoption pathways (ballot initiatives, legislative adoption, judicial remedies) but doesn't adequately address the political economy obstacles to adoption. Why would politicians voluntarily cede redistricting control? What coalition would support this?

**Why this matters**: APSR readers expect serious engagement with political feasibility, not just normative arguments. If the paper claims algorithmic redistricting is a "viable path" (abstract), it needs to explain the viable coalition politics.

**Specific gaps**:
- No discussion of which political actors benefit from current system and would resist reform
- Missing analysis of partisan asymmetry: Democrats clustered in cities face efficiency gaps under ANY compactness-optimizing method, so why would they support this?
- No engagement with the literature on why reform is rare (Issacharoff, Pildes, Persily on "lockup" effects)
- Insufficient discussion of how to build bipartisan coalitions when algorithm produces systematically unequal outcomes

**Recommendation**: Expand 6.3.2 "Coalition Politics and Partisan Asymmetry" to address:
1. Why would Democratic-controlled states adopt algorithms that produce efficiency gaps favoring Republicans?
2. Why would Republican-controlled states adopt algorithms that reduce their gerrymandering advantages?
3. What circumstances enable reform? (Divided government? Court orders? Ballot initiatives in states with weak parties?)
4. Case studies: Iowa succeeded (why?), California commission succeeded (why?), other attempts failed (why?)

This would demonstrate you understand the political constraints, not just the normative ideals.

## Minor Issues (Should Address)

### m1. VRA Discussion Too Brief (Section 5.6)

The Voting Rights Act discussion (Section 5.6) is only 3 paragraphs but raises a fundamental constitutional issue. Expansion needed:
- More detail on Section 2 requirements and how they conflict with pure compactness
- Discussion of recent Supreme Court VRA cases (*Brnovich*, *Allen v. Milligan*)
- Specific technical approach for VRA-constrained optimization (cite Duchin's work more extensively)
- Acknowledge tension: VRA requires seeing race; impossibility defense requires not seeing partisanship; but race correlates with partisanship

**Suggested addition**: 1-2 pages expanding 5.6 into comprehensive discussion of how algorithms can incorporate VRA constraints while maintaining impossibility defense for partisan manipulation.

### m2. Competitive Districts Discussion Incomplete (Section 5.3)

Section 5.3 correctly notes that geographic polarization limits competitiveness regardless of method. But it doesn't engage with the normative question: *should* we want competitive districts?

Literature debate:
- Pro-competitiveness: Electoral accountability requires competitive districts (Issacharoff, Karlan)
- Anti-competitiveness: Forcing competitiveness requires bizarre boundaries that split communities (Persily)
- Alternative view: Competitiveness in primaries matters more than general elections (McGhee)

**Recommendation**: Add 2-3 paragraphs engaging with this debate and explaining your position.

### m3. Edge-Weighted Optimization Teased But Not Explored (Section 6.4)

You mention edge-weighted optimization could achieve 50-60% compactness improvements (Section 6.4.1) but don't explain:
- How edge weights work (edge weight = boundary length)
- Why this improves compactness (minimizes perimeter, not just edge count)
- Whether this changes partisan outcomes
- Why you didn't use it in the main analysis

**Recommendation**: Either provide more detail (making it a substantive future work proposal) or remove the specific percentage claim (which feels like cherry-picking without justification).

### m4. Random Seed Non-Determinism Underexplained (Section 3.5)

You mention not setting random seeds (Section 3.5), causing <1% variation across runs. But:
- No empirical data showing this variation (how many runs? what's the distribution?)
- No discussion of whether this variation is feature or bug
- No engagement with ensemble methods literature that views variation as information-rich

**Recommendation**: Either fix seeds for full reproducibility or embrace variation with ensemble analysis showing boundary robustness.

## Strengths (Preserve These)

1. **Impossibility defense framing**: Novel, powerful, legally sophisticated. This is the paper's biggest contribution.

2. **Huntington-Hill precedent**: Excellent historical analogy. The six principles (objectivity, transparency, uniformity, etc.) are clearly articulated and persuasively applied to redistricting.

3. **Honest about geographic sorting**: Unlike many algorithmic redistricting papers that oversell neutrality, you correctly recognize efficiency gaps as geographic phenomena, not algorithmic bias.

4. **Process vs. outcome fairness**: Sophisticated philosophical stance appropriate for political geography realities.

5. **Technical quality**: Algorithm description is clear, implementation sounds solid, results are credible.

## Minor Suggestions

- **Abstract**: Too long (currently ~220 words). APSR prefers 150 words. Cut the methodology details.
- **Figure quality**: Check if Minnesota/Alabama figures (4.1, 4.2) are high enough resolution for print.
- **Citation updates**: Several references are working papers that may have been published (check Chen/Rodden 2013, Duchin 2018).
- **Terminology consistency**: You switch between "census tracts" and "tracts"—pick one.
- **County bridging explanation** (Section 3.2): Excellent innovation but could use a figure showing example bridges (e.g., Hawaii, Michigan islands).

## Recommendation

**Revise and Resubmit** (after addressing major issues)

This paper has significant potential for APSR publication. The impossibility defense is a genuinely novel contribution to redistricting reform debates. The Huntington-Hill framing is sophisticated and compelling. The technical work is solid.

However, the empirical analysis of geographic sorting needs deepening (M1), the communities of interest discussion needs strengthening (M2), and the political feasibility analysis needs honest engagement with adoption barriers (M3).

With these revisions, this would be an important contribution that advances both the algorithmic redistricting literature and broader debates about mathematical governance of political processes.

The paper's greatest strength is recognizing that procedural legitimacy matters more than outcome optimality—a lesson too often forgotten in reform debates. This insight, combined with technical innovation and philosophical sophistication, makes it APSR-worthy with revisions.

---

**Word count impact**: Addressing major issues will add ~2,500-3,000 words. Current draft (~17,600 words) would reach ~20,000 words. Consider cutting some methodology detail (Section 3) or results tables (Section 4) to stay within APSR's 12,000-15,000 word range. The content matters more than technical minutiae for this audience.
