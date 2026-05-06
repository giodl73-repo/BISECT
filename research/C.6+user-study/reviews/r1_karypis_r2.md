# Review 1 — George Karypis (Algorithmic / Computer Science Lens)
**Paper**: C.6 — Public Perceptions of Algorithmic vs. Human-Drawn Congressional Districts: A Survey Experiment
**Round**: R2
**Score**: 4/4 (Accept)

## Summary

The revision addresses all three Priority 1 issues I flagged in Round 1 satisfactorily. The treatment text is now reproduced in a clearly labeled figure box in Section 3. H2 has been rewritten to match what the design actually tests. H3 has been demoted to exploratory with an explicit power statement. My Round 1 concerns are resolved. I am recommending acceptance.

## Response to Round 1 Issues

**P1-A (Power adequacy for partisan moderation — LEAD).** The revision demotes H3 to ``exploratory'' and adds the power caveat directly in Section 5.3: ``per-subgroup $n \approx 130$--$150$ is sufficient to detect effects of $\geq 0.36$ SD at 80\% power, so the $0.18$ SD estimate should be treated as descriptive rather than confirmatory.'' This is exactly the right framing. The caution about over-interpreting before replication is appropriately placed in the results, not just the discussion. The Discussion (Section 6.3) still describes the partisan gap as ``substantively modest'' without a formal equivalence test (P2-C from my Round 1), but that was a Priority 2 issue and the power caveat now performs most of the necessary epistemic work. Resolved.

**P1-B (H2 testability — LEAD).** H2 is now rewritten as: ``Respondents will rate algorithmic maps no less fair than commission-drawn maps (null hypothesis of equivalence, tested via one-sided $t$-test). A plain-language description of the algorithmic process will increase perceived fairness for algorithmic and commission maps relative to the enacted baseline.'' This is testable from the existing design. The original ``but not enacted maps'' clause that had no corresponding design cell is gone. Resolved.

**P1-C (Treatment text not reproduced — LEAD).** Figure 2 (the treatment text box) reproduces both the algorithmic and commission-drawn process-description texts in full. The algorithmic text appropriately emphasizes what the algorithm cannot do (access partisan or racial data) and what it guarantees (equal population, compactness). The commission text is symmetrically presented. A reviewer can now assess whether the treatment is neutral with respect to the two reform alternatives. Resolved.

## Residual Observations (Not Blocking)

The technical appendix specifying algorithm parameters for stimulus generation (my Round 1 concern about reproducibility) is still missing. This is not a blocking issue for acceptance — the paper correctly delegates this to the companion technical paper — but including a one-paragraph description of the parameter choices (METIS configuration, edge weights, random seed) in an appendix would strengthen the methodological transparency claim the paper makes about algorithmic redistricting in general.

Item 2 of the fairness scale (``the people who drew these districts treated different communities equally'') refers to ``people'' in the algorithmic condition where no people drew the districts. The paper does not flag this as a Likert item validity issue (P4-D from Round 1). Given the high internal consistency (α=0.87) and CFA fit, I accept that this item does not substantially damage the scale, but it should be noted in Section 4.

## Recommendation

Accept. The three P1 issues are resolved. The paper makes a clean, well-powered empirical contribution with appropriate epistemic humility about the underpowered moderation analysis.
