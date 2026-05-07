# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Jowei Chen (University of Michigan)
**Expertise**: Automated redistricting, compactness, neutrality
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

This paper presents a clean, implementable algorithmic approach to redistricting with a novel "impossibility defense" against gerrymandering. The recursive bisection method is technically sound, computationally efficient, and produces reasonable districts. The philosophical framing around Huntington-Hill is creative and effective.

As someone who has spent years defending automated redistricting against skeptics, I appreciate the paper's honest acknowledgment of limitations—particularly that algorithms cannot overcome geographic polarization and that compactness itself has partisan consequences. This intellectual honesty strengthens rather than weakens the argument.

However, the paper needs significant strengthening in three areas: (1) comparison to existing automated methods (especially my simulation-based approaches), (2) empirical demonstration that results are truly insensitive to reasonable parameter variations, and (3) better handling of the compactness gap with enacted districts.

## Score: 3.0/4.0

**Accept with Major Revisions**

The core contribution is valuable: extending Huntington-Hill precedent to redistricting with structural manipulation-resistance. The impossibility defense sidesteps intent-based arguments effectively. With substantive revisions addressing comparison methodology and parameter sensitivity, this merits publication in a top venue.

## Major Issues (Must Address)

### M1. Insufficient Comparison to Simulation-Based Methods

**Issue**: Section 6.2 discusses alternative algorithmic approaches (MCMC, genetic algorithms, integer programming, shortest splitline) but doesn't adequately compare to simulation-based ensemble methods—the current gold standard for demonstrating neutrality.

**Why this matters**: My work with Rodden and others uses automated simulation to generate thousands of neutral plans, then shows whether enacted plans are statistical outliers. This approach has been accepted in court (*Gill v. Whitford*, *Rucho* dissent, *League of Women Voters v. Pennsylvania*). You can't claim algorithmic superiority without engaging seriously with this established method.

**Specific problems**:
- Only 1 paragraph on MCMC (Section 6.2), dismissing it as "diagnostic not prescriptive"
- No acknowledgment that ensemble methods provide statistical evidence your single plan is not itself a manipulation-favoring outlier
- Missing comparison: Is your Alabama map within the distribution of 10,000 simulated Alabama maps? Or is it an outlier?
- No discussion of how recursive bisection could be used within ensemble framework (varying random seeds, parameters)

**Recommendation**: Add substantial subsection 6.2.1 "Comparison to Ensemble Simulation Methods" that:

1. **Explains ensemble approach**: Generate thousands of plans satisfying traditional criteria (contiguity, compactness, population balance), analyze distribution of partisan outcomes
2. **Acknowledges strengths**: Ensemble methods provide statistical evidence for outlier detection and have legal precedent
3. **Empirically compares**: For 3-5 states, generate 1,000 recursive bisection plans (varying random seeds), plot partisan outcome distribution, show your single plan is near ensemble median
4. **Discusses trade-offs**:
   - Ensembles: statistical power but require selecting from millions of plans (how?)
   - Single algorithms: clear selection rule but lack statistical uncertainty quantification
5. **Proposes hybrid**: Your algorithm generates single "default" plan, but sensitivity analysis (varying parameters) provides uncertainty bounds

This would position your work as complementary to ensemble methods rather than competitor, which is more accurate and defensible.

### M2. Parameter Sensitivity Insufficiently Demonstrated

**Issue**: You claim structural immunity to manipulation because the algorithm can't see partisan data. But the algorithm DOES see parameters—and parameter choices might correlate with partisan outcomes even without intent.

**Specific concerns**:
- **Balance tolerance** (±0.5%): What if you used ±1% or ±0.1%? Does this change partisan outcomes?
- **Refinement iterations** (niter=100): Why 100? What happens with 10 or 1,000?
- **Adjacency definition** (queen vs. rook contiguity): Does this matter?
- **County bridging distance threshold**: Currently nearest-within-county. What if you used nearest-overall or 10km threshold?
- **Random seed**: You mention <1% variation but provide no empirical data

**Why this matters**: If small parameter changes produce large partisan swings, critics will argue you cherry-picked parameters to favor outcomes. The impossibility defense only works if results are robust across reasonable parameter ranges.

**Recommendation**: Add entire section 4.5 "Parameter Sensitivity Analysis" with:

1. **Parameter space definition**: List all parameters with reasonable ranges
2. **Systematic variation**: For 5-10 representative states, vary each parameter individually while holding others fixed
3. **Outcome metrics**: Track population deviation, compactness, partisan lean, efficiency gap
4. **Results presentation**:
   - Table showing outcome variation across parameter ranges
   - Finding: "Partisan outcomes vary by <3% across all reasonable parameter choices"
   - Or if large variation: honest discussion of which parameters matter and why default choices are defensible
5. **Random seed ensemble**: Show distribution across 100 runs with different seeds

This empirical demonstration is CRITICAL for the impossibility defense. Without it, skeptics will say "you can't gerrymander intentionally but you can achieve outcomes through parameter tuning."

### M3. Compactness Gap with Enacted Districts Needs Better Handling

**Issue**: Your algorithmic districts are LESS compact than enacted 2020 districts (0.220 vs. 0.305 Polsby-Popper). Section 4.3 acknowledges this but the explanation feels defensive rather than constructive.

**Problems with current explanation**:
- "Block vs. tract granularity" is valid but not the full story (blocks would help by ~10-15%, not 35% gap)
- "Manual optimization" is vague—what specifically do humans optimize that your algorithm doesn't?
- "Redistricting reforms" implies enacted plans are now good—but many 2020 plans were gerrymandered (TX, FL, OH)
- Missing key question: Are enacted plans MORE compact because of better methods, or because they violate other criteria you respect?

**Why this matters**: Readers will ask "if your algorithm is worse on compactness, why use it?" You need a more convincing answer than "future work will fix this."

**Recommendation**: Rewrite Section 4.3 with deeper analysis:

1. **Separate analysis by state redistricting process**:
   - Commission states (MI, CO, CA, AZ): Compare your compactness to theirs
   - Court-drawn states (PA, NC): Compare to court maps
   - Legislative states (TX, FL, OH): Compare to gerrymandered maps
   - **Hypothesis**: Your algorithm beats gerrymandered states, matches/loses to commission states

2. **Analyze trade-offs**: Do high-compactness enacted plans violate criteria you respect?
   - Check if high-compactness enacted districts have worse population balance
   - Check if they split more counties
   - Check if they ignore water barriers (forcing impossible districts)

3. **Case studies**: Pick 2 states where enacted beats you significantly (MI, IN)
   - Map both plans side-by-side
   - Explain specifically what geometric features the enacted plan achieves
   - Discuss whether those features could be incorporated algorithmically

4. **Edge-weighted optimization results**: If edge-weighted bisection achieves 50-60% improvement, show at least one state example NOW, not just mention as future work

This would transform weakness into nuanced analysis of trade-offs, which is more persuasive than defensive explanations.

## Minor Issues (Should Address)

### m1. Hierarchical Structure Advantage Underexplored

**Strength not emphasized**: Recursive bisection creates hierarchical district structure (state → regions → districts) that mirrors political geography. Many states DO have natural regional divisions (Northern CA vs. Southern CA, Upstate NY vs. NYC, North FL vs. South FL).

**Missed opportunity**: You could argue hierarchical structure is a FEATURE that matches nested community structures, unlike flat k-way partitioning which treats all districts symmetrically.

**Recommendation**: Add subsection 3.6 "Hierarchical Structure and Regional Geography" explaining:
- How recursive bisection creates implicit regional structure
- Whether these regions align with actual political/economic/geographic regions
- Case study showing Minnesota's 2-4-8 structure makes geographic sense (Twin Cities vs. Greater Minnesota split at round 1)

This would strengthen the methodological contribution.

### m2. Population Equality Legal Standards Unclear

**Section 4.1 claims**: "86% within ±5% compares favorably to enacted plans"
**Legal reality**: Supreme Court requires congressional districts "as nearly equal as practicable" (Karcher). In practice this means <1% deviation for congressional districts (unlike state legislative districts which allow more).

**Problem**: Your 86% within ±5% may not meet constitutional standards. You mention block-level would achieve ±0.1%, but why not implement it?

**Recommendation**:
- Add citations to recent redistricting cases showing accepted deviation ranges
- Either: implement block-level to meet standards, or explain why tract-level is sufficient given legal uncertainty
- Discuss whether "as nearly equal as practicable" allows algorithmic methods with inherent granularity constraints

### m3. Contiguity Guarantee Overstated

**Section 3.2 county bridging**: You add edges for islands/water barriers. But:
- Do these bridges represent LEGAL contiguity? (Can voters cross water to reach rest of district?)
- Or just GRAPH contiguity? (Artificial edges for algorithm to work)

**Example**: If Hawaiian island district requires 50km water crossing, is that meaningfully contiguous?

**Recommendation**: Distinguish graph-contiguous (algorithm requirement) from legally-contiguous (may require ferry service, bridges). Acknowledge some districts may be multi-component with water separation.

### m4. Election Data Limitations Underexplored

**Section 5 uses 2020 presidential results** but:
- Congressional races differ from presidential (incumbency, candidate quality, national tides)
- 2020 was unusual year (COVID, Trump)
- Using single election introduces temporal bias

**Recommendation**: If possible, analyze multiple elections (2016, 2020, 2022) to show partisan patterns robust across years. If not possible, discuss limitation more explicitly.

## Strengths (Preserve These)

1. **Impossibility defense**: This framing is excellent and novel. It sidesteps Rucho's intent-based problems elegantly.

2. **Computational efficiency**: 2-3 hours for 50 states is impressive and enables iteration/sensitivity analysis.

3. **Honest about geographic sorting**: You correctly identify that efficiency gaps reflect geography, not algorithm—this honesty strengthens credibility.

4. **Clean algorithm**: Recursive bisection is simple, understandable, and reproducible—important for public acceptance.

5. **Huntington-Hill precedent**: Creative historical analogy that provides legitimacy.

## Recommendations for Revision Priority

**High priority** (must address):
1. M2 - Parameter sensitivity analysis (add Section 4.5)
2. M1 - Serious comparison to ensemble methods (expand Section 6.2)
3. M3 - Better compactness gap analysis (rewrite Section 4.3)

**Medium priority** (should address):
4. m1 - Hierarchical structure advantage (add Section 3.6)
5. m2 - Population equality legal standards (clarify Section 4.1)

**Low priority** (nice to have):
6. m3 - Contiguity distinction (clarify Section 3.2)
7. m4 - Election data limitations (expand Section 5 intro)

## Overall Recommendation

**Major Revisions Required, Then Accept**

This paper makes a valuable contribution to automated redistricting literature. The impossibility defense is novel and powerful. The Huntington-Hill framing provides legitimacy. The technical work is generally solid.

However, three critical gaps must be addressed:
1. Serious engagement with ensemble simulation methods (current gold standard)
2. Empirical demonstration of parameter insensitivity (essential for impossibility defense)
3. Better analysis of why enacted districts are more compact (not just future work promises)

With these revisions, this would be a strong publication. Without them, reviewers will find the impossibility defense unconvincing and the compactness gap troubling.

---

**Personal note**: I've spent years defending automated redistricting in courts and academic venues. Your approach is sound and your philosophical framing is excellent. But the devil is in the empirical details. Skeptics will focus on (1) parameter sensitivity and (2) compactness gaps. Address these head-on with data, and you'll have a compelling contribution that advances the field.
