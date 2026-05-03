# Review: Jonathan Rodden
**Paper**: GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection for Congressional Redistricting
**Reviewer**: Jonathan Rodden (Stanford — political geography, urban-rural divide, geographic sorting, Why Cities Lose)
**Date**: 2026-05-02
**Score**: 3.0 / 4

---

## Summary

GeoSection adds isoperimetric normalisation to METIS-based recursive bisection to prevent the "caterpillar pathology" — the systematic isolation of compact urban Democratic cores as pure D+1 districts, with surrounding suburban ring districts clustered near the partisan tipping point. The algorithm selects the split ratio that minimises EC/sqrt(min(i,k-i)) rather than always using near-equal splits. A 50-state sweep reports natural ratios for 45 multi-district states; a head-to-head comparison shows 38/44 states produce identical seat counts under GeoSection and the standard MEC baseline.

This paper asks the right question: does changing the bisection ratio structure change the partisan geography encoded in single-member districts? The answer — largely no — is important and correct, and it directly extends my own empirical work on the geographic efficiency gap. The paper makes a genuine political science contribution in demonstrating that the proportionality deficit in competitive states is robust to changes in the bisection algorithm.

However, the paper's treatment of the caterpillar pathology's partisan implications is incomplete in ways that matter for a political science audience. I recommend acceptance with major revisions to strengthen the political geography analysis.

---

## Strengths

**S1. The caterpillar-to-partisan-packing analogy is sharp and correct.**
The paper correctly identifies why the caterpillar is legally dangerous: it produces safe Democratic urban seats ringing Republican-leaning suburban districts without any partisan input. This is the algorithmic manifestation of the geographic efficiency gap I documented in *Why Cities Lose* — Democrats' spatial concentration makes their votes "wasted" in both deliberate gerrymanders and neutral algorithmic maps. The paper correctly attributes this structural property to geography, not to algorithmic bias.

**S2. The "38/44 stable" finding is the central political science result.**
The finding that 86% of states produce identical seat counts under GeoSection and MEC is a meaningful empirical result. It confirms that the proportionality deficit is a property of geographic voter sorting, not of the specific bisection algorithm. This is exactly what the geographic sorting literature would predict, and it is valuable to see it confirmed in an algorithm-level experiment.

**S3. The Wisconsin case is the strongest political geography story.**
Wisconsin is the best-chosen case study. GeoSection's 3D/5R vs. MEC's 2D/6R is a meaningful change in a state where the geographic sorting problem is well-documented. The mechanism is clear: the MEC tree isolates Milwaukee as a pure D+1 peel; GeoSection's 1:7 confirmation gives Milwaukee a single compact district and places the suburban ring tracts differently, creating a second lean-D district. This is exactly the suburban redistribution mechanism that makes partisan outcomes sensitive to bisection structure.

**S4. Proportionality gap table for competitive states is exactly right.**
Table 5 reporting the -7.0 pp mean gap across 8 competitive states is the correct analysis for this paper's claims. The states are well-chosen (all 46-55% D vote) and the results are consistent with the broader literature on geographic sorting. Presenting this as "the Rodden effect" is flattering but appropriate — geographic sorting dominates the outcome regardless of bisection algorithm.

---

## Weaknesses

**W1. The partisan direction of the 7 differing states is not explained mechanistically.**
The paper identifies 6 states where GeoSection gains D seats and 1 (Pennsylvania) where MEC gains D seats, and notes that "the partisan direction of GeoSection vs. MEC is not systematic." This is partially correct — there is no algorithmic bias — but it undersells the opportunity to explain the mechanism. Why does Wisconsin gain a D seat under GeoSection while Pennsylvania loses one? The Wisconsin explanation is given (suburban Milwaukee tract placement); the Pennsylvania explanation is vague ("MEC's standard bisection happens to produce a better D outcome than GeoSection's confirmed Philadelphia peel"). A proper political geography analysis would show, for Pennsylvania, which specific tracts are placed differently under the two algorithms and why that flips the seat.

**W2. The "genuine urban asymmetry confirmed" category merits more scrutiny.**
Table 1 categorises 17 states as having "genuine urban compactness confirmed" with 1:(k-1) splits. For several of these states (IL: 1:16, PA: 1:16, TX: 1:37, CA: 1:51), this means the algorithm is creating a maximally asymmetric bisection tree — effectively a caterpillar, just now labelled as "confirmed" by the normalisation. The paper argues that the isoperimetric confirmation makes this legally distinguishable from the unnormalised caterpillar, but politically the outcome may be similar: safe urban D seats ringing suburban ring districts. The paper should report whether the "confirmed caterpillar" states (IL, PA) have similar proportionality gaps to the normalisation-shifted states (NC, WI). If the proportionality gap is similar regardless of whether the peel is confirmed or prevented, the normative distinction between the two is weaker.

**W3. Cross-census stability is a significant gap.**
The paper studies only 2020 Census data. My work documents significant urbanisation trends from 2000-2020 that have changed the geographic sorting structure in several competitive states (Arizona, Georgia, North Carolina). If the natural ratio for Wisconsin shifted from 1:7 in 2020 to something different in 2010 or 2000, this would mean GeoSection's bisection tree — and its partisan outcomes — is not stable across apportionment cycles. For three high-interest states (WI, NC, PA), running the natural ratio analysis on 2000 and 2010 data would be a valuable addition.

**W4. The competitive-state classification is not justified.**
The paper defines "competitive states" as those with 46-55% D presidential vote share (Table 5). This threshold is stated but not justified. Presidential vote share is a noisy measure of district-level competitiveness. State House elections, Senate elections, or composite partisan performance measures might classify states differently. For a paper making proportionality claims, using a single election cycle (2020) and a single office (presidential) is a limitation worth acknowledging explicitly.

---

## P1 Items (Must Fix Before Acceptance)

**P1-I. Provide a mechanistic explanation for Pennsylvania.**
The paper explains why Wisconsin gains a D seat under GeoSection but does not explain why Pennsylvania loses one. Pennsylvania is a high-profile competitive state where the bisection structure matters. Add a brief analysis (analogous to the Wisconsin district-level analysis in Section 4.2) explaining which tracts are placed differently under GeoSection's 1:16 "confirmed" Philadelphia peel vs. MEC's standard bisection, and why this produces a worse D outcome under GeoSection.

**P1-II. Analyse proportionality for "confirmed caterpillar" states.**
For Illinois and Pennsylvania — both confirmed at 1:16 — report the proportionality gap under GeoSection and compare it to the MEC baseline. If the proportionality gap under confirmed caterpillar bisection (IL, PA) is similar to the gap under normalisation-shifted bisection (NC, WI), the legal distinction between "confirmed peel" and "prevented caterpillar" does not produce different partisan outcomes. This is an important null result that the paper should report directly.

---

## P2 Items (Should Fix)

**P2-I.** Run the natural ratio analysis for Wisconsin and North Carolina using 2010 and 2000 Census data (or at minimum state whether this is feasible with the current pipeline) to address cross-census stability.

**P2-II.** Add a table or supplementary analysis comparing proportionality gaps in "confirmed asymmetric" states (IL, PA, TN) vs. "normalisation-shifted" states (NC, WI) to characterise whether the two categories have different partisan implications.

**P2-III.** Justify the 46-55% D vote competitive state threshold, or report sensitivity of the mean -7.0 pp gap to alternative thresholds (e.g., 47-53%, 45-55%).

**P2-IV.** The paper mentions in Section 5.3 that "the B.7 seed-sweep convergence result establishes this rigorously for the MEC case." For a political science audience, briefly summarise the B.7 result in one sentence rather than requiring the reader to consult a companion paper.

**P2-V.** The Oklahoma finding (GeoSection gains 1 D seat from 33.1% D vote) is puzzling. At 33.1% D, no algorithm should be producing 1 D seat in a 5-district state without some geographic isolation of a D plurality. Explain this result: which Oklahoma district is D-leaning and why GeoSection creates it.

---

## Verdict

Accept with Major Revisions. This paper makes a genuine and important political science contribution: it demonstrates empirically that changing the bisection algorithm's ratio structure does not change the fundamental proportionality deficit in competitive states. This confirms, at the algorithm level, the geographic sorting hypothesis from the political geography literature.

The weaknesses are primarily ones of depth in the political geography analysis. The Pennsylvania anomaly deserves explanation. The "confirmed caterpillar" category (IL, PA at 1:16) needs proportionality analysis to determine whether the isoperimetric confirmation actually changes political outcomes. Cross-census stability is a significant gap for a paper making claims about natural geographic structure.

With P1-I and P1-II addressed, the paper will have a much cleaner story: GeoSection prevents the caterpillar in states where geography does not warrant it (NC, WI), confirms it in states where geography does (IL, PA), and produces similar proportionality deficits in both categories. That would be a strong and coherent finding.

**Score: 3.0 / 4**
