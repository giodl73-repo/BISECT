# Review: Percy Liang
**Paper**: GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection for Congressional Redistricting
**Reviewer**: Percy Liang (Stanford — empirical evaluation, NLP/ML systems, reproducibility)
**Date**: 2026-05-02
**Score**: 3.0 / 4

---

## Summary

GeoSection introduces isoperimetric normalisation of the edge-cut criterion in METIS-based recursive bisection: all feasible split ratios 1:(k-1) through k/2:k/2 are scanned, and the ratio minimising EC/sqrt(min(i,k-i)) is selected. A secondary directional penalty (Phase 2) is defined but not evaluated. The 50-state sweep reports "natural ratios" for all 45 multi-district states and a head-to-head partisan comparison against the B.7 MEC baseline across 44 states (6 with partisan data gaps).

The paper's empirical ambition is appropriate for the claim. Running all 50 states and reporting gaps honestly (TX, CA, FL missing; MO missing) is the correct scientific posture. The head-to-head comparison showing 38/44 identical is a strong and concrete finding.

My concerns are primarily about the evaluation design: the study is observational, several important confounds are not controlled, and three of the most politically consequential states are missing from the partisan analysis. The paper should be accepted, but with revisions that strengthen the empirical claims.

---

## Strengths

**S1. 50-state empirical scope with honest gap reporting.**
The sweep covers all 45 multi-district states and marks CA, TX, FL, and MO as having uncertain or missing natural ratios. Reporting the gaps rather than excluding them silently is the correct scientific posture.

**S2. Head-to-head comparison is the right evaluation design.**
Comparing GeoSection to the B.7 MEC baseline on the same pipeline, same data, and same 44 states is methodologically sound. The 38/44 identical finding is credible because the comparison holds constant everything except the ratio selection criterion.

**S3. The Wisconsin case study is well-designed.**
Table 2 (bisection tree) and Table 3 (district partisan shares) together give a complete account of how the algorithm's structural choices propagate to district-level outcomes. The identification of the specific mechanism — suburban Milwaukee tracts placed differently under GeoSection vs. MEC, creating one additional D-leaning district — is the most convincing causal story in the paper.

**S4. Competitive-state proportionality analysis.**
The 8-state competitive-state table (Table 5) with -7.0 pp mean gap is a clean, focused presentation that directly addresses the most important fairness question without overclaiming. Situating this in the context of the Rodden/geographic sorting literature is appropriate.

---

## Weaknesses

**W1. Three large states with missing partisan data represent 27% of the House.**
CA (52 seats), TX (38 seats), and FL (28 seats) are missing from the partisan analysis. Together they account for 118 of 435 congressional seats. TX and FL lean Republican in partisan composition; CA leans strongly Democratic. Excluding all three creates a potentially systematic bias in the head-to-head comparison. The paper notes the technical reason (uncertain natural ratios at 50 seeds) but does not bound the effect on the partisan findings.

**W2. The +5D net finding is presented without uncertainty quantification.**
The abstract and conclusion state "net effect: +5 D seats under GeoSection across 44 states." This is a point estimate from a single run of each algorithm with no variance reported. The natural question — is +5 statistically distinguishable from noise given the seat-count distributions across the 44 states? — is not addressed. With only 7 states differing by 1 seat and 38 identical, a permutation test or bootstrapped confidence interval would characterise whether +5 is a systematic finding.

**W3. Algorithm comparison is not controlled for seed randomness.**
The head-to-head comparison (Table 4) uses the minimum edge-cut over 50 seeds for GeoSection and the MEC baseline. But the two algorithms explore different parts of METIS's solution space: GeoSection runs up to floor(k/2) * N seeds at the first level, while MEC runs N seeds only. The GeoSection result has seen more of the solution space at the first level. To ensure the comparison is fair, the MEC baseline should also be run with floor(k/2) * N first-level seeds and the best result reported.

**W4. Phase 2 (directional penalty) is defined but produces zero data.**
Section 3.4 defines the directional penalty weight formula and Remark 3.2 states all empirical results use lambda=0. Phase 2 appears in the algorithm description, the abstract mentions "Phase 2: PCA orientation per subregion," and the paper claims "noticeably straighter boundaries" from preliminary tests. But no data supports this claim. A paper that describes a contribution in the abstract and introduction but presents no evaluation for it is incomplete.

**W5. The "Rodden Effect" subsection heading overstates the connection.**
The paper labels Section 4.4 "The Rodden Effect" to describe the consistent Republican over-representation in competitive states under GeoSection. But Rodden's geographic sorting hypothesis is about the structural cause of proportionality deficits (urban concentration), not about any specific algorithm. Calling GeoSection's proportionality deficit the "Rodden Effect" implies a causal connection that is not demonstrated — the proportionality deficit could arise from algorithm-specific features that happen to produce the same pattern. A more neutral label (e.g., "Proportionality in Competitive States") would be more accurate.

---

## P1 Items (Must Fix Before Acceptance)

**P1-I. Expand the partisan analysis to bound the large-state exclusion effect.**
For CA, TX, and FL, run GeoSection with 200 seeds per ratio at the first level (or until convergence) and report the natural ratio and partisan outcome. If 200 seeds does not converge, report the range of partisan outcomes across seeds. This bounds the uncertainty in the "+5D net" finding and addresses the selection bias in excluding three of the largest states.

**P1-II. Report variance on the seat-count comparison.**
For the 44-state head-to-head comparison, report a permutation test or bootstrapped confidence interval for the net D-seat difference. The +5D finding needs a denominator: how large is this relative to natural variation in seat counts across seeds?

---

## P2 Items (Should Fix)

**P2-I.** Either add Phase 2 results (even for one state) or explicitly remove Phase 2 from the abstract and contributions list. Describing a contribution without evaluation is misleading.

**P2-II.** Report the range of natural ratios observed across 50 seeds for the 5 states where the normalisation most narrowly selects the winner (i.e., where the normalised scores for the top-2 ratios are closest). This would characterise how stable the ratio selection is near the decision boundary.

**P2-III.** Rename "The Rodden Effect" subsection to a neutral label and clarify that the proportionality finding is consistent with the geographic sorting hypothesis but not uniquely explained by it.

**P2-IV.** Run the MEC baseline with the same total seed budget as GeoSection (floor(k/2) * N first-level seeds) to confirm the head-to-head comparison is not confounded by search budget.

**P2-V.** Clarify whether the "net effect: +5D seats" claim in the abstract refers to the 44-state comparison or a 50-state comparison. If 44 states, the abstract should say so.

---

## Verdict

Accept with Major Revisions. GeoSection is a principled and well-implemented contribution to the redistricting algorithms literature. The caterpillar pathology diagnosis is correct, the normalisation is motivated, and the 50-state sweep is the most comprehensive evaluation in the paper series. The weaknesses are primarily ones of evaluation rigor: the three large missing states create a potential systematic bias, Phase 2 has no empirical grounding, and the net D-seat finding lacks uncertainty quantification.

If the authors bound the large-state exclusion effect (P1-I), add variance on the seat comparison (P1-II), and either evaluate Phase 2 or remove it from the contributions (P2-I), the revised paper will be substantially stronger. The core finding — that 38/44 states are unchanged under GeoSection, with the 6 differing states shifting in both directions — is a clean and honest result that deserves publication in a venue like Political Analysis or ALENEX.

**Score: 3.0 / 4**
