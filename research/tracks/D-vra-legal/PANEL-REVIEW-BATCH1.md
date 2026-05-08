# D-VRA-Legal Track — Panel Review Batch 1

**Date**: 2026-05-07
**Reviewers**: R1 Karypis (graph algorithms), R2 Rodden (political science), R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/AI)
**Scoring**: 0–4 per reviewer. Average ≥ 3.0 → Accept; ≥ 2.5 → Minor Revision; ≥ 2.0 → Major Revision; < 2.0 → Reject.

---

## D.0 — VRA Compliance (Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning)

### Reviewer Scores

**R1 — Karypis (3)**: The edge-weighting scheme is technically sound: assigning higher METIS edge weights to edges between minority tracts directly encodes "don't split minority communities" into the graph objective function. The ablation study (7 weight factors × 4 thresholds × 5 states = 140 configurations) is well-designed. The finding that 5x–10x weights are optimal, with diminishing returns above 100x, is consistent with how METIS handles large edge weight ratios (extreme weights can distort the coarsening phase). The 80% success rate (4/5 states) for per-state-optimal configurations vs 40% for multi-constraint is credible. One concern: the adaptive formula (V4 production) only achieves 2/5 states at full legal targets, while the ablation achieves 4/5 — this discrepancy (ablation = best per-state config, production = single adaptive formula) is acknowledged in footnotes but the abstract headline "80% success" conflates these. The abstract should lead with the production result.

**R2 — Rodden (3)**: The 5-state pilot (Alabama, Georgia, Louisiana, Mississippi, South Carolina) is an appropriate set for VRA Section 5 testing. The critical threshold finding (~42% state-wide minority for success) is important and properly documented. The discrepancy between ablation performance (80%: 4/5 states) and V4 production performance (40%: 2/5 states) creates a headline accuracy problem — the abstract claims 80% success rate, but this is the ablation result with per-state-optimal fixed configurations, not the production system. A reader deploying the production `bisect --structure ratio-optimal-vra` would achieve 40%, not 80%. The abstract should clarify "80% in ablation, 40% in V4 production."

**R3 — Duchin (3)**: The paper correctly identifies geographic clustering as the primary determinant of VRA feasibility, not algorithmic sophistication. The Alabama breakthrough (49.6% maximum with multi-constraint → 50.8% with edge-weighting) is the key empirical result and it is cleanly demonstrated. The South Carolina failure is correctly attributed to fundamental demographic limits. The ubvec (constraint relaxation) analysis is appropriately secondary. One concern: the paper's note that "Post-Callais (2026), the VRASection algorithm (`--structure ratio-optimal-vra`) implemented in the `redist` Rust binary provides the recommended production implementation, superseding the Python-based workflow originally used" is important — this means the core results were generated with a *different* codebase than the current production tool. The METIS parameters used in the Python workflow may differ from the Rust implementation. This should be explicitly flagged as a limitation.

**R4 — Stephanopoulos (3)**: The VRA Section 2 framework is correctly invoked. The paper correctly cites *Thornburg v. Gingles* (1986) and *Bartlett v. Strickland* (2009) for the majority-minority district requirement. The 42% threshold finding is framed appropriately as "guidance for assessing geographic feasibility of proportional MM representation (Gingles prong 1)," not as a legal rule — this is the correct framing. *Allen v. Milligan* (2023) is mentioned in the implementation note, correctly. No invented citations. The paper does not claim the 42% threshold is a legal standard — only an empirical guideline.

**R5 — Liang (3)**: The 140-configuration ablation is adequately powered for the conclusions drawn. The V4 production results are separately footnoted with Mississippi's stochastic boundary condition clearly explained (District 1 at 49.99% — METIS randomness at the threshold). The seed is not reported for production runs, which is a reproducibility concern. The implementation note correctly identifies that results labeled "V4 production" reflect the adaptive formula, and ablation results reflect fixed per-state-optimal configurations — these are two different experimental conditions and should be clearly distinguished in the abstract, not just footnotes.

**Average**: (3+3+3+3+3) / 5 = **3.0**
**Verdict**: **Accept** (marginal; the ablation-vs-production conflation in the abstract is a P1 issue that warrants revision before journal submission)

### Priority Issues (P1)
1. **Abstract conflates ablation (80%) and production (40%) success rates**: "This novel approach doubles success rate to 80% (4/5 states)" — this is the ablation result with per-state-optimal fixed configs, not the adaptive formula production result. The adaptive V4 formula achieves 40% (2/5 states at full legal targets). The abstract should distinguish these.

### P3 Issues
- `\texttt{redist} Rust binary` in multiple methodology sections. Should be `bisect`. **P3 flag.**

---

## D.1 — Threshold Analysis (The 42% Threshold: Geographic Limits of VRA Compliance)

### Reviewer Scores

**R1 — Karypis (3)**: The comprehensive 43-state, 645-configuration analysis is the largest empirical study in the D track. The correlation r=0.78 (p<1e-8) between state minority percentage and optimization success is a strong finding. The jackknife sensitivity analysis (threshold shifts by at most ±1-2 pp when any single state is excluded) supports robustness. The logistic regression approach to threshold identification is appropriate. The paper correctly notes that Mississippi's V4 production result (49.99%) is a stochastic boundary condition, not a reliable failure. One concern: the paper conflates total population minority percentage with CVAP in some places — the threshold analysis uses total population, but VRA enforcement uses CVAP for effectiveness. This distinction matters for threshold applicability.

**R2 — Rodden (4)**: This is exactly the kind of empirical work courts and legislatures need. The 40-44% confidence interval for the effectiveness threshold provides actionable guidance: states ≥44% can proceed with confidence; states ≤40% face genuine geographic constraints; the 40-44% range requires case-specific analysis. The regional jackknife (excluding any single region leaves threshold at 41-43%) demonstrates national applicability. The geographic effects section (Moran's I analysis) correctly shows that metropolitan concentration can enable sub-proportional MM success at below-threshold state-level minority percentages. The table at the end (practical feasibility guidelines) is the right policy format. One note: Alabama's "below threshold" classification (36.9%) despite achieving its *legal* VRA target (2 MM districts, not the proportional 3) is handled correctly in the footnote — the paper consistently uses proportional targets for cross-state comparability, not legal obligations.

**R3 — Duchin (4)**: The distinction between "any success (≥1 MM district)" criterion (threshold ~15-20%) and "proportional success" criterion (threshold ~42%) is important and correctly drawn. The borderline state analysis (Illinois, Louisiana, North Carolina) correctly identifies metropolitan concentration as the mediating variable. The statistical power analysis (power >0.999 for r=0.78 at N=43) is correctly computed. The confidence interval for the correlation ([0.63, 0.88] at 95%) is correctly stated. The paper appropriately acknowledges that the threshold is for *proportional* MM representation, not for creating *any* MM district — courts need both benchmarks and the paper provides both.

**R4 — Stephanopoulos (4)**: The legal framing is careful. The paper does not assert the 42% threshold as a legal rule — it is explicitly described as "empirically validated guidance for assessing geographic feasibility of proportional MM representation (Gingles prong 1)." The footnote on Alabama correctly notes that Alabama's *legal* VRA target under *Allen v. Milligan* (2023) is 2 MM districts, which is achievable; the "failure" in the below-threshold classification is relative to the proportional target (3), not the legal target. This distinction is legally critical and handled correctly. *Allen v. Milligan* is correctly cited. No invented citations.

**R5 — Liang (4)**: The 645-configuration analysis (43 states × 15 configurations each) is large and well-documented. The methodological robustness section tests multiple success criteria, multiple statistical tests, and multiple threshold identification methods — all yielding stable estimates around 42%. The jackknife analysis is appropriate for identifying influential observations. The power analysis is correctly computed. The sample of 43 multi-district states (86% of all states with multiple districts) represents the complete relevant population, so sampling concerns do not apply. This is a reproducible, rigorous empirical paper.

**Average**: (3+4+4+4+4) / 5 = **3.8**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. The abstract claims (42% threshold, r=0.78, p<1e-8, 62% success rate above threshold, 0% below 37%) are all consistent with Table 1 and Table 2 in the paper.

### P3 Issues
- No `redist` binary references in this paper's main sections. Clean.

---

## D.2 — N-Way vs Recursive VRA (N-way vs Recursive Bisection for VRA-Compliant Redistricting)

### Reviewer Scores

**R1 — Karypis (4)**: The 1,760-run ablation (880 n-way + 880 recursive bisection, 5 weight factors × 4 thresholds × 44 states × 2 methods) is the most computationally intensive study in the D track. The finding of statistical equivalence (n-way: 47.5%, recursive: 48.3%, p=0.634, Cohen's d=-0.018) is the central empirical result and is reported correctly with the null result: the p-value is high (not rejecting equivalence) and the effect size is negligible. The 67.5% faster runtime for n-way (3.68s vs 11.33s) is correctly attributed to the hierarchical overhead of recursive bisection. The state-specific advantages (Virginia +35% for n-way, Connecticut +45% for recursive) are sensible given geographic structure differences. One minor concern: the paper notes that n-way prefers 40% threshold while recursive prefers 50% — this parameter sensitivity finding would benefit from a brief mechanistic explanation.

**R2 — Rodden (3)**: The equivalence finding is policy-relevant: practitioners can choose either method based on computational constraints without sacrificing VRA effectiveness. The paper correctly characterizes this as "the methods achieve comparable VRA compliance when properly configured" — not that all configurations are equivalent, but that peak performance is similar. The statement that n-way offers "67.5% faster runtime... making it preferable for rapid prototyping and large-scale analysis" is an appropriate practical recommendation. The paper does not overclaim equivalence in all settings — it correctly identifies the higher performance ceiling for recursive (56.8% vs 52.3% best configuration) as a meaningful difference for careful tuning.

**R3 — Duchin (3)**: The equivalence test is appropriate for the main claim. The 44-state scope is nearly the complete population of multi-district states. The disaggregated state-specific results (Connecticut, Virginia) provide evidence that equivalence is not simply averaging out large differences in opposite directions. The paper could more clearly address whether equivalence holds at all parameter values or only near-optimal — the current presentation suggests it holds overall but not state-by-state. A table of per-state results (or at minimum per-region) would strengthen the equivalence claim.

**R4 — Stephanopoulos (3)**: The paper is primarily methodological and makes no direct legal claims. The implications section correctly frames the equivalence finding as guidance for redistricting commissions and courts considering method choice — "practitioners can confidently choose either method based on computational constraints rather than effectiveness concerns." This is a reasonable non-legal practical recommendation. No invented citations. The absence of legal analysis is appropriate for a methods-comparison paper.

**R5 — Liang (4)**: The 1,760-run experiment is the most rigorous in this track. Runtime measurements (3.68s vs 11.33s) from a consumer laptop (Windows 11, 16GB RAM) are appropriately contextualized. The `redist states --workers 4` specification is reproducible. The statistical test for equivalence (p=0.634) correctly frames the null hypothesis as equivalence and interprets the high p-value as failure to reject H₀ — this is standard but still worth checking for null hypothesis interpretation errors, and the paper does it correctly. The Cohen's d of -0.018 confirms negligible effect size.

**Average**: (4+3+3+3+4) / 5 = **3.4**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claims (n-way 47.5%, recursive 48.3%, p=0.634, d=-0.018, 67.5% faster runtime) are stated in the abstract and consistent with methodology section descriptions.

### P3 Issues
- `\texttt{redist}` binary name in methodology section (`redist states --workers 4`, `redist Rust binary with --partition-mode metis-vra`). Should be `bisect`. **P3 flag.**

---

## D.3 — Compactness Tradeoff (Quantifying the VRA-Compactness Tradeoff)

### Reviewer Scores

**R1 — Karypis (4)**: The 105-configuration ablation (1 baseline + 5 weight factors × 4 thresholds × 5 states) is well-designed. The compactness metrics (edge cut, Polsby-Popper, Reock, convex hull ratio) with cross-metric correlations (r≈-0.6 between edge cut and geometric metrics, r>0.7 between geometric metrics) provide a robustness check on the measurement framework. The ensemble validation against 10,000 ReCom plans for Alabama is technically rigorous: the edge-weighted METIS optimal configuration (5×@45%) is dominated by only 0.4% of the ensemble, placing it near the Pareto frontier. The feasibility ratio analysis (MM%/minority%) provides a clean geometric explanation for South Carolina's infeasibility (ratio 1.22 > feasible range). The Georgia "both gain" result is surprising but mechanistically explained through the three mechanisms (geographic clustering, clearer non-MM boundaries, non-optimal baseline). Technically sound.

**R2 — Rodden (4)**: The non-MM districts gain finding (+7.5% average, statistically significant at p=0.039 with Cohen's d=0.169) is the key empirical result and is carefully reported — small effect size acknowledged, consistent direction across states emphasized. The CVAP robustness section is important for VRA applicability: a district with 50% total minority may have ~47.4% CVAP, but the paper correctly notes that (1) most MM districts exceed 50% by comfortable margins and (2) coalition districts below 50% CVAP can still provide effective representation. The Pareto frontier analysis provides courts with a rigorous tool for assessing whether challenged plans are optimal. Louisiana's both-lose result is honest about the limitations of the approach.

**R3 — Duchin (4)**: The ensemble comparison (Alabama vs 10,000 ReCom plans) is the gold standard for validating that edge-weighted METIS achieves genuine Pareto optimality rather than algorithm-specific artifacts. Finding that the optimal configuration is dominated by only 0.4% of ReCom plans (35/10,000) demonstrates it is near the true Pareto frontier. The three mechanisms (geographic clustering enables joint optimization, non-MM districts benefit from clearer boundaries, baseline partitions are locally not globally optimal) are explanatory and testable. The "baseline paradox" in Mississippi (baseline already achieves VRA compliance without optimization) is correctly identified as reflecting high state-level minority percentage. Statistical significance testing is comprehensive (t-test, bootstrap CI, permutation test).

**R4 — Stephanopoulos (4)**: The legal implications are carefully stated. The compactness defense ("our maps are reasonably compact, therefore any partisan bias is geographic") is refuted by the evidence that Georgia achieves better compactness *with* MM districts than without — this is a genuinely useful finding for VRA litigation. The Pareto frontier framework is proposed as a litigation tool: plans below the frontier are unjustifiable. The *Shaw v. Reno* (1993) citation for the "bizarre shapes" doctrine is correctly used. The feasibility ratio threshold (≈1.2) is proposed as a quantitative tool for courts assessing Gingles Prong 1 — this is appropriately framed as empirical guidance, not a legal rule. *Thornburg v. Gingles* (1986) is correctly cited. No invented citations.

**R5 — Liang (4)**: The CVAP adjustment section is technically precise: the age adjustment (Black VAP ratio 0.72, total VAP ratio 0.76) is correctly computed to yield ~47.4% CVAP for a 50% total population district. The impact on main findings (only 1/18 MM districts risks reclassification) is carefully documented. The ensemble comparison methodology (ReCom Markov chain, 10,000 plans, dominance computation) is reproducible. The parametric statistics (t-tests), bootstrap (10,000 resamples), and permutation tests (10,000 permutations) are all reported with correct methodological descriptions. The supplement structure (appendices A-E with clear contents list) supports reproducibility.

**Average**: (4+4+4+4+4) / 5 = **4.0**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claims (non-MM +7.5% compactness gain, MM -25.3% loss, Georgia win-win at +22.2%, South Carolina infeasibility with feasibility ratio 1.22) are all consistent with Table 1 (cross-state summary) and Table 2 (MM vs non-MM breakdown).

### P3 Issues
- `\texttt{redist}` binary in methodology section (`redist Rust binary (redistricting computation)`). Should be `bisect`. **P3 flag.**

---

## D.4 — Legal Implementation (Adopting Algorithmic Congressional Redistricting: Legal Pathways)

### Reviewer Scores

**R1 — Karypis (3)**: The technical description of the algorithm is appropriately abstract for a legal paper. The population balance claim (±0.5% for 98% of districts across all 50 states and three census years) is correctly cited to the foundational paper. The Huntington-Hill analogy (mathematical formula mandated by statute for apportionment) is structurally apt. The technical aspects of the model statute (Section 2 of the model: algorithm certification requirements) are algorithmically sound. Not the reviewer's primary domain, but no technical errors found.

**R2 — Rodden (3)**: The paper correctly identifies the "structural problem" (access to partisan data) as distinct from the "intent problem" (partisan motivation). The footnote correctly notes that geographic sorting produces Democratic advantage in algorithmic maps, tracing this to urban-rural sorting rather than design — this is accurate and appropriately placed. The commission adoption evidence (Arizona, California, Michigan, Colorado) is cited correctly. The paper does not claim commissions eliminate partisan bias — it accurately states they "reduce the most egregious partisan bias but do not eliminate it." The footnote reference to D.0's geographic sorting efficiency gap finding is appropriate. One concern: the paper would benefit from a brief discussion of what the algorithmic EG floor means for legislative buy-in — legislators who lose under algorithmic plans will observe the Democratic tilt and may oppose adoption on those grounds.

**R3 — Duchin (3)**: The constitutional analysis is thorough for a legal article. The nondelegation analysis is the most technically demanding section — the paper correctly distinguishes Congress mandating an algorithm (which delegates no discretion to an executive agency) from Congress delegating rulemaking authority to an agency with standards. This is legally sound. The VRA conflict analysis correctly notes that the algorithm's VRA mode satisfies Gingles Prong 1 through geographic alignment scores (D.5) rather than racial targeting. The model statute text in Section 7 is the most operationally valuable contribution.

**R4 — Stephanopoulos (4)**: This is a careful and rigorous legal article. *Smiley v. Holm* (1932) is correctly cited for Elections Clause breadth. *Ex parte Siebold* (1879) is correctly cited. *Wesberry v. Sanders* (1964) and *Karcher v. Daggett* (1983) are correctly cited for population equality requirements. *Arizona State Legislature v. AIRC* (2015) is correctly cited for the commission constitutional authority precedent. *Rucho v. Common Cause* (2019) is correctly used throughout — the paper does not claim *Rucho* bars federal legislative action, only federal judicial action, which is the correct reading. *Louisiana v. Callais* (2026) is correctly cited in the model statute. All citations verified as real and correctly characterized. The state-pathways analysis (42/50 states compatible without amendment) is a specific claim that the reviewer did not independently verify but the categorization methodology is clearly described.

**R5 — Liang (3)**: The model statute text is the most reproducible artifact in this paper — it specifies concrete technical requirements (certification process, algorithm audit, seed derivation, variance reporting). The paper correctly references the bisect CLI for the worked example commands in C.9 (cross-reference is appropriate). The paper itself contains no empirical claims requiring validation. The concern is that the technical annexes to the model statute (Section 2 technical appendix referenced in the statute text) are not included in the paper — the statute mandates "certified algorithm" and "technical appendix" but these are specified only at the framework level. The implementation paper (C.9) provides the practical complement.

**Average**: (3+3+3+4+3) / 5 = **3.2**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. The legal claims are accurately stated and citations are verified. The model statute is self-contained.

### P2 Issues
- No CLI flag usage in this paper. Clean.

### P3 Issues
- No `redist` binary references in this paper's sections. The paper uses system-level references ("the algorithm," "the redistricting system") without naming the binary. Clean.

---

## D.5 — Gingles Bloc Voting Methodology (Quantifying VRA Section 2 Evidence)

### Reviewer Scores

**R1 — Karypis (3)**: The VRASection geographic alignment score ($A_d$) is technically defined as the share of a minority community captured in the optimal algorithmic district relative to a statewide baseline. This is a reasonable operationalization for Prong 1. The WLS regression framework for Prongs 2 and 3 is not primarily algorithmic, but the bootstrap CI from the ensemble is correctly motivated. The HC3 heteroskedasticity-consistent standard errors are the appropriate choice for ecological inference with heterogeneous tract sizes. No algorithmic correctness issues found.

**R2 — Rodden (4)**: This is a methodologically careful paper in an area where expert witness practice has historically been inconsistent. The WLS+HC3+Holm framework for Prong 3 is the correct post-*Callais* approach. The Prong 2 analysis using primary election data (not general election, which is confounded by party loyalty) is the right methodological choice — using primary elections isolates minority cohesion from partisan cohesion. The Alabama worked example correctly disentangles racial from partisan bloc voting: β_race drops from 0.71 (without partisan control) to 0.43 (with control), leaving a residual 0.43 racial component. This is methodologically clean and consistent with the *Callais* disentanglement requirement. The finding that all five elections yield corrected p-values < 0.01 under Holm correction is well-documented.

**R3 — Duchin (4)**: The operationalization of Prong 1 using VRASection alignment scores (threshold 0.5: the district captures at least 50% of the minority community) is principled and reproducible, replacing ad hoc visual inspection. The bootstrap CI for Prong 2 cohesion estimates (from the ensemble) is the correct approach for quantifying uncertainty in ecological inference. The multi-election LOO analysis for Prong 3 (excluding any single election does not change the conclusion) is appropriate for testing robustness. The expert witness checklist in Section 7 maps `bisect` outputs to each prong, which is operationally useful. The methodology is algorithm-agnostic for Prongs 2 and 3, which broadens applicability appropriately.

**R4 — Stephanopoulos (4)**: *Thornburg v. Gingles* (1986) is correctly cited and the three prongs are accurately described. *Allen v. Milligan* (2023) is correctly cited and its significance accurately characterized (reaffirmed Gingles, rejected race-neutral comparator). *Louisiana v. Callais* (2026) is correctly cited for the disentanglement requirement. *Rucho v. Common Cause* (2019) is correctly cited for the partisan-defense context. The LOO analysis for Prong 3 addresses the standard legal challenge to expert testimony that relies on a few elections. The "totality of circumstances" inquiry is correctly noted as following after all three prongs are satisfied. This is the strongest legal paper in the D track for legal citation precision.

**R5 — Liang (4)**: The Alabama worked example provides concrete numbers for each prong (Prong 1: A_d = 0.73 and 0.61; Prong 2: β_min = 0.84–0.91 across 5 elections, p<0.001; Prong 3: β_race = 0.41–0.45 with partisan control, p<0.001 after Holm correction). These numbers are internally consistent and the table (Table 1, Prong 3 results) is fully populated with real data — no TBD entries. The `bisect analyze --types bloc-voting` command is cited as the implementation reference. The bootstrap CI methodology is correctly described. The paper is the most reproducible in the D track.

**Average**: (3+4+4+4+4) / 5 = **3.8**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. All three prong operationalizations are internally consistent. The Alabama worked example is fully populated with concrete numbers. The abstract claims (Prong 1 alignment score threshold, Prong 2 WLS+HC3, Prong 3 WLS+HC3+Holm with partisan control) are consistent with the methodology sections.

### P1 Note: Mixed `redist`/`bisect` naming
The abstract references `\texttt{redist}` system; Section 1 introduction uses both `\texttt{redist}` and `\texttt{bisect}` in the same paragraph (line 39: "using the \texttt{redist} algorithmic redistricting system" then line 46: "`\texttt{bisect analyze --types bloc-voting}`"). This mixed naming is confusing but is not a P1 empirical inconsistency — it is a **P3 naming issue**.

### P3 Issues
- Mixed `redist`/`bisect` naming: abstract, conclusion, and some section introductions use `redist`; method references use `bisect`. **P3 flag.** The expert witness checklist (Section 7) correctly uses `bisect` throughout — apply this consistently to the abstract and introduction.

---

## Summary Table

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 Count | Top P1 Issue |
|-------|----|----|----|----|-----|-----|---------|----------|--------------|
| D.0 VRA Compliance | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 1 | Abstract conflates ablation (80%) and production (40%) success rates |
| D.1 Threshold Analysis | 3 | 4 | 4 | 4 | 4 | 3.8 | Accept | 0 | — |
| D.2 N-Way vs Recursive VRA | 4 | 3 | 3 | 3 | 4 | 3.4 | Accept | 0 | — |
| D.3 Compactness Tradeoff | 4 | 4 | 4 | 4 | 4 | 4.0 | Accept | 0 | — |
| D.4 Legal Implementation | 3 | 3 | 3 | 4 | 3 | 3.2 | Accept | 0 | — |
| D.5 Gingles Bloc Voting | 3 | 4 | 4 | 4 | 4 | 3.8 | Accept | 0 | — |

### Track-Level Observations

**Strengths**: D.3 (Compactness Tradeoff) and D.5 (Gingles Methodology) are the strongest papers. D.3's ReCom ensemble validation against 10,000 plans is the most rigorous empirical validation in the entire D track. D.5's post-*Callais* WLS+HC3+Holm methodology is immediately usable as an expert witness framework and all three prong operationalizations are internally consistent with populated data tables.

**D.1 (Threshold Analysis)** is also strong — the 43-state, 645-configuration analysis with jackknife robustness testing provides the most comprehensive empirical basis for the 42% threshold finding. The legal framing (empirical guidance for Gingles Prong 1, not a legal rule) is exactly right.

**D.0 has one P1 issue**: The abstract's "80% success rate" is the ablation result, not the production system result (40%). For papers that will be cited in expert testimony or submitted to journals, this distinction is material. Revision recommended before journal submission, though the paper is accepted at this stage.

**D.4 (Legal Implementation)** is solid legal scholarship with no citation errors. The model statute text in Section 7 is operationally valuable. Minor opportunity: the paper could address the political economy of legislative adoption (the Democratic tilt concern mentioned by R2) more directly.

**Recurring P3 issue**: The binary rename from `redist` to `bisect` is incomplete across D.0 (methodology), D.2 (methodology), D.3 (methodology), and D.5 (abstract/introduction). D.4 and D.1 are clean. A global find-and-replace pass is needed across the track for production-level documents.

**No serious legal citation errors** found in any D-track paper. All case citations (*Gingles*, *Allen v. Milligan*, *Callais*, *Rucho*, *Smiley v. Holm*, *Wesberry*, *Karcher*, *Shaw v. Reno*) are real cases correctly characterized. The D track is legally sound.
