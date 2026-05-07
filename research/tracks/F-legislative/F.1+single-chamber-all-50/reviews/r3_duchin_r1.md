# Review 3 — Moon Duchin
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R1
**Score**: 3/4

## Summary

F.1 is the empirical anchor of the F track. My review focuses on the mathematical and methodological claims: the resolution selection threshold, the PP results, and the single-seed design. These are the claims most likely to be challenged in ensemble-aware redistricting research contexts.

## Strengths

The empirical design is clear and reproducible in principle. The use of seed 42 as a primary seed with 5-seed sensitivity checks for case study states is a reasonable approach, and the reported result (PP scores within ±0.005 across 5 seeds for WA, TX, NH, NE) is encouraging. The paired t-tests for county splits (t=4.2, p<0.001) and population balance (t=3.8, p<0.001) are appropriate statistical summaries of the 50-state comparison.

The methodology section's description of the nearest-power-of-2 strategy for non-power-of-2 k values, with citation to B.5/B.6 (dellaLibera2026nwayRecursive), grounds the high-k extension in prior methodological work.

## Concerns

**C1 — Single-map versus ensemble framing.** F.1 generates one map per state (seed 42) and compares it to one enacted map. Modern redistricting analysis — particularly the Markov chain and sequential Monte Carlo literature — evaluates redistricting plans by comparing them to the distribution of valid plans, not to a single alternative. F.1 cannot currently answer: is the algorithmic map near the center of the distribution of valid state house maps, or is it an outlier that happens to score well on compactness? This limitation should be explicitly acknowledged (not buried in the limitations section as "single-seed results") and framed as a direction for future work.

**C2 — PP inflation from block-group resolution needs decomposition.** F.1 reports mean PP = 0.401 at block-group resolution versus 0.381 at tract resolution. F.5 provides the decomposition of this difference into scale effects and resolution effects. However, F.1 presents the block-group PP figure without the decomposition, creating the impression that block-group resolution is simply better than tract resolution for state house redistricting. The resolution effect (approximately +0.013 PP units from F.5's analysis) should be distinguished from the scale effect (+0.027 PP units) even in F.1's presentation.

**C3 — Nebraska k/n_bg = 0.040 versus the 0.05 threshold.** F.1 Table 2 lists Nebraska (49 districts, 570 tracts, k/n = 0.086) as requiring block-group resolution because 0.086 > 0.05. At block-group resolution, Nebraska has 1,231 block groups, giving k/n_bg = 0.040 < 0.05. The paper uses block-group resolution for Nebraska and achieves the best PP score in the block-group tier (0.421). But technically, by the resolution rule's own logic, Nebraska at 0.040 does not require block-group resolution — it falls in the "adequate" tier. This creates a circularity: the rule classifies states based on k/n_tract, which then determines whether block-group is used, but the resulting k/n_bg ratio may be well within the adequate range. The paper should clarify why Nebraska is classified as requiring block-group resolution despite k/n_bg < 0.05.

**C4 — Compactness comparison confound: smaller districts have shorter perimeters.** F.1's headline finding (state house maps more compact than congressional) is mathematically expected (F.5 derives this) but is presented in F.1 without the mathematical justification. A reader comparing the WA congressional map (k=10) to the WA House map (k=98) will observe higher PP in the latter even if both are drawn with identical algorithm quality. F.1 should reference F.5's decomposition to avoid giving readers the impression that the compactness advantage is evidence of better algorithmic performance per se.

## Recommendation

Accept with revisions. The empirical contribution is significant but the framing of the PP results needs clarification. C1 (ensemble context) and C3 (Nebraska threshold circularity) are the most important concerns.
