> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Quantifying the VRA-Compactness Tradeoff

**Reviewer**: Richard Pildes (NYU Law)
**Expertise**: Election Law, Constitutional Doctrine, Voting Rights Act
**Date**: 2026-02-08
**Round**: 1

---

## Overall Assessment

This paper addresses a question that has vexed courts and legislatures for three decades: whether compliance with Section 2 of the Voting Rights Act inherently requires sacrificing traditional redistricting principles like compactness. The Supreme Court's jurisprudence from *Thornburg v. Gingles* (1986) through *Shaw v. Reno* (1993) and *Miller v. Johnson* (1995) has wrestled with this tension, often treating VRA compliance and compactness as competing constitutional values requiring delicate balancing.

The paper's central empirical finding—that non-majority-minority districts generally *improve* in compactness when majority-minority districts are created—has profound implications for redistricting law. If true, this undermines the "necessary tradeoff" defense that legislatures use to justify non-compact MM districts and that courts reluctantly accept under *Shaw*. The Alabama case study, where VRA compliance actually *improves* overall compactness, is particularly striking.

However, from a legal perspective, the paper needs stronger engagement with constitutional doctrine, clearer definitions of what constitutes VRA "compliance," and more careful treatment of the relationship between algorithmic optimization and constitutionally permissible redistricting practices. The paper also overlooks critical legal distinctions between Section 2's requirements and Shaw's constraints.

**Score**: 3.0/4 (Accept with Major Revisions)

---

## Major Strengths

### 1. Direct Challenge to Judicial Assumptions
Courts have long operated under the assumption articulated in *Shaw v. Reno* that creating majority-minority districts "often" requires "bizarre" shapes that subordinate traditional redistricting principles. This paper provides quantitative evidence that this assumption is empirically unfounded in many contexts. The policy implications are substantial—courts should not defer to legislative claims that "VRA compliance forced us to draw non-compact districts" without demanding algorithmic evidence that no compact alternative exists.

### 2. Pareto Frontier as Legal Standard
The Pareto frontier concept translates naturally into legal doctrine. A redistricting plan that lies below the Pareto frontier—achieving neither maximal VRA compliance nor maximal compactness—could be challenged as failing to satisfy *both* Section 2 and state constitutional compactness requirements. This provides courts with an administrable standard for evaluating whether plans represent good-faith efforts to balance competing objectives or pretextual manipulations.

### 3. Geographic Feasibility Threshold
The identification of feasibility ratios (MM% / minority%) above which VRA targets become arithmetically infeasible has immediate application in *Gingles* litigation. The first *Gingles* precondition requires that the minority population be "sufficiently large and geographically compact to constitute a majority in a single-member district." South Carolina's ratio 1.22 demonstrates that "sufficiently large" has a quantifiable threshold—courts can use this framework to determine when Section 2 claims fail the first *Gingles* prong as a matter of mathematical impossibility.

### 4. Mechanistic Explanation
The three mechanisms (geographic clustering, non-MM boundary clarification, baseline suboptimality) provide a causal framework that could guide judicial review. Courts applying *Miller v. Johnson*'s "predominant factor" test ask whether race was the "predominant" consideration in drawing district lines. If compactness and VRA compliance are jointly optimizable through edge-weighting (as Alabama shows), then race is not "predominating" in a constitutionally problematic way—it's simply being incorporated as one factor among several.

---

## Major Issues (Must Address)

### M1: Constitutional Standards and "VRA Compliance"

**Issue**: The paper repeatedly claims that certain configurations achieve "VRA compliance" by creating the target number of majority-minority districts. However, Section 2 of the VRA does not *mandate* any specific number of MM districts—it prohibits vote dilution. The target numbers you use (Alabama: 2 MM, Georgia: 5 MM, etc.) appear to be derived from proportional representation goals or consent decree requirements, not directly from Section 2's text or *Gingles* preconditions.

This conflation creates legal confusion. Section 2 requires that districts provide minority voters with an equal *opportunity* to elect representatives of their choice—which could be satisfied by districts with 45-48% minority voting-age population (coalition districts) rather than 50%+ majority-minority districts. Your 50% threshold is defensible but not the only constitutionally permissible interpretation.

Moreover, *Bartlett v. Strickland* (2009) held that Section 2 does not require creation of "crossover districts" where minorities constitute less than 50% but potentially elect candidates with white crossover votes. Your paper doesn't engage with these doctrinal nuances.

**Recommendation**:
1. **Clarify VRA standard**: Add a Background subsection explaining that your analysis tests *one interpretation* of Section 2 compliance (proportional representation via 50%+ MM districts), not the only interpretation. Acknowledge that courts have recognized influence districts (30-40%), coalition districts (40-45%), and crossover districts as potential Section 2 remedies.

2. **Justify 50% threshold**: Explain why you focus on 50%+ MM districts specifically. Is this based on state-specific litigation history, consent decrees, or a policy judgment about effective representation? Be explicit about this choice's normative foundations.

3. **Alternative thresholds analysis**: Consider testing 40-45% thresholds ("coalition districts") to show whether feasibility improves. If South Carolina achieves its 3-district target at 45% minority thresholds, this has legal relevance for *Gingles* analysis.

**Severity**: P1 (Blocking) - Legal readers will question your entire framework if "VRA compliance" is not clearly defined.

---

### M2: Shaw v. Reno and Racial Predominance

**Issue**: The paper's policy recommendations (Section 5.4) state that "Courts should not accept that VRA compliance inherently requires compactness sacrifice without demanding algorithmic evidence." However, this oversimplifies the *Shaw* framework. *Shaw v. Reno* established that districts drawn "so bizarrely on their face that they are unexplainable on grounds other than race" may violate equal protection, even if intended to comply with Section 2.

The key *Shaw* question is not "Does VRA compliance require compactness sacrifice?" but rather "Did race predominate over traditional principles to such an extent that the plan is unexplainable except by reference to race?" *Miller v. Johnson* clarified that racial considerations can be *a* factor but not *the dominant* factor.

Your edge-weighted optimization—which assigns 5× or 10× weights to minority-minority edges—could be challenged under *Shaw*/*Miller* as making race the predominant factor. A legislature using your algorithm would need to defend that race was merely one factor among several (population balance, contiguity, compactness), not the predominant consideration that dictated district lines.

**Recommendation**:
1. **Add Shaw/Miller analysis**: In Discussion Section 5.4 (Policy Implications for Courts), add a subsection addressing how edge-weighted optimization interacts with *Shaw*/*Miller* doctrine. Argue that because edge-weighting produces compact districts that satisfy multiple traditional criteria simultaneously, it avoids the "race as sole factor" problem that *Shaw* prohibits.

2. **Defend multi-factor balancing**: Explain that edge-weighting implements a multi-objective optimization where race is *one* weighted factor, not the exclusive criterion. This distinguishes your approach from the North Carolina I-85 district, where compactness was entirely subordinated to racial considerations.

3. **Suggest doctrinal refinement**: Propose that courts evaluate redistricting plans based on Pareto efficiency rather than subjective "predominant factor" analysis. A plan on the Pareto frontier balances race and compactness optimally; a plan below the frontier suggests race was used pretextually (insufficient MM districts) or excessively (unnecessary compactness sacrifice).

**Severity**: P1 (Blocking) - Without addressing *Shaw*/*Miller*, your policy recommendations are incomplete.

---

### M3: Evenweil and Population vs Voting Power

**Issue**: Your paper optimizes on total population (Section 6.4 acknowledges this limitation), but Section 2 VRA litigation increasingly focuses on voting-age population (VAP) or citizen voting-age population (CVAP). The Supreme Court's *Evenweil v. Abbott* (2016) held that states *may* use total population for redistricting purposes, but the question of whether Section 2 *requires* considering VAP/CVAP for vote dilution analysis remains unsettled.

If your Alabama "2 MM districts" have 50%+ total population but only 42% CVAP (due to age, citizenship, registration gaps), they may not satisfy Section 2's requirement that minorities have an equal opportunity to *elect* representatives of their choice. Lower courts have split on this issue, and the Supreme Court has not definitively resolved it.

This matters legally because Section 2 plaintiffs must prove (under *Gingles* prong 3) that the white majority votes as a bloc to defeat the minority's preferred candidate. If minority *electoral* strength is substantially lower than minority *population* percentage, *Gingles* prong 3 may not be satisfied even if prong 1 (sufficiently large and geographically compact) is.

**Recommendation**:
1. **Calculate CVAP percentages**: Obtain Census Bureau CVAP data (available for 2020) and recalculate how many districts qualify as "majority-minority" under CVAP definitions. Report this alongside total population results.

2. **Acknowledge doctrinal uncertainty**: In Background Section 2, add a discussion of the population-base debate (*Evenweil*, *Garza v. County of Los Angeles*, recent circuit court decisions). Explain that your analysis uses total population but that alternative measures might yield different results.

3. **Provide sensitivity analysis**: If CVAP data reduces your MM district counts (e.g., Alabama 2 MM → 1.5 effective MM), discuss how this affects feasibility thresholds and Pareto frontiers. Does the win-win pattern persist under CVAP definitions?

**Severity**: P2 (Important) - Affects legal interpretation but doesn't invalidate core findings.

---

## Minor Issues

### m1: *Gingles* Precondition 1 ("Sufficiently Large and Geographically Compact")
You claim South Carolina's feasibility ratio 1.22 demonstrates when *Gingles* prong 1 fails (page 37). This is correct but deserves more explicit doctrinal framing. Courts applying *Gingles* ask: "Is the minority population sufficiently large and geographically compact?" Your analysis answers this quantitatively: when (MM% / minority%) > ~1.2, the answer is "no" as a mathematical matter. Make this connection explicit in Section 5.4.

### m2: State Constitutional Compactness Requirements
Your paper focuses on VRA-compactness tradeoffs but doesn't specify which study states have constitutional compactness mandates. Florida, for example, has explicit compactness requirements (Fair Districts Amendment), while other states have only statutory or aspirational compactness criteria. Specify in Section 3.2 which states have enforceable compactness requirements, as this affects the legal stakes of your findings.

### m3: One Person, One Vote and Population Deviation
You mention "strict population balance (±0.5%)" but don't cite constitutional authority. Congressional districts must meet *Karcher v. Daggett*'s near-perfect population equality standard (deviations justified only by good-faith efforts to achieve legitimate state objectives). Clarify that your ±0.5% threshold exceeds constitutional requirements (which permit essentially no deviation absent justification) but represents a reasonable algorithmic tolerance.

### m4: Multi-Member vs Single-Member Districts
Section 2 analysis historically focused on single-member districts (as required by federal law for congressional redistricting). However, some states use multi-member districts for state legislative redistricting. Clarify in Background Section 2 that your analysis applies to single-member congressional districts only, and that multi-member contexts present different Section 2 considerations.

### m5: Retrogression and Section 5 Preclearance
Your study states (AL, GA, LA, MS, SC) were all covered by Section 5 preclearance until *Shelby County v. Holder* (2013) struck down the coverage formula. Section 5's "retrogression" standard (plans cannot worsen minority representation) differs from Section 2's "dilution" standard. Clarify that your analysis evaluates Section 2 compliance, not Section 5 retrogression, since Section 5 preclearance no longer applies.

---

## Recommendations for Revision

### High Priority (Must Address)
1. **Define "VRA compliance" clearly** (M1) - Distinguish proportional representation from Section 2 requirements; justify 50% threshold; consider alternative thresholds.
2. **Address Shaw/Miller predominant factor test** (M2) - Explain how edge-weighted optimization avoids racial predominance concerns.
3. **Incorporate CVAP analysis** (M3) - Calculate voting-age population percentages for MM districts; provide sensitivity analysis.

### Medium Priority (Strengthen Paper)
4. **Explicit *Gingles* prong 1 connection** (m1) - Frame feasibility ratios as quantitative answers to "sufficiently large and geographically compact."
5. **State constitutional compactness specifications** (m2) - Clarify which states have enforceable compactness requirements.

### Low Priority (Polish)
6. **Population deviation constitutional standard** (m3) - Cite *Karcher v. Daggett* for equal population requirement.
7. **Single-member district scope** (m4) - Clarify analysis applies to congressional districts only.
8. **Section 2 vs Section 5 distinction** (m5) - Note that Section 5 preclearance no longer applies post-*Shelby County*.

---

## Detailed Comments by Section

### Introduction (Section 1)
- **Strength**: Framing around *Shaw v. Reno* and I-85 district effectively contextualizes the legal stakes.
- **Missing**: Should reference *Miller v. Johnson* and racial predominance test, as this is the current doctrinal framework courts apply.

### Background (Section 2)
- **Missing**: Needs subsection on VRA Section 2 doctrine—*Gingles* three preconditions, *Bartlett* crossover districts, *LULAC* coalition districts, *Evenweil* population-base debate. Currently treats VRA as background context rather than legal framework.
- **Suggestion**: Add 1-2 pages on VRA legal doctrine to ground your compliance metrics in constitutional standards.

### Methodology (Section 3)
- **Clarity needed**: What does "VRA compliance" mean operationally? Is it achieving target MM counts, or satisfying *Gingles* preconditions, or both? Define this explicitly.

### Results (Section 4)
- **Strength**: Alabama case study (Section 4.3) demonstrates that VRA compliance can *improve* compactness, which directly contradicts legislative "necessary tradeoff" defenses.
- **Legal application**: This result could be used by Section 2 plaintiffs to challenge plans that sacrifice compactness while claiming VRA necessity.

### Discussion (Section 5)
- **Missing**: Shaw/Miller analysis (M2). Section 5.4 discusses policy implications for courts but doesn't engage with equal protection doctrine governing racial redistricting.
- **Suggestion**: Add subsection "Constitutional Permissibility of Edge-Weighted Optimization" discussing how your approach satisfies *Shaw*/*Miller* multi-factor balancing.

### Conclusion (Section 8)
- **Strength**: "I-85 district represents algorithm failure, not inevitable cost" is a powerful reframing that courts could adopt.
- **Missing**: Should end with proposed legal standard: "Courts should evaluate redistricting plans based on Pareto efficiency—plans below the frontier are constitutionally suspect regardless of whether they claim VRA necessity or compactness priority."

---

## Legal Significance and Impact

This paper has the potential to reshape Section 2 litigation by providing courts with quantitative tools (Pareto frontiers, feasibility ratios) for evaluating redistricting plans. The finding that VRA-compactness tradeoffs are state-dependent rather than universal could influence how courts apply *Shaw*/*Miller* balancing tests and assess *Gingles* preconditions.

However, the paper's legal impact will be limited if it doesn't engage more deeply with constitutional doctrine. Addressing the *Shaw*/*Miller* predominant factor issue (M2) and clarifying what "VRA compliance" means legally (M1) are essential for law review publication or judicial citation.

With revisions, this could be a landmark paper cited in redistricting litigation for the next decade. Without them, it remains a valuable empirical study but misses its full legal potential.

---

## Final Recommendation

**Accept with Major Revisions**

The paper makes significant empirical contributions but requires:
1. Clearer definition of VRA compliance standards (M1)
2. Engagement with Shaw/Miller doctrine (M2)
3. CVAP analysis for legal applicability (M3)

With these revisions, the paper will be suitable for publication in APSR, election law symposia, or top law reviews.

**Estimated revision time**: 2-3 weeks for CVAP analysis and legal doctrine additions.
