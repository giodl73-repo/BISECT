---
reviewer: Moon Duchin
round: 1
score: 2
date: 2026-05-05
---

## Summary

B.5 presents a systematic comparison of recursive bisection and direct k-way partitioning across 727 chamber configurations and concludes that RB dominates on compactness by a small but consistent margin. The paper is well-organised and the experimental scale is impressive. However, from a mathematical perspective, the paper has substantive gaps in how it characterises the solution space, how it justifies the Polsby-Popper metric as the appropriate single outcome variable, and whether the statistical analysis adequately accounts for the multiplicity of configurations. These gaps affect both the strength of the conclusions and the paper's positioning relative to the redistricting mathematics literature.

## Strengths

- **Prime-k analysis.** The identification of prime-k as an edge case for RB, with quantification of the advantage loss and a recovery mitigation, is the most mathematically novel part of the paper. The finding that k=17 (Pennsylvania) retains +0.003 PP despite three levels of asymmetric splits is interesting and appears robust.
- **Polsby-Popper connection to edge-cut objective.** The methodology section correctly notes that minimising PP for boundary-length-weighted edges is equivalent to minimising total edge weight. This formal connection between the graph-partitioning objective and the geometric compactness measure is stated correctly and justifies the choice of PP as the primary outcome.
- **Block-group resolution confirmation.** Confirming the RB advantage at 3x larger graphs (block-group resolution) provides evidence of scale robustness. Many redistricting papers stop at tract resolution; this extension is valuable.

## Weaknesses / P1 Items (Required Fixes)

- **The statistical analysis does not account for the clustering structure.** The paper reports paired t-tests pairing by state, which treats each state as an independent observation. But the 727 chambers are not independent — state senate and state house chambers for the same state share geography, and the 50 states in the congressional comparison partially overlap with the same 50 states in senate and house comparisons. The Bonferroni correction for three comparisons is insufficient because it assumes the three tests are independent. A mixed-effects model with state as a random effect (treating chamber type as the fixed effect of interest) would properly account for the clustering. Without this, the reported p-values are potentially anti-conservative.
- **The Polsby-Popper metric has known sensitivity to measurement scale.** PP depends on the perimeter-to-area ratio, which is sensitive to the resolution at which boundaries are measured. The paper uses two resolutions (tract and block-group) and finds "identical" results, but does not report PP values at block-group resolution. If block-group PP values are systematically higher (finer boundaries produce smoother perimeters) or lower (more jagged boundaries), the comparison between strategies at block-group resolution may not be meaningful. The paper should report block-group PP values for at least the four state-level case studies.
- **The 727-chamber count is not fully documented.** Section 3.1 lists 435 congressional + "99 unicameral-or-senate chambers" + "all 50 state house chambers" = 584, not 727. The discrepancy (727 - 584 = 143) is not explained. Is this chambers at two resolutions? Is it multiple census years? The Methods section must clarify how 727 is computed, or the abstract claim is misleading.

## P2 Items (Suggestions)

- **Compare to an ensemble baseline.** The paper benchmarks RB vs. n-way, but the redistricting mathematics literature would expect comparison against a ReCom ensemble (at least for a small state like Wisconsin). Knowing whether RB's mean PP falls within the ensemble distribution is more informative than the RB-vs-n-way comparison alone.
- **The FM post-processing mitigation for prime-k should be implemented and measured.** Section 5.2 proposes the mitigation conceptually and claims it recovers ~60% of lost compactness, but Table 2 does not show post-processing results. Either implement the mitigation and show the numbers, or hedge the claim as "preliminary."

## Score: 2 — Major Revision

The chamber-count discrepancy and the clustering problem in the statistical analysis are substantive issues that require additional work. The block-group PP reporting gap weakens a key claim. I would reconsider with a revised analysis that fixes the count, uses a proper mixed model, and reports block-group PP values.
