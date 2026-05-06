# Revision Plan — B.16 ConvergenceSweep: T=600 as Statutory Stopping Criterion
Round 1 avg: 3.5/4
Round 2 avg: 4.0/4 — ALL REVIEWERS AT 4 — ACCEPT

## Round 2 Summary (2026-05-05)
- Karypis: 4/4 — All P1 resolved. Theorem 3.1 exponent corrected, Gumbel KS test added, EC_norm dual definition added.
- Rodden: 4/4 — j* column populated, Gumbel SD error corrected, Georgia partisan study deferred to P2.
- Duchin: 4/4 — EC_norm definitions accepted, Theorem 3.1 corrected, Gumbel KS test + independence acknowledgement satisfied.
- Stephanopoulos: 4/4 — 24h vs 30d reconciled, EAC T_stat process deferred to B.02, version string deferred.
- Liang: 4/4 — METIS single-threaded requirement added, repository URL deferred to journal submission P2.

## P1 — Required (from reviewers)

- [x] [Karypis R1.1] Fix Theorem 3.1 proof sketch — corrected O(n^{2k}) → O(n^{2(k-1)}); proof revised to correctly count (k-1)-tuple of cuts
- [x] [Karypis R1.2] Justify the Gumbel distribution choice — KS test added (D=0.11, p≈0.52), heterogeneity acknowledged, Gumbel framed as "descriptive envelope"
- [x] [Karypis R1.3] Resolve EC_norm definition for full k-way partition vs. bisection tree — two-case definition added (recursive bisection: sum of level EC_norms; k-way: EC/sqrt(k/2))
- [~] [Rodden R2.1] Georgia partisan outcome analysis — framing added (compactness-proportionality tradeoff noted), full case study deferred to P2
- [x] [Rodden R2.2] Populate the complete j* column in Table 1 — all 50 states now have j* values
- [x] [Rodden R2.3] Fix the 89-seed margin / Gumbel SD claim — corrected to 0.46 standard deviations (not 1); SD correctly computed as σπ/√6 ≈ 192
- [x] [Duchin R3.1] EC_norm definition inconsistency — resolved (same as Karypis R1.3)
- [x] [Duchin R3.2] Fix Theorem 3.1 proof — resolved (same as Karypis R1.1)
- [x] [Duchin R3.3] Gumbel independence assumption — KS test added, heterogeneity and non-i.i.d. acknowledged
- [~] [Stephanopoulos R4.1] EAC administrative process for raising T_stat — table caption notes EAC authority; full statutory clause deferred to B.02
- [~] [Stephanopoulos R4.2] Version string versioning problem — deferred to B.02 as statutory text question
- [x] [Stephanopoulos R4.3] Reconcile 24-hour vs. 30-day deadline — 24h characterised as computation sub-requirement within 30-day submission window
- [x] [Liang R5.1] Qualify METIS determinism for parallel implementations — METIS_OPTION_NTHREADS=1 specified, OpenMP non-determinism explained
- [~] [Liang R5.2] Table 5 measured vs. derived entries — partially addressed; full clarification for journal submission
- [~] [Liang R5.3] Repository URL and commit hash — deferred to journal submission (P2)

## P2 — Suggested

- [ ] [Karypis R1.P2a] Add T_prac=500 vs T_stat=600 computational cost comparison for worst-case states
- [ ] [Karypis R1.P2b] Provide theoretical argument for how convergence tail scales with n
- [ ] [Rodden R2.P2a] Georgia partisan case study (D seat count at T=500 vs T=600 termination)
- [ ] [Rodden R2.P2b] Stratify Table 1 convergence data by state characteristics (k, n, p*)
- [ ] [Duchin R3.P2a] Specify exact byte concatenation order for SHA-256 domain separation
- [ ] [Duchin R3.P2b] Address planarity assumption for block-level graphs
- [ ] [Stephanopoulos R4.P2a] Address the litigation procedure if opponent finds lower EC_norm at seed s_0+650
- [ ] [Stephanopoulos R4.P2b] Specify mandatory vs. optional depo_log.jsonl fields for EAC submission
- [ ] [Liang R5.P2a] Clarify --search convergence interaction with --structure prime-factor
- [ ] [Liang R5.P2b] Address ConvergenceSweep applicability to NestSection multi-chamber problem

## P2 — Suggested

- [ ] [Karypis R1.P2a] Add T_prac=500 vs T_stat=600 computational cost comparison for worst-case states — show seed count and runtime at both thresholds for Georgia, Wisconsin, Florida
- [ ] [Karypis R1.P2b] Provide theoretical argument for how convergence tail scales with n (graph size) — relevant for block-level redistricting assessment
- [ ] [Rodden R2.P2a] Add political economy paragraph on the compactness-proportionality tradeoff in ConvergenceSweep's optimality claim — acknowledge that certifying minimum-EC_norm may certify the most efficient Republican geometry in some states
- [ ] [Rodden R2.P2b] Stratify Table 1 convergence data by state characteristics (k, n, p*) — identify which characteristics predict longer tails
- [ ] [Duchin R3.P2a] Specify exact byte concatenation order for SHA-256 domain separation in the statutory text — `SHA-256("DIA_SEED_V1" || census_release_id)` vs. `SHA-256(census_release_id || "DIA_SEED_V1")` — both are secure but the order must be specified for implementation consistency
- [ ] [Duchin R3.P2b] Address planarity assumption for block-level graphs — planar separator theorem bounds may not hold for urban census block geometries; discuss implications for block-level convergence certification
- [ ] [Stephanopoulos R4.P2a] Address the litigation procedure if an opponent finds a lower EC_norm at seed s_0+650 — specify the legal response to a "missed optimum" challenge
- [ ] [Stephanopoulos R4.P2b] Specify mandatory vs. optional depo_log.jsonl fields for EAC submission — reduce court verification burden by identifying which seed-level fields are legally required
- [ ] [Liang R5.P2a] Clarify --search convergence interaction with --structure prime-factor when AR zero seed variance means sweep terminates at seed 1
- [ ] [Liang R5.P2b] Address whether ConvergenceSweep Algorithm 1 is applicable to the NestSection multi-chamber constrained graph problem
