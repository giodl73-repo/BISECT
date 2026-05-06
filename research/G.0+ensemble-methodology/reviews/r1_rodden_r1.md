# Review: G.0 — Ensemble Methodology
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 1
**Score**: 3/4

## Summary

G.0 provides a framework paper that situates the AR algorithm within ensemble redistricting methodology. From a political science perspective, the paper's most important contribution is establishing clear vocabulary: ensembles characterise what geography permits (Goal A), while the AR algorithm computes one canonical plan (Goal B). The statutory implications section makes a novel claim — that a deterministic plan can bypass the Rucho manageability problem entirely — that merits engagement from political scientists even if it is primarily a legal argument.

## Strengths

The paper correctly distinguishes the ensemble as characterising the geographically determined distribution of outcomes versus the AR plan as a single point in that distribution. This is essential context for understanding why the Georgia result (38th percentile of Democratic seats) is not evidence of algorithmic bias but of geographic sorting. The treatment of this distinction in Section 6 is the paper's strongest contribution to political science readers.

The acknowledgment that "ensemble does not tell you what the correct map is" is refreshing. Much of the ensemble literature in political science papers overstates what ensemble comparisons establish — they identify outliers, they do not define correct outcomes. G.0 is appropriately modest.

The short-burst methods discussion correctly characterises these as optimisation tools rather than distributional samplers. Courts and political scientists sometimes conflate the two, and the clarification matters for evidence interpretation.

## Weaknesses

**The paper does not engage with the geographic-sorting literature on its own terms.** The Rodden effect is mentioned in passing (Section 5.2), but the framework paper should more explicitly address what it means for the AR plan to fall "near the partisan median" when the geographic median is itself skewed relative to proportionality. A plan at the 54th percentile of an ensemble that centers on 7D/7R for a state with 49.1% Democratic vote share is consistent with slight Republican over-representation — but this nuance is absent. The framework paper should acknowledge that ensemble-median outcomes are not the same as fair outcomes.

**The legal argument in Section 6 is asserted more than argued.** The claim that the AR plan "bypasses Rucho by operating at the Article I §2 level" is a significant legal claim that requires more than a paragraph to establish. The argument is that courts can adopt the AR plan without engaging in the gerrymander-detection inquiry — but this requires showing that the Article I §2 derivation is genuinely mandatory rather than permissive. The paper treats it as given.

**Table 1 omits the Polsby-Popper-optimised variants of ensemble methods** (e.g., Metropolized Forest ReCom, which does weight for compactness). The table entry for "Compactness weight: None" for all ensemble methods is incorrect once Metropolized variants are included.

## Minor Issues

- The paper should define "geographic baseline" more precisely. Different ensemble studies use different electoral baselines (presidential vote, composite index, etc.), and the percentile result depends on this choice. G.0 should flag this as a calibration parameter that must be held constant when comparing AR plans across states.
- The assertion that the AR plan is "politically inert" is correct in the sense that no partisan data enters the algorithm, but it is somewhat misleading for the framework paper: the output of the algorithm does have partisan implications that vary by state, and a naive reader might misinterpret "politically inert" as "partisan-neutral."

## Recommendation

Accept with moderate revisions. Add a paragraph acknowledging that ensemble-median outcomes are not normatively neutral in sorted states, and sharpen the legal argument in Section 6.
