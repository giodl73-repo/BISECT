# U.1 ConvergenceSweep — R1 Panel Synthesis

**Round**: 1
**Date**: 2026-05-07
**Paper**: U.1 ConvergenceSweep: T=600 Statutory Seed Formula

## Scores

| Reviewer | Score | Verdict |
|----------|-------|---------|
| Karypis | 3/4 | Minor Revision |
| Rodden | 3.5/4 | Minor Revision |
| Duchin | 3/4 | Major Revision |
| Liang | 3/4 | Minor Revision |
| Stephanopoulos | 3.5/4 | Minor Revision |
| **Mean** | **3.2/4** | |

## Consensus

All five reviewers affirm the paper's central empirical contribution: Georgia's 511-seed convergence tail is a concrete, verifiable result that establishes T=500 as certifiably insufficient and T=600 as adequate with an 89-seed margin. Karypis (METIS developer) confirms the characterisation of METIS as a heuristic and the convergence sweep design as the strongest practical guarantee available. Rodden endorses the statutory engineering design. Stephanopoulos finds the paper close to publication-ready as a legal contribution. The SHA-256 content-derived seed formula and the Certificate of Seed Neutrality are praised across the panel as genuine innovations — Duchin specifically notes the correct distinction between unpredictability and temporal neutrality, and Stephanopoulos identifies the Certificate as a litigation artifact no prior redistricting proposal has offered.

Two algorithmic specification issues carry blocking weight. Duchin and Karypis independently flag the same pair of problems: EC_norm is defined with a recursive bisection tree denominator but applied to full k-way partitions in Algorithm 1, creating an underspecified comparison in the sweep loop; and Theorem 1's exponent is stated as O(n^{2k}) when the argument given yields O(n^{2(k-1)}). The Gumbel model's i.i.d. assumption across 50 structurally heterogeneous graphs is flagged by Duchin and Karypis as requiring explicit acknowledgment and a goodness-of-fit test. Rodden flags a missing complete j* column in Table 1 and an arithmetic error in the "one standard deviation" characterisation of the 89-seed margin. Liang identifies a METIS determinism overstatement for parallel builds that could allow verifiers to obtain a different plan and claim non-reproducibility. Stephanopoulos identifies three statutory drafting gaps, the most consequential being the absence of any administrative mechanism to raise T_stat if the 2030 census produces a harder state.

The paper is strong enough to accept after revision and does not require a new round of data collection. All P1 items are corrections to existing text or additions of specified missing content (Table 1 data, statutory clause). No new empirical runs are required beyond the already-completed B.7 sweep.

## P1 Items (Blocking)

1. **[Duchin + Karypis] EC_norm definition inconsistency.** Section 2.1 defines EC_norm using a recursive bisection tree formulation with denominator sqrt(min(i, k_l - i)) at each bisection level l. Algorithm 1 applies EC_norm to full k-way partitions produced by a direct METIS k-way call, for which no recursive tree exists. The paper must resolve this with a single unambiguous definition: either (a) METIS is always called as a bisector in a recursive tree, with EC_norm summing over bisection levels, or (b) METIS produces a full k-way partition, with EC_norm = EC(Pi) / sqrt(k/2). This also raises comparability of convergence tails tau across states using different call modes.

2. **[Duchin + Karypis] Theorem 1 exponent error.** The paper claims O(n^{2k}) distinct minimum-edge-cut k-partitions, but the stated argument — a recursive bisection tree with k-1 internal nodes, each a bisection cut, giving O(n^2)^{k-1} = O(n^{2(k-1)}) — yields O(n^{2(k-1)}). Additionally, the Euler-formula bound invoked to get O(n^2) cuts-per-level applies to s-t cuts, not balanced bisection cuts. Fix the exponent to O(n^{2(k-1)}) and either correct the cuts-per-level argument or replace Theorem 1 with the simpler correct statement: "the METIS seed space is finite because each local optimum is a distinct partition of a finite set, and the sweep terminates in finite time."

3. **[Duchin] Gumbel i.i.d. assumption must be stated explicitly.** The 50 convergence tails are from 50 structurally different graphs (different n, k, geography). The Gumbel extreme-value theorem applies to the maximum of i.i.d. samples from the same distribution — which is not the case. The paper must explicitly state that the Gumbel model treats these 50 heterogeneous observations as approximately i.i.d. (an empirical assumption, not a consequence of extreme-value theory), add a KS goodness-of-fit test, and clarify that the statutory recommendation rests primarily on the empirical 89-seed margin, not on the Gumbel tail bound.

4. **[Rodden] Table 1 j* column is incomplete.** The j* (last improving seed index) column in Table 1 is populated only for Georgia (489) and Wisconsin (1023), with em-dashes for all other states. The full column must be populated from the B.7 sweep log, as j* together with tau determines T_needed and practitioners running the 2030 sweep need the complete dataset.

5. **[Rodden] Georgia partisan case study missing.** The paper shows T=500 fails Georgia geometrically but does not report whether the T=500 termination plan and the T=600 certified optimum differ in Democratic seat count. A Georgia case study comparing D seat counts and EC_norm at the two plans is required for a political science audience to assess whether the certification gap is politically consequential.

6. **[Rodden] "Approximately one standard deviation" arithmetic error.** The 89-seed margin is characterised as "approximately one standard deviation" above the observed worst case. For Gumbel(mu, sigma=150), the standard deviation is sigma * pi / sqrt(6) ≈ 1.28 * 150 ≈ 192 seeds. The 89-seed margin is approximately 0.46 standard deviations, not 1.0. Correct this arithmetic throughout.

7. **[Liang] METIS determinism claim too strong for parallel builds.** Proposition 1 states ConvergenceSweep is deterministic "regardless of hardware, operating system, or parallelisation strategy applied to independent METIS calls." METIS's OpenMP-parallel variants use a non-deterministic work-stealing scheduler, so two runs with the same seed but different thread-scheduling outcomes may differ. Qualify Proposition 1 to specify that the determinism guarantee requires single-threaded METIS (METIS_OPTION_NTHREADS=1) and add a note that the statutory build specifies this flag explicitly.

8. **[Stephanopoulos] Section 5.4 administrative process clause for T_stat is absent.** The conclusion states the statute provides for an administrative process to raise T_stat without full legislative amendment, but Section 5.4 clause (C) contains no such mechanism. Add statutory language specifying how T_stat can be raised after the 2030 sweep — either an EAC regulatory clause with a public comment period, or a provision requiring congressional action, with an interim provision for states whose tails exceed 600.

9. **[Stephanopoulos] Version string not bound to census cycle.** Section 4.3 notes a future DIA amendment would use "DIA_SEED_V2," but neither Section 4.3 nor Section 5.4 specifies that the version string in effect at census release governs the full redistricting cycle. Without this, a 2032 challenge to a 2021 V1 map could argue V2 should have been used. Add statutory language binding the version string to the census cycle.

10. **[Stephanopoulos] "24-hour statutory window" reference is inconsistent with the 30-day submission deadline.** Section 2.4 compares the Texas sweep runtime to "the statutory 24-hour window," but standard redistricting statutes specify a 30-day submission deadline. Either add the 24-hour computation sub-requirement to the statutory text in Section 5.4, or remove the comparison and reference only the 30-day deadline.

## P2 Items (Important)

1. **[Karypis] T_prac vs. T_stat computational cost comparison.** Table 5 should add a column showing seed count and runtime at T=500 versus T=600 for the five worst-case states, to help practitioners assess the cost of choosing T_prac for research runs.

2. **[Karypis] Block-level resolution scaling argument.** The paper correctly flags n ~ 1.5M block-level graphs as untested. Add a theoretical argument or empirical bound on how convergence tail length scales with n (since METIS complexity is O(m log n), a 100x increase in n changes per-seed time by ~100x and may change the tail distribution).

3. **[Rodden] State characteristics as predictors of convergence tail length.** Table 1 sorts by tau but does not include n, k, or a geographic concentration measure. Add an informal regression note or footnote identifying which state properties (k, n, urban concentration) predict longer tails, to help practitioners flag likely hard cases before the 2030 sweep.

4. **[Rodden] Acknowledge the compactness-proportionality tradeoff.** ConvergenceSweep produces the minimum EC_norm plan, which in states with urban Democratic cores tends to pack Democratic votes. Add a brief acknowledgment that the DIA accepts this tradeoff in exchange for algorithmic determinism, with a cross-reference to B.0's established analysis.

5. **[Liang] Table 5 measured vs. estimated runtime labelling.** The caption says "estimated ... based on observed METIS call times" but the j_stop column implies sweeps were actually run. Clarify: if times are measured, present them as such; if derived from per-seed T(M), show T(M) as a measured quantity and the total as a derived quantity.

6. **[Liang] Cargo.lock URL.** Provide a URL to the specific repository commit containing the Cargo.lock used for the 50-state B.7 sweep, or state it will be provided upon acceptance.

7. **[Duchin] SHA-256 concatenation order.** Section 4.3 discusses the version string for domain separation but does not specify whether the version string comes before or after the census_release_id in the SHA-256 input. Specify the byte-level concatenation order in the statutory text to prevent implementation divergence.

8. **[Stephanopoulos] Legal procedure if opponent claims lower EC_norm.** The Certificate of Seed Neutrality invites opponents to "run the sweep and show us the lower edge cut." Add a sentence specifying the legal procedure if this nevertheless occurs — which agency or court adjudicates whether a lower-EC_norm plan found by an opponent must replace the statutory map.

## P3 Items (Minor)

1. **[Liang] ConvergenceSweep behavior with zero-seed-variance structures.** If --structure prime-factor (ApportionRegions) has zero seed variance (as established in T.4), ConvergenceSweep would terminate at T=1. Clarify whether a statutory AR+convergence run terminates immediately or runs the full 600 seeds, to avoid misleading users about sweep behavior.

2. **[Duchin] Explicit null hypothesis for T=600 adequacy.** Add a precise null hypothesis (e.g., "the B.7 50-state dataset contains all worst-case congressional graph structures; the 2030 census will not produce a state with tau > 600") to clarify what the Gumbel model and empirical margin are jointly claiming.

## Panel Verdict

Accept after major revision. The empirical core — Georgia's 511-seed tail certifying T=600 with an 89-seed margin — is solid and independently verifiable from the B.7 dataset. The legal contributions (Certificate of Seed Neutrality, statutory enforcement clause, T_stat/T_prac distinction) are genuinely novel. However, the paper has two mathematical errors that would embarrass the paper if cited in court (Theorem 1 exponent, EC_norm definition gap), an overstatement of METIS determinism that could allow a verifier to claim non-reproducibility, and three statutory drafting omissions that leave T_stat elevation, version-cycle binding, and the Georgia partisan outcome unresolved. All P1 items are text corrections or additions — no new empirical runs are required. A single revision pass addressing all P1 items and the most important P2 items should bring the paper to acceptance.
