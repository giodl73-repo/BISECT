---
reviewer: Percy Liang
round: 2
score: 4
date: 2026-05-05
---

## Summary

The authors have resolved two of my three Round 1 P1 items. The abstract CI is now correct ([+15%, +29%] for all three sources), and the Section 7.2 synthesis text clarifies that the narrower [+18%, +26%] corresponds to seed+resolution only. The LaTeX math mode error in the align environment (which was causing missing-dollar warnings) is also fixed. My P1.2 (PES calibration documentation for $\sigma_\epsilon$) and P1.3 (state-level seed variance heterogeneity in the joint CI) remain unaddressed, but the abstract fix — the most visible statistical error in Round 1 — is resolved. The bootstrap exchangeability argument in Section 2.3 is a substantive improvement that goes beyond my original requests.

## Evaluation of Revisions

**P1.1 (Abstract/body CI inconsistency)**: Fully resolved. The abstract now states [+15%, +29%] for the all-three-sources CI. The synthesis section's clarification that [+18%, +26%] corresponds to seed+resolution only, and the note that the full three-source CI is "the correct headline figure," is well-stated. The fact that the LaTeX `$|\Delta PP|$` fix in Section 7.2 was also corrected as part of this revision process removes a distracting technical error that would have drawn attention in review.

**P1.2 (PES calibration documentation)**: Not addressed. Section 3.4 still attributes $\sigma_\epsilon \approx 0.015$ to "the Census Bureau's tract-level undercount estimates from the 2020 PES experimental microdata" without specifying the exact PES product, whether small-area estimation was used to derive tract-level values from state-level PES estimates, or what the standard error of $\sigma_\epsilon$ itself is. This remains an important transparency gap for a paper that will be cited as a statistical foundation for legal testimony. The calibration uncertainty in $\sigma_\epsilon$ propagates directly through the delta method CI for census uncertainty, and ignoring it slightly understates the width of the census component.

**P1.3 (State-level seed variance heterogeneity)**: Not addressed. Section 7.2 still uses $\sigma_\text{seed} = 0.010$ as a fixed national mean rather than a population-weighted combination of state-level seed standard errors. The large-state (high-delegation) states (California $k=52$, Texas $k=38$, Florida $k=28$) have systematically higher seed variances than the national mean, and their population weights dominate the national PP average. Using the unweighted national mean in the joint CI computation understates the true seed component of uncertainty for the national +22% claim.

**Bootstrap exchangeability paragraph (Section 2.3)**: Well-executed beyond what I requested. The three-way interpretation (DIA seed point, expected random seed distribution, future application seed) is statistically precise, and the connection to C.0/C.2's geographic dominance finding to justify 50-state exchangeability is the right evidential cross-reference. This is the clearest statement of what the bootstrap CI is actually estimating that appears anywhere in the paper.

**Legal-relevance mapping (Section 7)**: A useful addition that was not in my original requests but directly improves the paper's legal utility. The Wesberry compliance framing for census uncertainty is particularly well-done.

## Remaining Concerns

P1.2 (PES calibration documentation) is the more important of the two unresolved items: it affects the credibility of the census uncertainty CI. P1.3 (heterogeneous seed variances) affects the accuracy of the joint CI, though in the conservative direction (underestimating joint uncertainty is conservative for the headline +22% claim). For a paper intended as a technical foundation for expert testimony, both deserve resolution before journal submission.

## Score: 4 — Accept

The abstract CI fix resolves the most important numerical inconsistency from Round 1. The bootstrap exchangeability argument substantially improves Section 2.3. The remaining items (PES calibration, state-level seed heterogeneity) are documented as limitations in the REVISION-PLAN but do not prevent acceptance given the paper's overall statistical rigor.
