# Quality Assessment: Algorithmic Objectivity for Congressional Redistricting

**AI Persona**: Richard Pildes (based on work at NYU School of Law)
**Expertise Area**: Election law, Voting Rights Act, constitutional law
**Round**: 1
**Date**: 2026-02-08

> **Simulation Notice**: This is AI-generated feedback for quality improvement, not a real peer review. Use these insights to strengthen your work.

---

**Content Mode**: full

---

## Overall Assessment

This paper presents a sophisticated technical approach to redistricting with potentially significant legal implications. The "impossibility defense" concept—that algorithms cannot gerrymander because they cannot access partisan data—is a genuinely novel legal argument that deserves serious consideration post-*Rucho*. The empirical finding that neutral algorithms produce 69 more majority-minority districts than enacted plans is constitutionally important: it suggests that *Shaw* concerns about race-conscious districting may be overstated if neutral criteria produce comparable or better outcomes.

However, as an election law scholar, I have serious concerns about the paper's legal analysis. First, the impossibility defense is legally naive: courts care about **effects**, not just intent. An algorithm that systematically advantages one party due to geography will still face partisan gerrymandering claims even if the designer couldn't see partisan data. Second, the VRA analysis conflates **ability to create MM districts** with **legal obligation to do so**. Section 2 requires showing vote dilution, not just that more MM districts are geometrically possible. Third, the paper ignores the *one person, one vote* nuances: ±0.5% deviation may be permissible for states but the Supreme Court has demanded near-zero deviation for congressional districts.

Most critically, the paper doesn't engage with the **justiciability problem** that motivated *Rucho*: even if algorithmic maps are "better," courts still need **manageable standards** to evaluate them. Saying "use this algorithm" replaces one non-justiciable question (is this map too partisan?) with another (are these parameters correct?). The paper needs to address how courts would adjudicate disputes over parameter choices.

For Science, this level of legal analysis may suffice to convey the broad implications. But the paper should acknowledge these legal complexities more clearly to avoid overstating what algorithmic redistricting can achieve legally.

## Score

**Score**: 3/4 — Accept (with legal analysis revisions)

## Major Issues (Blocking)

### M1: Impossibility Defense Legally Insufficient

The paper's core legal claim (Section 5) is that algorithmic redistricting provides an "impossibility defense" stronger than intent-based tests because the algorithm cannot access partisan data. This misunderstands equal protection doctrine. Under *Arlington Heights* (1977), discriminatory effects can violate the Constitution even without discriminatory intent if effects are sufficiently severe and unexplained by neutral factors.

An algorithm that produces 56.5% Democratic districts when Democrats win 52% of votes (efficiency gap) could face equal protection claims regardless of designer intent. The algorithm's "structural inability" to see partisan data doesn't immunize its effects from scrutiny—it just shifts the burden to showing the outcome is justified by legitimate state interests (compactness, contiguity, etc.).

**Required**: Revise Section 5 to acknowledge that:
- Intent is only one factor in equal protection analysis
- Effects-based challenges remain viable under *Arlington Heights*
- The defense is procedural (transparent, reproducible) not substantive (immune from challenge)
- Courts would still apply rational basis review to parameter choices

The impossibility defense is valuable for demonstrating good faith, but it's not a silver bullet.

### M2: VRA Analysis Conflates "Can" with "Must"

Section 4, Finding 2 reports that algorithms *can* create 137 MM districts vs. 68 enacted, suggesting this vindicates algorithmic approaches for VRA compliance. But Section 2 of the VRA doesn't require maximizing MM districts—it prohibits **vote dilution**. The *Gingles* test (1986) requires showing:
1. Minority group is sufficiently large and geographically compact
2. Minority group is politically cohesive
3. White majority votes sufficiently as a bloc to defeat minority-preferred candidates

Your analysis shows (1) is satisfied for more states than currently have MM districts, but doesn't address (2) or (3). Without racially polarized voting analysis, we can't conclude that 137 MM districts are *required* by Section 2, only that they're geometrically *possible*.

**Required**: Clarify that your analysis addresses only the first *Gingles* prong. Add discussion of when geometric possibility triggers Section 2 obligations—this requires showing actual vote dilution, not just that more districts could be drawn. Distinguish between **opportunity districts** (minority population sufficient) and **performing districts** (minority actually elects preferred candidates).

### M3: Population Deviation Standard Too Lenient

The paper reports mean population deviation of 2.79% and describes this as "comparable to human maps and well within... one-person-one-vote tolerance." This is incorrect for **congressional districts**. While state legislative districts allow ~10% total deviation (*Brown v. Thomson*, 1983), congressional districts require "absolute equality" absent extraordinary justification (*Karcher v. Daggett*, 1983). Courts have struck down congressional plans with deviations under 1%.

A 2.79% mean deviation with some districts at ±2.79% or higher would likely violate *Karcher*. If your algorithm can achieve ±0.5% as mentioned in Section 3, why does the aggregate report 2.79%? This suggests either measurement error or the algorithm sometimes produces non-compliant plans.

**Required**: Clarify the population deviation metric:
- Is 2.79% the mean of absolute deviations or mean of maximum deviations per state?
- What percentage of districts exceed ±1% deviation?
- Report max deviation per state and identify any states exceeding constitutional limits
- If algorithm allows user-specified tolerance, recommend ±0.5% or tighter for congressional use

## Minor Issues

### m1: *Rucho* Mischaracterized

Section 1 states *Rucho* "removed federal oversight" of partisan gerrymandering, suggesting a policy choice. More accurately, *Rucho* held partisan gerrymandering claims are **non-justiciable political questions** because courts lack manageable standards. The Court didn't endorse gerrymandering—it concluded federal courts are institutionally incapable of policing it. This matters because your algorithmic proposal must address the justiciability problem, not just provide an alternative method.

### m2: *Shaw* Implications Understated

Finding 2 states that neutral algorithms achieve VRA compliance "without explicit racial targeting" and cites *Shaw* as creating scrutiny for race-conscious redistricting. But *Shaw* doesn't prohibit race-conscious districting—it applies **strict scrutiny**, requiring narrow tailoring to a compelling interest. If edge-weighting systematically connects minority tracts (even without explicit racial classification), this could trigger *Shaw* review.

More fundamentally: if neutral algorithms *exceed* VRA requirements, this raises the question of whether race-conscious districting remains permissible. Post-*Students for Fair Admissions* (2023), there's tension between maximizing minority representation and avoiding racial classifications. The paper should address whether algorithmic methods that "happen" to create MM districts navigate this tension or exacerbate it.

### m3: Traditional Redistricting Principles Missing

Most state constitutions and redistricting criteria include principles beyond population equality, contiguity, and compactness:
- Respect for county and municipal boundaries
- Preservation of communities of interest
- Protection of incumbent pairings
- Competitive districts (some states)

The paper doesn't report whether algorithmic plans satisfy these traditional criteria. For legal adoptability, show whether algorithms respect or violate these state-specific requirements.

### m4: 42% Threshold as Legal Standard

Section 5 suggests the 42% threshold provides "empirical benchmarks for the *Gingles* geographic compactness prong." But *Gingles* doesn't establish percentage thresholds—it asks whether a minority group *can* constitute a majority in a reasonably compact district. A state at 41% minority might still have geographic clustering sufficient for one MM district.

The threshold is analytically useful but legally imprecise. Courts apply *Gingles* district-by-district, not state-by-state. A statewide threshold risks both false positives (state at 43% with evenly dispersed population) and false negatives (state at 40% with highly clustered population enabling one district).

### m5: Algorithmic Transparency vs. Black Box

The paper touts algorithmic transparency but doesn't address the **black box problem**: METIS is proprietary, complex software that redistricting commissioners cannot meaningfully audit. If two commissioners propose different edge-weighting schemes and get different results, how do courts choose? The paper needs to address whether algorithmic governance trades political manipulation for technical opacity.

## Strengths

1. **Impossibility defense is novel**: Even if not legally dispositive, this framing advances the discourse and provides courts with a new conceptual tool.

2. **VRA findings are timely**: Post-*Allen v. Milligan* (2023), demonstrating that neutral methods can satisfy Section 2 is constitutionally significant.

3. **Huntington-Hill analogy is apt**: The parallel to apportionment provides a historical model for mathematical governance that courts and legislators will understand.

4. **Addresses *Rucho* gap**: By proposing manageable standards (use this algorithm, these parameters), the paper responds to the Court's call for alternatives.

## Questions for Authors

1. How would courts adjudicate disputes over parameter settings? If plaintiffs argue edge-weights are insufficiently tuned for VRA compliance, what standard of review applies?

2. If an algorithmic plan produces partisan effects (efficiency gap), can losing party challenge it? Under what legal theory?

3. Does the algorithm's inability to see partisan data insulate it from **First Amendment** claims? *Rucho* hints that extreme partisan gerrymandering might violate free speech/association rights.

4. Have you analyzed whether algorithmic plans comply with state constitutional requirements (e.g., Texas Constitution Art. III § 28 prohibiting county splits)?

5. What's the constitutional status of edge-weighting parameters? Are they legislative choices (entitled to deference) or technical decisions (subject to judicial review)?

## Recommendations

- Revise impossibility defense to acknowledge effects-based challenges remain
- Clarify VRA analysis distinguishes "geometric possibility" from "legal obligation"
- Correct population deviation standard for congressional districts (report max per state)
- Add discussion of how courts would adjudicate parameter disputes (justiciability)
- Analyze whether edge-weighting triggers *Shaw* strict scrutiny
- Address *Shaw*/*SFFA* tension between maximizing MM representation and avoiding racial classifications
- Report compliance with traditional redistricting principles (county splits, communities of interest)
- Refine 42% threshold discussion to avoid implying bright-line legal rule
- Discuss algorithmic transparency vs. technical opacity tradeoff
- Consider adding a "Justiciability and Judicial Review" subsection to Section 5

---

**Verdict**: This paper makes important contributions at the intersection of computer science, political science, and law. The technical work is impressive and the normative vision—mathematical governance for redistricting—is compelling. However, the legal analysis needs substantial revision to accurately represent constitutional doctrine and avoid overstating what algorithmic methods can achieve legally. The impossibility defense is a valuable concept that needs more careful legal grounding. With revisions addressing the constitutional law issues, this will be a significant contribution to election law scholarship and could influence future redistricting reform efforts. The work is rigorous; the legal framing needs refinement to match.
