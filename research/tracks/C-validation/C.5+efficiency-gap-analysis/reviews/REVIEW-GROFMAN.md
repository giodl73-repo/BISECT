> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Bernard Grofman (UC Irvine, Political Science & Law)
**Expertise**: Voting Rights Act, redistricting law, electoral systems
**Date**: 2026-02-08
**Venue**: American Political Science Review

---

## Overall Assessment

This paper provides valuable quantitative benchmarks for partisan fairness in redistricting, and the national scope is commendable. However, the paper has a critical omission: it entirely ignores Voting Rights Act compliance and racial fairness. Algorithmic redistricting that reduces partisan bias while destroying minority opportunity districts is not "neutral"—it violates federal law. The paper needs substantial additions addressing VRA compliance and the tension between partisan fairness and minority representation.

**Verdict**: Weak Accept (contingent on addressing VRA concerns)
**Score**: **2.5/4**

---

## Major Issues

### 1. Voting Rights Act Compliance Entirely Absent (P1)

You mention VRA once in passing (footnote: "prior work shows algorithmic plans exceed VRA compliance") but provide NO analysis of how your algorithmic plans affect minority representation. This is legally and ethically unacceptable.

**Critical missing analysis:**
- How many majority-minority districts exist in enacted vs algorithmic plans?
- Do algorithmic plans maintain or destroy Section 2 opportunity districts?
- What happens to Black and Latino representation when you optimize for compactness?
- Are there VRA-protected districts in Rust Belt states (Detroit, Milwaukee, Cleveland)? Do algorithmic plans pack or crack these communities?

**Why this is P1 (blocking):** Section 2 of the VRA requires maintaining minority opportunity districts where possible. If your algorithmic plans reduce minority representation by 20-30%, they are legally indefensible regardless of partisan fairness improvements.

**Example concern:** Pennsylvania's enacted plan includes PA-02 (Philadelphia, majority-Black) and PA-03 (Pittsburgh, plurality-Black). Do your algorithmic plans maintain these districts? If not, you've solved partisan gerrymandering by committing racial gerrymandering.

**Required revision:**
1. Add complete Section 4 subsection: "Minority Representation and VRA Compliance"
2. Report majority-minority district counts for algorithmic vs enacted plans
3. Analyze whether algorithmic plans maintain or destroy Section 2 opportunities
4. Discuss tension between compactness (packs minorities) and descriptive representation

### 2. Intersectionality of Partisan and Racial Gerrymandering (P1)

The Rust Belt states you highlight (PA, WI, MI) have significant Black populations concentrated in Detroit, Milwaukee, Pittsburgh, Philadelphia, and Cleveland. When you report that algorithmic plans produce -2.8% EG while enacted plans show +7.2% EG, you're not accounting for:

**Potential confound:** Enacted plans may pack Black voters (who are overwhelmingly Democratic) into majority-minority districts to comply with VRA. This creates:
- High Democratic efficiency gap (surplus votes in majority-minority districts)
- But also: descriptive representation for Black voters

**Problem:** Your algorithmic plans might achieve better efficiency gaps by cracking Black voters across multiple districts, diluting both their partisan AND racial voting power.

**Required analysis:**
1. Compare your results to alternative "VRA-compliant algorithmic plans" that explicitly maintain majority-minority districts
2. Show whether enacted plans' Republican advantage comes from:
   a) Packing minorities (VRA compliance), or
   b) Additional manipulation beyond VRA requirements
3. Quantify the "VRA cost": how much does maintaining majority-minority districts increase efficiency gap?

### 3. Coalition Districts and Section 2 "Gingles" Criteria (P2)

Section 2 requires opportunity districts for racial minorities where the three *Thornburg v. Gingles* preconditions are met:
1. Minority group sufficiently large and geographically compact
2. Minority group politically cohesive
3. Majority votes as bloc to defeat minority-preferred candidates

**Missing analysis:**
- Did you test whether algorithmic plans satisfy *Gingles* preconditions in relevant regions?
- Do you maintain "coalition districts" (e.g., Black+Latino districts in Texas)?
- How many "influence districts" (30-45% minority) do algorithmic vs enacted plans create?

**Suggested addition:** Brief discussion of *Gingles* criteria and how algorithmic compactness optimization interacts with geographic compactness of minority communities.

---

## Minor Issues

### 4. Limited Engagement with Redistricting Law Literature (P2)

Your references are heavily weighted toward quantitative political science (Stephanopoulos, McGhee, Rodden, Chen) but light on legal scholarship. For APSR submission, you should engage:

- *Shaw v. Reno* line of cases on racial gerrymandering
- State constitutional provisions on redistricting (PA, FL, NY, CO)
- Recent *Alabama v. Milligan* decision on Section 2 enforcement

**Suggested addition:** Add 5-6 legal citations discussing VRA compliance, state constitutional requirements, and tension between partisan and racial fairness.

### 5. "Neutral" Algorithm Assumption Problematic (P3)

You claim METIS is "neutral" because it can't access partisan data. But it CAN access census data, which is highly correlated with race. And race is highly correlated with partisanship (90% of Black voters support Democrats).

**Implication:** Your "neutral" algorithms indirectly access partisan data through racial demographics. This doesn't invalidate your findings, but you should acknowledge it.

**Suggested addition:** Paragraph discussing how demographic data serves as proxy for partisan patterns, and why this is unavoidable (VRA requires using racial data).

### 6. Alternative Electoral Systems Underexplored (P3)

Section 5.4 mentions multi-member districts as alternative to single-member districting, but gives this only 2 sentences. Given your finding that single-member districts produce unavoidable -3.2% bias, you should discuss:

- At-large elections with cumulative voting (used in some jurisdictions)
- Multi-member districts with ranked-choice voting
- Mixed-member proportional representation (like Germany)

**Suggested addition:** Expand discussion of alternative electoral systems, with focus on how they interact with VRA compliance (multi-member districts can maintain minority representation while improving proportionality).

---

## Positive Aspects

1. **Quantitative rigor**: National-scale efficiency gap analysis is valuable
2. **Geographic honesty**: Acknowledging that algorithms produce Democratic bias due to urban concentration is important
3. **Legal relevance**: Benchmarks useful for state constitutional litigation
4. **Temporal stability**: Showing EG persistence across elections is valuable
5. **Regional variation**: Identifying Rust Belt as extreme case is helpful

---

## Specific Recommendations

### Section 2 (Background)
- Add subsection on Voting Rights Act requirements and tension with partisan fairness
- Cite *Shelby County*, *Allen v. Milligan*, and recent Section 2 litigation

### Section 4 (Results)
- Add subsection "4.7 Minority Representation and VRA Compliance"
- Report majority-minority district counts for all 50 states
- Analyze whether algorithmic plans maintain Section 2 opportunities

### Section 5 (Discussion)
- Add subsection "5.5 Partisan Fairness vs. Minority Representation"
- Discuss tradeoffs: compact districts may pack minorities, less compact districts allow coalition-building
- Compare to "VRA-compliant algorithmic plans" that explicitly maintain opportunity districts

### Section 6 (Conclusion)
- Acknowledge that partisan fairness and racial fairness may conflict
- Discuss how reformers should prioritize when these goals diverge

---

## Questions for Authors

1. Did you analyze minority representation at all? If so, why isn't it reported?
2. Do your algorithmic plans maintain existing majority-minority districts?
3. Have you compared your results to "VRA-constrained" algorithmic plans?
4. What happens to Latino representation in Texas, California, Arizona, Florida?

---

## Historical Context

As someone who has worked on redistricting since the 1970s, I've seen multiple waves of reform focus on single objectives:
- 1960s-70s: Population equality (*Reynolds v. Sims*)
- 1980s-90s: Minority representation (VRA amendments)
- 2000s-10s: Partisan fairness (efficiency gap)
- 2020s: Compactness and algorithmic redistricting

Each wave ignores tradeoffs with prior objectives. Your paper continues this pattern by focusing solely on partisan fairness while ignoring racial fairness. A complete analysis must address BOTH.

The good news: your methodology is sound for measuring partisan fairness. The bad news: measuring partisan fairness in isolation is insufficient for redistricting evaluation.

---

## Verdict Justification

This paper makes a valuable empirical contribution to the partisan fairness literature, but it has a critical gap: no analysis of Voting Rights Act compliance or minority representation. For a paper claiming to evaluate "neutral" redistricting, ignoring racial fairness is problematic.

I'm giving this paper 2.5/4 (Weak Accept) contingent on adding substantial VRA analysis. If the authors can show:
1. Algorithmic plans maintain majority-minority districts
2. Or, if not, quantify the VRA-partisan fairness tradeoff

Then this becomes a 3.5/4 (Strong Accept). But without racial fairness analysis, this paper is incomplete.

**Recommendation**: Major revisions required, specifically adding VRA compliance analysis and discussing partisan-racial fairness tradeoffs. The partisan fairness findings are solid, but they're only half the story.
