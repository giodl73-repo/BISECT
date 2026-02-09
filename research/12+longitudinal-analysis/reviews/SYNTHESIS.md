# Review Synthesis: Twenty Years of Congressional Redistricting

**Paper**: 12+longitudinal-analysis
**Round**: 1
**Date**: 2026-02-08
**Reviewers**: 5 (McGhee, Duchin, Levitt, Altman, Grofman)

---

## Overall Verdict

**Average Score**: 3.0 / 4.0 (Good paper with significant contributions but major revisions needed)

**Score Distribution**:
- 3.5/4.0 (Strong): Duchin
- 3.0/4.0 (Good): McGhee, Altman, Grofman
- 2.5/4.0 (Adequate): Levitt

**Consensus**: Accept with major revisions. The paper makes meaningful empirical contributions—first longitudinal algorithmic redistricting study, quantified commission effectiveness, geographic stability metrics—but requires substantial improvements in several areas before publication.

---

## Key Strengths (Consensus)

All reviewers praised:

1. **Methodological rigor**: Consistent algorithm (METIS) with identical parameters across 2000/2010/2020 is exemplary longitudinal design
2. **Comprehensive scope**: All 50 states, 3 census decades, 435 districts
3. **Novel metrics**: IoU-based geographic stability analysis (61% districts maintained >0.7 overlap)
4. **Commission quantification**: First multi-decade empirical assessment of commission effectiveness (+7.3pp improvement)
5. **Clear presentation**: Well-written, logical structure, effective figures

---

## Critical Issues Summary

The paper faces **three major thematic critiques** from multiple reviewers:

### Theme 1: Missing Partisan Analysis (McGhee, Levitt, Grofman)
- Compactness ≠ fairness: geometric metrics without partisan outcomes
- Cannot assess whether algorithmic districts are actually less biased
- "Impossibility defense" misapplied (irrelevant for academic research)

### Theme 2: Legal/Practical Constraints Underestimated (Levitt, Grofman)
- VRA compliance ignored (Section 2 requires majority-minority districts)
- Commission recommendations overstated (misunderstand how they operate)
- Implementation feasibility not addressed

### Theme 3: Reproducibility Deficits (Altman, Duchin)
- No code/data access
- Missing computational details (graph construction, exact METIS invocation)
- Data provenance unclear (which shapefiles? when downloaded?)

---

## P1 Issues (Blocking — Must Address)

These issues prevent publication and must be resolved:

### P1.1: Add Partisan Metrics or Reframe Scope (McGhee, Grofman)

**What**: Paper omits partisan fairness analysis entirely, focusing only on geometric compactness. This is a critical gap for redistricting scholarship.

**Why it's P1**:
- Partisan bias is the central concern in redistricting debates
- Compactness alone doesn't guarantee fairness (compact districts can be biased)
- Missing table-stakes analysis for Science/APSR venue

**Options**:
1. **Add partisan analysis** (preferred): Use historical election data (2000-2020 presidential/congressional results) to compute:
   - Efficiency gaps for algorithmic vs enacted districts
   - Partisan bias metrics (mean-median difference)
   - Seats-votes curves
2. **Reframe scope** (fallback): Explicitly present as "geometric fairness only" study
   - Add limitation to abstract and introduction (not just discussion)
   - Retarget to methods journal (Political Analysis, Statistics and Public Policy)

**Where to address**: New Section 5 or prominent limitation in abstract/intro

---

### P1.2: Strengthen Statistical Analysis (McGhee, Duchin)

**What**: Commission impact analysis (7.3pp improvement, t=2.89, p=0.006) lacks critical statistical details.

**Why it's P1**:
- Small sample (N=6 commission, N=44 non-commission) requires careful treatment
- No effect size, confidence intervals, or robustness checks reported
- Readers cannot assess reliability of findings

**Required additions**:
1. **Effect size**: Report Cohen's d
2. **Uncertainty**: Add 95% confidence intervals for 7.3pp difference
3. **Robustness**:
   - Test with alternative compactness metrics (Reock, Convex Hull)
   - Exclude outliers (CA +8.2%) and retest
   - Control for confounds (state size, prior compactness, partisanship)
4. **Sensitivity**: Show results hold across different statistical specifications

**Where to address**: Section 5.4 (Reform Impact) — expand from 1 table to 2-3 tables + robustness paragraph

---

### P1.3: Address VRA Compliance (Levitt)

**What**: Paper generates districts without considering Voting Rights Act Section 2 requirements for majority-minority districts.

**Why it's P1**:
- VRA compliance is a legal requirement, not optional constraint
- Algorithmic approach as described would violate VRA in states like Alabama
- Undermines implementation viability claims

**Options**:
1. **Integrate VRA** (substantial work): Add racial demographics to METIS edge weights, validate majority-minority district preservation
2. **Acknowledge limitation** (lighter lift): Explicitly frame as "pre-VRA baseline requiring manual adjustment"
   - Add VRA discussion to methodology and limitations
   - Note that real implementation requires VRA-compliant post-processing
   - Cite Gingles factors and Section 2 case law

**Where to address**: Section 3 (Methodology) and Section 7 (Discussion/Limitations)

---

### P1.4: Provide Code and Data (Altman)

**What**: Paper lacks computational reproducibility—no code repository, no data access, insufficient implementation details.

**Why it's P1**:
- Violates computational science standards
- Results cannot be independently verified
- Prevents replication and extension by other researchers

**Required**:
1. **GitHub repository** with:
   - METIS configuration files
   - Census tract preprocessing scripts
   - IoU computation code
   - District assignment CSVs (50 states × 3 years)
2. **Data archive**: Zenodo or Dataverse with permanent DOI
3. **Methods supplement**: Computational pipeline, software versions, hardware specs
4. **Data provenance**: URLs, download dates, MD5 hashes for all inputs

**Where to address**: Add "Data and Code Availability" section before bibliography

---

### P1.5: Temper Causality Claims (McGhee, Levitt)

**What**: Paper claims commissions "improved outcomes" and "reverse the negative trend" but observational design cannot support causal claims.

**Why it's P1**:
- Commission adoption is non-random (selection bias)
- All commission states are blue/purple (confounded with partisanship)
- Results could reflect partisan sorting, not commission effectiveness

**Required changes**:
1. **Reframe throughout**: "Associated with" not "caused" or "improved"
2. **Add DiD analysis** (mentioned in limitations, should be in methods):
   - Compare commission states pre/post adoption to matched controls
   - E.g., CA 2000 vs CA 2010/2020, with AZ as comparison
3. **Acknowledge selection bias**: Commission adoption is not random
4. **Soften recommendations**: Commissions "correlate with improved compactness" not "improve compactness"

**Where to address**: Abstract, Section 5.4, Discussion (throughout)

---

## P2 Issues (Important — Strongly Recommended)

These issues significantly limit the paper but don't block publication if addressed with clear limitations:

### P2.1: Expand Literature Review (Grofman, McGhee)

**What**: Background section cites only 5 papers, omitting vast redistricting literature.

**Missing**: Partisan metrics (Stephanopoulos & McGhee 2015), ensemble methods (DeFord et al. 2019), VRA literature (Grofman & Handley), legal cases (Shaw v. Reno, Rucho)

**Fix**: Expand Section 2 from ~800 words to 1,500-2,000 words with subsections:
- 2.1: Redistricting criteria
- 2.2: Measuring gerrymandering
- 2.3: Reform interventions
- 2.4: Longitudinal studies

---

### P2.2: Add Theoretical Framing (Grofman)

**What**: Paper claims "theoretical contributions" (Discussion 7.3) but doesn't advance theory—findings are empirical/descriptive.

**Fix**:
- Engage with representation theory (Pitkin, Mansbridge, Dahl)
- Explain *why* compactness matters for representation quality
- Formulate testable hypotheses linking compactness to outcomes (turnout, responsiveness)
- Or: Drop "theoretical contributions" claim and present as purely empirical

---

### P2.3: Test Alternative Compactness Metrics (Duchin)

**What**: Exclusive use of Polsby-Popper without comparing to Reock, Convex Hull, or Cut Edges.

**Fix**:
- Add at least one alternative metric (Reock recommended)
- Show results are robust across metrics
- Discuss metric choice and limitations (PP penalizes perimeter, sensitive to coastlines)

---

### P2.4: Ensemble Method Comparison (Duchin)

**What**: Single METIS run per state/year doesn't characterize distribution of possible algorithmic plans.

**Fix**:
- Generate N=100-1000 ensemble plans using ReCom (recombination)
- Compare METIS compactness to ensemble mean/median
- Validate that METIS is representative of algorithmic approaches generally

---

### P2.5: Disaggregate Commission Types (Levitt, Grofman)

**What**: All commissions lumped together (CA, AZ, CO, MI, VA, NY) but they vary:
- Independent (CA, AZ) vs bipartisan (VA) vs advisory (NY)

**Fix**: Test whether independent commissions outperform bipartisan/advisory commissions

---

### P2.6: Add REDMAP Subgroup Analysis (McGhee)

**What**: REDMAP targeted specific states (OH, PA, WI, NC, MI), not all states. Paper attributes general decline to REDMAP without disaggregation.

**Fix**: Compare REDMAP-targeted states to non-targeted states. Test whether targeted states declined more.

---

## P3 Issues (Minor — Nice to Have)

These issues would improve the paper but are not essential:

### P3.1: Graph Construction Details (Duchin, Altman)
Add pseudocode or reproducibility supplement specifying how census tracts become graph nodes/edges

### P3.2: Population Balance Justification (Duchin)
Justify ±0.5% tolerance choice. Test sensitivity to ±0.1% and ±1%

### P3.3: Computational Workflow Documentation (Altman)
Add Methods Supplement with pipeline diagram, software versions, runtime estimates

### P3.4: International Comparative Context (Grofman)
Add paragraph on commission experience in Canada, UK, Australia

### P3.5: Counterfactual Analysis (Grofman)
Model what 2020 would look like if all states adopted commissions in 2010

### P3.6: Case Study Boxes (Grofman)
Add detailed case studies for CA 2010 commission and MI 2018 ballot initiative

### P3.7: Legal Citations (Levitt)
Add citations to key SCOTUS cases (Shaw, Miller, Rucho, Brnovich)

### P3.8: Remove Predictive Claims or Add Uncertainty (McGhee)
2030 predictions (TX +2, FL +2) have false precision. Add uncertainty or remove.

---

## Revision Priorities

**Must address** (will block acceptance):
- P1.1: Partisan metrics or scope reframing
- P1.2: Statistical rigor (effect sizes, CIs, robustness)
- P1.3: VRA discussion
- P1.4: Code/data access
- P1.5: Causality tempering

**Should address** (will significantly strengthen):
- P2.1: Literature expansion
- P2.2: Theoretical framing or claim removal
- P2.3: Alternative compactness metrics
- P2.4: Ensemble comparison (if feasible)

**Nice to have** (polish):
- P3 items as time permits

---

## Recommended Action Plan

1. **Week 1-2**: Address P1 issues
   - Add partisan analysis or reframe scope (P1.1)
   - Strengthen statistics (P1.2)
   - Write VRA discussion (P1.3)
   - Create GitHub repo + data archive (P1.4)
   - Revise causality language throughout (P1.5)

2. **Week 3**: Address P2 issues
   - Expand literature review
   - Test alternative compactness metric
   - Clarify theoretical vs empirical contributions

3. **Week 4**: Polish and P3 items
   - Add case studies, computational details
   - Final pass for clarity and consistency

4. **Resubmit**: Target Science if partisan analysis added, or Political Analysis if scope remains geometric-only

---

## Bottom Line

This is **solid empirical work with meaningful contributions** but needs substantial revision before publication in a top-tier venue. The methodological rigor (consistent algorithm across 20 years) is exemplary, and the commission effectiveness quantification is valuable for policy debates. However, the omission of partisan analysis, weak statistical treatment, and reproducibility deficits must be addressed.

**Estimated revision effort**: 4-6 weeks for thorough response to P1+P2 issues

**Expected outcome after revision**: Strong acceptance likelihood at Science (if partisan analysis added) or Political Analysis (geometric-only focus)
