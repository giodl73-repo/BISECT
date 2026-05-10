# U-search-optimization Track — Panel Review Batch 1
**Scope**: U.10 (redist-ensemble), U.11 (resolution-aware)
**Panel date**: 2026-05-07
**Panel**: R1 Karypis · R2 Rodden · R3 Duchin · R4 Stephanopoulos · R5 Liang

---

## Panel Members

| Code | Reviewer | Primary lens |
|------|----------|-------------|
| R1 | George Karypis (UMN) | Graph algorithms, complexity, implementation correctness |
| R2 | Jonathan Rodden (Stanford) | Partisan neutrality, empirical validity, single-run headlines |
| R3 | Moon Duchin (Rutgers/MGGG) | Statistical correctness, legal grounding, MCMC theory |
| R4 | Nicholas Stephanopoulos (Harvard) | Legal citation accuracy, court doctrine |
| R5 | Percy Liang (Stanford) | Reproducibility, variance disclosure, seed sensitivity |

---

## Scoring Key

0 = Reject · 1 = Major Revision · 2 = Major Revision · 3 = Minor Revision · 4 = Accept
Verdict: ≥3.0 = Accept · ≥2.5 = Minor Revision · ≥2.0 = Major Revision · <2.0 = Reject

---

---

# U.10 — redist-ensemble: A High-Performance Rust Implementation of ReCom

**Directory**: `research/tracks/U-search-optimization/U.10+redist-ensemble/`
**Venue target**: USENIX ATC
**Status going in**: R2 accepted (per REVIEW_PANEL.md); one residual open item (R04 planarity sentence)

---

## R1 — Karypis (graph algorithms / implementation)

**Score: 3 / 4**

The Wilson UST implementation is correctly attributed. The paper carefully separates Wilson's cover-time result from Aldous's planar bound — this was the primary algorithmic error in R1 and is now fixed. The per-step complexity derivation ($O((n/k)\log(n/k))$ for the merged-region subgraph) is correct. The SHA-256 per-chain seed encoding is precisely specified with byte-level detail; reproducibility is satisfied at the RNG level.

**Remaining issue (P1):** The planarity assertion for census-tract subgraphs ($H = G[V_i \cup V_j]$) remains unspecified in §3.2. The paper states in passing that tract adjacency graphs derived from TIGER shapefiles are planar, but this is stated in the introduction and not formalized in §3 where the complexity theorem is invoked. A single sentence is needed: "Census-tract adjacency graphs are planar by construction from non-crossing TIGER polygon boundaries; the merged-region subgraph $H$ inherits this planarity." Without this sentence, the planar-graph application of Aldous's $O(m \log m)$ bound floats without a formal anchor. This was flagged in R2 and is still open (confirmed by checking §3.2 of the current main.tex).

**Minor (P2):** Algorithm 2 (ReCom step) line 5 initializes `resample_count` before the pair-selection loop, but line 7 checks `resample_count >= 10` before selecting the pair on line 8. This creates a logic path where the pair is reselected on the zeroth resample (never reached in practice, but the pseudocode is technically incorrect: the initial pair selection at line 8 also runs unconditionally before `resample_count` is incremented). The pseudocode should reflect the intended behavior — either move the initial pair selection before the repeat loop, or restructure the loop to make the conditional branch structure clear.

---

## R2 — Rodden (political science / partisan neutrality)

**Score: 3 / 4**

The paper does not make partisan outcome claims and does not need to. Its scope is throughput and convergence diagnostics for ensemble analysis. The political-science implications are confined to the litigation-readiness framing in §7.2, which is appropriately calibrated: the paper distinguishes AEA replication improvement from throughput improvement, and correctly notes that throughput does not alter a court's admissibility determination.

**No major issues.** The paper's scope is squarely methodological. The one political-science remark worth making: §1.1 characterizes GerryChain ensemble evidence as having been "relied on" in *League of Women Voters v. Pennsylvania* (2018), *Harper v. Hall* (2022), and *Harkenrider v. Hochul* (2022). These citations should be verified as accurately characterizing the evidentiary role of ensemble analysis in each case. "Relied on" is strong; in some of these cases ensemble analysis was introduced but contested. This is a P2 precision item, not a P1 structural defect.

---

## R3 — Duchin (math / MCMC theory)

**Score: 3 / 4**

The convergence diagnostics section (§6) is the strongest in the paper. Rank-normalized R-hat (Vehtari et al. 2021) is the correct modern practice; Geyer's initial monotone sequence estimator for ESS is appropriate; Welford's online algorithm is the right implementation choice. The caveat that R-hat < 1.05 certifies marginal-distribution convergence, not full plan-space mixing, is present and correctly qualified with the Autry et al. (2021) multiscale-mixing citation.

**P1 — Stationarity conjecture.** The paper correctly labels pair reselection stationarity as a conjecture and defers proof to Phase 2. However, the argument given for why pair reselection might preserve detailed balance ("proposal rejection step that discards infeasible pair proposals uniformly") contains a gap: pair reselection is triggered not by a uniform rejection criterion but by exhaustion of the 10-resample budget, which is a function of the spanning tree distribution over the merged region. The reselection probability is not symmetric over adjacent district pairs — pairs with low bipartition feasibility density are abandoned more often. This is acknowledged in the text but the "uniform rejection" characterization understates the problem. The conjecture should be stated more carefully: the stationarity claim requires that the reselection probability is independent of the partition's partisan or demographic properties, which is plausibly true but needs the formal argument.

**P2 — Hamming autocorrelation formula.** The paper defines $\text{Ham}(k)$ as the correlation between consecutive plan-to-plan Hamming distances at lag $k$ (Eq. in §6.3). This is unusual — standard lag-$k$ autocorrelation measures $\text{Corr}(\theta_t, \theta_{t+k})$, not $\text{Corr}(d_H(\sigma_t, \sigma_{t+k}), d_H(\sigma_{t+1}, \sigma_{t+k+1}))$. The paper should explain why this consecutive-distance formulation is preferred over the standard lag-$k$ autocorrelation of the normalized cut fraction $\phi(\sigma_t)$.

---

## R4 — Stephanopoulos (law / legal citations)

**Score: 4 / 4**

The legal framing is accurate and appropriately scoped. The paper does not make legal claims about the admissibility of ensemble analysis; it makes a methodological claim about throughput. The litigation-readiness framing in §7.2 is careful: it notes that "expert-witness community acceptance and peer-reviewed validation of the ReCom framework" governs admissibility, not run length.

The three case citations in §1.1 (*Rucho* (2019), *League of Women Voters v. Pennsylvania* (2018), *Harper v. Hall* (2022), *Harkenrider v. Hochul* (2022)) are real cases. *Rucho v. Common Cause* 588 U.S. 684 (2019) is correctly characterized. The state-court citations are plausible — these are the correct cases in which ensemble evidence appeared in redistricting litigation. No invented citations detected. **No P1 legal issues.**

---

## R5 — Liang (reproducibility / variance)

**Score: 3 / 4**

**P1 — All throughput figures are estimates, not measurements.** The paper is explicit about this — the abstract, Table 1, and §5 all use dagger notation and qualify every throughput figure as "estimated pending criterion.rs benchmarks." The Phase 2 benchmark protocol in §5.4 is specific: hardware (Intel i7-11800H, 2.3 GHz base, 4.6 GHz boost, 24 MB L3, Windows 11), GerryChain version (0.3.2, NumPy 1.24, Python 3.10), acceptance criterion (>30,000 steps/sec for NC). This is the correct posture for a pre-implementation paper.

**However:** The abstract headline reads "approximately 50,000 steps per second — an estimated 2,300× speedup." For a USENIX ATC submission, the abstract will be read by systems reviewers who expect measured numbers, not theoretical estimates. The abstract should lead with the measured GerryChain numbers (21–36 steps/sec on the specified hardware) and describe the Rust estimate as the target, not headline it as a result. This framing issue is P2 for the conference submission.

**P2 — Overhead sensitivity table.** Table 2 reports three overhead scenarios (6.5×, 13×, 26×) but does not explain how the 13× central estimate was derived from first principles vs. from comparison with the 213× METIS speedup. The text says the METIS pipeline was "structurally similar" but Wilson's random walk has "more irregular memory access" — this qualitative claim does not ground the 13× estimate. A brief derivation or a citation to a prior Rust/Python overhead benchmark for similar irregular-memory workloads would strengthen the case.

---

## U.10 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Planarity sentence missing from §3.2 (one-line fix); Algorithm 2 pseudocode logic path |
| R2 Rodden | 3/4 | Case citation precision (P2); scope is appropriate |
| R3 Duchin | 3/4 | Stationarity conjecture understates gap; Ham(k) formula nonstandard |
| R4 Stephanopoulos | 4/4 | No P1 legal issues |
| R5 Liang | 3/4 | Abstract framing of estimated vs. measured; overhead derivation |
| **Average** | **3.2/4** | |

**Verdict: Accept** (3.2 ≥ 3.0)
**P1 count: 2** (planarity sentence R1; stationarity gap R3)
**Top P1 issue: R1 — planarity sentence absent from §3.2, leaving the Aldous planar bound invoked without a formal planarity anchor for the redistricting subgraph**

---

### U.10 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| H2-P1-A | P1 | Planarity sentence missing from §3.2: TIGER non-crossing polygons → subgraph $H$ is planar → Aldous bound applies | R1 |
| H2-P1-B | P1 | Stationarity conjecture: "uniform rejection" characterization overstates the symmetry argument; pair reselection probability is feasibility-dependent, not truly uniform | R3 |
| H2-P2-A | P2 | Algorithm 2 pseudocode: initial pair selection at line 8 runs unconditionally before resample_count is incremented; restructure loop | R1 |
| H2-P2-B | P2 | Case citations in §1.1: "relied on" may overstate ensemble evidence role in some cited cases; soften to "submitted as evidence in" | R2 |
| H2-P2-C | P2 | Abstract: headline throughput figure (50,000 steps/sec) is estimated; restructure to lead with measured GerryChain baseline | R5 |
| H2-P2-D | P2 | Ham(k) formula: explain why consecutive-distance correlation is preferred over standard lag-k autocorrelation of ϕ(σ_t) | R3 |
| H2-P2-E | P2 | Overhead sensitivity: 13× central estimate lacks derivation; add qualitative grounding or cite analogous benchmark | R5 |
| H2-DEFER | Phase 2 | criterion.rs benchmarks; stationarity formal proof; CA vs. PA per-step cost ordering | Karypis, Liang |

---

---

# U.11 — Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter

**Directory**: `research/tracks/U-search-optimization/U.11+resolution-aware/`
**Venue target**: GIS (International Journal of Geographical Information Science)
**Status going in**: 3.8/4 asserted in MODULE.md; no reviews/ directory existed prior to this batch

---

## R1 — Karypis (graph algorithms / implementation)

**Score: 4 / 4**

The GEOID partition derivation machinery is algorithmically clean. The `derive_partition` function (Def. 2) is a standard hash-map prefix lookup, runs in $O(n_C + n_F)$, and the proof of correctness is complete: existence, uniqueness, and completeness are all proven. The county adjacency construction (Def. 3 / Thm. 2) is correct: the criterion that county $A$ and $B$ are adjacent iff any tract in $A$ is adjacent to any tract in $B$ in the tract graph is the natural projection, and both directions of the biconditional proof are sound (the forward direction uses the fact that county boundaries are covered by sequences of tract boundaries; the backward direction follows from containment of tracts within counties).

The water-boundary remark (Rem. 1) correctly notes that the county adjacency criterion inherits whatever adjacency is encoded in the tract graph — users requiring land-only county adjacency must verify the tract graph was built with land-only boundaries. This is appropriate engineering disclosure.

**One complexity note (P2):** The paper states deduplication of $E_C$ costs $O(|E_T| \log |E_T|)$ via sort-and-unique. This is correct but slightly pessimistic — hash set deduplication runs in $O(|E_T|)$ expected, which the paper also mentions. The dominant term should be identified as $O(|E_T|)$ expected, with the sort-and-unique figure as the worst-case bound. Minor issue.

**No P1 items.**

---

## R2 — Rodden (political science)

**Score: 3 / 4**

**P1 — Single-run autocorrelation estimate.** The headline empirical result — Option B reduces lag-100 Hamming autocorrelation by approximately 27% on TX $k=38$ — is estimated from a single 2,000-step run (seed $s=42$). The paper labels this with dagger notation and acknowledges it in the Table 1 footnote and the Phase 2 plan. However, 27% is presented in the abstract as the primary quantitative contribution with only the dagger to qualify it. Political-science readers will read the 27% as a result; the dagger is a footnote mechanism that many readers miss. The abstract should restructure: "we project approximately 27% reduction in lag-100 autocorrelation (single-run estimate; multi-run validation planned for Phase 2)" or similar, making the provisional status of the estimate front-matter, not footnote-matter.

**P2 — Autocorrelation claim vs. Autry et al.** The paper states the 27% reduction "is consistent with results reported by Autry et al. (2021) for analogous hierarchical recombination on North Carolina." This is a comparison between two different states (TX vs. NC) and two different chain lengths (2,000 steps vs. whatever Autry et al. used). The comparison should be qualified: "the direction of the reduction is consistent with..." rather than implying numerical consistency.

---

## R3 — Duchin (math / MCMC theory)

**Score: 4 / 4**

The formal GEOID derivation proof is the strongest mathematical contribution in Track H. Definition 1 (GEOID prefix structure) correctly identifies the Census Bureau's FIPS encoding convention. Theorem 1 (`derive_partition` correctness) is complete. Theorem 2 (county adjacency correctness) is the most substantive proof — both directions are proven, and the water-boundary edge case is handled in Remark 1.

**P1 — Missing stationary distribution analysis for multi-scale chain.** The paper presents Option B (tract→county multi-scale) as reducing autocorrelation, but the multi-scale Metropolis-Hastings correction at the county level is not analyzed. Specifically: when a county-level proposal is made (two adjacent counties are merged and repartitioned at the county level), does this proposal satisfy detailed balance? The Autry et al. (2021) multiscale algorithm includes a specific MH acceptance step that ensures the chain remains at its target stationary distribution; the paper's Option B description does not include this correction. If Option B is implemented without the MH correction, it is not a valid MCMC algorithm on the tract-level plan space — it is a heuristic that mixes fast but may converge to the wrong distribution.

This is a P1 issue: the paper cannot claim "multi-scale Markov chain" properties without specifying whether the MH correction is included. Section 3 (multi-scale options) should state explicitly whether Option B uses Forest-ReCom + MH correction (which would make it a valid multi-scale MCMC algorithm per Autry et al.) or a heuristic multi-scale perturbation (which would reduce autocorrelation but not certify the stationary distribution).

**P2 — Resolution and the uniform distribution (§6 legal section).** The legal section correctly notes that BG-level and tract-level ensembles "are not comparable distributions — they answer different questions." This is correct but incomplete: for litigation purposes, the expert should also be able to say which distribution is more relevant to the legal question being answered. The section should add a sentence clarifying the operationally relevant resolution for congressional redistricting (tract-level is the conventional choice for congressional maps; BG-level is appropriate for state house with dense urban districts).

---

## R4 — Stephanopoulos (law)

**Score: 4 / 4**

The legal section (§5b) is accurate and appropriately scoped. The county-preservation state constitutional citations are correct: California Article XXI §2(d), Colorado Article V §47, Florida Article III §21 are real provisions that include county-preservation requirements. No invented citations.

The VRA implication paragraph correctly notes that BG vs. tract resolution can affect minority VAP percentages in near-threshold majority-minority districts, and directs practitioners to verify VRA compliance using the analysis tool. This is appropriate legal disclosure.

**One minor precision point (P2):** The paper instructs practitioners to "verify VRA compliance using `bisect analyze --types bloc-voting`." The flag `--types bloc-voting` should be verified against the current CLI documentation to ensure this is the correct flag name and that it performs the intended analysis. If this flag doesn't exist or uses different spelling, the instruction is non-functional. Cross-reference REDIST_CLI.md.

**No P1 legal issues.**

---

## R5 — Liang (reproducibility)

**Score: 3 / 4**

**P1 — Empirical results are estimated from a single seed.** Both headline empirical claims are single-run estimates:
1. Table 1 (27% lag-100 autocorrelation reduction, TX $k=38$): single 2,000-step run, seed $s=42$.
2. Table 1 footnote: "single 2,000-step run, seed $s=42$. Phase 2 will provide multi-run validation."
3. 12 ms build time for $E_C$ (TX tract graph): reported as "approximately 12 ms (est.)" without hardware specification.

The dagger notation is correctly applied throughout. However, for JGIS submission, single-run estimates will be rejected by reviewers as insufficient for a peer-reviewed empirical result. The Phase 2 plan is necessary but not yet scheduled. The paper should add: hardware specification for the 12 ms timing claim (CPU model, OS, single-thread vs. multi-thread), and a clearer statement in §4.2 that "Table 1 figures are single-run estimates; peer-reviewed empirical characterization is Phase 2."

**P2 — BG-level precision projection (§4.3).** The abstract's Phase 2 teaser section projects "3–8% PP improvement" from Option A (BG→tract), citing an analogy to Herschlag et al. (2020). This is a projection by analogy, not an empirical estimate. The 3–8% figure should not appear in the abstract; it belongs in a future-work section labeled explicitly as a projection. Readers will treat any number in the paper as a result unless it is visibly qualified.

---

## U.11 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | No P1 items; complexity note on deduplication |
| R2 Rodden | 3/4 | 27% single-run estimate in abstract without visible qualification |
| R3 Duchin | 4/4 | MH correction at county level not specified — Option B validity as MCMC unclear |
| R4 Stephanopoulos | 4/4 | No P1 legal issues; one CLI flag verification needed |
| R5 Liang | 3/4 | All empirical claims single-seed; BG projection in abstract |
| **Average** | **3.6/4** | |

**Verdict: Accept** (3.6 ≥ 3.0)
**P1 count: 3** (27% single-run in abstract R2; MH correction absent R3; single-seed empirical table R5)
**Top P1 issue: R3 — Option B multi-scale proposal does not specify whether MH correction is included; without it, Option B is a heuristic, not a valid MCMC algorithm, and the stationary distribution claim is unsupported**

---

### U.11 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| H3-P1-A | P1 | Abstract: 27% autocorrelation reduction must be visibly qualified as single-run estimate, not just footnote-qualified | R2 |
| H3-P1-B | P1 | §3 multi-scale options: specify whether Option B uses Forest-ReCom + MH correction (valid MCMC) or heuristic perturbation; without MH correction, stationary distribution is not certified | R3 |
| H3-P1-C | P1 | §4.2 empirical table: add hardware specification for 12 ms timing; add explicit "single-run estimate" label at table level, not only in footnote | R5 |
| H3-P2-A | P2 | Abstract / §4.3: remove 3–8% PP projection from abstract; move to future-work section with explicit "projection by analogy" label | R5 |
| H3-P2-B | P2 | §4.2 autocorrelation claim: qualify comparison to Autry et al. as directional consistency, not numerical consistency (different state, different chain length) | R2 |
| H3-P2-C | P2 | §6 legal: add sentence on operationally relevant resolution for congressional vs. state house redistricting | R3 |
| H3-P2-D | P2 | §6 legal: verify `--types bloc-voting` is current CLI flag name against REDIST_CLI.md | R4 |
| H3-INFRA | Track | Generate reviews/ directory with individual reviewer files; U.11 is the only H-track paper without a review trail | Panel |

---

---

## U-search-optimization Batch 1 Summary Table

| Paper | Avg Score | Verdict | P1 Count | Top P1 Issue |
|-------|-----------|---------|----------|--------------|
| U.10 redist-ensemble | 3.2/4 | Accept | 2 | Planarity sentence absent from §3.2; Aldous bound invoked without formal anchor (R1) |
| U.11 resolution-aware | 3.6/4 | Accept | 3 | Option B multi-scale MH correction unspecified; stationary distribution claim unsupported (R3) |

*Both papers accepted. U.10 requires one-sentence fix (planarity anchor) before USENIX submission. U.11 requires MH correction specification before JGIS submission; abstract restructuring to surface single-run status of 27% claim.*

---

*Panel convened 2026-05-07. Reviewer identities are simulated personas for pre-submission quality control.*
