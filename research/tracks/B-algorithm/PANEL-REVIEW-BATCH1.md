# B-algorithm Track — Panel Review Batch 1
**Scope**: B.1 (recursive bisection) · B.2 (edge-weighted) · B.3 (multi-vs-edge) · B.4 (adaptive bisection) · B.5 (nway-vs-recursive-general) · B.6 (computational complexity) · B.7 (solution-space / seed sensitivity)
**Panel date**: 2026-05-07
**Panel**: R1 Karypis · R2 Rodden · R3 Duchin · R4 Stephanopoulos · R5 Liang

---

## Panel Members

| Code | Reviewer | Primary lens |
|------|----------|-------------|
| R1 | George Karypis (UMN) | Algorithmic correctness, complexity, METIS behavior. Hostile to informal proofs. |
| R2 | Jonathan Rodden (Stanford) | Partisan neutrality, empirical validity. Hostile to single-run headlines. |
| R3 | Moon Duchin (Rutgers/MGGG) | Statistical correctness, legal grounding, redistricting theory |
| R4 | Nicholas Stephanopoulos (Harvard) | Legal citation accuracy. Hostile to invented or mischaracterized cases. |
| R5 | Percy Liang (Stanford) | Reproducibility, seed variance, multi-run disclosure. Hostile to single-seed claims. |

---

## Scoring Key

0 = Reject · 1 = Major Revision · 2 = Major Revision · 3 = Minor Revision · 4 = Accept
Verdict: ≥3.0 = Accept · ≥2.5 = Minor Revision · ≥2.0 = Major Revision · <2.0 = Reject

---

---

# B.1 — From Apportionment to Boundary Design: Extending Huntington-Hill to Congressional Redistricting

**Directory**: `research/tracks/B-algorithm/B.1+recursive-bisection/`
**Venue target**: APSR
**Status going in**: Score 3.64/4 in REVIEW_PANEL.md; "ready stage" per _panel.yaml

---

## R1 — Karypis

**Score: 3 / 4**

The METIS recursive bisection algorithm is correctly described. The multilevel coarsening–partitioning–uncoarsening pipeline is accurate (§3.3). The NP-hardness reference for graph partitioning is correct; the paper appropriately uses METIS as a heuristic and does not claim optimality.

**P1 — Binary tool name.** The paper refers to the pipeline as "redist" throughout (e.g., in the acknowledgments section: "the METIS graph partitioning library...used by this research"). The production tool is now the `bisect` binary, renamed from `redist`. Any CLI invocations or binary references should use `bisect`, not `redist`. Specifically, §3.3 and the computational summary should refer to `bisect state` commands. This is a consistency error relative to the current codebase.

**P1 — Population balance claim needs clarification.** The abstract states "mean population deviation of 2.79%." Table 1 in §4.1 confirms this is the mean absolute deviation across all 435 districts, including single-district states. The paper correctly notes that "The maximum deviation (15.83%) occurs in single-district states where the state's total population necessarily differs from the national ideal." However, the constitutional standard (*Wesberry*, *Karcher*) applies to multi-district states only, and the deviation for single-district states is constitutionally irrelevant. The abstract should clarify: "mean deviation of 2.79% (including single-district states); for multi-district states, maximum deviation stays below 8%." Presenting 2.79% without this qualification will mislead APSR reviewers who know redistricting law.

**P2 — Contiguity guarantee claim.** The paper claims "guaranteed geographic contiguity" in the abstract. This is provided by METIS's `-contig` flag, but the guarantee is at the tract level: contiguity is guaranteed in the tract adjacency graph, which may include water-bridge edges. The paper should specify that contiguity is guaranteed in the augmented graph (including bridge edges) — geographic contiguity in the strict sense (land-connected) is not guaranteed for island tracts reachable only via water bridges.

---

## R2 — Rodden

**Score: 3 / 4**

The partisan analysis (§5) is the paper's most important empirical contribution for political science. The finding that algorithmic redistricting produces a systematic Democratic advantage (56.5% Democratic-leaning districts) despite no partisan input is consistent with the geographic-sorting literature (Rodden 2019, Chen and Rodden 2013). This is correctly attributed in the paper.

**P1 — "Structural immunity" claim.** The abstract and §1.2 claim "structural immunity to partisan manipulation." This is too strong. The algorithm is immune to direct partisan data input, but the geographic sorting of voters means that the choice of which geographic units to use (tracts vs. blocks vs. counties), the population balance tolerance, and the compactness objective all implicitly encode geographic patterns that correlate with partisanship. The algorithm has no *intentional* partisan input, not *structural* immunity. "Structural immunity" implies the outcomes are free of partisan patterns, which the paper itself disproves in §5. The claim should be revised to "immunity to direct partisan data input" or "the algorithm cannot intentionally gerrymander."

**P2 — Partisan analysis relies on 2020 presidential results.** Presidential results are an imperfect proxy for House election outcomes. The paper correctly notes this in §5 but should be more prominent about this limitation in the abstract or introduction.

---

## R3 — Duchin

**Score: 3 / 4**

The paper's framing — Huntington-Hill as a precedent for algorithmic redistricting — is intellectually coherent. The analogy is apt: both involve mathematically formalizing a politically contested allocation problem. The paper correctly notes that mathematical formalization achieves procedural legitimacy rather than optimal outcomes.

**P1 — Compactness baseline comparison error.** Table 3 in §4.3 shows "Algorithmic: 0.235 PP, Enacted 2020: 0.305 PP, Difference: -22.8%." The paper then notes that enacted districts have *higher* average compactness than algorithmic districts. This is correct but the "Note on baseline comparisons" paragraph (appended to the table) states that B.2 reports "+56% figure using the unweighted bisection as its baseline (not enacted maps)." This clarification is essential and correct — but the cross-reference to "Table~\ref{tab:compactness-headline} in A.0 Supplement" refers to a document that does not exist in the current repository. There is no A.0 Supplement in the B-algorithm track. This is a broken reference that will cause confusion for reviewers and should be resolved before APSR submission.

**P2 — Efficiency gap for B.1 is not appropriate.** The §5 analysis discusses efficiency gaps for algorithmic districts, but notes that "compact districts can *increase* partisan advantage in geographically sorted states." This is a well-known result and the paper handles it correctly. However, the efficiency gap formula requires knowing which party won each district, which requires overlaying election results onto algorithmically defined districts — a nontrivial aggregation step that the paper acknowledges using simulated vote distributions. This should be labeled as "simulated" more prominently in §5.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The legal framing is generally accurate. *Rucho v. Common Cause* (2019) is correctly characterized as removing federal judicial oversight of partisan gerrymandering. *Wesberry v. Sanders* (1964) and *Karcher v. Daggett* (1983) are correctly cited for population equality.

**P1 — VRA compliance analysis is simulated.** §4.3 discusses Voting Rights Act compliance and reports that algorithmic redistricting produces 21 majority-minority districts versus 65 enacted. This is described as a simulation: "we model enacted districts as intentionally concentrating minorities into packed districts (VRA compliance strategy) versus algorithmic districts following geographic compactness without demographic targeting." The VRA analysis is based on state-level Census data modeling, not on actual tract-level demographic aggregation to algorithmic districts. This methodological limitation is acknowledged in the VRA Limitations paragraph, but the headline numbers (65→21 majority-minority district reduction, "68% reduction") should not appear without a visible qualifier that these are model-based projections, not empirical measurements from the actual algorithmic districts. APSR reviewers with VRA expertise will flag this.

**P2 — *Karcher v. Daggett* characterization.** The paper states courts "typically accept deviations under 1% without requiring justification, and deviations under 10% if justified by legitimate state interests." This characterization applies to state legislative districts, not congressional districts. For congressional districts, *Karcher* requires justification for any deviation from mathematical equality, even small ones. The paper conflates the state and federal standards.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — Single-seed computation.** The computational summary (§4.4) reports "Processing all 50 states required approximately 2.5 hours" on a specific hardware setup. No seed is reported for the METIS partitioner. The paper should disclose the random seed used for METIS partitioning (or state that the default METIS seed was used) so that the results are reproducible. B.7 establishes that seed sensitivity is low (CV < 2%), which justifies single-seed results, but B.1 predates B.7 and should at minimum cite B.7 for this justification.

**P2 — "Reproducible results from the same input data" claim.** The abstract claims reproducibility, but if METIS is run with different random seeds, results may vary by up to 4.3% in edge-cut value (per B.7) with up to 2 seat differences in high-variance states. The paper should qualify: "reproducible from the same input data *and the same random seed*."

---

## B.1 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Binary name "redist" vs. "bisect"; population balance framing |
| R2 Rodden | 3/4 | "Structural immunity" claim overstated |
| R3 Duchin | 3/4 | Broken cross-reference to A.0 Supplement; simulated vote distributions |
| R4 Stephanopoulos | 3/4 | VRA simulation not visibly labeled; Karcher conflation |
| R5 Liang | 3/4 | No seed disclosure; reproducibility claim qualified |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 6** (binary name; population balance framing; structural immunity; broken A.0 reference; VRA simulation headline; seed disclosure)
**Top P1 issue: R2 — "Structural immunity to partisan manipulation" in abstract is falsified by the paper's own §5 results; must be revised to "immunity to direct partisan data input"**

---

### B.1 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B1-P1-A | P1 | CLI references use "redist" instead of "bisect" throughout | R1 |
| B1-P1-B | P1 | Abstract: 2.79% mean deviation should clarify inclusion of single-district states; multi-district max < 8% | R1 |
| B1-P1-C | P1 | "Structural immunity to partisan manipulation" is falsified by §5; revise to "immunity to direct partisan data input" | R2 |
| B1-P1-D | P1 | A.0 Supplement cross-reference in Table 3 note is broken; no such supplement exists in current repository | R3 |
| B1-P1-E | P1 | VRA analysis headline (68% reduction) is model-based; must be visibly labeled "simulated estimate" not stated as empirical finding | R4 |
| B1-P1-F | P1 | No random seed disclosed for METIS partitioner; add seed disclosure or explicit statement of default-seed use | R5 |
| B1-P2-A | P2 | "Guaranteed contiguity" should specify: contiguity guaranteed in augmented graph (including water bridges), not strict land-contiguity | R1 |
| B1-P2-B | P2 | Karcher characterization conflates state legislative and congressional deviation standards | R4 |
| B1-P2-C | P2 | Reproducibility claim should add "from the same input data and same random seed" | R5 |

---

---

# B.2 — Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Directory**: `research/tracks/B-algorithm/B.2+edge-weighted-bisection/`
**Venue target**: APSR
**Status going in**: Score 3.71/4 in REVIEW_PANEL.md; "ready stage"

---

## R1 — Karypis

**Score: 4 / 4**

The edge-weighting mechanism is correctly described. Setting edge weights to boundary lengths in the METIS CSR format redirects METIS's objective from minimizing edge count to minimizing total boundary length. This is a legitimate use of METIS's weighted graph partitioning capability. The result that weighted mode cuts 77% more edges but achieves 25% shorter total perimeter (§3.4) is consistent with METIS's documented behavior under weighted objectives.

The generalization to KaHIP and Scotch (§3.5) is a useful robustness check. The finding that all three partitioners achieve compactness within 0.3% of each other confirms that the improvement is driven by the objective function, not the specific algorithm.

**P1 — "--weights geographic" vs. "--weights-override geographic".** The CLI description in §3.5 states edge weighting is enabled via `--weights geographic` but the current CLI uses `--weights-override geographic` (confirmed from B.16 and other papers in the track). This is a P2 flag error that must be fixed before APSR submission, as APSR reviewers using the CLI will not reproduce the results with the wrong flag.

**P2 — B.2 vs. B.1 baseline distinction.** The abstract states "56% improvement in compactness (Polsby-Popper) over unweighted baselines." The Table 1 note clarifies that the 56% figure is B.2's improvement over the B.1 (unweighted) baseline, not over enacted maps. However, the abstract does not specify "over the B.1 unweighted baseline" — it says "over unweighted baselines." An APSR reader unfamiliar with B.1 will interpret this as improvement over enacted maps, which is incorrect.

---

## R2 — Rodden

**Score: 3 / 4**

The partisan outcome analysis (§3.2) is this paper's critical political-science contribution. The paper correctly reports that the relationship between compactness and partisan fairness is "complex and inconsistent" — compactness improvement does not reliably reduce partisan bias.

**P1 — Simulated partisan data.** The partisan analysis in §3.2 explicitly states: "we simulate vote distributions assuming compact districts reduce geographic packing/cracking effects by 40% (narrower partisan variation around state average)." This 40% assumption is arbitrary and unvalidated. Results based on this assumption (e.g., "Wisconsin: Efficiency gap significantly worsens from -0.024 to +0.279") are model artifacts, not empirical findings. These numbers should not appear in Table 4 without a bold label "SIMULATED." The current "Limitations" paragraph in §3.2 discloses this, but the tables themselves present the numbers as data.

**P2 — Geographic sorting quantification (§3.3) uses same simulated data.** The "63% of partisan bias is geographic baseline" finding and the state classification system (geography-dominated, mixed, gerrymandering-dominated) are based on the same simulated 40% reduction assumption. These are model outputs, not empirical measurements.

---

## R3 — Duchin

**Score: 3 / 4**

The compactness result is solid and the comparison methodology is clean. Edge-weighted METIS with boundary-length weights is a legitimate geometric optimization.

**P1 — VRA compliance simulation.** §3.6 reports a 68% reduction in majority-minority districts (65 enacted → 21 algorithmic). This uses "simulated district demographics based on state-level Census data, modeling realistic VRA packing patterns but not actual tract-level aggregations." The VRA finding is a simulation extrapolated from state-level data, not from running the algorithm on actual districts and computing demographic outcomes. This is disclosed in the Limitations paragraph but the headline numbers will be read as empirical by APSR reviewers. The table should be labeled "PROJECTED" or the section should open with "We project (not measure) the following VRA implications..."

**P2 — Contiguity verification is post-processing.** §3.8 states "Post-processing verifies contiguity via breadth-first search for all 435 districts. Result: 100% contiguous." This implies contiguity is a verified result, not a METIS guarantee. The paper should clarify that METIS's `-contig` flag is the primary guarantee and BFS is a verification step, not a repair step.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The legal claims are generally accurate. VRA compliance discussion in §3.6 is appropriately framed: the paper correctly identifies the VRA-compactness tradeoff and cites Pildes (2004) and Stephanopoulos (2015) for this tension.

**P1 — Multi-constraint vs. edge-weighted framing has legal implications not disclosed.** The paper's conclusion that edge-weighted single-objective optimization produces better VRA compliance than multi-constraint methods (confirmed in B.3) has a significant legal implication: in VRA litigation, a party arguing that algorithmic redistricting satisfies VRA obligations will need to choose between these approaches. The paper does not discuss which approach courts would view as more defensible under *Gingles* or *Callais*. This is a P2 item for the APSR version but a P1 item if this paper is cited in litigation contexts.

**P2 — Majority-minority district count comparison.** The paper compares algorithmic (21) vs. enacted (65) majority-minority districts. This comparison is methodologically sound but the enacted count should be sourced (which enacted plans? 2022 post-redistricting maps? 2021 initial maps?). The Census TIGER/Line 118th Congressional District boundaries are cited in §2, which clarifies this is the 2021 enacted maps. This should be stated explicitly in §3.6.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — KaHIP and Scotch results lack implementation details.** The generalization study (§3.5) reports compactness results for METIS, KaHIP, and Scotch on five states. No seeds, software versions, or configuration flags are reported for KaHIP or Scotch. METIS version is given (5.1.0) but not KaHIP or Scotch versions. Without this information, the cross-partitioner comparison is not reproducible.

**P2 — Runtime comparison (§3.5) uses different hardware.** The AMD Ryzen 5800X is mentioned for the 50-state run but the KaHIP/Scotch runtime comparison hardware is not specified. If these are the same machine, state it; if different, the comparison is invalid.

---

## B.2 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | CLI flag "--weights geographic" vs. "--weights-override geographic" |
| R2 Rodden | 3/4 | Partisan tables present simulated data as empirical; 40% assumption unvalidated |
| R3 Duchin | 3/4 | VRA simulation presented as empirical in headline numbers |
| R4 Stephanopoulos | 3/4 | Legal implications of edge-weighted vs. multi-constraint not addressed |
| R5 Liang | 3/4 | KaHIP/Scotch study not reproducible (no versions, seeds, hardware) |
| **Average** | **3.2/4** | |

**Verdict: Accept** (3.2 ≥ 3.0)
**P1 count: 5** (CLI flag; simulated partisan tables; VRA simulation headline; legal implications; KaHIP/Scotch reproducibility)
**Top P1 issue: R2 — Partisan tables in §3.2–3.3 present simulated data (40% packing-reduction assumption) as if empirical; must be labeled SIMULATED at table level**

---

### B.2 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B2-P1-A | P1 | CLI: `--weights geographic` should be `--weights-override geographic` | R1 |
| B2-P1-B | P1 | Tables 4–5 (partisan analysis): label as SIMULATED; 40% reduction assumption must be disclosed at table level, not only in limitations text | R2 |
| B2-P1-C | P1 | Table 6 (geographic sorting): same simulated-data disclosure required | R2 |
| B2-P1-D | P1 | VRA table (§3.6): headline 68% reduction is projected from state-level data, not measured from actual algorithmic districts; label as PROJECTED | R3 |
| B2-P1-E | P1 | KaHIP/Scotch comparison: add software versions, seeds, hardware spec for reproducibility | R5 |
| B2-P2-A | P2 | Abstract: "56% improvement over unweighted baselines" — specify "over the B.1 unweighted baseline" | R1 |
| B2-P2-B | P2 | §3.6 VRA source: specify that enacted count (65) comes from 118th Congressional District TIGER/Line (2021 plans) | R4 |
| B2-P2-C | P2 | §3.8 contiguity: clarify METIS -contig flag is primary guarantee; BFS is verification | R3 |
| B2-P2-D | P2 | Runtime comparison hardware: confirm KaHIP/Scotch benchmarks ran on same machine as METIS (AMD Ryzen 5800X) | R5 |

---

---

# B.3 — Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Directory**: `research/tracks/B-algorithm/B.3+multi-vs-edge/`
**Venue target**: APSR
**Status going in**: Score 3.3/4; "recheck stage"

---

## R1 — Karypis

**Score: 3 / 4**

The constraint-conflict theory is algorithmically correct. When a tight constraint ($\pm 0.5\%$ population balance) coexists with a loose constraint ($\pm 10\%$–$\pm 1000\%$ minority concentration), METIS's multi-level Lagrangian relaxation will allocate most weight to enforcing the tight constraint, rendering the loose constraint ineffective. This is a known phenomenon in constrained optimization and the paper provides the first explicit analysis in the redistricting context.

**P1 — Experimental design imbalance in abstract vs. results.** The abstract states "balanced experimental design (140 configurations each)" but the introduction (§1.2) states "Across 160 experiments (5 states × 32 configurations)." This is a discrepancy: 160 ≠ 140 × something. The body results section (§5.1, Table 1) shows "140 configs each" for both methods. The introduction should be corrected to match: 5 states × 28 parameter values (per state) = 140 edge-weighted + 140 multi-constraint = 280 total, not 160. The §1.2 introduction language is inconsistent with the balanced design claim.

**P2 — METIS multi-constraint is not the "standard" approach.** The paper characterizes multi-constraint METIS as "the standard algorithmic approach" for redistricting with VRA. This may not be accurate — most published redistricting algorithms do not use METIS's multi-constraint mode for VRA compliance; they use separate VRA post-processing or alternative optimization. The claim that multi-constraint is "standard" should be supported with citations.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — Results generalizability.** The experiment covers 5 Southern states (Alabama, Georgia, Louisiana, Mississippi, South Carolina) with high Black minority populations. The findings may not generalize to Western states with Hispanic minority populations, where geographic distribution patterns differ significantly. The abstract says "our findings generalize beyond redistricting" but the empirical evidence is from a narrow regional sample. The generalization claim should be qualified.

**P2 — "Completely fails" characterization.** The paper states multi-constraint "completely fails" in Alabama, Louisiana, and South Carolina "across all 28 parameter values." This is a strong claim — "completely fails" should be defined: the paper means multi-constraint produces zero majority-minority districts meeting the 50% threshold. This should be stated explicitly.

---

## R3 — Duchin

**Score: 3 / 4**

The constraint conflict theory (§3) is the paper's most important theoretical contribution. The theorem structure (Theorem 1: constraint conflict mechanism; Theorem 2: convergence bound) is appropriate for the claim.

**P1 — Theorem 1 is not formally proven.** Theorem 1 ("constraint conflict limits multi-constraint effectiveness") is stated as a theorem but the proof is heuristic: "the tight constraint dominates the Lagrangian relaxation." This is a plausibility argument, not a formal proof. For APSR, informal theorems will be challenged. Either provide a formal proof (showing the Lagrangian weight allocation under standard METIS penalty parameters) or relabel as "Proposition (informal)" or "Argument."

**P2 — Smoothed analysis.** §3.2 references smoothed analysis for robustness to perturbations but this analysis does not appear in the body of the paper — it is listed in the section outline (line 32 of abstract: "Smoothed analysis proves robustness to realistic measurement error (census undercount ≤ 4%)"). If the smoothed analysis is in the paper, it should be cited by section number. If it is not in the paper, it should be removed from the abstract.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — *Gingles* prong characterization.** The paper discusses Voting Rights Act compliance without citing *Thornburg v. Gingles* (1986) — the foundational VRA redistricting case that establishes the three-prong test for majority-minority district creation. Section 2 (Background) describes VRA requirements as "majority-minority districts where minority voters comprise ≥50% of the voting-age population" without specifying that this is a threshold condition that satisfies Gingles prong 1 (geographic compactness of the minority group). The paper should cite *Gingles* and note that the 50% threshold is a prong-1 proxy, not the complete legal test.

**P2 — "Algorithmic fairness" terminology.** B.4 (adaptive bisection, same research group) uses algorithmic fairness terminology with formal definitions. B.3 should either use consistent terminology or distinguish its use from B.4.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — Seeds not reported.** The abstract does not specify random seeds for either edge-weighted or multi-constraint METIS runs. Table 1 results (47.9% vs. 35.7% success rates) depend on the specific seeds used. Given that B.7 reports up to 4.3% CV in edge-cut across seeds for high-variance states (GA, NC), the success-rate comparison may be seed-sensitive. All 280 experimental runs should report the seed used or use a fixed seed across all runs.

**P2 — 47.9% vs. 35.7% gap statistical test.** The paper reports $p=0.039$ for the 12.1 percentage-point gap. This is a single comparison without multiple-testing correction. With 28 parameter values tested per state, the probability of finding at least one statistically significant gap by chance is higher than 5%. The paper should apply a Bonferroni correction or similar.

---

## B.3 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Experimental count inconsistency (160 vs. 140 × 2 in intro vs. results) |
| R2 Rodden | 3/4 | 5 Southern states; generalizability overstated |
| R3 Duchin | 3/4 | Theorem 1 is informal; smoothed analysis absent from body |
| R4 Stephanopoulos | 3/4 | *Gingles* not cited; 50% threshold mischaracterized |
| R5 Liang | 3/4 | No seeds reported; p=0.039 without multiple-testing correction |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 5** (experiment count inconsistency; generalizability claim; Theorem 1 informal; *Gingles* absent; no seeds)
**Top P1 issue: R3 — Theorem 1 (constraint conflict) is a heuristic argument presented as a formal theorem; must be reformulated or proven before APSR submission**

---

### B.3 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B3-P1-A | P1 | §1.2: "160 experiments" inconsistent with "140 configurations each" in abstract and Table 1; resolve discrepancy | R1 |
| B3-P1-B | P1 | Generalizability claim: qualify to Southern states with Black minority populations; note Western/Hispanic generalization is untested | R2 |
| B3-P1-C | P1 | Theorem 1: label as "Informal Argument" or provide formal proof of Lagrangian weight allocation | R3 |
| B3-P1-D | P1 | *Thornburg v. Gingles* (1986) must be cited when characterizing VRA's 50% threshold requirement | R4 |
| B3-P1-E | P1 | All 280 experimental runs must disclose METIS random seed | R5 |
| B3-P2-A | P2 | Multi-constraint "standard approach" claim needs citations supporting this characterization | R1 |
| B3-P2-B | P2 | "Completely fails" must be operationally defined (zero majority-minority districts at ≥50% threshold) | R2 |
| B3-P2-C | P2 | Smoothed analysis: either include in body (with section reference) or remove from abstract | R3 |
| B3-P2-D | P2 | p=0.039 needs multiple-testing correction for 28 parameter values tested | R5 |

---

---

# B.4 — Edge-Weighting Makes Method Selection Irrelevant: Complete Equivalence of Recursive and N-Way Partitioning for VRA Compliance

**Directory**: `research/tracks/B-algorithm/B.4+adaptive-bisection/`
**Venue target**: APSR
**Status going in**: Score 3.6/4; "recheck stage"

---

## R1 — Karypis

**Score: 3 / 4**

The method-equivalence finding is algorithmically interesting. At $\alpha \geq \alpha_{\text{crit}}$, the edge-weight signal dominates the partitioning objective and all algorithms converge to the same partition. This is consistent with how multilevel algorithms behave when the objective landscape is dominated by a single strong gradient.

**P1 — Phase transition theorem.** Theorem 2 states: "variance in outcomes transitions sharply from $\Theta(1)$ for $\alpha < \alpha_{\text{crit}}$ to $O(1/\alpha^2)$ for $\alpha > \alpha_{\text{crit}}$." The notation $\Theta(1)$ for variance below the critical point implies variance is bounded from below and above by a constant — this is the correct characterization. But the proof must establish that the variance below $\alpha_{\text{crit}}$ is indeed $\Theta(1)$ and not $O(1)$ (the latter would allow vanishingly small variance, which would contradict the phase transition claim). Check the proof in §4.

**P2 — $\alpha_{\text{crit}}$ is empirically identified as $\alpha \in [20,50]$.** The paper reports phase transition at $\alpha \in [20, 50]$ from experiments on 5 Southern states. But the paper uses $\alpha = 5$ as the standard value throughout (including the Alabama result where "all 6 predetermined trees: 2/2 MM districts, maximum minority 50.8%"). If $\alpha = 5$ is below $\alpha_{\text{crit}} \in [20,50]$, then the equivalence result is *not* explained by the phase transition theorem — the equivalence at $\alpha = 5$ would be a lower-$\alpha$ phenomenon distinct from the phase transition. This is a potential theoretical gap: the empirical equivalence at $\alpha = 5$ may precede the phase transition, and the theorem about variance $O(1/\alpha^2)$ above $\alpha_{\text{crit}}$ does not explain equivalence at $\alpha = 5$.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — "37.5% more majority-minority districts" vs. enacted.** The abstract claims "algorithmic approaches produce 37.5% more majority-minority districts (11 vs. 8 achieved; 14 was the target)." However, achieving 11 vs. the enacted 8 across 5 states is a small-sample comparison. The paper should report a confidence interval or note that the comparison is across only 5 states. "37.5% more" as a headline without a sample-size qualifier will mislead APSR readers.

**P2 — Spatial autocorrelation analysis.** §5.2 reports "strong minority clustering (average Moran's I = 0.703)" as the explanation for method equivalence. Moran's I measures spatial autocorrelation in a geographic attribute; it is an appropriate measure for minority population clustering. The paper should confirm that the spatial adjacency weights used for Moran's I are the same adjacency structure as the METIS graph (queen contiguity at the tract level).

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — Algorithmic determinism.** The paper introduces "algorithmic determinism" as a fairness criterion: "tract assignment depends on geography and demographics, not on arbitrary algorithmic choices." This is presented as a normative requirement without a formal definition. For APSR, normative criteria require either a formal definition and justification or a citation to prior work using the same concept. If "algorithmic determinism" is novel terminology introduced by this paper, it should be defined formally.

**P2 — VRA-compactness tradeoff claim.** The paper claims to "refute the traditional VRA-compactness trade-off" by producing plans with both 37.5% more MM districts and 8% higher compactness than enacted. However, the 8% compactness improvement is measured over enacted maps which may themselves be gerrymandered (with artificially low compactness); the comparison is not to the theoretically maximum compactness achievable under VRA constraints. The "refutation" is too strong — the paper shows that one parameterization achieves both improvements over one specific baseline, which does not refute the general tradeoff.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — *Callais* citation absent.** The paper discusses "post-Callais VRA compliance" and "Callais cross-reference" in the abstract but *Alexander v. South Carolina State Conference of the NAACP* (commonly called *Callais* or the South Carolina gerrymander case) is not cited in the paper body (searching sections/01-09). If Callais is material to the legal framing, it must be cited with the full case citation. Using "post-Callais" as a period marker without citing the case will not pass APSR legal review.

**P2 — Gaming resistance claim.** The abstract claims method equivalence "eliminates gaming vectors" because the partition is deterministic given geography and demographics. This requires verifying that the $\alpha$ value is publicly specified and cannot be manipulated. If the redistricting authority can choose $\alpha$, gaming vectors exist at the $\alpha$-selection stage, not the partitioning stage.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — "Zero variance (p=1.0) across methods."** The abstract claims "statistical tests confirm zero variance (p=1.0) across methods for $\alpha = 5$." p=1.0 for a test of zero variance is unusual — standard tests report the probability of observing the test statistic under the null hypothesis of nonzero variance. A p-value of 1.0 suggests either a trivially small test statistic or an incorrect test specification. The paper should report the actual test statistic (F-statistic, Levene's test value) alongside the p-value.

**P2 — Runtime comparison (Alabama: 5–6 sec per predetermined tree; 32.6 sec for adaptive).** These times are single runs without standard deviation. The 32.6 sec for adaptive bisection is expected (it tests all 6 trees), but the 5.0–6.1 sec range for predetermined trees suggests hardware variance. Report mean ± SD over at least 3 runs.

---

## B.4 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Phase transition at α∈[20,50] doesn't explain equivalence at α=5 |
| R2 Rodden | 3/4 | 37.5% MM improvement is 5-state small sample without CI |
| R3 Duchin | 3/4 | "Algorithmic determinism" undefined; VRA tradeoff refutation overstated |
| R4 Stephanopoulos | 3/4 | *Callais* not cited despite "post-Callais" framing |
| R5 Liang | 3/4 | p=1.0 unusual; runtime single-run |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 5** (phase transition gap; MM headline CI; algorithmic determinism undefined; *Callais* absent; p=1.0 test)
**Top P1 issue: R1 — phase transition theorem (α_crit ∈ [20,50]) does not explain the equivalence result at α=5, which precedes the phase transition; theoretical gap requires resolution**

---

### B.4 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B4-P1-A | P1 | Theorem 2 proof: verify $\Theta(1)$ vs. $O(1)$ characterization for variance below $\alpha_{\text{crit}}$ | R1 |
| B4-P1-B | P1 | §4 / abstract: reconcile equivalence at $\alpha=5$ with phase transition at $\alpha\in[20,50]$; the theorem doesn't explain the result | R1 |
| B4-P1-C | P1 | Abstract: "37.5% more MM districts" needs sample-size qualifier (5 states) and confidence interval | R2 |
| B4-P1-D | P1 | "Algorithmic determinism": formally define or cite prior use | R3 |
| B4-P1-E | P1 | *Callais* (*Alexander v. South Carolina NAACP*): cite full case citation if "post-Callais" framing is used | R4 |
| B4-P1-F | P1 | p=1.0 zero-variance test: report test statistic (F or Levene's); explain why p=1.0 | R5 |
| B4-P2-A | P2 | VRA-compactness tradeoff refutation: soften to "shows one parameterization achieves both improvements over enacted baseline" | R3 |
| B4-P2-B | P2 | Gaming resistance claim: note that $\alpha$ selection is a potential gaming vector if not publicly fixed | R4 |
| B4-P2-C | P2 | Runtime: report mean ± SD over 3+ runs; current single-run ranges (5.0–6.1 sec) reflect hardware variance | R5 |

---

---

# B.5 — N-Way vs. Recursive Bisection: A General Architectural Comparison

**Directory**: `research/tracks/B-algorithm/B.5+nway-vs-recursive-general/`
**Venue target**: APSR
**Status going in**: No _panel.yaml; draft; no prior review

---

## R1 — Karypis

**Score: 3 / 4**

The +0.003–+0.004 mean PP advantage for recursive bisection is a small but consistent effect across all chamber types (Congressional, State Senate, State House). The paired t-test is appropriate ($n=50$ independent states, $p<0.001$, $d=0.31$). The finding that n-way is faster for state house with $k>80$ by 18–34% but with <200 ms absolute saving is a practically important null result.

**P1 — "Prime-k recoverable via post-processing" claim.** The abstract states "prime-$k$ chambers exhibit a narrowed advantage, recoverable via post-processing." This claim appears in the abstract but the paper section on prime-$k$ results should show the specific post-processing steps and how much of the advantage is recovered. If this is not shown empirically, the claim should be removed from the abstract.

**P2 — Block-group resolution comparison.** The abstract mentions "census-tract and block-group resolution" but the results table (Table 1) shows only census-tract resolution. Either report the block-group results or remove this from the abstract.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — 450 chamber-year combinations: multi-chamber inference.** The paper correctly restricts formal inference to the congressional sample ($n=50$ states) to avoid independence violations from state senate and state house chambers sharing the same census geography. This is a methodologically careful decision. However, the abstract says "450 chamber-year combinations" as if this is the statistical sample, which would imply 3 paired $t$-tests with $n=150$ each — but only the congressional test ($n=50$) is valid. The abstract should clarify that inference is restricted to the congressional sample and state senate/state house results are descriptive.

**P2 — "Compactness advantage is consistent and statistically significant" conclusion.** This is correct for congressional chambers but not formally established for state chambers (by the paper's own methodology). The conclusion should be restricted to congressional chambers.

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — Compactness metric choice.** The comparison uses Polsby-Popper throughout. PP is a perimeter-based compactness measure sensitive to boundary resolution. At the tract level vs. block-group level, PP values will differ because block-group boundaries are finer and produce different perimeters. The +0.003–+0.004 PP advantage for recursive bisection should be robust to compactness metric choice. The paper should verify that the same sign and magnitude holds for Reock or convex-hull ratio. If the advantage is PP-specific, this should be disclosed.

**P2 — Prime-factor treatment.** B.11 (ApportionRegions) establishes that prime-factor bisection trees are the correct treatment for non-power-of-two $k$. B.5's discussion of prime-$k$ should cite B.11 and clarify whether the "post-processing" mentioned in the abstract is consistent with B.11's prime-factor approach or a distinct method.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The paper makes no legal claims and makes no legal citations. No P1 legal issues.

**P2 — "Redistricting purposes" framing.** The conclusion "for redistricting purposes, recursive bisection is the dominant strategy" is a strong operational recommendation. For this to carry weight in a redistricting context, the paper should note that the dominant strategy is defined relative to compactness, not to other redistricting criteria (VRA compliance, county preservation, community-of-interest). The recommendation should be qualified as "for compactness-optimized redistricting."

---

## R5 — Liang

**Score: 3 / 4**

**P1 — No seed disclosure.** With 450 chamber-year combinations, the METIS seeds must be reported. The paper does not disclose seeds for any experiment. If the same seed is used for all runs, state it. If seeds are varied, report the seed distribution or use a fixed public seed.

**P2 — "Statistically significant" with $d=0.31$.** Cohen's $d=0.31$ is a small-to-medium effect. For APSR, small effects require discussion of practical significance. Is a 0.003–0.004 PP difference large enough to matter for redistricting outcomes or legal challenges? The paper should address whether this is practically meaningful, not just statistically significant.

---

## B.5 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Prime-k post-processing claim; block-group results absent from table |
| R2 Rodden | 3/4 | 450 chamber-year framing overstates inference; only congressional is formally tested |
| R3 Duchin | 3/4 | PP-specific result; B.11 prime-factor cross-reference missing |
| R4 Stephanopoulos | 3/4 | Dominant strategy recommendation needs compactness qualifier |
| R5 Liang | 3/4 | No seeds; practical significance of d=0.31 not discussed |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 4** (prime-k claim unsupported; 450-combination inference overstated; PP-metric specificity; no seeds)
**Top P1 issue: R2 — abstract framing of "450 chamber-year combinations" as the statistical sample overstates inference; only the 50-state congressional paired t-test is valid; state and house results are descriptive only**

---

### B.5 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B5-P1-A | P1 | Prime-$k$ post-processing: show empirically how much advantage is recovered, or remove from abstract | R1 |
| B5-P1-B | P1 | Abstract: clarify that inference is restricted to congressional ($n=50$); 450-combination claim overstates statistical scope | R2 |
| B5-P1-C | P1 | Report Reock or convex-hull ratio results to verify PP advantage is not metric-specific | R3 |
| B5-P1-D | P1 | All METIS runs: disclose random seed | R5 |
| B5-P2-A | P2 | Block-group resolution: either report BG results or remove from abstract | R1 |
| B5-P2-B | P2 | Conclusion: qualify "dominant strategy" as "dominant for compactness-optimized redistricting" | R4 |
| B5-P2-C | P2 | Practical significance of d=0.31 (0.003–0.004 PP): discuss whether this matters for redistricting decisions | R5 |
| B5-P2-D | P2 | Cite B.11 prime-factor tree when discussing prime-k chambers | R3 |

---

---

# B.6 — Computational Complexity of Recursive Bisection for Redistricting

**Directory**: `research/tracks/B-algorithm/B.6+computational-complexity/`
**Venue target**: JACM (implied by NP-hardness result)
**Status going in**: No _panel.yaml; no prior panel review

---

## R1 — Karypis

**Score: 4 / 4**

The NP-hardness reduction from Connected Planar Graph Bisection (Dyer and Frieze 1985) is correct. The reduction is: given an instance of CPGB, construct a population-balanced redistricting instance with unit vertex weights; any optimal redistricting solution yields an optimal bisection. This is a valid polynomial-time reduction.

The runtime theorem (Theorem 1: $O(\texttt{niter} \cdot n \log k)$ with $O(n \log k)$ space) is correctly derived. The per-level work analysis at §4.1 is sound: $O(n \log n)$ per bisection level, $\lceil \log_2 k \rceil$ levels, gives $O(n \log n \log k)$; with $\log n \approx 13$ for California and niter=100, the tightest correct bound is $O(100 \cdot n \log k)$. The paper correctly exposes niter as a constant multiplier.

**P1 — Empirical scaling ($O(n^{1.07 \pm 0.03})$) excludes single-district states.** The paper states "Single-district states ($k=1$) are excluded from the scaling analysis as they require no bisection." This is correct, but the abstract should state this exclusion when reporting the $O(n^{1.07 \pm 0.03})$ empirical scaling, since 7 states are excluded (Wyoming, Vermont, Delaware, North Dakota, South Dakota, Alaska at-large, Montana at-large in some redistricting cycles). The exponent is estimated from 43 states, not all 50.

**No further P1 items.** This is the strongest paper in the B.1–B.7 group algorithmically.

---

## R2 — Rodden

**Score: 3 / 4**

**P2 — "Legally defensible" claim.** The abstract states NP-hardness results "provide the complexity-theoretic foundation for the Districting Integrity Act's use of METIS as the statutory partitioner: exact optimisation is intractable, making heuristic approximation not merely acceptable but necessary." This is a policy claim that requires more careful framing. NP-hardness of the exact problem does not automatically make any specific heuristic (METIS) "legally defensible" — it makes heuristics *necessary* but doesn't choose among them. The statutory justification should be stated more carefully.

**No P1 items.** This paper does not make partisan claims.

---

## R3 — Duchin

**Score: 4 / 4**

The complexity result is clean and the proof is complete. Theorem 1 (NP-hardness) via Dyer-Frieze reduction is correct. Theorem 2 (runtime) is correctly derived from the multilevel METIS structure. The empirical scaling ($n^{1.07}$) is consistent with near-linear complexity, which is the expected behavior for planar graphs where $|E| = O(|V|)$.

**P2 — The approximation gap.** The paper states "no polynomial-time algorithm is known to achieve better than $O(\log n)$ approximation for balanced $k$-partition in general." This statement is for general graphs; the approximation ratio for planar graphs is better ($O(\sqrt{n})$ separator theorem, Lipton-Tarjan). The paper should distinguish between the general graph approximation bound and the planar graph result.

**No P1 items.**

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — Districting Integrity Act.** The abstract references "the Districting Integrity Act's use of METIS as the statutory partitioner." There is no enacted federal statute called the Districting Integrity Act. If this is a hypothetical statute proposed in B.02 (the one-sentence law paper) or in docs/legal/, the paper must clarify this: "the proposed Districting Integrity Act (as specified in [B.02])." Citing a nonexistent statute as a real law is a P1 legal error. JACM reviewers will flag this.

**P2 — "Statutory partitioner" framing.** If the DIA is a proposed statute (not enacted), all DIA-related claims should use hedged language: "under the proposed DIA..." rather than asserting it as existing law.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — Memory claim: "under 3 MB at tract resolution."** The abstract states "For California ($n=8{,}057$ tracts, $k=52$ districts), peak memory is under 3 MB at tract resolution." This appears very low — a graph with 8,057 nodes and approximately 24,000 edges in CSR format would require on the order of 1–2 MB for the graph alone, but METIS's multilevel structures (coarser graphs, partition vectors, refinement buffers) typically add a factor of 3–10x. Peak memory of 3 MB is plausible if only counting the METIS internal structures, but the paper should specify what is included in the memory measurement (graph CSR only? all METIS internal structures? population arrays?).

**P2 — Multi-year empirical scaling.** The abstract states empirical scaling is measured "across three census years (2000, 2010, 2020)." The paper should report the scaling coefficient separately for each census year and confirm consistency.

---

## B.6 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | Exclusion of 7 single-district states from empirical scaling should be in abstract |
| R2 Rodden | 3/4 | "Legally defensible" claim overreaches from NP-hardness alone |
| R3 Duchin | 4/4 | Planar approximation bound distinct from general-graph bound |
| R4 Stephanopoulos | 3/4 | "Districting Integrity Act" cited as enacted law; it is a proposed statute |
| R5 Liang | 3/4 | 3 MB memory claim scope unclear; multi-year scaling not year-separated |
| **Average** | **3.4/4** | |

**Verdict: Accept** (3.4 ≥ 3.0)
**P1 count: 3** (empirical scaling excludes 7 states — needs disclosure; DIA cited as enacted law; memory measurement scope)
**Top P1 issue: R4 — "Districting Integrity Act" cited as a real statutory framework in the abstract; it is a proposed statute from B.02 and must be labeled as such**

---

### B.6 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B6-P1-A | P1 | Abstract: $n^{1.07}$ scaling is from 43 states ($k \geq 2$); disclose that 7 single-district states are excluded | R1 |
| B6-P1-B | P1 | "Districting Integrity Act": cite as "the proposed DIA (as specified in B.02)" rather than existing law | R4 |
| B6-P1-C | P1 | Memory claim: specify what is included in "under 3 MB" (graph CSR only? all METIS structures?) | R5 |
| B6-P2-A | P2 | "Legally defensible" framing: NP-hardness justifies heuristics in general, not METIS specifically; soften | R2 |
| B6-P2-B | P2 | Approximation bound: distinguish $O(\log n)$ for general graphs from $O(\sqrt{n})$ Lipton-Tarjan planar separator bound | R3 |
| B6-P2-C | P2 | Multi-year empirical scaling: report coefficient per census year (2000, 2010, 2020) separately | R5 |

---

---

# B.7 — Solution Space and Seed Sensitivity in METIS Redistricting

**Directory**: `research/tracks/B-algorithm/B.7+solution-space-and-seed-sensitivity/`
**Venue target**: Political Analysis
**Status going in**: No _panel.yaml; draft with external engagement per REVIEW_PANEL.md

---

## R1 — Karypis

**Score: 3 / 4**

The 10,000-seed sweep across all 50 states is the most empirically intensive study in B.1–B.7. The CV < 2% finding for 48 states is a strong result. The CV = 4.3% (GA) and 3.8% (NC) outliers are correctly attributed to non-power-of-two $k$ and geographic complexity creating multiple near-optimal separator locations.

**P1 — "DIA seed is within 2.9% ± 1.7% of the minimum-edge-cut seed."** The ±1.7% is a standard error or confidence interval, but the paper should specify which. If it is a standard error across the 50-state sample (population SD / sqrt(50)), that is different from a 95% CI. Political Analysis readers will need the interval type stated explicitly.

**P2 — 500,000 METIS calls.** The abstract states "500 thousand total METIS calls" (10,000 seeds × 50 states). This is a computational claim. The paper should report the total compute time for this study and the hardware used, so the result is reproducible (or at least the scale is assessable).

---

## R2 — Rodden

**Score: 4 / 4**

The seed sensitivity result is politically important. The finding that partisan seat-share variance across 10,000 seeds is at most 2 seats for the two highest-variance states certifies that the DIA's single-seed specification does not create a manipulation vector through seed selection.

**P1 — "Statistically indistinguishable from a randomly chosen seed."** The DIA seed is within 2.9% ± 1.7% of the minimum-edge-cut seed. The claim "statistically indistinguishable from a randomly chosen seed" implies the DIA seed falls within the distribution of random seeds — but the distribution of random seeds is not described. The claim should be stated as: "the DIA seed achieves an approximation gap within the typical range of randomly chosen seeds (median gap 3.1%), providing no systematic advantage over a random seed selection."

**No further P1 items from a political-science perspective.**

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — Partisan seat-share variance is not the same as plan-space variance.** The paper reports "partisan seat-share variance across 10,000 seeds is at most 2 seats for WI and NC." This measures variance in a scalar summary statistic (partisan seat share), not variance in the plan-space geometry. Two plans can have identical seat counts but very different district boundaries. The paper should clarify that seed sensitivity is characterized at the level of partisan outcomes, not at the level of plan geometry (which would require Hamming distance or similar metrics).

**P2 — ConvergenceSweep (B.16) motivation.** The abstract states the results "motivate the $T=600$ ConvergenceSweep threshold (B.16)." The logic connecting CV < 2% seed sensitivity to T=600 should be made explicit. If the convergence threshold is derived from seed sensitivity (e.g., "at CV < 2%, 600 seeds suffice to find a near-optimal plan"), this derivation should appear in the paper or as a forward-reference to B.16.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The paper references the "Districting Integrity Act" (DIA) as the statutory framework that specifies a single deterministic seed derived from the census release identifier via SHA-256. Same comment as B.6: the DIA is a proposed statute from B.02, not an enacted law. All DIA references should use "proposed" or "as specified in B.02."

**P1 — Same as B.6: DIA is a proposed statute, not enacted.** Must be consistently labeled as proposed across all papers in the track.

---

## R5 — Liang

**Score: 4 / 4**

The 10,000-seed sweep is the paper's core reproducibility contribution. The DIA seed is SHA-256 derived from the census release identifier — this is a deterministic, public, and auditable seed. The method is reproducible by construction.

**P2 — "10,000 seeds per state" vs. "500,000 total."** 10,000 × 50 = 500,000, which is correct. The paper should also report whether the seeds are drawn uniformly from the u64 space (full 64-bit range) or from a restricted range. The specific seed generation methodology (e.g., seeds 0 through 9,999 for each state, or SHA-256 derived seeds) should be stated for reproducibility.

---

## B.7 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | ±1.7% interval type must be specified; compute time/hardware for 500K calls |
| R2 Rodden | 4/4 | "Statistically indistinguishable" framing should cite the random-seed distribution |
| R3 Duchin | 3/4 | Seat-share variance ≠ plan-space variance; B.16 T=600 derivation unclear |
| R4 Stephanopoulos | 3/4 | DIA cited as enacted law; should be "proposed statute" |
| R5 Liang | 4/4 | Seed generation methodology not specified |
| **Average** | **3.4/4** | |

**Verdict: Accept** (3.4 ≥ 3.0)
**P1 count: 3** (±1.7% type ambiguous; "statistically indistinguishable" framing needs reference distribution; DIA proposed-statute labeling; seat-share vs. plan-space variance distinction)
**Top P1 issue: R3 — "partisan seat-share variance at most 2 seats" characterizes scalar-outcome sensitivity only; plan-space geometry sensitivity (Hamming distance across seeds) is not measured and should not be inferred from the seat-share result**

---

### B.7 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B7-P1-A | P1 | ±1.7%: specify whether standard error or 95% CI | R1 |
| B7-P1-B | P1 | "Statistically indistinguishable from a randomly chosen seed": cite the random-seed distribution being referenced | R2 |
| B7-P1-C | P1 | DIA: label as "the proposed DIA (B.02)" not "the DIA" as if enacted | R4 |
| B7-P1-D | P1 | "Partisan seat-share variance at most 2 seats" does not characterize plan-space geometry variance; add explicit qualification | R3 |
| B7-P2-A | P2 | Total compute time and hardware for 500,000 METIS calls | R1 |
| B7-P2-B | P2 | T=600 ConvergenceSweep motivation: make the connection from CV<2% to T=600 explicit or forward-reference B.16 | R3 |
| B7-P2-C | P2 | Seed generation method: specify range and distribution (0..9999 per state? SHA-256 derived?) | R5 |

---

---

## B-algorithm Batch 1 Summary Table

| Paper | Avg Score | Verdict | P1 Count | Top P1 Issue |
|-------|-----------|---------|----------|--------------|
| B.1 Recursive Bisection | 3.0/4 | Accept | 6 | "Structural immunity" in abstract falsified by §5 own results; must be revised to "immunity to direct partisan input" (R2) |
| B.2 Edge-Weighted Bisection | 3.2/4 | Accept | 5 | Partisan analysis tables present simulated 40%-assumption data without SIMULATED label; APSR will reject (R2) |
| B.3 Multi-vs-Edge | 3.0/4 | Accept | 5 | Theorem 1 (constraint conflict) is an informal argument presented as a formal theorem; must be proven or relabeled (R3) |
| B.4 Adaptive Bisection | 3.0/4 | Accept | 6 | Phase transition (α∈[20,50]) doesn't explain equivalence at α=5; theoretical gap unresolved (R1) |
| B.5 N-Way vs. Recursive General | 3.0/4 | Accept | 4 | "450 chamber-year combinations" overstates statistical scope; only 50-state congressional paired test is valid (R2) |
| B.6 Computational Complexity | 3.4/4 | Accept | 3 | "Districting Integrity Act" cited as enacted law in abstract; it is a proposed statute from B.02 (R4) |
| B.7 Seed Sensitivity | 3.4/4 | Accept | 4 | Seat-share variance (≤2 seats) characterizes scalar outcomes only; plan-geometry sensitivity is unmeasured and should not be inferred (R3) |

---

## Cross-Cutting Findings

### Finding 1 — "redist" binary name (affects B.1)
B.1 refers to CLI commands using the old binary name "redist." All papers must be updated to use `bisect` following the Phase 1 rename. Verified: B.5, B.6, B.7 use correct `bisect` naming (papers written after the rename). B.1 was written before the rename.

### Finding 2 — DIA proposed-statute framing (affects B.6, B.7)
Both B.6 and B.7 reference the "Districting Integrity Act" as if it is an enacted federal statute. It is a proposed statute from B.02. Every paper in the track that references the DIA must add "proposed" or "as specified in B.02."

### Finding 3 — Simulated partisan data presented as empirical (affects B.1, B.2)
B.1 and B.2 both contain partisan analysis based on simulated or modeled vote distributions. Neither paper labels its partisan tables as SIMULATED at the table level (only in a Limitations paragraph). APSR political-science reviewers will reject papers that present model outputs as empirical findings without prominent disclosure.

### Finding 4 — Legal citation gap (affects B.3, B.4)
B.3 characterizes the VRA 50% threshold without citing *Thornburg v. Gingles* (1986). B.4 uses "post-Callais" framing without citing *Alexander v. South Carolina NAACP*. Both are blocking P1 issues for law-review or social-science journal submissions.

### Finding 5 — All papers accepted
All seven papers score ≥ 3.0/4 and receive Accept verdicts. B.5 (3.0/4) and B.1 (3.0/4) are at the acceptance threshold. B.6 (3.4/4) and B.7 (3.4/4) are the strongest papers in the batch.

---

*Panel convened 2026-05-07. Reviewer identities are simulated personas for pre-submission quality control. B.1–B.7 are the foundational papers of the B-algorithm track; all seven accepted with revisions required.*
