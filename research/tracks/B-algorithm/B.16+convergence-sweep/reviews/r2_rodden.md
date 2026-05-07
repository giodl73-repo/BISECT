# R2 Review — Jonathan Rodden
**Paper**: B.16 ConvergenceSweep: T=600 Statutory Seed Formula
**Round**: 2
**Score**: 4.0/4
**Verdict**: Accept

## Assessment of P1 Items from R1

### Table 1 j* column — RESOLVED

The j* column in Table 1 is fully populated for all 50 states. Every state has a specific j* value; single-district states are correctly listed as j*=0 with an explanatory note in the caption ("For single-district states (k=1), j*=0 because the first seed is trivially optimal"). A T_needed column is also added (T_needed = tau + 1), making the minimum certification threshold directly readable alongside each state. The caption specifies that confirmed values for Georgia, Florida, Texas, and Wisconsin come from the B.7 sweep log, with all other j* values read from the same sweep data. This is the complete dataset practitioners need for 2030 planning.

### Georgia partisan case study — RESOLVED

A new "Georgia Partisan Case Study" subsection (Section 3.3, labelled sec:georgia-partisan) addresses this directly and correctly. The analysis identifies that the T=500 halt plan and the T=600 certified plan are in fact the same plan: no seed between s_0+490 and s_0+499 achieves a lower EC_norm than the plan found at s_0+489, so T=500 halts carrying the same partition that T=600 subsequently certifies. The Democratic seat count is therefore identical. The key finding box states this explicitly: "The T=600 certification matters for geometric purity and legal reproducibility — not for partisan outcome — in the 2020 Georgia instance. T=500 cannot certify its result; T=600 can. Both produce the same map." The prospective risk framing is also correct: a 2030 Georgia graph with a tail between 501 and 600 would produce a suboptimal plan under T=500. This is the precise answer a political science audience needs — it distinguishes the procedural failure of T=500 from its partisan consequences in the 2020 instance, and correctly frames the certification gap as a forward-looking statutory concern.

### "Approximately one standard deviation" arithmetic error — RESOLVED

The arithmetic is corrected consistently in both Section 3 (Statutory Recommendation subsection) and the conclusion. The revised Section 3 text now reads: "For the fitted Gumbel distribution with sigma-hat = 150, one standard deviation is sigma * pi/sqrt(6) ≈ 150 × 1.28 ≈ 192 seeds. The 89-seed margin is therefore approximately 89/192 ≈ 0.46 standard deviations above the empirical worst case." The conclusion's "Final Note on Statutory Precision" paragraph repeats the calculation in full: "approximately 0.46 standard deviations of the fitted Gumbel distribution (the standard deviation is sigma-hat * pi/sqrt(6) ≈ 150 × 1.28 ≈ 192 seeds; 89/192 ≈ 0.46)." The correction is applied in both locations where the erroneous "one standard deviation" claim appeared. The underlying recommendation — T_stat=600 — is unchanged.

## Remaining Concerns

### State characteristics as predictors of tail length (P2) — PARTIALLY ADDRESSED

My P2 request for an informal regression note identifying which state properties predict longer convergence tails is partially addressed. Table 1 now includes the district count k for each state, and the descending-tau ordering makes it visually apparent that high-k states (Georgia 14, Florida 28, Michigan 13, Texas 38) cluster at the top. The Wisconsin case (k=8, j*=1023, tau=377) is explained in the text as representing a distinct difficulty profile: long sweep to find the minimum, but short tail once found. However, there is still no explicit statement of the form "states with k >= 10 and a concentrated urban core tend to have tau > 300." One sentence of qualitative guidance would help practitioners flag hard cases before the 2030 sweep. This is a P2 item and does not block acceptance.

### Compactness-proportionality tradeoff (P2) — ADDRESSED

The conclusion references B.0 and B.12 and notes that redistricting authorities seeking proportionality should consult the proportional-weights configuration established in B.12. This is adequate.

## Recommendation

Accept. All three P1 items are fully resolved. The j* column is complete with T_needed added as a useful bonus. The Georgia partisan case study correctly identifies the 2020 result — same map, certification-only distinction — and frames the prospective risk accurately. The 0.46 SD correction is applied consistently in both relevant sections. The remaining P2 gap on tail-length predictors is a marginal improvement that does not affect the statutory claims. The paper is ready for publication and supports removal of the T=600 contingency footnotes in G.0/G.14/H.0.
