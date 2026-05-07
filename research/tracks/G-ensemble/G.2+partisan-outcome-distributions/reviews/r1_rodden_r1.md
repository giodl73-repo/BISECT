# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 1
**Score**: 3/4

## Summary

G.2 addresses the question I care most about: does the AR algorithm systematically favor one party? The answer — "no, in five of six states" — is defensible given the geometric logic of minimum edge-cut partitioning on a geographically sorted voter map. The paper's treatment of the Rodden effect in the Georgia context is correct. My concerns are with the proportionality corridor analysis and with the paper's framing of what "near the ensemble median" means in terms of seat share fairness.

## Strengths

The binomial-with-overdispersion model for the ensemble partisan distribution is a useful simplification. The identification of $p_{\rm geo}$ as the geographically determined win probability correctly captures why ensemble medians differ from proportional outcomes in sorted states. This is the right model for explaining the Georgia result.

The proportionality corridor analysis (Section 4) is a genuine contribution. Defining the corridor as outcomes within one seat of strict proportionality and computing the corridor fraction $\phi$ for each state provides a natural benchmark. The finding that AR falls inside the corridor for competitive states (NC, WI, PA) and outside for sorted Georgia is exactly as expected from the geographic sorting theory.

The paper correctly notes that "falling inside the proportionality corridor does not mean the plan is proportional by design" — this is an important disclaimer that prevents misuse of the corridor result.

## Weaknesses

**The paper's framing in the abstract is misleading: "near the 50th percentile of partisan outcomes" is not the same as "near proportional."** For North Carolina with $v_D = 0.491$, the 54th percentile of the ensemble means the AR plan matches the ensemble median, which corresponds to approximately 7D/7R. But strict proportionality would be 6.87 Democratic seats — essentially 7D. So the AR plan happens to be both at the median and near-proportional for NC. For Georgia, however, the ensemble median is 6D/8R for a state with $v_D = 0.464$, which corresponds to $6/14 = 42.9\%$ Democratic seats vs. 46.4% Democratic vote share. The ensemble median itself represents approximately 3.5 points of Republican over-representation due to the Rodden effect. The paper does not acknowledge this: being at the ensemble median in a sorted state means being systematically skewed relative to proportionality, even if one is not an outlier relative to geography.

The paper should add a paragraph explicitly acknowledging that the ensemble median in sorted states does not represent a proportional outcome, and that the "near median" finding means "consistent with geography" not "consistent with proportionality."

**The "algorithm choice determines which tail you land in" (Section 5) is a significant claim that deserves empirical support.** The paper asserts that the minimum-edge-cut objective positions the AR plan in the "interior" of the partisan distribution rather than in either tail. This is presented as a general principle, but the logic is not obvious: why would minimum edge cut produce median partisan outcomes rather than, say, the most compact-Democrat outcome? The B.0 bakeoff is referenced but not described. A figure comparing several algorithm choices (minimum edge cut, maximum compactness, population-variance minimization) on their partisan distributions would make this section empirically grounded.

**The corridor fraction for Georgia (28%) implies that 28% of random valid GA plans produce 6D or 7D seats.** The AR plan produces 5D, which is outside the corridor. But the paper does not adequately address the question of whether a court should care about this. If 28% of random plans achieve the corridor, and the AR plan does not, is this a problem? The paper's answer is essentially "no, because geography explains the deviation" — but this is exactly the debate that Rodden (2019) and Stephanopoulos-McGhee (2015) have extensively litigated. The paper should engage more directly with whether geographic inevitability is a sufficient explanation.

## Minor Issues

- The Georgia $p_{\rm geo} \approx 0.36$ value corresponds to approximately 5D seats in expectation under a pure binomial, which would make the AR result of 5D exactly equal to the geometric expectation — not below it. But the paper describes the ensemble median as 6D. This needs explanation: if the geometric mean is 5D, why is the ensemble median 6D?
- The Minnesota result in Section 4.3 lacks a source for the ensemble data. This is an additional state introduced without published ensemble comparison and should be marked with \est.

## Recommendation

Accept with moderate revisions. The key addition needed is an explicit acknowledgment that ensemble-median outcomes in sorted states are themselves not proportional, and that this has implications for interpreting the Georgia result.
