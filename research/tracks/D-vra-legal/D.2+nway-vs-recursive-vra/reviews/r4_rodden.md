# Review — D.2: N-Way vs. Recursive VRA
**Reviewer**: Jonathan Rodden (Political geography, gerrymandering)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

D.2 provides a rigorous empirical comparison that political scientists and legal scholars will find useful. The central finding — that n-way and recursive bisection produce equivalent VRA success rates at 1/3 the computational cost — is both credible and important for practice. My primary concern is the lack of geographic analysis of where and why the two methods diverge at the state level, which would help practitioners understand when to expect divergence from the national pattern.

## Strengths
- The scale (1,760 runs, 44 states) enables reliable national-scale generalization.
- The geographic variation table showing state-level wins for each method is appropriately honest: equivalence nationally does not mean interchangeability locally.
- The parameter preference analysis (n-way favors lower thresholds, recursive favors higher) connects algorithm structure to parameter sensitivity in a way that has practical guidance value.

## Concerns
- **Geographic clustering as missing variable**: The paper does not analyze whether geographic clustering of minority populations (Moran's I or similar) predicts which method performs better at the state level. Virginia's +35% n-way advantage and Connecticut's +45% recursive advantage may reflect geographic structure (dispersed vs. clustered minority populations) rather than arbitrary method behavior. A regression of method_advantage ~ geographic_clustering would provide insight and potentially actionable guidance.
- **Regional patterns**: The state-specific results are presented in a table but not analyzed regionally. Are the n-way wins concentrated in a particular type of state (e.g., Midwest urban-dispersed minority populations)? Are the recursive wins in New England states with compact urban minority communities? This regional analysis would strengthen the paper's contribution to the political geography literature.
- **Partisan implications**: The paper focuses on minority opportunity district formation but does not examine whether method choice affects the partisan composition of non-minority districts. If one method systematically produces more compact districts (and compactness affects partisan outcomes via geographic sorting), then method choice may have partisan implications beyond VRA.

## Required Changes (P1/P2)
- **P2**: Add a brief regression analysis of state-level method advantage as a function of geographic clustering (Moran's I or minority population concentration) and state size. This would move the finding from "some states differ" to "here is why."
- **P2**: Add a regional summary of state-specific results (South, Midwest, Northeast, West) to identify whether method advantages cluster geographically.
