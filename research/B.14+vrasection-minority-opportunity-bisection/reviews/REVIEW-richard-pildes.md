# Review: VRASection: Geographic Alignment Score for Minority Opportunity District Bisection

**Reviewer**: Richard Pildes (NYU School of Law)
**Expertise**: Voting rights law, constitutional law of democracy, Shaw v. Reno doctrine, Section 2 jurisprudence, election law
**Date**: 2026-05-02

## Overall Assessment

This paper makes a genuine advance in algorithmic redistricting by designing a VRA-responsive partitioning method that avoids the constitutional and numerical pitfalls of explicit minority-concentration constraints. The insight that Gingles Prong 1's geographic compactness requirement is the same geographic signal encoded in the census-tract adjacency graph is elegant and legally coherent.

My assessment as a voting rights scholar is mixed. The algorithm design is well-reasoned; the legal analysis contains important errors that, if not corrected, would make this paper harmful to practitioners who cite it in expert testimony. The paper's treatment of *Shaw v. Reno* is too confident about a bright-line threshold that does not exist in the case law, and the paper underplays the significance of *Cooper v. Harris* (2017), which substantially narrows the circumstances under which a race-conscious redistricting decision survives strict scrutiny. These are not merely academic quibbles: a redistricting expert who testifies based on the paper's current Shaw analysis would be advancing a legally incorrect argument.

The empirical contribution is also incomplete. The paper presents first-level bisection ratios but does not demonstrate that those ratios produce compliant final plans. For a paper targeted at Penn Law Review or Election Law Journal, this gap is disqualifying in the current draft.

## Score: 3/4

**My score**: 3/4 --- Important contribution to the algorithmic redistricting literature, but legal analysis needs substantial revision before publication in a law review.

## Major Strengths

1. **MetisVra failure analysis as motivation**: The constraint scale mismatch explanation from B.3 is a clean, non-legal argument for why explicit minority constraints fail. This sets up VRASection's architectural alternative well.

2. **Legal design architecture**: Applying the alignment bonus only at level 1 of the bisection tree, and bounding it to 40% of the selection score, represents genuine legal engineering. The "race observed once, at the highest geographic scale" framing translates well into expert testimony.

3. **Gingles Prong 1 operationalization**: Connecting the alignment score directly to Gingles' geographic concentration requirement is the right theoretical move. If the minority community is geographically compact enough to satisfy Prong 1, the alignment score will be non-trivially positive; if not, the algorithm reduces to GeoSection. This is a proper operationalization of the legal standard.

4. **The Alabama case study is well-chosen**: *Allen v. Milligan* (2023) provides a specific, binding legal standard against which the algorithm can be tested: does Alabama's map produce a second Black-opportunity district? Using this as the lead case gives the paper an empirical anchor.

## Major Issues (Must Address)

### Issue 1: The Shaw v. Reno Analysis Mischaracterizes the Doctrine
**Severity**: High
**Description**: Section 3.3 and Section 5.2 claim that the Shaw/Miller racial predominance test is satisfied whenever $w_\text{vra} < 0.5$ because compactness then "exceeds" the alignment component in the objective function. This is not how the predominance test works.

The predominance test is a factual inquiry into whether racial considerations were the actual explanation for specific boundary choices. *Miller v. Johnson* (1995) held that the question is whether race was "the predominant factor motivating the legislature's decision to place a significant number of voters within or without a particular district." *Bush v. Vera* (1996) applied this at the district level, not at the statewide algorithm level. *Cooper v. Harris* (2017) further clarified that even when a legislature has a strong basis in evidence for a racial classification, strict scrutiny applies whenever race is the predominant factor, and remedial intent does not save the plan from that scrutiny.

The paper's analysis fundamentally errs by treating the objective function weight as equivalent to the "predominant factor" inquiry. Courts do not inspect objective function coefficients; they examine whether specific boundary decisions are explicable on non-racial grounds. A plan drawn with a 1% racial weight could still exhibit racial predominance if that 1% explains the critical boundary deviation from a geographically natural split.

What VRASection can legitimately argue under current doctrine: (1) the specific boundary choices it makes are geographically explicable (the Black Belt boundary is real, not manufactured); (2) compactness is the primary criterion by design; (3) the racial signal functions as a bounded tie-breaker where the geographic boundary would otherwise be ambiguous. This is a stronger argument than "$w < 0.5$," and it is the argument the paper should be making.

**Recommendation**: Replace Section 3.3's $w_\text{vra} < 0.5$ predominance analysis with an analysis based on the counterfactual question: for Alabama's 2:5 split, is the boundary explicable on geographic grounds (the Black Belt's natural northern boundary) or does it require racial data to explain? Engage with *Cooper v. Harris* (2017) on the limits of the remedial justification.

### Issue 2: The Paper Does Not Engage with the Narrow Tailoring Requirement
**Severity**: High
**Description**: Even if VRASection survives the racial predominance threshold (i.e., strict scrutiny does not apply), the paper should address the separate question of narrow tailoring. If VRASection is used in a jurisdiction with a demonstrated Section 2 obligation (like Alabama post-*Allen v. Milligan*), the use of minority VAP in the selection score is justified by a compelling interest --- VRA compliance. But narrow tailoring requires that the racial classification be no broader than necessary to achieve the compelling interest.

For VRASection, narrow tailoring raises a specific question: does applying the alignment bonus only at level 1, using minority VAP (not racial identity) as the input, and bounding the weight to 40% constitute the least-race-conscious means of achieving the Section 2 remedy? The paper implicitly argues yes --- the design is minimally race-conscious --- but does not make this argument explicitly. A court applying strict scrutiny would want to see this analysis.

**Recommendation**: Add a paragraph in Section 5.2 addressing narrow tailoring: (1) the Section 2 obligation provides compelling interest; (2) VRASection is narrowly tailored because it uses geographic distribution rather than racial identity, applies alignment at a single level, and bounds the weight below the predominance threshold; (3) compare to less-tailored alternatives (MetisVra, explicit racial set-asides) to show VRASection is at the minimally-intrusive end of the spectrum.

### Issue 3: The Absence of End-to-End Results for Alabama
**Severity**: High
**Description**: This paper is motivated by *Allen v. Milligan*'s requirement that Alabama draw 2 Black-opportunity congressional districts. The paper presents VRASection's first-level bisection choice (2:5 instead of 1:6) and asserts that this "creates preconditions" for a second MM district, but does not verify that VRASection actually produces 2 MM districts in the full 7-way partition.

For a law review or election law publication, this omission is disqualifying. A practitioner or court relying on this paper needs to know whether the algorithm achieves VRA compliance, not merely that it chooses a first-level bisection that has better theoretical properties. The claim in the conclusion that VRASection "responds directly to the *Allen v. Milligan* (2023) finding" is overstated if the paper cannot show it produces the required second MM district.

**Recommendation**: Run the full pipeline for Alabama and report: (a) the number of districts with >50% Black VAP in the final 7-way partition under VRASection; (b) the same count under GeoSection for comparison; (c) whether the result is stable across the 50 seeds tested. A single-state full result is feasible and necessary.

## Minor Issues

- **Section 2 vs. Section 5**: The paper addresses only Section 2 (vote dilution). Section 5 preclearance, while substantially weakened by *Shelby County* (2013), remains relevant in certain contexts. A footnote clarifying the paper's Section 2 focus and the limited remaining applicability of Section 5 would help readers situate the analysis.

- ***Abbott v. Perez* (2018)**: The paper does not cite *Abbott v. Perez*, which substantially narrowed the circumstances under which a district can be found to be an unconstitutional racial gerrymander. *Abbott* held that the burden of proof on the racial predominance question is heavily on plaintiffs once the legislature states race-neutral justifications. VRASection's documented geographic rationale for each bisection choice would benefit from this framing.

- **The 50% predominance threshold as presented**: Table 4's "Shaw compliant?" column implies a binary legal classification. This is misleading. Legal compliance under Shaw is a fact-specific inquiry that no algorithm can guarantee. Replace the column with "Within proposed safety margin" or similar, and add a footnote clarifying that legal compliance depends on the specific factual record.

- **The Callais standard**: The discussion of *Callais v. Landry* (2025) in Section 5.5 conflates two different questions: (1) whether VRASection produces a plan that satisfies Section 2's map-drawing obligation, and (2) whether the bloc-voting evidence from the Callais analysis supports a Section 2 claim in the jurisdiction. These are separate questions and should be analyzed separately.

- **The CVAP question**: Courts in Section 2 litigation often examine Citizen VAP (CVAP) rather than total VAP because non-citizens cannot vote and their inclusion can inflate apparent minority percentages. For Alabama and Mississippi (low non-citizen populations) this distinction is minor, but the paper should flag it as a methodological choice with a citation to cases discussing CVAP.

## Questions for Authors

1. Does the full pipeline run of VRASection on Alabama produce 2 Black-majority districts? If not, what is the result, and does the paper's conclusion need to be revised?

2. How does the paper respond to the argument that Alabama's 2:5 first-level split --- with the Black Belt on one side --- is explicable entirely on geographic grounds, without any alignment score? If the geographic argument alone justifies the split, then VRASection is redundant for Alabama and the hard cases are elsewhere. What is the paper's best case study where the alignment score is doing non-obvious work?

3. For states where VRASection and GeoSection agree on the first-level ratio (MS, GA, LA), does VRASection produce the same or different MM district counts in the final partition? If the same, what is the argument for VRASection over GeoSection in those states?

4. The paper targets Penn Law Review and Election Law Journal. Would it benefit from a law review co-author who could take responsibility for the legal analysis sections?

## Recommendation

Revise with emphasis on legal accuracy. The algorithm and the Gingles operationalization are strong. Rewrite the Shaw predominance analysis to connect to actual doctrine, add narrow tailoring analysis, and report end-to-end MM district counts for Alabama before submitting to a law review venue.
