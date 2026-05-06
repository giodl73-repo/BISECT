# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 1
**Score**: 3/4

## Summary

G.5 is primarily a theory paper — the mixing time bounds and spectral gap analysis are mathematical results — so my review focuses on the practical and legal implications. The paper's most important political-science contribution is the empirical mixing-time scaling law ($\hat{t}_{\rm mix} \approx 400k$) and the "when ensemble is irreplaceable" framework. These results speak directly to when redistricting practitioners can trust ensemble evidence and when they should use the AR plan instead.

## Strengths

The two-use-case framework in Section 6 is the clearest articulation in the G-series of what ensembles are actually good for: auditing enacted maps and characterising the feasible space. These are exactly the two things that matter in redistricting litigation. The statement "CS generates one plan; it cannot answer the question 'how unusual is this plan?'" is the key insight that explains why both methods are needed.

The hybrid approach for special masters (Section 6.4) is practically valuable: generate with CS, audit with ensemble, adjust only if the CS plan is an outlier (expected frequency $< 1\%$). This is a workable protocol that a court-appointed special master could actually follow.

The empirical mixing time table (Table 4 in G.4, cross-referenced in G.5) showing that NC chains mix in approximately 2,000 steps and CA chains require 20,000 steps is directly relevant to evaluating the adequacy of published ensemble studies.

## Weaknesses

**The paper does not discuss the political implications of the large gap between theoretical ($10^8$ steps for NC) and empirical ($2{,}000$ steps) mixing times.** If the chain mixes in 2,000 steps in practice because it starts near compact plans and "concentrates near compact regions," then the ensemble distribution the chain explores is effectively weighted toward compact plans — not the uniform distribution over all valid plans. For political geography, this means: plans in the tail of compactness (very non-compact) are under-represented in the ensemble, and the ensemble's partisan distribution reflects the compact region's partisanship, not all valid plans. The paper acknowledges this vaguely ("concentration effect") but does not address its implication for interpreting the ensemble's partisan distribution as a geographic baseline.

**The "expected frequency $< 1\%$" claim (Section 6.4) that the CS plan would be a partisan outlier is stated without derivation or citation.** This is a specific quantitative claim about the G-series' most important practical result. If the CS plan is an outlier in fewer than 1% of cases, this follows from the B.7 50-state sweep data — but the paper does not show this. The claim should either be derived from the sweep data or labeled as an estimate.

**The practical mixing rate $\hat{t}_{\rm mix} \approx 400k$ has not been validated for large states.** For NC ($k = 14$), the prediction is $400 \times 14 = 5{,}600$ steps — but the measured mixing time is 2,000 steps. For WI ($k = 8$), it predicts $3{,}200$ but measured is 1,500. For CA ($k = 52$), it predicts $20{,}800$ and the measured is 20,000 — good agreement. The formula over-predicts for small states and under-predicts for large states (not shown, but the CA prediction is close). A confidence interval for this scaling relationship would make it more credible.

## Minor Issues

- The paper mentions that most published redistricting studies use "50,000–100,000 steps without reporting convergence diagnostics." This is an important critique but should be supported by a census of published ensemble studies — how many actually lack convergence reporting?
- The claim that starting from a compact plan gives "faster apparent convergence but would not certify stationarity" (noted in G.4) is not discussed in G.5. Since G.5 proposes a hybrid approach where the initial plan is the CS output (a compact plan), this could affect the interpretation of ensemble certification.

## Recommendation

Accept with minor revisions. Add derivation or citation for the $< 1\%$ outlier frequency claim. Discuss the implication of start-from-compact-plan concentration for the ensemble's representativeness of the full plan space.
