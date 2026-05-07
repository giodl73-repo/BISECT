# Review 5 — Reviewer: Christina Liang (Computational Social Science / Causal Inference)
**Paper:** C.8 — Do Algorithmic Districts Produce Competitive Elections? Evidence from All 50 States
**Round:** 1
**Score:** 3/4

## Summary

A clearly written empirical paper with a well-defined research design. The main finding (85 vs. 65 competitive districts) is plausible and the methodology is appropriate. My concerns are about causal identification and the absence of uncertainty quantification. The paper is framed as a test of the "competitiveness argument" for algorithmic redistricting, but the research design is observational — the comparison between algorithmic and enacted maps is not a randomized experiment — and the causal language should be softened accordingly.

## Strengths

The definition of the outcome variable is clear and well-justified. Using presidential vote share rather than congressional vote share avoids incumbency confounding, which is the right choice for a cross-sectional comparison of map configurations. The acknowledgment of the presidential vote proxy's limitations (Section 2.4) is honest and appropriate.

The durability analysis (Section 6) is the methodological highlight. The finding that 61/85 competitive algorithmic districts remain competitive under all shifts from -4 to +4 percentage points is more robust than a single-year cross-section. This is the paper's most convincing evidence that competitive algorithmic districts are structurally rooted in geography rather than in the specific 2020 electoral environment.

The partisan symmetry analysis (Section 5) is well-constructed. The test comparing 50.6% competitive Democratic share to 51.3% national Biden share is the right kind of calibration test, even if it's not a perfect partisan-symmetry test.

## Weaknesses and Concerns

The biggest methodological gap is the absence of uncertainty quantification for the central finding. The paper reports 85 competitive districts as a point estimate from a single random seed. The standard deviation of this estimate across random seeds is not reported, and the reference to "dellaLibera2026uncertainty" for ensemble analysis deflects to a paper that is not available for review.

To support the headline claim, the paper needs a confidence interval. The simplest approach: run the redistricting algorithm with 20 different seeds and report the 5th-95th percentile range of competitive district counts. If the range is, say, [80, 91], then the 85 figure is a robust central estimate and the comparison to 65 under enacted maps is convincing. If the range is [55, 115], the comparison is not convincing.

The causal language in the paper is too strong. The abstract says algorithmic maps "produce approximately 85 competitive districts compared with 65 under enacted plans." This is accurate as a descriptive comparison, but the implicit causal claim — that algorithmic redistricting *causes* the increase in competitive districts — is not established by the research design. The counterfactual question is: if enacted maps had been drawn with the same process as algorithmic maps, how many competitive districts would they have produced? The paper assumes that the difference between algorithmic and enacted maps is due to the redistricting process, but it could also be due to the specific algorithmic parameters, the 2020 electoral environment, or state-specific demographic changes. The paper should use language like "algorithmic maps are associated with more competitive districts" rather than "produce."

The comparison of "algorithmic maps" to "enacted plans" conflates two things: the redistricting *process* (algorithmic vs. human) and the redistricting *criteria* (compactness-only vs. multiple criteria including communities of interest). A state that draws a compact, communities-of-interest-respecting map through a human process might produce similar competitive district counts as the algorithmic map. The paper should acknowledge that its comparison isolates the effect of geometric neutrality plus compactness optimization, not the effect of algorithmic vs. human redistricting per se.

## Minor Issues

- Table 1 shows Safe Democratic seats falling from 208 to 189 while Safe Republican seats stay roughly constant (162→161). The footnote mentions area-weighted interpolation for the comparison. But the enacted map vote-share reaggregation might have different measurement error than the algorithmic map reaggregation, since enacted district boundaries are drawn at the block level while algorithmic district boundaries are at the tract level. This could systematically affect the competitive district count comparison. The paper should check whether reaggregation error affects the comparison.
- The reference to "dellaLibera2026stability" in Section 6.3 for temporal stability of algorithmic district boundaries is not in the reference list and appears to be a forward reference to work not yet completed. If this citation cannot be resolved, the forward-looking claim about 2030-cycle stability should be removed or caveated.

## Recommendation

Accept with minor revisions. Add ensemble-based confidence interval for the competitive district count, soften the causal language to associational language, and resolve the forward citation to "dellaLibera2026stability."
