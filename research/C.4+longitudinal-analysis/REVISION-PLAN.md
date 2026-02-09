# Improvement Plan: Twenty Years of Congressional Redistricting

**Paper**: 12+longitudinal-analysis
**Round**: 1 → 2
**Date**: 2026-02-08
**Source**: Synthesis of 5 AI-simulated expert reviews

> **Purpose**: This plan is based on AI-generated quality assessment feedback. Use it to strengthen your work, not as "reviewer responses." These are suggestions for improvement, not requirements.

---

## Summary

The quality assessment (average score: 3.0/4.0) identifies this as solid empirical work with meaningful contributions but requiring major revisions before publication. Key strengths include methodological rigor (consistent algorithm across 20 years) and novel findings (commission effectiveness quantification, geographic stability). Critical improvements needed: add partisan analysis or reframe scope, strengthen statistical rigor, address VRA compliance, provide code/data access, and temper causality claims.

## Simulated Feedback Panel

| # | AI Persona | Based On | Score | Assessment |
|---|---------|-------------|-------|---------|
| 1 | Eric McGhee | Public Policy Institute of California | 3.0/4 | Good but missing partisan metrics |
| 2 | Moon Duchin | Tufts University / MGGG | 3.5/4 | Strong methods, needs ensemble comparison |
| 3 | Justin Levitt | Loyola Law School | 2.5/4 | Adequate but VRA compliance ignored |
| 4 | Micah Altman | MIT Libraries | 3.0/4 | Good work, lacks reproducibility |
| 5 | Bernard Grofman | UC Irvine | 3.0/4 | Good empirical, weak theoretical framing |

---

## P1: Critical Enhancements (High Impact)

### P1.1: Add Partisan Metrics or Reframe Scope
**Source**: P1.1 in SYNTHESIS.md
**Identified by**: McGhee, Grofman, Levitt
**Enhancement**:
- [ ] Option A: Add partisan analysis section using historical election data (2000-2020)
- [ ] Option B: Reframe abstract and introduction as "geometric fairness only" study
- [ ] Add limitation prominently if choosing Option B
**Target section**: New Section 5 OR sections/00-abstract.tex, 01-introduction.tex, 07-discussion.tex

### P1.2: Strengthen Statistical Analysis
**Source**: P1.2 in SYNTHESIS.md
**Identified by**: McGhee, Duchin
**Enhancement**:
- [ ] Report Cohen's d effect size for commission improvement (7.3pp)
- [ ] Add 95% confidence intervals
- [ ] Test robustness with Reock compactness metric
- [ ] Exclude outliers (CA +8.2%) and retest
- [ ] Control for confounds (state size, prior compactness, partisan lean)
**Target section**: sections/05-compactness.tex (Section 5.4)

### P1.3: Address VRA Compliance
**Source**: P1.3 in SYNTHESIS.md
**Identified by**: Levitt
**Enhancement**:
- [ ] Add VRA discussion to methodology
- [ ] Acknowledge algorithmic approach requires VRA post-processing
- [ ] Cite Section 2 case law (Gingles factors)
- [ ] Frame as "pre-VRA baseline" not implementation-ready
**Target section**: sections/03-methodology.tex, 07-discussion.tex

### P1.4: Provide Code and Data Access
**Source**: P1.4 in SYNTHESIS.md
**Identified by**: Altman
**Enhancement**:
- [ ] Create GitHub repository with code and district CSVs
- [ ] Archive data in Zenodo with DOI
- [ ] Add "Data and Code Availability" section
- [ ] Create Methods Supplement with computational details
**Target section**: New section before bibliography

### P1.5: Temper Causality Claims
**Source**: P1.5 in SYNTHESIS.md
**Identified by**: McGhee, Levitt
**Enhancement**:
- [ ] Replace "improved" with "associated with" throughout
- [ ] Add selection bias caveat
- [ ] Note commission states are blue/purple (confound)
- [ ] Soften policy recommendations
**Target section**: sections/00-abstract.tex, 05-compactness.tex, 07-discussion.tex

---

## P2: Substantial Improvements (Medium Impact)

### P2.1: Expand Literature Review
- [ ] Expand Section 2 from ~800 to 1,500-2,000 words
- [ ] Add missing citations (Stephanopoulos & McGhee 2015, DeFord et al. 2019)
- [ ] Create subsections for criteria, metrics, reforms, longitudinal studies
**Target section**: sections/02-background.tex

### P2.2: Clarify Theoretical Claims
- [ ] Either engage with representation theory OR drop "theoretical contributions" claim
- [ ] Explain why compactness matters for representation quality
**Target section**: sections/07-discussion.tex (Section 7.3)

### P2.3: Test Alternative Compactness Metrics
- [ ] Add Reock compactness metric
- [ ] Show results robust across metrics
- [ ] Discuss metric limitations (PP vs Reock)
**Target section**: sections/03-methodology.tex, 05-compactness.tex

### P2.4: Disaggregate Commission Types
- [ ] Separate independent vs bipartisan vs advisory commissions
- [ ] Test whether independent outperform others
**Target section**: sections/05-compactness.tex (5.4)

---

## P3: Refinements (Optional)

- Graph construction pseudocode
- Population balance justification (±0.5%)
- International comparative context
- Case studies (CA 2010, MI 2018)
- Legal citations (Shaw, Rucho, etc.)
- Soften 2030 predictions

---

## Estimated Effort

- P1 items: 3-4 weeks
- P2 items: 1-2 weeks
- Total: 4-6 weeks for P1+P2

## Expected Outcome

After revision:
- Science: Strong acceptance if partisan analysis added
- Political Analysis: Very strong if geometric-only scope
