# Review 3 — Reviewer: Moon Duchin (Metric Geometry / Ensemble Methods)
**Paper:** D.5 — Quantifying VRA Section 2 Evidence with Algorithmic Redistricting: A Gingles Prong-by-Prong Methodology
**Round:** 1
**Score:** 3/4

## Summary

This paper addresses the intersection of ensemble redistricting methodology and VRA expert testimony — a connection that the redistricting mathematics community has not fully developed. The alignment score for Prong 1 is a novel contribution that formalizes what has previously been done by visual inspection. The Prong 3 methodology is technically sound. My concerns are about the ensemble-based Prong 1 analysis and the interaction between the Prong 1 compactness threshold and the ensemble distribution.

## Strengths

The alignment score formula (Equation 2.1) is a principled, reproducible alternative to visual inspection for Prong 1. The ratio of district-level minority CVAP to COI-level minority CVAP is the right geometric quantity: it measures how well the proposed district captures the minority community's geographic core, independent of the total size of the community. The 0.5 threshold is conservative (capturing at least half the community's core) and empirically grounded in prior judicially accepted districts.

The ensemble-based Prong 1 robustness check (fraction of configurations satisfying Prong 1 across 1000 ensemble runs) is an important methodological innovation. It moves from "can we draw a compliant map?" (yes/no) to "how frequently does a neutral algorithm draw a compliant map?" (probability), which is a stronger legal standard. An opponent cannot argue that the single proposed map is an outlier if 95% of ensemble configurations also satisfy Prong 1.

The deposition readiness discussion (Section 7.2) and the redist depo module are important practical contributions. The SHA-256 hash chain on all outputs addresses a real discovery problem: expert witnesses in redistricting cases frequently face challenges about whether reported outputs match what the algorithm produced, and a tamper-evident log answers this.

## Weaknesses and Concerns

The ensemble-based Prong 1 robustness check has a technical problem that the paper does not address. The ensemble generates maps with different random seeds, all satisfying population balance and compactness constraints. For Prong 1, we want to know: across the ensemble, does the minority community appear as a majority in a single district? But the ensemble generates *full state maps* — it does not specifically generate the proposed minority-opportunity district configuration. A map in the ensemble might have a majority-minority district in a different location than the "proposed" district (because the algorithm found a different compact configuration). The alignment score for "the district" in an ensemble configuration is only meaningful if the ensemble configurations are compared to the same geographic core.

The paper should clarify: when it says "Prong 1 is robust if the alignment score exceeds 0.5 across at least 90% of ensemble configurations," does it mean (a) 90% of ensemble configurations have *some* district with alignment score > 0.5 relative to the COI, or (b) 90% of ensemble configurations have the *same* district (in the same geographic location) with alignment score > 0.5? These are different questions with different legal implications. Option (a) answers "can a compliant map be drawn?" while option (b) answers "does this specific configuration arise consistently?"

The conclusion (Section 8) notes as a limitation that "the Holm correction controls family-wise error rate but may be conservative in elections with high correlations; the Benjamini-Hochberg procedure may be more appropriate in such settings." This is correct, but it should be addressed earlier in the paper as a methodological choice rather than left to the limitations. If Holm is conservative and B-H is more appropriate, why not use B-H? For legal contexts, conservatism in the multiple-testing correction is actually desirable from the plaintiff's perspective (if the finding survives a conservative correction, it is harder to attack), so Holm is defensible. But the paper should make this argument explicitly rather than treating B-H as an alternative without reasoning.

## Minor Issues

- The paper uses "disentanglement" throughout to describe the *Callais* methodology requirement. This is consistent with the *Callais* framing but the term does not have a standard statistical meaning. The paper should note that "disentanglement" refers specifically to partial-correlation analysis that isolates racial from partisan effects, to avoid confusion with other statistical "disentanglement" methods.
- The LOO analysis is described in the text (Section 4.4) and referenced in the expert checklist (Table 2), but the Alabama LOO table is mentioned ("The LOO analysis shows that excluding any single election does not change the Prong 3 conclusion") without being shown. The LOO table should be included in the worked example.
- Equation numbers: the paper has three sections with equations but the Prong 2 WLS model (Section 3.2) does not have a numbered equation, while the Prong 3 model (Section 4.3) does (Equation 3.1). This inconsistency should be corrected: the Prong 2 model should also be numbered for easy reference.

## Recommendation

Accept with minor revisions. Clarify the ensemble-based Prong 1 robustness check (option (a) vs. option (b) interpretation), defend the Holm vs. B-H choice explicitly, and include the Alabama LOO table in the worked example.
