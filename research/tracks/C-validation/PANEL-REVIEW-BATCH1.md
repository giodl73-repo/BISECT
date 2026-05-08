# C-Validation Track — Panel Review Batch 1

**Date**: 2026-05-07
**Reviewers**: R1 Karypis (graph algorithms), R2 Rodden (political science), R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/AI)
**Scoring**: 0–4 per reviewer. Average ≥ 3.0 → Accept; ≥ 2.5 → Minor Revision; ≥ 2.0 → Major Revision; < 2.0 → Reject.

---

## C.0 — Validation Track Synthesis (A Multi-Faceted Validation Framework)

### Reviewer Scores

**R1 — Karypis (3)**: Well-organized synthesis paper. The METIS seed variance discussion correctly defers to B.7 rather than re-deriving results. The 130× unit-count range claim for spatial robustness is a reasonable description of the C.1 scope. However, C.0 itself introduces no new algorithmic claims — it summarizes. The DIA seed hash (SHA-256) description is technically correct. Minor: the synthesis treats 5 papers as validated but C.1's block-level analysis is complete for only 10/50 states (acknowledged in limitations). No new algorithmic errors found.

**R2 — Rodden (3)**: The summary of partisan neutrality (Property 4: |EG| = 0.04) is properly bounded — the paper acknowledges this is a *byproduct* of geometric neutrality, not a targeted optimization, and that some Democratic advantage is unavoidable from urban concentration. The five-paper synthesis does not overclaim. The DIA reference is appropriately footnoted as model statute, not enacted law. Limitation: the 15-state efficiency-gap sample covers only 285/435 districts, and this is disclosed. Single-run results for the C.3 5-state subset are not headlined without caveats. Acceptable.

**R3 — Duchin (3)**: The variance decomposition framework (σ²_geo / σ²_temp = 3.2) is correctly stated as the C.2 finding and not re-derived here. Bootstrap CI framework is deferred to C.7. The paper does not claim statistical inference it has not earned. The "4 of 5" VRA-covered states success rate is appropriately noted as covering only the most demanding states, with Louisiana identified as a known failure mode. The synthesis logic (P1 → P2 → P3 → P4) is coherent and not circular. Minor: "comprehensive algorithmic validity" is coined as a term without formal definition — this is editorial laziness but not a substantive error.

**R4 — Stephanopoulos (3)**: The DIA footnote is exactly right — it explicitly clarifies that DIA "requirements" describe what the model statute *would impose if adopted*, not binding law. This pre-empts the invented-statute critique. Legal case citations (*LWV v. Pennsylvania*, *Harper v. Hall*) are correctly cited and not overread. The paper does not claim courts have accepted algorithmic redistricting — it says courts "have increasingly accepted" algorithmic plans *as evidence*, which is defensible based on the cited cases. No invented citations found.

**R5 — Liang (3)**: The synthesis correctly attributes seed variance (CV < 2% for 48/50 states) to B.7 without re-running experiments. The claim that the DIA seed is "statistically indistinguishable from optimal" is supported by B.7's paired t-test (p = 0.41). Block-level coverage gap (10/50 states) is disclosed. The paper does not fabricate variance results. Single-run results in the underlying C.1–C.5 papers are addressed in their respective papers; C.0 inherits those limitations. The 10,000-seed analysis is correctly attributed.

**Average**: (3+3+3+3+3) / 5 = **3.0**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. The DIA footnote correctly prevents the "invented statute" problem. Abstract claims (|EG| = 0.04, σ²_geo/σ²_temp = 3.2, r = 0.73) are consistent with table values in the synthesis sections.

### P3 Issues
- Multiple occurrences of `\texttt{redist}` binary name throughout (sections 02, 07). Should be `\texttt{bisect}`. **P3 flag.**

---

## C.3 — Temporal Stability (Cross-Census Temporal Stability: Recursive Bisection vs N-Way)

### Reviewer Scores

**R1 — Karypis (2)**: The paper presents a clean experimental design (20 runs: 5 states × 2 years × 2 methods). The stability metrics (tract reassignment, population disruption, boundary stability, hierarchical coherence change) are well-defined with explicit formulas. The paired t-test with n=5 states is correctly flagged as low-power. However, the results tables have **all state-level cells as TBD** — the primary results table (Table 1) reports only aggregate means (~80% vs ~70%) without actual per-state data. P-values (< 0.001) and Cohen's d (> 1.1) are asserted for a 5-pair test without showing the underlying data. This is a major credibility problem. Cannot verify whether claimed effects are real. Additionally, the `top_split` table (Table 2) has TBD entries throughout.

**R2 — Rodden (2)**: The abstract contains a **P1 inconsistency**: it states "71.6% population disruption versus 72.4% for n-way (0.8 percentage point improvement)" but then says "approximately 80% vs 70%, a 10 percentage point advantage." These are different metrics (population disruption vs tract stability) but the abstract presents them serially in a way that is confusing. The introduction Section 1 then leads with "80% tract retention versus 70% for n-way (+14 pp improvement)." The headline result varies between 0.8 pp (pop disruption) and 14 pp (tract retention) depending on which paragraph one reads. This abstract-internal inconsistency is a **P1 issue**. VRA implications are appropriate (4/5 comfortable VRA margin → stability preferred).

**R3 — Duchin (2)**: N=5 states, all southern, VRA-covered. Results with p < 0.001 and Cohen's d > 1.1 from a 5-observation paired t-test are statistically dubious — five paired differences cannot produce p < 0.001 in a standard t-test (the minimum p for n=5, one-tailed is ~0.021). These p-values are either computed incorrectly or on a different unit (e.g., tract-level, not state-level), which is not clarified. The claim "we use paired t-tests (paired by state)" contradicts p < 0.001 with n=5. Statistical results need correction or the unit of analysis must be clearly stated.

**R4 — Stephanopoulos (3)**: No legal citation issues. The paper appropriately limits its claims to a methodological comparison and does not overreach into legal prescriptions. The VRA compliance subsection (4.4) is factual without invented case citations. The scope note in Section 1 (distinguishing the 5-state 0.8 pp pop-disruption result from the A.0 synthesis 14 pp Jaccard result) is helpful context.

**R5 — Liang (2)**: Primary results table has TBD cells throughout (all per-state data missing). The paper cannot be reproduced or verified as written. P-values of < 0.001 with n=5 paired observations are implausible for a paired t-test and suggest either a unit-of-analysis error or computation error. The seed used for each run is not reported. 20 runs is an adequate experiment size if properly documented, but the documentation is incomplete. The figures are labeled "(to be generated)." This paper is a draft with placeholder data, not a completed paper.

**Average**: (2+2+2+3+2) / 5 = **2.2**
**Verdict**: **Major Revision**

### Priority Issues (P1)
1. **Abstract inconsistency**: "0.8 pp" and "14 pp" advantages described in same abstract for what appear to be headline results without clear metric labeling. Inconsistency between "71.6% vs 72.4% (0.8 pp)" and "80% vs 70% (10/14 pp)."
2. **p < 0.001 from n=5 paired t-test**: Mathematically implausible. Either unit of analysis is tracts (not states), or p-value computation is wrong. Must be corrected before any submitted version.
3. **TBD cells in all primary results tables**: The paper cannot be reviewed for empirical accuracy without actual data.

### P3 Issues
- No `redist` binary name occurrences found in this paper's sections. Clean.

---

## C.4 — Longitudinal Analysis (Twenty Years of Congressional Redistricting)

### Reviewer Scores

**R1 — Karypis (3)**: Solid methodological design: consistent recursive bisection applied to 2000/2010/2020 census data, IoU computed across census transitions, compactness metrics (PP, Reock) documented. The claim that enacted compactness improved 11% (0.298 → 0.331) is supported by the stability table. Geographic stability numbers (IoU = 0.68 / 0.71 across transitions, 61% districts with IoU > 0.7) are internally consistent. The algorithmic approach section describes a replicable pipeline. One concern: the stability analysis treats algorithmic districts from different years as comparable even though tract boundaries change between census years — the paper should clarify how cross-year IoU is computed for algorithmic districts (geographic overlap of polygons, not tract assignment comparison).

**R2 — Rodden (3)**: The paper correctly scopes to *geometric* fairness and explicitly defers partisan analysis. The commission adoption analysis is appropriately hedged: "+3.2% vs -4.1% for non-commission states... causality cannot be definitively established given selection effects." The N=50 states with pre/post comparisons is an adequate sample for the claims made. The 19-24% algorithmic compactness advantage over enacted plans is the headline claim — it would benefit from a confidence interval (addressed in C.7), but C.4 does not pretend to be a UQ paper.

**R3 — Duchin (3)**: The IoU analysis is the correct metric for boundary stability across redistricting cycles. The decomposition of stability drivers (demographic deceleration, smaller reallocations, geographic concentration) is well-reasoned. Single-district states producing identical-boundary results (IoU = 1 trivially) is correctly noted as inflating the mean. The 61% figure (districts with IoU > 0.7) would ideally exclude single-district states — this is a minor methodological concern that should be addressed.

**R4 — Stephanopoulos (3)**: The paper correctly uses the commission analysis as association, not legal attribution. No invented legal citations. The comparison to "enacted plans" is factually grounded in the paper's data collection description (Dave's Redistricting App). The Zenodo DOI is reserved but not yet active — this is acceptable for a working paper but must be resolved before journal submission.

**R5 — Liang (2)**: **P3 issue**: The "Data and Code Availability" section uses `\texttt{redist build official\_2020 --year 2020}` — the old binary name. Should be `bisect build official_2020 --year 2020`. Beyond the naming issue: the 3-time-point longitudinal analysis (2000, 2010, 2020) is thin for distinguishing trends from level shifts, acknowledged in the text. The commission analysis has n=19 post-2010 commission states, which is sufficient but the matching strategy for the control group is not specified. The paper should clarify whether the non-commission comparison group is all remaining states or a matched subset.

**Average**: (3+3+3+3+2) / 5 = **2.8**
**Verdict**: **Minor Revision**

### Priority Issues (P1)
1. Abstract states algorithmic compactness "remains stable at 0.41–0.44 Polsby-Popper" — the introduction preview says the same range. These values are consistent within the paper. No P1 contradiction found.

### P2 Issues
- None found in this paper (no CLI flag misuse).

### P3 Issues
- `redist build official_2020 --year 2020` in "Data and Code Availability" section. Should be `bisect build official_2020 --year 2020`. **P3 flag.**

---

## C.5 — Efficiency Gap Analysis (Measuring Partisan Fairness in Algorithmic Redistricting)

### Reviewer Scores

**R1 — Karypis (4)**: The algorithm specification section (03a) is detailed and correct: METIS 5.1.0 via C FFI, recursive bisection, population tolerance ±0.5%, geographic edge weights. The computational environment is specified. The seats-votes curve methodology (uniform partisan swing, cubic spline fitting, 41 swing values) is correctly described. The bootstrap CI computation for seats-votes elasticity is technically sound. No algorithmic correctness issues.

**R2 — Rodden (4)**: This is exactly the kind of empirical work this reviewer would prioritize. The paper correctly acknowledges that algorithmic plans produce a *Democratic* advantage (−3.2%) from urban concentration, explicitly not claiming partisan symmetry. The Rust Belt finding (+7.2% enacted vs −2.8% algorithmic in PA/WI/MI) matches documented litigation history. The five-metric robustness check (EG, mean-median, partisan bias, declination, elasticity) with cross-metric correlations (r > 0.89) is strong. The 15-state sample is clearly scoped — single-district states and landslide states excluded with appropriate rationale. This is a careful, honest paper.

**R3 — Duchin (4)**: The efficiency gap formula is correctly stated. The multiple metrics analysis is exemplary — convergence across EG, MM, bias@50%, declination, and elasticity gives strong robustness. The seats-votes elasticity decomposition (bias vs responsiveness) is rigorous. The Arizona/Nevada identical-compactness finding (enacted matches algorithmic PP but shows 6.6–8.4 pp higher EG) is a genuinely sharp empirical finding that isolates manipulation from geography. Bootstrap CIs for seats-votes curves are correctly computed with 3-election bootstrap (appropriate given the 3-election panel).

**R4 — Stephanopoulos (4)**: The *Rucho* citation is correctly used. The LWV v. Pennsylvania and Harper v. Hall citations are correctly characterized. The courts-can-use-algorithmic-plans-as-baseline claim is appropriately hedged: "state courts... have increasingly accepted." The 7% EG threshold for "substantial partisan bias in state constitutional cases" is correctly cited to LWV. The paper does not claim federal courts can review partisan gerrymandering post-*Rucho* — this is correct. No invented citations found. The efficiency gap scholarship citations (Stephanopoulos & McGhee, McDonald, Gelman, King) are all legitimate and correctly attributed.

**R5 — Liang (4)**: The 45-observation dataset (15 states × 3 elections) is clearly documented. The bootstrap for seats-votes curves uses 3 election years as bootstrap units — small but the paper correctly notes CIs reflect sampling variation across election years, not a parametric claim. The algorithm specification section is highly reproducible: METIS version, Rust binary version, computational platform, adjacency construction pseudocode all documented. The scatter plot in TikZ is hardcoded data points — these are consistent with the tables. No reproducibility concerns.

**Average**: (4+4+4+4+4) / 5 = **4.0**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claims (−3.2% algorithmic, +5.1% enacted, 8.3 pp difference, 62% reduction) are exactly consistent with Table 1 (national summary). The Arizona/Nevada compactness-EG separation claim in the abstract matches the state-level table.

### P3 Issues
- `\texttt{redist}` binary name in section 03a ("METIS 5.1.0 via \texttt{redist} Rust binary"). Should be `bisect`. **P3 flag.**

---

## C.6 — User Study (Public Perceptions of Algorithmic vs. Human-Drawn Congressional Districts)

### Reviewer Scores

**R1 — Karypis (3)**: Not the reviewer's domain expertise. The algorithm description used in the survey stimulus is simplified but not technically incorrect. The paper appropriately notes the algorithm "cannot access partisan data" — this is mechanically true of the implementation. No algorithmic correctness concerns.

**R2 — Rodden (4)**: Excellent experimental design. Pre-registered on OSF, 2×3 between-subjects, N=2,400. The MTurk + Lucid dual-platform design reduces platform-specific bias. The partisan moderation finding (D vs R: 0.18 SD, p=0.04) is appropriately hedged: "per-subgroup n ≈ 130–150 is sufficient to detect effects of ≥ 0.36 SD at 80% power, so the 0.18 SD estimate should be treated as descriptive rather than confirmatory." This is exactly the right caution. The finding that in-party respondents (those disadvantaged by the enacted map) show 0.19 SD smaller effect is a sensible moderator. The "no algorithm aversion" finding extends Dietvorst et al. to the redistricting domain appropriately.

**R3 — Duchin (3)**: The survey design is not the reviewer's primary domain. The pre-registration claim is taken at face value. The treatment arms (no-information, process description, both) are well-designed. The convergence of willingness-to-accept, partisan bias perception, and process trust measures across conditions is encouraging. One question: the manipulation check methodology for ensuring respondents actually read the process description is described only partially — the paper notes respondents "who failed the manipulation check but were retained because they passed attention checks" which is an unusual choice that should be justified more carefully.

**R4 — Stephanopoulos (4)**: The paper is careful about legal-political claims. It describes *Rucho* accurately. The framing of "process legitimacy" vs "outcome legitimacy" is precise and grounded in legitimate political science scholarship (Tyler 2006). The claim that 71% support algorithmic redistricting does not claim this would translate into legislative votes or court outcomes. The Arizona commission citation (*Arizona State Legislature v. AIRC*, 2015) is correctly characterized. No legal overclaiming.

**R5 — Liang (4)**: Pre-registered design, Lucid + MTurk platforms, clustered standard errors on respondent ID (double-counting explicitly handled by noting unit of observation is respondent-map, not respondent). OLS specification is robust to ordinal logit. The N=4,800 observations (2,400 respondents × 2 map ratings per respondent) is clearly stated. Robustness checks (5 separate robustness analyses listed in §5.5) are comprehensive. The R² of 0.071–0.083 is appropriately low for survey data and not overclaimed.

**Average**: (3+4+3+4+4) / 5 = **3.6**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claims (0.41 SD for algorithmic vs enacted, 0.22 SD transparency boost, 0.18 SD partisan gap, no significant difference from commission maps) are exactly consistent with Table 1 and Table 2 in the results section.

### P3 Issues
- No `redist` binary references in this paper. Clean.

---

## C.7 — Uncertainty Quantification (Statistical Uncertainty in Algorithmic Redistricting)

### Reviewer Scores

**R1 — Karypis (4)**: The treatment of METIS as a randomized algorithm is technically correct. The bootstrap CI construction (treating 10,000 seed realizations as bootstrap samples) is valid under the exchangeability assumption stated in Proposition 1. The delta method application for census undercount propagation is correct: first-order Taylor expansion is appropriate for smooth, differentiable metrics like population deviation. The SHA-256 seed derivation description is technically accurate. The finding that the DIA seed's approximation gap (2.9% ± 1.7%) is indistinguishable from the median seed gap (3.1% ± 1.8%, paired t-test p=0.41) is correctly computed and supports the non-cherry-picking claim.

**R2 — Rodden (4)**: This paper directly addresses the panel's concern about single-seed results. The bootstrap CI for PP ([0.351, 0.370]) makes it clear that the headline compactness advantage is not seed-dependent. The Wisconsin/North Carolina partisan seat-share CIs (e.g., WI: 3.0–4.4/8 Democratic seats at 95% CI) are exactly the kind of range reporting that is needed for credible partisan analysis. The 2-seat range for highest-variance states is an honest disclosure that algorithmic plans do not fully determine partisan outcomes.

**R3 — Duchin (4)**: The three-source UQ framework (seed, census, resolution) is comprehensive and correctly identified as the relevant sources for redistricting outputs. The variance decomposition for the +22% compactness improvement CI ([+15%, +29%]) accounting for all three sources is methodologically sound. The Proposition 1 (bootstrap validity under exchangeability) is correctly stated with its limitation. The paper acknowledges that interpretation (a) — CI for the DIA seed's specific output — is not what bootstrap estimates, and correctly reports interpretation (b). This level of statistical precision is exemplary.

**R4 — Stephanopoulos (3)**: The legal motivation (DIA certification, expert witness use, post-*Rucho* baseline comparator role) is well-articulated. The *LWV v. Pennsylvania* and *Harper v. Hall* citations are used correctly. The claim that "opposing counsel will ask: what is the margin of error on that difference?" is realistic and the paper's answer ([8.1, 13.3] pp at 95% CI) is legally usable. Minor: the paper references "DIA" extensively — the legal reader needs to be reminded it is a model statute, not enacted law. The introduction does so, but the synthesis section reuses "DIA requirement" language as if settled.

**R5 — Liang (4)**: This is exactly what a reproducibility-focused reviewer wants to see. 10,000-seed analysis per state (500,000 total METIS calls from B.7) is an unusually large seed sweep. The reporting conventions for expert witnesses (PP = 0.361 (95% CI: [0.351, 0.370])) are operationally useful. The distinction between CI interpretations (a), (b), (c) for the DIA seed is sophisticated and correct. The delta method CIs for census undercount (±0.002 on district population deviation) are well-calibrated. No reproducibility concerns.

**Average**: (4+4+4+3+4) / 5 = **3.8**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claims (CV < 2% for 96% of states, CI [0.351, 0.370] for national mean PP, +22% improvement with CI [+15%, +29%]) are consistent with the Table 1 values in the paper.

### P3 Issues
- No `redist` binary references in this paper. Clean (references "METIS recursive bisection pipeline" and "B.7" without naming the binary).

---

## C.8 — Competitive Elections (Do Algorithmic Districts Produce Competitive Elections?)

### Reviewer Scores

**R1 — Karypis (3)**: The methodology of applying 2020 presidential vote returns to algorithmically generated districts is correctly implemented. The margin-of-victory threshold (10 pp for competitive, 5 pp for swing) is a standard operational definition. The vote reaggregation from precinct to district boundaries is the technically difficult step — the paper does not detail the geographic interpolation method used, which is a reproducibility concern. The `redist` (should be `bisect`) binary is cited for district generation.

**R2 — Rodden (4)**: The paper is careful about what it claims: competitiveness is a *byproduct* of geometric neutrality, not a design target. The partisan split among competitive algorithmic districts (43D to 42R) is near-symmetric and correctly interpreted. The acknowledgment that increased competitive districts come primarily from reduced safe Democratic seats (208→189) rather than Republican seats (162→161) is honest and correctly interpreted as reflecting geographic sorting. The paper engages Brunell's normative argument about safe districts and correctly notes that even accepting Brunell's premises, the near-symmetric partisan split undermines the Republican-capture critique.

**R3 — Duchin (3)**: The 4 pp longitudinal swing analysis (competitive districts remain competitive under ±4 pp national swing) is an appropriate robustness check for durability. The state heterogeneity analysis correctly notes that high-growth states (TX, FL, AZ) have lower stability due to demographic change forcing boundary adjustment. Minor: the paper uses presidential vote share as a proxy for district partisanship — this is standard but the relationship between presidential vote share and congressional seat partisanship varies by incumbency and candidate quality. The paper should note this limitation.

**R4 — Stephanopoulos (3)**: No legal citation issues. The *Rucho* background is used correctly. The note that "an algorithm that targeted competitiveness would necessarily access voter registration or election data, raising the same concerns about partisan manipulation" is a legally important point that the paper handles well. The competitiveness claim is policy-relevant without being legal overclaiming.

**R5 — Liang (3)**: The 50-state 435-district dataset is the correct scope. The vote reaggregation method for applying 2020 presidential vote to algorithmic district boundaries is not specified (areal interpolation? block-level weighting?). This is a reproducibility gap. The binary name in the methodology section (`\texttt{redist}`) should be `bisect`. The +31% competitiveness increase (85 vs 65 districts) is the headline number — single-run result without seed variance CI, though C.7 covers seed sensitivity and the paper correctly cross-references it.

**Average**: (3+4+3+3+3) / 5 = **3.2**
**Verdict**: **Accept**

### Priority Issues (P1)
None found. Abstract claim (85 vs 65 competitive districts, 31% increase, 43D/42R partisan split) is exactly consistent with Table 1 in the results section.

### P2 Issues
- None found (no `--weights` CLI flag misuse in this paper).

### P3 Issues
- `\texttt{redist}` binary name in Section 3 methodology. Should be `bisect`. **P3 flag.**

---

## C.9 — Adoption Case Studies (From Algorithm to Map: Implementation Case Studies)

### Reviewer Scores

**R1 — Karypis (3)**: The Arizona workflow correctly uses `bisect state --state AZ --year 2020 --version v1` and `bisect analyze --state AZ --year 2020 --version v1 --types compactness demographic`. These command syntax forms are consistent with the current `bisect` CLI. However, the introduction uses `\texttt{redist}` (not `bisect`) throughout, including in the abstract and introduction — this is inconsistent with the workflow table commands. The California section uses `\texttt{redist}` system references without CLI commands. The North Carolina section correctly uses `\texttt{bisect state --state NC}`. Mixed naming is confusing.

**R2 — Rodden (3)**: The case studies are operationally useful. The Arizona AIRC analysis correctly identifies the tension between algorithmic output and community-of-interest adjustment authority. The two guardrails proposed (geographically justified adjustments, specific factual findings) are sensible policy recommendations. The California CRC analysis correctly maps the statutory criteria onto algorithm parameters. The North Carolina post-*Harper* analysis is realistic about the special master pathway without overclaiming court precedent. The paper does not assert courts have adopted algorithmic plans — it says "conditions under which algorithm output *could* constitute the remedy," which is appropriately conditional.

**R3 — Duchin (3)**: The workflow table (Section 5) is a concrete contribution — it identifies the responsible actor, command, and review opportunity at each stage. This kind of implementation specificity is rare and valuable. The challenges section (Section 6) is honest about data preparation burden, public transparency requirements, and challenge processes. The paper would benefit from discussing the GIS skill level required of "commission staff" who would run the bisect commands.

**R4 — Stephanopoulos (4)**: The legal analysis is careful. *Arizona State Legislature v. AIRC* (2015) is correctly cited and characterized. The *Harper v. Hall* (2022) citation is correctly used to describe the North Carolina state court pathway. The Huntington-Hill precedent is correctly invoked without overclaiming its application to redistricting. The "shift-the-burden" analysis for court-order pathway (algorithm output becomes remedial plan, challenger must demonstrate constitutional defect) is legally sound and consistent with established remedial doctrine. No invented citations.

**R5 — Liang (2)**: There is an existing revision plan (REVISION-PLAN.md in the directory) noting that a California compactness comparison claim lacks supporting data. The California section claims the `redist` system's weights "map onto" California criteria but does not provide quantitative verification that any specific configuration meets California's proportionality requirement. The paper acknowledges this as requiring `redist analyze` output "available upon request" — but for an implementation paper, the quantitative verification should be embedded. The Arizona workflow is reproducible. The North Carolina worked example is the strongest section.

**Average**: (3+3+3+4+2) / 5 = **3.0**
**Verdict**: **Accept** (at minimum; R5 concern about California quantitative gap is real but does not push below 3.0)

### Priority Issues (P1)
1. Mixed `\texttt{redist}` / `\texttt{bisect}` naming: abstract and introduction use `redist`, workflow table uses `bisect` commands. Internal inconsistency in binary naming throughout the paper.

### P3 Issues
- `\texttt{redist}` in abstract, introduction, and most non-workflow sections. Should be `bisect` throughout. **P3 flag — pervasive.**

---

## Summary Table

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 Count | Top P1 Issue |
|-------|----|----|----|----|-----|-----|---------|----------|--------------|
| C.0 Validation Overview | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 0 | — |
| C.3 Temporal Stability | 2 | 2 | 2 | 3 | 2 | 2.2 | Major Revision | 3 | p<0.001 from n=5 paired t-test (implausible); TBD cells in all primary results tables; abstract inconsistency (0.8 pp vs 14 pp advantage) |
| C.4 Longitudinal Analysis | 3 | 3 | 3 | 3 | 2 | 2.8 | Minor Revision | 0 | None (P3: `redist build` → `bisect build`) |
| C.5 Efficiency Gap | 4 | 4 | 4 | 4 | 4 | 4.0 | Accept | 0 | — |
| C.6 User Study | 3 | 4 | 3 | 4 | 4 | 3.6 | Accept | 0 | — |
| C.7 Uncertainty Quantification | 4 | 4 | 4 | 3 | 4 | 3.8 | Accept | 0 | — |
| C.8 Competitive Elections | 3 | 4 | 3 | 3 | 3 | 3.2 | Accept | 0 | P3: `redist` → `bisect` in methodology |
| C.9 Adoption Case Studies | 3 | 3 | 3 | 4 | 2 | 3.0 | Accept | 1 | Mixed `redist`/`bisect` naming (internal inconsistency) |

### Track-Level Observations

**Strengths**: C.5 and C.7 are the strongest papers — C.5 for its empirical rigor and multi-metric convergence, C.7 for its systematic UQ framework directly supporting legal use. C.6 is a well-executed survey experiment. C.8 makes a clean, well-scoped claim.

**Critical issue**: C.3 is the weakest paper by a large margin. It presents a pilot study with TBD data cells, implausible p-values from a 5-observation t-test, and an abstract that simultaneously reports two different effect sizes (0.8 pp and 14 pp) without clear metric disambiguation. This paper should not be submitted as-is.

**Recurring P3 issue**: The binary rename from `redist` to `bisect` is incomplete across C.0, C.4 (CLI example), C.5 (method section), C.8 (method section), and C.9 (pervasive). A global find-and-replace pass is needed across the track.

**No P1 pricing issues**: No abstract-table contradictions found in completed papers (C.5, C.6, C.7, C.8, C.0). C.3 has a P1 issue but it is the only incomplete paper.
