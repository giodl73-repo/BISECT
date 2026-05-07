# Revision Plan — B.12 ProportionalSection

**status: ACCEPTED**

**Current round**: 3  
**Round 1 avg score**: 2.90 / 4.0  
**Round 2 avg score**: 3.20 / 4.0 (+0.30)  
**Round 3 avg score**: 3.54 / 4.0 (+0.34)  
**Stage**: accepted (post Round 3)

## Round 3 Scores

| Reviewer | R1 | R2 | R3 | Delta R2→R3 |
|---|---|---|---|---|
| Karypis | 3.0 | 3.4 | 3.6 | +0.2 |
| Rodden | 3.0 | 3.1 | 3.5 | +0.4 |
| Duchin | 3.0 | 3.2 | 3.6 | +0.4 |
| Stephanopoulos | 3.0 | 3.3 | 3.6 | +0.3 |
| Liang | 2.5 | 3.0 | 3.4 | +0.4 |
| **Average** | **2.90** | **3.20** | **3.54** | **+0.34** |

## Round 2 Scores (historical)

| Reviewer | R1 | R2 | Delta |
|---|---|---|---|
| Karypis | 3.0 | 3.4 | +0.4 |
| Rodden | 3.0 | 3.1 | +0.1 |
| Duchin | 3.0 | 3.2 | +0.2 |
| Stephanopoulos | 3.0 | 3.3 | +0.3 |
| Liang | 2.5 | 3.0 | +0.5 |
| **Average** | **2.90** | **3.20** | **+0.30** |

## What Changed Round 2 → Round 3 (P1 items resolved)

- **Theorem 2 scope**: Added Remark after Theorem 2 (Lorenz feasibility) stating contiguity as the binding constraint and noting non-contiguous assignments may admit proportional solutions in more cases.
- **MAUP sensitivity**: Added "Sensitivity to Geographic Resolution" subsection to §6 (compromise/discussion) reporting qualitatively identical results at block-group resolution (WI improves at η=1.10, NC worsens at all η). Cites Chen & Rodden (2013).
- **"What this paper does not show" disclaimer**: Added "Scope of Claims" subsection to §7 (legal) with 5 explicit disclaimers: courts cannot order this method, no partisan outcome is constitutionally required, algorithm is explicitly partisan, σ<0.05 does not imply gerrymandering, B.12 may worsen outcomes (NC example).
- **Nevada +23.8pp**: Added dedicated paragraph in §5 explaining the outlier: low k=4 (55 tracts/district average), METIS granularity limitation, D_votes constraint forces over-concentration in Las Vegas urban area.

## Round 3 Open Items (P2 — for journal submission)

- [ ] Add niter and ncuts to B.12 METIS parameter specification (Karypis)
- [ ] Specify whether C(G) values are Lorenz-analytical or METIS-empirical (Karypis)
- [ ] Clarify Table 1 seed determinism: "all 30 seeds identical" or report SD (Duchin, Liang)
- [ ] Strengthen Scope of Claims item (4): σ<0.05 does not immunise from gerrymandering claims (Stephanopoulos)
- [ ] Report quantitative σ at block-group resolution for WI and NC (Rodden, Karypis, Liang)
- [ ] Add Harvard Dataverse DOI for VEST 2020 precinct returns (Liang, Stephanopoulos)
- [ ] Test 2022 Senate returns sensitivity for NC and WI σ classification (Rodden)

## What Changed Round 1 → Round 2

**Resolved in Round 2:**
- **P1-Karypis / P1-Liang (empirical validation)**: Added §5.1 with 6-state × 3-η empirical METIS table (Table 1). Separates empirical METIS results from analytical bounds throughout. METIS version (5.1.0), seed count (30), and balance tolerance (1.5%) specified.
- **P1-Stephanopoulos / P1-Liang (formal gap definition)**: Added Definition 2.1 (proportionality gap Δ = s_D/k − d) to §3.2 (framework). Explicitly distinguished from efficiency gap (Stephanopoulos 2015).
- **P1-Stephanopoulos / P1-Liang (50-state appendix)**: Added Appendix A with σ values for all 42 multi-district states, organized by free/cheap/expensive category.
- **P1-Stephanopoulos (Rucho engagement)**: Expanded §7.4 to four paragraphs engaging Rucho v. Common Cause (2019) directly. Three post-Rucho legal developments characterized: state court landscape, algorithmic neutrality standard, procedural fairness claim.
- **P1-Duchin (C×σ bound)**: Corollary 4.1 revised to characterize C(G) as state-specific (not universal). Geometric interpretation as Lorenz curve slope at x = k_R/k. Three example values (WI ≈ 4,200; GA ≈ 720; AZ ≈ 85) grounded in empirical runs.

## Outstanding P1 Items After Round 2

### High priority (consensus across ≥3 reviewers)

- [ ] **Theorem 2 scope overstated** (Duchin, Liang): "If and only if" in Lorenz feasibility theorem covers the non-contiguous case only. Should be stated as necessary condition or biconditional with contiguity caveat clearly flagged.
- [ ] **Vote data source sensitivity untested** (Liang, Rodden): σ classification tested only for 2020 presidential returns. Sensitivity to 2022 Senate returns not examined. NC and WI in particular had unusual presidential vs. Senate splits.
- [ ] **MAUP sensitivity unaddressed** (Rodden): Does σ change at block-group resolution? At minimum, report GA and WI σ at block-group resolution or explain why this is deferred.

### Medium priority (1–2 reviewers)

- [ ] **METIS full parameter vector** (Karypis, Liang): ncuts, niter, numbering not reported. Version 5.1.0 specified but defaults differ from 5.0.x.
- [ ] **C(G) estimation procedure** (Karypis): Is C(G) computed from Lorenz curve analytically or from METIS runs empirically? The two procedures should give the same answer but the paper should specify which was used.
- [ ] **Nevada over-proportionality explanation** (Duchin): +23.8pp at η=1.10 is the most striking result in Table 1 but receives less explanation than WI and NC. Deserves a dedicated short paragraph.
- [ ] **"What This Paper Does Not Show" disclaimer** (Stephanopoulos): A paragraph clarifying that σ < 0.05 does not imply an enacted plan is a gerrymander. Prevents misuse in litigation.
- [ ] **Confidence intervals for Table 1** (Liang): 30-seed runs produce single point estimates. Are results deterministic (zero inter-seed variance) or averaged? If averaged, report SD or CI.
- [ ] **B.9 baseline verification** (Duchin): The B.9 column in Table 1 should be verified against the B.9 paper's reported figures, not assumed.
- [ ] **Appendix A data source citation** (Stephanopoulos): Footnote specifying VEST/Fekrazad interpolation source for D vote share figures.
- [ ] **Monocentric vs. polycentric Lorenz disaggregation** (Rodden): Analytical framing of why WI (monocentric) and GA (polycentric) behave differently would strengthen the political geography contribution.

## Round 2 Themes

**What moved the score (large moves):**
- Liang: +0.5 — The empirical table resolved the analytical-vs-empirical conflation that was his primary Round 1 concern.
- Karypis: +0.4 — The empirical table plus C(G) state-dependency resolution.
- Stephanopoulos: +0.3 — The Rucho engagement and formal gap definition.

**What produced more modest moves:**
- Rodden: +0.1 — MAUP sensitivity remains unaddressed, which was his most important Round 1 concern.
- Duchin: +0.2 — Theorem 2 scope issue is the most technically significant remaining concern.

## Trajectory Assessment

**Score trajectory**: 2.90 → 3.20.  
**Target**: 3.2/4 (accept territory). **Achieved at Round 2.**

The paper has crossed the 3.2 threshold with the Round 2 revisions. The remaining P1 items fall into two categories:

1. **Quick fixes** (Theorem 2 scope, Nevada explanation, "What This Paper Does Not Show," data source citation): Can be addressed in a final author revision.
2. **Deferred empirical work** (MAUP sensitivity, vote data sensitivity, monocentric/polycentric disaggregation): These are valid future-work items that strengthen the paper but are not required for acceptance at the current score level.

**Recommendation**: Ready for author final revision and journal submission. Priority for next author revision: Theorem 2 scope (one-sentence fix), Nevada explanation (one paragraph), "What This Paper Does Not Show" disclaimer (one paragraph), METIS full parameter vector (one footnote). These four items would likely push the score to 3.4+.
