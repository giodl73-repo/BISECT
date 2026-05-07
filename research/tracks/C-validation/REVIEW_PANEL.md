# REVIEW PANEL — Track C: Validation
**Module**: track-C
**Review Date**: 2026-05-07
**Convened by**: Panel Secretariat, Apportionment Research Group

---

## Panel Composition

| # | Reviewer | Affiliation | Primary Expertise |
|---|----------|-------------|-------------------|
| R1 | Michael Goodchild | UC Santa Barbara | GIS theory, spatial analysis, MAUP |
| R2 | Jonathan Rodden | Stanford University | Electoral geography, geographic sorting |
| R3 | Moon Duchin | Rutgers / MGGG | Gerrymandering, metric geometry, ensemble methods |
| R4 | Nicholas O. Stephanopoulos | Harvard Law School | Efficiency gap, partisan gerrymandering law |
| R5 | Jowei Chen | University of Michigan | Computational redistricting, neutral benchmarks |
| R6 | Roger Tourangeau | University of Michigan | Survey methodology, public opinion measurement |
| R7 | Kriti Bhargava Liang | MIT / Independent | Experimental methods, statistical UQ |

**Rationale**: Track C spans two sub-tracks requiring distinct expertise. R1–R3 cover the spatial statistics and computational dimensions of the robustness sub-track. R4–R5 supply political methodology (efficiency gap, competitive elections, legal adoption). R6–R7 provide the public opinion / survey-experiment and statistical UQ expertise needed to assess C.6 and C.7 rigorously. This composition deliberately excludes graph-partitioning specialists (dominant in Track B panels) and emphasises adversarial political-science, geospatial, and survey-methods voices.

---

## Individual Assessments

### R1 — Michael Goodchild (GIS / MAUP)

**Overall module score**: 6.5 / 10
**Track robustness score**: 7.0 / 10
**Track political-science score**: 5.5 / 10

**Strengths.** C.2's slice-based framework is methodologically innovative and earned a well-deserved Round 3 Strong Accept. The Moran's I = 0.42 spatial autocorrelation measurement in C.1 and the K = 3/5/7 sensitivity sweep address MAUP at the level of the field's current standard. C.7's three-source variance decomposition is the kind of quantitative rigour GIS journals need more of.

**Weaknesses.** C.1 carries a critical open issue: the block-level data claim (130× unit-count range) rests on projected rather than fully empirical results, and the REVISION-PLAN.md in that directory is an unfilled template — no actual revision has been applied to the manuscript. C.3 is not a paper; it is a collection of Python scripts and CSV partition files with no LaTeX manuscript. C.4 lacks ensemble comparison to alternative partitioners, a standard now expected at mid-tier computational venues.

**Paper rankings** (robustness sub-track, descending quality):
1. C.2 — Ready, R3 strong accept, empirically grounded slice framework
2. C.7 — Ready, strong UQ, several P1 items still open in revision plan
3. C.0 — Ready, coherent synthesis, but P1 list unresolved in manuscript
4. C.4 — Ready, P1 items addressed per _panel.yaml, venue mismatch with Science
5. C.1 — Round 0 effective; no revisions applied; 130× claim is projected
6. C.3 — No manuscript; experimental artefacts only

**Responses to key validation questions.**
Does C.1 adequately control for spatial autocorrelation? The Moran's I measurement and K-slice sensitivity are adequate for the IJGIS venue. The spatial lag model is absent; MAUP effects are measured but their causal mechanism is not modelled. This is a gap, not a fatal flaw for the resolution-robustness claim. Does C.2 properly account for changing tract boundaries? Yes — the 18.2% instability rate is measured and population-weighted centroid correspondence is documented in Section 3.2 and Table 1. This is adequate for ACM SIGSPATIAL.

---

### R2 — Jonathan Rodden (Electoral Geography)

**Overall module score**: 6.0 / 10
**Track robustness score**: 5.5 / 10
**Track political-science score**: 7.0 / 10

**Strengths.** C.5's near-zero efficiency gap is the most surprising and publishable result in the module. The geographic sorting acknowledgement — "unavoidable geographic concentration of Democratic voters" — is exactly right and prevents C.5 from overclaiming. C.8's asymmetric safe-seat conversion (19-seat reduction in safe Democratic seats, 1-seat change for Republicans) is an empirically important finding that cuts against simplistic "algorithmic maps are neutral" narratives. C.9 shows genuine legal sophistication.

**Weaknesses.** C.3 has no manuscript. The five Deep South states used for the temporal analysis (AL, GA, LA, MS, SC) are not nationally representative; any cross-decade stability generalisation is severely constrained by this sample. C.8 uses 2020 presidential vote as the sole electoral signal — a single high-polarisation election year — and the efficiency gap and mean-median difference for enacted plans use cross-election SDs from only three elections (2016, 2018, 2020), understating true decadal uncertainty. C.4's commission-type heterogeneity discussion (independent vs. bipartisan vs. advisory) remains shallow despite being flagged in the revision plan.

**Paper rankings** (political-science sub-track, descending quality):
1. C.5 — Ready, unanimous R2 strong accept; the module's highest-confidence finding
2. C.6 — Ready, R2 4.0/4; strongest experimental design in the module
3. C.8 — Ready; ensemble uncertainty range [81, 90] resolves the core critique
4. C.9 — Ready; Harper reversal and Allen VRA correction are genuine legal improvements

**Responses to key validation questions.**
Does C.8 adequately address reverse causality? Partially. The paper correctly labels competitiveness as a byproduct, not a target, and softens causal language to "are associated with." The reaggregation of 2020 presidential returns to algorithmic district boundaries is itself a counterfactual projection — no election has been run under these maps — and the model does not account for how candidate entry and voter turnout would adapt to new boundaries. The longitudinal projection under ±4pp uniform shifts partially addresses durability but not behavioural adaptation. The limitation is real but beyond the paper's scope; it should appear as a limitations sentence. Is geographic sorting properly acknowledged in C.5? Yes, more carefully than in most neutral-benchmark papers. The persistent negative efficiency gap is attributed to residential geography, not algorithmic bias, and this framing is consistent throughout.

---

### R3 — Moon Duchin (Mathematical / Ensemble)

**Overall module score**: 6.5 / 10
**Track robustness score**: 7.5 / 10
**Track political-science score**: 6.0 / 10

**Strengths.** C.7 is the standout paper for mathematical quality. The three-source variance decomposition with analytically derived delta-method CIs and the legal-use framing (each uncertainty source mapped to a class of legal challenge) is technically rigorous and genuinely useful for expert witnesses. C.6's pre-registration and 2×3 design are exemplary. C.2's Moran's I = 0.42 and K-slice robustness are adequately reported and the neutrality language correction (process neutrality vs. outcome neutrality) is the right move.

**Weaknesses.** The module contains no ensemble comparison whatsoever. Single-seed analyses dominate C.1 (block-level projection), C.3, and large portions of C.4. C.8 uses a 20-seed ensemble only for the 10 largest states. GerryChain and recom-based ensemble benchmarks are now standard in competitive redistricting research (DeFord et al. 2021, Herschlag et al. 2020); without one, the "robustness" claim has no comparison class. C.7's P1.3 (shapefile simplification error not separated from resolution error) and P1.4 (PES tract-level calibration) remain unresolved per the revision plan. The abstract/body CI inconsistency in C.7 ([+18%,+26%] vs. [+15%,+29%]) is a factual error still present in the manuscript.

**Paper rankings** (overall descending quality):
1. C.7 — Best UQ framework; P1 items are correctible, not invalidations
2. C.2 — Slice framework is genuinely original; empirical gap manageable
3. C.6 — Pre-registered; meets survey-experiment standards
4. C.5 — Strong; overclaim issue resolved in R2
5. C.0 — Coherent synthesis but DIA status and VRA omission unresolved
6. C.8 — Adequate; causal framing softening satisfactory
7. C.4 — Adequate; selection bias in commission-state comparison
8. C.1 — Needs empirical data at block level; 150 projected runs inadequate
9. C.9 — Adequate case studies; Harper correction critical and completed
10. C.3 — No manuscript

**Responses to key validation questions.**
Does C.7 cover model uncertainty or just estimation uncertainty? Estimation uncertainty only. The three sources (seed variance, census undercount, shapefile resolution) are all estimation uncertainties within the METIS recursive bisection framework. Model uncertainty — whether METIS recursive bisection is the right partitioner, whether Polsby-Popper is the right compactness metric — is explicitly out of scope. The paper does not acknowledge this distinction, and it should, particularly for JASA where model selection uncertainty is expected in formal UQ papers. Does C.1 adequately control for spatial autocorrelation? The Moran's I measurement is adequate for IJGIS. The absence of a spatial lag model is a gap but not a fatal one given that the paper's primary claim is resolution-robustness rather than spatial-process inference.

---

### R4 — Nicholas O. Stephanopoulos (Election Law)

**Overall module score**: 6.0 / 10
**Track robustness score**: 5.0 / 10
**Track political-science score**: 7.0 / 10

**Strengths.** C.5's framing of efficiency gap as a validation criterion rather than a design target is exactly the right normative move. The result — near-zero EG as byproduct of compactness — is a genuine contribution to the legal literature on partisan gerrymandering. C.9's case studies are legally sophisticated: the Harper reversal is correctly reframed as the "Harper model" applicable to states maintaining partisan gerrymandering jurisdiction under their state constitutions, and the Allen v. Milligan VRA district-count analysis addresses the Section 2 multiplicity requirement. C.6 demonstrates non-trivial cross-partisan public support (0.18 SD partisan gap), which is directly relevant to commission-based adoption proposals.

**Weaknesses.** C.0's DIA legal status ambiguity is the most serious problem in the module for downstream legal use. Citing a model statute as though it were enacted law, without disclosure of its status, could undermine the credibility of expert witnesses relying on this synthesis. The omission of VRA compliance from C.0's four-property validation framework means the synthesis does not address the most frequently litigated redistricting claim. C.9's effects-based state constitutional challenge discussion (Pennsylvania free-and-equal clause) is treated as a moderate issue in the revision plan; given the post-Rucho litigation landscape, this warrants P1 treatment in a final revision.

**Paper rankings** (political-science sub-track):
1. C.5 — Most publishable and legally useful
2. C.6 — Cross-partisan acceptance finding has direct policy relevance
3. C.9 — Strong after Harper correction; legally sophisticated
4. C.8 — Good; observational framing adequately hedged

**Responses to key validation questions.**
Do C.8/C.9 adequately address reverse causality? C.9 does not face a reverse causality problem — it is a process analysis, not an outcome study. C.8 faces the causality problem and treats it appropriately as a counterfactual projection. The framing "byproduct of geometric neutrality, not a design target" is legally useful and does not overstate causal identification. The remaining gap — no analysis of how electoral outcomes would differ if candidates adapted strategy to algorithmic district boundaries — should appear as a one-sentence limitation. Is the 71% post-survey support figure in C.6 adequately caveated? In the revision plan it is flagged as a post-treatment measure, but this caveat must appear in the published paper text, not only in revision notes.

---

### R5 — Jowei Chen (Computational Redistricting)

**Overall module score**: 6.5 / 10
**Track robustness score**: 6.0 / 10
**Track political-science score**: 7.0 / 10

**Strengths.** The module correctly uses a single algorithm (METIS recursive bisection) consistently across all papers, enabling apples-to-apples comparison across resolutions, years, and states. C.8's enacted-map stratification — partisan legislature 58 competitive districts vs. commission 76 vs. algorithmic 85 — transforms a blunt comparison into a nuanced one and is the most important analytic addition of the revision round. C.5's 50-state coverage is methodologically superior to most neutral-benchmark papers, which typically use 10–15 states.

**Weaknesses.** The module's most significant weakness is the complete absence of ensemble comparison. A GerryChain or recom-based ensemble benchmark is now standard (DeFord et al. 2021; Herschlag et al. 2020). Without one, the claim that METIS recursive bisection produces "robust" results has no comparison class — the module demonstrates the algorithm is internally consistent across resolutions and years but does not show where it falls in the space of valid redistricting plans. This gap is present across C.1, C.4, and C.8, and is flagged but deferred in C.2's revision plan. C.3 is a collection of experimental scripts, not a paper; it cannot support the temporal stability claims attributed to it in C.0's synthesis.

**Paper rankings** (computational rigour, descending):
1. C.5 — 50 states, correct EG methodology
2. C.2 — Pilot validation adequate; slice framework is new
3. C.7 — Strong UQ; P1 items correctible
4. C.8 — Ensemble uncertainty adequate for Pol. Analysis
5. C.1 — Adequate methodology; empirical gap remains
6. C.4 — Adequate; selection bias in commission sample
7. C.0 — Synthesis depends partly on C.3 which is not a paper
8. C.6 — Strong experiment; not a computational paper
9. C.9 — Case studies; not a computational paper
10. C.3 — No paper

---

### R6 — Roger Tourangeau (Survey Methodology)

**Overall module score**: 6.8 / 10
**Track political-science score**: 7.5 / 10 (for C.6)

**C.6 assessment.** The 2×3 between-subjects design with pre-registration, MTurk+Lucid dual platform, and N = 2,400 meets acceptable power for the primary effects (d ≥ 0.20 at 80% power per pre-registration). The exclusion rate (14.6%) and non-differential exclusion across conditions (χ² = 8.4, p = 0.14) are satisfactory. The treatment text reproduced in Section 3 (Figure 2 box, added in R2) meets the replication standard. The partisan moderation finding (H3, 0.18 SD Democrats minus Republicans, p = 0.04) is appropriately reframed as exploratory given that subgroup Ns (~124–148 per cell) are insufficient to detect effects this small at 80% power. The H2 revision — rewritten as an equivalence test rather than a directional hypothesis — resolves the testability integrity issue.

**Outstanding concerns (non-blocking).** Stage 4 sequencing (whether the manipulation check precedes or follows the debriefing) remains ambiguous in the manuscript text and must be specified in the final published version. The Likert item 2 validity issue ("the people who drew these districts treated different communities equally" in a condition where no people drew the districts) has been flagged but not resolved with a sensitivity analysis excluding that item. The two-state design (NC, MD) limits ecological validity; both states are high-salience gerrymandering cases, and the high-awareness sensitivity analysis recommended in R2 has not been executed. The standardisation procedure (per-respondent mean then cross-respondent standardisation vs. all 4,800 observations) is unspecified and produces materially different standardised effect sizes.

**Ecological validity assessment.** The use of actual enacted and algorithmically generated maps for NC and MD, with professional cartographic design and realistic treatment text, provides reasonable ecological validity for the specific comparison tested. Generalisation to all states or all algorithmic systems requires caution and is appropriately caveated in the current limitations section.

---

### R7 — Kriti Bhargava Liang (Experimental Methods / UQ)

**Overall module score**: 6.5 / 10

**C.7 assessment.** The delta method for census undercount propagation is technically sound. The CI hierarchy (seed-only → seed+resolution → all three sources) is well-presented and the legal-use mapping (Section 8, added per P1.6) is a genuine contribution. Two P1 items remain unresolved in the current manuscript. First, the abstract reports [+18%, +26%] while the synthesis section (Section 7.4, Table 7.4) reports [+15%, +29%] for all three sources — a factual inconsistency that must be corrected before JASA submission. Second, σε ≈ 0.015 is attributed to "Census Bureau's tract-level undercount estimates from the 2020 PES experimental microdata," but official PES provides state-level and demographic-group estimates, not tract-level estimates; the small-area extrapolation methodology is undocumented and must be specified with its own uncertainty propagated through the delta method CI.

**C.6 assessment.** The standardisation procedure (P4-C) is unspecified and produces materially different standardised effect sizes depending on whether standardisation is performed per respondent or across all 4,800 observations. This must be resolved before publication. The pilot sample platform alignment question (P4-E: whether the n = 200 pilot was drawn from MTurk only while the main study pools MTurk and Lucid) is a minor but real validity concern for the Likert scale properties.

**Module-level UQ assessment.** The module has one formal UQ paper (C.7) and one paper with informal ensemble uncertainty (C.8, range [81, 90]). C.1, C.4, and C.9 report no uncertainty on their headline figures. C.3 has no manuscript. The module's UQ coverage is therefore partial: adequate for C.7 and C.8 but absent for C.1's block-level claims and C.4's twenty-year trajectory. A stated limitation in C.0's synthesis regarding which papers carry formal uncertainty quantification would strengthen the module's self-assessment.

---

## Module-Level Synthesis

### Module Score

**Score: 6.4 / 10**
**Tier: Conditionally Adequate** (target for Track C synthesis input to Track A: 8.0+)

| Dimension | Score | Notes |
|-----------|-------|-------|
| Robustness sub-track | 6.2 / 10 | C.3 absent; C.1 no revisions applied; C.0 DIA/VRA gaps |
| Political-science sub-track | 7.0 / 10 | C.5 and C.6 strong; C.8/C.9 adequate |
| Cross-track coherence | 6.5 / 10 | C.0 synthesis coherent but DIA status and VRA property absent |
| UQ coverage | 5.5 / 10 | Formal UQ only in C.7 and C.8; absent from C.1, C.4 |
| Empirical grounding | 6.0 / 10 | C.2/C.5/C.6 empirically strong; C.1/C.3 weak or absent |

### Track Scores

**Track robustness: 6.2 / 10**
Chain: C.0 → C.1 → C.2 → C.3 → C.4 → C.7
Strongest links: C.7 (formal UQ), C.2 (slice validation framework, R3 strong accept)
Weakest links: C.3 (no manuscript exists), C.1 (no revisions applied; empirical data gap at block level), C.0 (DIA legal status unresolved)

**Track political-science: 7.0 / 10**
Chain: C.5 → C.6 → C.8 → C.9
Strongest links: C.5 (APSR, unanimous 4.0/4 in R2), C.6 (AJPS, pre-registered, 4.0/4 in R2)
Weakest link: C.9 (Harper correction complete; effects-based state constitutional challenge needs strengthening)

---

## Priority Items

### PP1 — C.3 Has No Manuscript (Blocking for Module Coherence)
**Track**: robustness
**Severity**: High

C.0's synthesis (sections/04-temporal.tex) explicitly attributes temporal stability findings to C.3 as a published result. C.3's directory contains Python scripts, CSV partition files, pickled tree objects, and status reports — but no LaTeX manuscript. The _panel.yaml records a Round 1 review history with a 3.1/4 score, but the reviewed artefact was the abstract and experimental data plan, not a paper. C.0's claim that Track C "provides temporal stability analysis" in C.3 is currently unsupported by any publishable document.

**Required action**: Either (a) write the C.3 manuscript from existing experimental data, clearly scoped as a 5-state (AL, GA, LA, MS, SC) sub-national pilot study for 2010–2020 recursive vs. n-way comparison, and revise C.0 accordingly; or (b) demote C.3 from "paper" to "experimental data archive," remove the C.3 citation from C.0's synthesis section, and note the gap explicitly in C.0's limitations.

### PP2 — C.1 Block-Level Empirical Data Gap and Blank Revision Plan (Blocking for IJGIS)
**Track**: robustness
**Severity**: High

C.1's headline claim — robustness across a 130× unit-count range — requires empirical block-level redistricting runs. The _panel.yaml records a Round 3 review history with a 3.6/4 strong-accept score, all P1 items marked addressed, and stage: ready. However, the REVISION-PLAN.md file in the C.1 directory is a blank template (the default panel:setup placeholder with no tasks filled in), and Section 3.3 of the methodology contains a TODO comment. This indicates that the block-level runs were never executed and no revisions were written into the paper. The Round 3 review score reflects the revision plan assessment, not assessment of a completed paper with block-level empirical data.

**Required action**: Either run the 10-state block-level subset (AL, CA, FL, GA, ID, ME, NY, TX, VT, WV) as described in the methodology, report results with 95% CIs from the 10-run stochasticity analysis, and populate the revision plan with completed tasks; or clearly label Section 4's block-level results as representative projections, reduce the claim to the empirically verified tract–block-group comparison, and reposition the paper as a methodology validation pending full empirical execution.

### PP3 — C.7 Abstract/Body CI Inconsistency and PES Calibration Gap (Blocking for JASA)
**Track**: robustness
**Severity**: Medium-High

Two unresolved P1 items affect the paper's factual accuracy and methodological soundness. First (P1.1): the abstract states the 95% CI for the +22% compactness improvement is [+18%, +26%], but the synthesis section (Section 7.4 and Table 7.4) reports [+15%, +29%] for all three uncertainty sources. The abstract figure corresponds to seed variance and resolution only, omitting the census undercount component. Second (P1.4): σε ≈ 0.015 is attributed to "Census Bureau's tract-level undercount estimates from the 2020 PES experimental microdata," but official 2020 PES products do not provide tract-level estimates; the extrapolation is undocumented.

**Required action**: (a) Correct the abstract to cite [+15%, +29%] as the three-source CI. (b) Specify the exact PES product and small-area estimation methodology for deriving σε at tract level. If extrapolated from state-level SDs, document the extrapolation model and add a sensitivity table showing the CI under σε = 0.010, 0.015, 0.020.

### PP4 — C.0 DIA Legal Status and VRA Property Omission (Blocking for Downstream Legal Use)
**Track**: robustness
**Severity**: Medium

C.0's revision plan items P1.3, P1.4, and P1.5 are unresolved in the current manuscript. The Districting Integrity Act is cited throughout as though it were enacted federal law without disclosure of its status as a model statute. The four-property validation framework omits VRA compliance — the most frequently litigated redistricting claim.

**Required action**: (a) Add a footnote at the first DIA citation clarifying its status as a model statute. (b) Replace "satisfies" with "provides empirical evidence toward satisfying" in the compliance section. (c) Either add VRA stability as Property 5 or add a prominent limitations paragraph explaining why VRA compliance is not validated in this module.

### PP5 — C.6 Standardisation Procedure Unspecified and Likert Item Validity (Pre-Publication)
**Track**: political-science
**Severity**: Medium

The primary outcome standardisation procedure is unspecified, making the published effect sizes (d = 0.41, 0.22) unreplicable without this specification. Likert item 2 refers to "people" who drew the districts in a condition where no people drew them; a sensitivity analysis excluding this item has not been reported.

**Required action**: (a) Specify the standardisation procedure in Section 4. (b) Report whether Likert item 2 shows systematically different response patterns across conditions, and report the sensitivity analysis excluding that item.

### PP6 — C.8 and C.9 Reverse Causality Framing Needs One Sentence (Minor)
**Track**: political-science
**Severity**: Low

**Required action**: Add one sentence to C.8 conclusion: "Our estimates assume stable candidate entry and voter turnout patterns under algorithmic district boundaries; behavioural adaptation by candidates and parties to new boundary configurations could attenuate or amplify the competitive-district advantage reported here."

---

## Paper Scorecard

| Paper | Venue | Stage | Eff. Round | Best Score | Ready? | Primary Gap |
|-------|-------|-------|------------|------------|--------|-------------|
| C.0 validation-overview | internal | draft | R1 | 3.0/4 | No | P1.3–P1.6 unresolved in manuscript |
| C.1 maup-sensitivity | IJGIS | draft | R0 (effective) | 3.6/4 (projected) | No | REVISION-PLAN blank; empirical data absent |
| C.2 cross-census-validation | SIGSPATIAL | ready | R3 | 3.6/4 | Yes | Figures generated; ready to submit |
| C.3 temporal-stability | ACM-KDD | none | R1 (no paper) | 3.1/4 (abstract only) | No | No LaTeX manuscript exists |
| C.4 longitudinal-analysis | Science → Pol. Analysis | ready | R1 | 3.4/4 | Conditional | Venue mismatch; P2 items open |
| C.5 efficiency-gap | APSR | ready | R2 | 4.0/4 | Yes | Unanimous accept; ready to submit |
| C.6 user-study | AJPS | ready | R2 | 4.0/4 | Yes | Three non-blocking items remain |
| C.7 uncertainty-quant | JASA | draft | R1 | 3.0/4 | No | P1.1 CI inconsistency; P1.4 PES calibration |
| C.8 competitive-elections | Pol. Analysis | ready | R2 | 3.0/4 | Yes | One limitations sentence needed |
| C.9 adoption-case-studies | State Politics | ready | R2 | 3.0/4 | Yes | Effects-based constitutional challenge |

**Papers genuinely ready for submission**: C.2, C.5, C.6, C.8, C.9 (5 of 10)
**Papers requiring substantive revision**: C.0, C.1, C.7 (3 of 10)
**Paper requiring creation**: C.3 (1 of 10)
**Paper conditionally ready (venue reassignment)**: C.4 (1 of 10)

---

## Cross-Track Dependency Assessment

Track C depends on Track B's B.2 (+22% compactness claim) and B.7 (CV < 2% seed stability). These upstream dependencies are handled as follows:
- B.2 is validated by C.7 (CI [+15%, +29%]) — adequately handled, pending P1 resolution
- B.7 is validated by C.7 Section 2 (CV < 2% for 96% of states) — adequately handled
- B.2 at finer spatial resolution is claimed but not empirically validated by C.1

Track C feeds Track A's synthesis (A.0) and Track D's legal-implementation case (D.4, C.8, C.9). The current module state is **insufficient for Track A synthesis input**: C.3 does not exist as a paper, C.0's DIA framing is legally ambiguous, and C.1 has no revisions applied. Track D's reliance on C.8 and C.9 is adequately supported by the current paper states.

**Minimum condition for Track A readiness**: PP1 (C.3 manuscript or demotion), PP4 (C.0 DIA and VRA), and PP3 (C.7 abstract CI) must be resolved before C.0 can serve as Track A input.

---

*Panel convened 2026-05-07. This review supersedes all prior paper-level review summaries for module-level assessment purposes. Seven reviewers; quorum satisfied.*
