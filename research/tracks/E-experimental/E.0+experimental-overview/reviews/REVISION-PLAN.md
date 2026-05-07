# Revision Plan — E.0 Alternative Representation Systems: A Design Space Analysis
Round 1 avg: 3.0/4

## Score Summary

| Reviewer | Score | Verdict |
|----------|-------|---------|
| Karypis  | 3/4   | Minor Revision |
| Rodden   | 3/4   | Minor Revision |
| Duchin   | 3/4   | Minor Revision |
| Stephanopoulos | 3/4 | Minor Revision |
| Liang    | 3/4   | Minor Revision |
| **Average** | **3.0/4** | **Minor Revision** |

**Gate Status**: PASSED (avg 3.0 >= 2.5, no score < 2.0)

---

## P1 — Required Fixes

### P1.1 — Revise the Pareto-optimality claim for the DIA (Rodden, Stephanopoulos)
**Issue**: The abstract and conclusion claim the DIA "occupies the Pareto frontier for feasible systems" and is "Pareto-optimal." This is not demonstrated --- the paper shows no alternative Pareto-dominates the DIA, but this is not the same as Pareto optimality, and the claim overstates what the analysis shows.
**Fix**: Replace "Pareto-optimal" with "Pareto-non-dominated among the six examined alternatives" throughout. Revise conclusion Section 6.2 to: "Among the six systems examined, no alternative outperforms the DIA on all four dimensions within the same constitutional and statutory constraints. This is not a claim of Pareto optimality for all possible systems, but of non-dominance within the examined portfolio."
**Target**: main.tex abstract, sections/06-conclusion.tex

### P1.2 — Define explicit quantitative thresholds for the 1-3 Pareto scores (Liang, Karypis)
**Issue**: The scoring rubric ("substantially outperforms," "comparable," "substantially below") uses qualitative labels without documented quantitative criteria, making scores non-replicable.
**Fix**: Add a methodology paragraph in Section 3.2 specifying the thresholds used: e.g., "score 3 if the system exceeds the DIA baseline by more than X% on the relevant metric; score 2 if within ±Y%; score 1 if below by more than X%." Report the underlying continuous metric values for each system in an appendix table, so readers can verify the ordinal assignments.
**Target**: sections/03-tradeoffs.tex (Pareto Scoring Rubric subsection)

### P1.3 — Add CI or sensitivity analysis for the 0.015 PP/pp exchange rate (Liang, Rodden)
**Issue**: The compactness--proportionality exchange rate is the paper's most quantitatively precise claim but is presented as a single point estimate derived from the E.5 parametric optimization.
**Fix**: Report the R² of the linear relationship between compactness and proportionality across the E.5 weight parameter range, to test whether linearity is justified. Report a bootstrap CI for the exchange rate. Test whether the rate differs significantly across geographic regions (states with high vs. low urban concentration may show different exchange rates).
**Target**: sections/03-tradeoffs.tex (The Steep Pareto Frontier subsection)

### P1.4 — Specify the VRA benchmark operationalisation (Duchin)
**Issue**: The minority representation dimension is measured against a "VRA Section 2 benchmark derived from Thornburg v. Gingles" without specifying how the benchmark is calculated.
**Fix**: Add a paragraph in Section 3.1 (minority representation dimension) describing the benchmark calculation method: what counts as a majority-minority district, how the Gingles preconditions are operationalised algorithmically (or what proxy is used), and the data source for minority VAP estimates.
**Target**: sections/03-tradeoffs.tex (minority representation dimension)

### P1.5 — Acknowledge the normative status of proportionality as a criterion (Rodden, Stephanopoulos)
**Issue**: Proportionality is presented as an objective evaluation dimension without noting its contested normative and legal status (post-Rucho, courts have declined to treat partisan symmetry as a constitutional requirement).
**Fix**: Add 2--3 sentences in Section 3 (Four Evaluation Dimensions) noting that proportionality is one normative criterion among several, that its legal status in federal courts is unsettled post-Rucho but available as a state constitutional standard in some states, and that the paper uses it as a descriptive dimension rather than a prescriptive requirement.
**Target**: sections/03-tradeoffs.tex

---

## P2 — Suggested Improvements

### P2.1 — Add Moore v. Harper implications to the constitutional taxonomy (Stephanopoulos)
**Issue**: The introduction notes Moore v. Harper but does not draw out its implications for the constitutional taxonomy (expanded state-court review venue).
**Suggestion**: Add 2--3 sentences in Section 4 noting that Moore has created new state-court venues for redistricting challenges and that all "immediately available" systems are now subject to state constitutional review.
**Target**: sections/05-constitutional.tex

### P2.2 — Remove the Wolpert-Macready "no free lunch" citation (Duchin, Karypis)
**Issue**: The NFL theorem applies to optimization algorithms, not to representation systems. The citation is technically incorrect and could be challenged by opponents.
**Suggestion**: Replace with a statement about Pareto dominance: "No system Pareto-dominates all others --- every improvement on one dimension is accompanied by a degradation on at least one other, a structural constraint of the design space."
**Target**: sections/03-tradeoffs.tex

### P2.3 — Distinguish reform contexts: commission states vs. legislative states (Stephanopoulos)
**Issue**: The conclusion advocates DIA adoption without distinguishing states that already have independent commissions from states with legislative redistricting.
**Suggestion**: Add a paragraph in Section 6.3 distinguishing two reform contexts: (a) states with legislative redistricting where DIA adoption is a major reform, and (b) states with independent commissions where DIA adoption is an incremental improvement and multi-member districts are the more significant structural option.
**Target**: sections/06-conclusion.tex

### P2.4 — Report baseline uncertainty from C.7 for the four dimension values (Liang)
**Issue**: The DIA baseline values (PP = 0.367, etc.) are point estimates without uncertainty. C.7 provides CIs for PP.
**Suggestion**: Add a footnote to Table 1 noting that the DIA baseline PP = 0.367 has a 95% CI of approximately [0.356, 0.378] (from C.7) and that alternative system scores that overlap this CI may not be statistically distinguishable from the baseline.
**Target**: sections/03-tradeoffs.tex (Table 1)
