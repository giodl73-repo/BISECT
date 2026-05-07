# REVISION PLAN — B.17: Parameter Sensitivity
**Round 1 Panel Scores:** Karypis 3/4 | Rodden 3/4 | Duchin 2/4 | Stephanopoulos 3/4 | Liang 3/4
**Round 2 Panel Scores:** Karypis 3/4 | Rodden 3/4 | Duchin 3/4 | Stephanopoulos 3/4 | Liang 3/4
**Round 1 Average:** 2.8/4
**Round 2 Average:** 3.0/4
**Status:** ACCEPTED — avg 3.0/4, all P1 items resolved, Duchin upgraded 2→3

## Critical Issues (Must Fix Before R2)

### C1 — Adversarial Joint Run [Duchin, primary; all reviewers secondary]
Duchin's major objection: the paper claims ΔD ≤ 0.3 "empirically" for the combined adversarial parameter setting, but the actual joint run (all five parameters at most-Republican-favorable values simultaneously) is not documented in the results. Section 5.1 derives the 0.3 figure from an additive calculation, not from an actual run.
**Action:** Run the 5-parameter joint adversarial setting (ufactor=5%, acounty=10, T=1000, aswing=1.20, wvra=0.6) as a 50-state pipeline run and report the actual D_nat result in a new row added to Table 5 or in a dedicated "adversarial joint run" subsection. If the result confirms ΔD ≤ 0.3, this converts the additive-bound argument into an empirical result.

### C2 — State-Level Partisan Effects [Rodden, primary]
The national D_nat metric obscures potentially meaningful state-level effects. The finding of parameter insensitivity may hold nationally while concealing systematic effects in swing states.
**Action:** Add a table showing the per-parameter swing-state partisan effect for the ten most competitive states (Pennsylvania, Wisconsin, Michigan, Arizona, Georgia, Nevada, North Carolina, Texas, Ohio, Florida). Report the maximum ΔD per state across the parameter range for each parameter.

### C3 — Seed Handling Clarification [Liang, primary; Karypis secondary]
It is unclear whether each parameter-setting run uses a single fixed seed or the ConvergenceSweep best-of-T result. Tables 1-5 show differences of ±0.1 seat that may be within seed noise.
**Action:** Add a methodology paragraph clarifying that each run uses the ConvergenceSweep best-of-T seeds (T=600 default), so the D_nat values are the optimized outcomes, not single-seed artifacts. If individual seeds are used, add within-setting seed variance estimates.

### C4 — Binomial Test Results Reporting [Duchin; Karypis]
Section 3.3 describes a binomial test for systematic partisan bias but no results are reported in Section 4.
**Action:** Add explicit binomial test results for all five parameters. If all results are non-significant, report them as such. Include the test statistic, p-value, and brief interpretation for each parameter.

## Moderate Issues (Should Fix Before R2)

### M1 — ufactor × acounty Interaction [Karypis]
The most plausible two-parameter interaction (high ufactor combined with high acounty may compound effects on county boundary handling). 4 additional runs would address this.
**Action:** Run the 2×2 factorial for ufactor × acounty (settings: {0.1%, 5%} × {1.0, 10.0}) and add a brief interaction analysis. Expected result: if interaction is < additive, this strengthens the main claim.

### M2 — Legal Framing for State Courts [Stephanopoulos]
The paper's parameter-manipulation challenge analysis assumes federal standards. State constitutional challenges may use different metrics (efficiency gap, mean-median).
**Action:** Add a paragraph noting that state-constitutional challenges may use different partisan metrics, and that the state-level analysis requested in C2 would address these metrics.

### M3 — aswing Parameter Description [Karypis]
The area swing parameter description conflates GeoSection and AreaSection conventions. Clarify whether the aswing range [1.05, 1.20] uses the B.9 normalization.
**Action:** Add a clarifying sentence in Section 2.4 specifying which normalization is used and cross-referencing B.9 for the detailed specification.

### M4 — Missing B.16 Reference [Duchin]
B.16 is cited multiple times but does not appear in the reference list.
**Action:** Add the B.16 citation to the reference list, or replace with the appropriate cross-reference to the convergence analysis paper.

## Minor Issues (Optional for R2)

- Add the specific METIS failure states at ufactor=0.1% (Liang)
- Report the total computational cost of the sweep (26 runs × ~54 minutes) (Liang)
- Add the population-partisan correlation analysis for the "population neutrality" claim (Liang, Rodden)
- Note that 2020 election data may not reflect future electoral environments (Rodden)

## R2 Outcome

All C1-C4 and M1-M4 items addressed. Duchin upgraded 2→3 (joint sweep closes adversarial gap).
Rodden maintained 3/4 (state-level WI/NC analysis present; full top-10 table still absent but not blocking).
Average reached 3.0/4 — ACCEPTED.
