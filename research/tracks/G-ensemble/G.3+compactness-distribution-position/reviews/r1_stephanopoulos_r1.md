# Review: G.3 — Compactness Distribution Position
**Reviewer**: Nicholas Stephanopoulos (Election law, redistricting doctrine)
**Round**: 1
**Score**: 3/4

## Summary

G.3 addresses the legal defensibility of compact redistricting plans — a topic with direct doctrinal relevance since Shaw v. Reno established compactness as a constitutional proxy for non-gerrymandering. The paper correctly notes that very high compactness (above the 90th percentile) could itself be suspicious if it correlates with partisan packing. The AR plan's 61st–72nd percentile position is therefore in the legal "sweet spot" — more compact than average but not suspiciously extreme.

## Strengths

The legal defensibility argument (Section 6) is the paper's most valuable contribution. The paper correctly notes that a plan at the 99th percentile of compactness could be challenged as a "compactness route" to partisan packing (a concern raised in the VRA context, where concentrating minority voters produces compact minority-opportunity districts). The 61st–72nd percentile range avoids this challenge while still demonstrating non-gerrymandered geometry.

The Shaw v. Reno citation is appropriate, though the paper should also cite the later cases (Miller v. Johnson, Bush v. Vera) that refined the role of compactness in racial gerrymandering analysis. These cases established that "irregular" district shapes are probative but not dispositive of racial sorting — the same doctrinal framework could apply to partisan compactness analysis.

The compactness-partisan correlation documentation ($\rho \approx -0.09$ to $-0.31$) is directly relevant to the "compactness as partisan weapon" concern. The paper correctly argues that since the correlation is a property of geography (not algorithm design), and since the AR plan's partisan outcome is near the ensemble median despite its above-median compactness, the algorithm has not exploited the compactness-partisan correlation to achieve a partisan end.

## Weaknesses

**The paper does not engage with the line of cases addressing algorithmic compactness as a redistricting criterion.** In Rucho v. Common Cause, the majority noted that plaintiffs had proposed compactness-based standards for identifying gerrymanders, and found them insufficient — not because compactness is irrelevant, but because any standard must be judicially manageable. An algorithmic plan that optimizes compactness directly raises similar manageability concerns: who decides that minimum edge cut is the right compactness criterion? This objection should be addressed.

**The distinction between compactness as a method (what the algorithm optimizes) and compactness as a result (the PP score) is legally significant but underdeveloped.** The AR algorithm optimizes edge cut, not Polsby-Popper. Courts examining compactness typically use PP or Reock scores as the metric. The paper should discuss whether a court would treat "minimum edge cut" and "Polsby-Popper in the 68th percentile" as equivalent for legal purposes, or whether the optimization metric matters separately from the outcome metric.

**The "not an outlier" framing could work against the AR plan in some legal contexts.** If the AR plan is not an outlier on compactness, a challenger might argue it is also not "specially compact" in any distinctive sense — the algorithm doesn't produce a uniquely compact map, just a typical one. The paper should address how to frame the compactness result as a positive legal attribute rather than merely the absence of a negative one.

## Minor Issues

- The paper cites Shaw v. Reno via `\shaw` macro but does not include the full citation in the text. Courts require full citations.
- The "right to representation" discussion in the legal defensibility section (Section 6, referenced in the outline) should distinguish between racial gerrymandering claims (where compactness plays a specific Shaw role) and partisan gerrymandering claims (where compactness is a general non-gerrymandering proxy).

## Recommendation

Accept with moderate revisions. Add engagement with the "algorithmic manageability" objection and clarify the relationship between edge-cut optimization and Polsby-Popper as a legal metric.
