# REVISION PLAN — C.8: Competitive Elections
**Round 1 Panel Scores:** Karypis 3/4 | Rodden 3/4 | Duchin 3/4 | Stephanopoulos 3/4 | Liang 3/4
**Average:** 3.0/4
**Status:** Conditional accept (minor revisions needed to solidify the main empirical claim)

## Critical Issues (Must Fix Before R2)

### C1 — Ensemble Uncertainty Bounds on Competitive District Count [Karypis, primary; Duchin; Liang]
The 85 competitive district figure is from a single random seed. Without an uncertainty range, this is a point estimate that opposing researchers (or litigation opponents) could challenge by showing that a different seed produces substantially fewer competitive districts.
**Action:** Run the redistricting algorithm with 20 independent seeds for the 10 largest multi-district states (accounting for ~75% of competitive districts) and 5 seeds for remaining states. Report the 5th-95th percentile range of competitive district counts across the ensemble. If the range is tight (e.g., 82-88), the 85 figure is robust. If wide, report the median instead of a single seed estimate. This is the single most important revision for this paper.

### C2 — Causal Language Softening [Liang, primary]
The paper's language ("algorithmic maps produce 85 competitive districts") implies a causal claim that the research design does not support. The comparison between algorithmic maps and enacted maps is observational.
**Action:** Soften causal language throughout. Replace "produce" with "are associated with" in the abstract and conclusion. Add a paragraph in the methodology clarifying that the paper estimates a correlation between redistricting method (algorithmic vs. enacted) and competitive district count, not a causal effect.

## Moderate Issues (Should Fix Before R2)

### M1 — Stratify Enacted Map Comparison by Process Type [Duchin]
The "65 competitive districts under enacted maps" aggregates over commission maps, court-ordered maps, bipartisan legislative maps, and partisan gerrymanders. A partisan gerrymander in Ohio produces fewer competitive districts than an AIRC map in Arizona; lumping them together understates the algorithmic advantage over specifically-partisan maps.
**Action:** Separate the enacted map comparison into at minimum two categories: (a) maps drawn by independent commissions or court order (AZ, CA, CO, MI, NY, etc.), and (b) maps drawn by partisan legislatures (TX, OH, FL, NC, GA, etc.). Report competitive district counts separately for each category.

### M2 — Asymmetric Safe-Seat Analysis [Rodden]
The main finding is that safe Democratic seats fall from 208 to 189 while safe Republican seats barely change (162→161). This asymmetry has important implications for algorithmic redistricting's partisan effects in different electoral environments. The paper notes this in passing but does not develop it.
**Action:** Add a section analyzing the asymmetric safe-seat conversion: which states account for the 19-seat reduction in safe Democratic seats? In which electoral environments (wave elections) does this asymmetry favor or disadvantage Democrats? Include the longitudinal projection table data showing how the competitive district count asymmetry evolves under ±4pt shifts.

### M3 — Efficiency Gap / Mean-Median Analysis [Stephanopoulos]
The partisan symmetry claim relies on the 43D/42R competitive-seat split, which is not a standard redistricting fairness metric. Courts and redistricting practitioners are more familiar with the efficiency gap and mean-median difference.
**Action:** Compute the efficiency gap and mean-median difference for both the algorithmic and enacted national maps using the 2020 presidential vote. Report these alongside the competitive-seat split in Table 2.

### M4 — Forward Citation Resolution [Liang]
"dellaLibera2026stability" is cited in Section 6.3 for temporal stability claims that the paper relies on for its cross-decade projection. This citation is not in the reference list.
**Action:** Either (a) remove the cross-decade projection paragraph, replacing it with "B.18 (temporal stability) characterizes boundary changes across reapportionment cycles," or (b) include the stability analysis from B.18 directly.

## Minor Issues (Optional for R2)

- Report states with losses of competitive districts under algorithmic maps (Stephanopoulos)
- Resolve the at-large state treatment (Montana at 2 seats, Karypis)
- Add *Davis v. Bandemer* to the legal background on competitive districts (Stephanopoulos)
- Add population-density correlation analysis to explain the urban-rural safe-seat asymmetry (Rodden)

## Target R2 Score

This paper is already at 3.0/4 average. With C1 (ensemble uncertainty) and M1 (stratified enacted-map comparison), the paper should reach 3.3-3.5/4.
The paper is on a clear path to 3.5/4 without requiring major structural changes.
