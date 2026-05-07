# PANEL REVISION PLAN — Track C: Validation
**Module**: track-C
**Plan Date**: 2026-05-07
**Source**: REVIEW_PANEL.md — panel convened 2026-05-07
**Target module score**: 8.0 / 10 (current: 6.4 / 10)

---

## Overview

Six priority items drive the gap between the current module score (6.4) and the Track A readiness threshold (8.0). Five of the six are concentrated in the robustness sub-track (PP1–PP4 plus PP3). The political-science sub-track is already adequate (7.0/10) and requires only minor text additions (PP5, PP6). The critical path to Track A readiness runs through PP1 → PP4 → PP3 in that order, since C.0's synthesis cannot be finalised until C.3's status is resolved and C.0's DIA/VRA gaps are corrected, and C.7's CI headline feeds directly into C.0 Section 2.

**Papers that need no revision work**: C.2, C.5, C.6, C.8, C.9 — submit as-is once PP5 and PP6 sentence additions are complete.

---

## PP1 — Write or Demote C.3 (Blocking)

**Priority**: Critical
**Track**: robustness
**Effort estimate**: 3–6 weeks (write path) or 1 day (demote path)
**Blocks**: C.0 finalisation, Track A synthesis

### Problem

C.3's directory contains complete experimental data: 20 partition CSV files (5 states × 2 algorithms × 2 years), hierarchical tree pickles, stability metrics CSV, VRA compliance results, and multiple Python analysis scripts. The FINDINGS_SUMMARY.md documents the key results. A paper can be written from this material. However, no LaTeX manuscript exists. C.0 Section 4 (sections/04-temporal.tex) cites C.3 results as though they are published findings.

### Option A — Write the Manuscript (Recommended)

**Scope**: Limit C.3 to what the data actually shows — a 5-state (AL, GA, LA, MS, SC) 2010–2020 comparison of recursive bisection vs. n-way partitioning on temporal stability metrics (tract retention rate, IoU, VRA compliance). Do not claim national generalisability; frame as a Deep South pilot with hypotheses for national validation.

**Tasks**:
- [ ] Write sections/00-abstract.tex from FINDINGS_SUMMARY.md
- [ ] Write sections/01-introduction.tex: temporal stability as a validation criterion; recursive vs. n-way as the comparison
- [ ] Write sections/02-background.tex: 1–2 pages; cite Karypis METIS, Duchin ensemble work, Herschlag 2020
- [ ] Write sections/03-methodology.tex: describe the 5-state dataset, graph construction, stability metrics (tract retention, IoU, VRA MM district stability)
- [ ] Write sections/04-results.tex: pull numbers from temporal_stability_metrics.csv and hierarchical_stability.csv; include dendrogram figures already in figures/hierarchical/
- [ ] Write sections/05-discussion.tex: acknowledge Deep South sample limitation; connect to C.2's slice-based framework; note P1.2 hierarchical validation and P1.3 VRA analysis as completed
- [ ] Write sections/06-conclusion.tex
- [ ] Create main.tex assembling all sections
- [ ] Update _panel.yaml: reset stage to draft, round to 0, populate from REVIEW_SUMMARY.md history
- [ ] Revise C.0 sections/04-temporal.tex to cite the completed C.3 manuscript accurately

**Key numbers to use** (from temporal_stability_metrics.csv and FINDINGS_SUMMARY.md):
- Recursive bisection 2010→2020 tract retention: 28.4% (correct figure; abstract previously misreported 80%)
- N-way partitioning 2010→2020 tract retention: 27.6%
- Advantage of recursive over n-way: 1.1 percentage points (not 10 points as early notes suggested)
- The hierarchical structure validation data is in results/hierarchical_stability.csv and figures/hierarchical/

**Scope constraint**: Do not claim the 1.1pp advantage is nationally representative. The five states are a convenience sample from a single region. Frame the contribution as: first empirical measurement of temporal stability difference between hierarchical and flat partitioning, with a pilot dataset establishing measurement methodology for national replication.

### Option B — Demote to Data Archive (Faster)

**Tasks**:
- [ ] Rename directory to C.3+temporal-stability-data or add a README.md clearly marking it as "experimental data archive, not a publication"
- [ ] In C.0 sections/04-temporal.tex, replace citations of C.3 with direct data citations: "Experimental data from [archive] show..."
- [ ] Add a sentence in C.0 Section 6.4 (limitations): "A full paper formalising the 5-state temporal stability analysis (C.3) is in preparation; the quantitative results cited in Section 4 are from experimental data pending peer review."
- [ ] Update MODULE.md to mark C.3 as "data archive" rather than "paper"

**Recommendation**: Option A. The data and analysis scripts are complete; the writing effort is manageable (estimated 20–30 pages of LaTeX). Option B is faster but leaves a permanent gap in the robustness chain at the temporal stability node.

---

## PP2 — Apply Revisions to C.1 (Blocking)

**Priority**: Critical
**Track**: robustness
**Effort estimate**: 2–4 weeks
**Blocks**: IJGIS submission, C.0 Section 2 empirical claims

### Problem

C.1's _panel.yaml records Round 3 strong-accept (3.6/4) with all P1 items marked addressed. The actual manuscript tells a different story: the REVISION-PLAN.md is the blank default template (unfilled), and sections/03-methodology.tex contains active TODO comments. This indicates that the block-level runs were never executed and no revisions were written into the paper.

### Tasks

**Phase 1: Populate the revision plan** (Day 1)
- [ ] Replace the blank REVISION-PLAN.md with an actual plan derived from the Round 3 review history in _panel.yaml and the five P1 items documented there
- [ ] For each P1 item (P1.1–P1.5), document what specific text changes were made in which section files

**Phase 2: Verify and complete manuscript text** (Week 1)
- [ ] Read the current sections/03-methodology.tex and confirm that the P1.2 METIS configuration (Table 3, 10-run stochasticity), P1.3 graph construction (Table 2, Rook contiguity), and P1.4 spatial autocorrelation (Moran's I = 0.42, K=3/5/7 sensitivity) sections are present in the .tex files
- [ ] Remove all TODO comments from methodology section
- [ ] Confirm sections/04-results.tex contains the K=3/5/7 sensitivity sweep and Moran's I results referenced in P1.4

**Phase 3: Block-level empirical data** (Week 2–4)
Choose one of three options:

- [ ] **Option A (Preferred)**: Run the 10-state subset (TX, CA, FL, GA, NY for high-minority; VT, ME, WV, NH, ID for low-minority) at block-group and block resolution. Report mean ± 95% CI from 10 runs per state-year. Expected runtime: 8–12 hours. Replace projected results with empirical data.
- [ ] **Option B (Acceptable)**: Retain projected results but label them explicitly as "representative projections based on validated tract-level methodology" in the abstract and Section 4. Add a data-status subsection (Section 3.7, following C.2's model) and add 95% CIs from the tract-level 10-run analysis as a lower-resolution proxy.
- [ ] **Option C (Minimum)**: Add a 3-state empirical validation subset (e.g., VT, DE, RI — single-district states where block-level runs are computationally tractable) as proof-of-concept, clearly labelled as not nationally representative.

**Phase 4: Figures** (concurrent with Phase 3)
- [ ] Generate Figure 1 (compactness by resolution, 3 panel), Figure 2 (MM district count by resolution), Figure 3 (MAUP sensitivity K=3/5/7) using the matplotlib/seaborn pipeline noted in P2.1
- [ ] Verify figures are referenced and built in the LaTeX compilation

---

## PP3 — Resolve C.7 CI Inconsistency and PES Calibration (Blocking)

**Priority**: High
**Track**: robustness
**Effort estimate**: 3–5 days
**Blocks**: JASA submission, C.0 headline CI claim

### Tasks

**P1.1 — Fix abstract CI** (Day 1)
- [ ] In main.tex abstract, change "[+18%, +26%]" to "[+15%, +29%]" (three-source joint CI)
- [ ] Add a parenthetical: "(accounting for METIS seed variance, shapefile resolution uncertainty, and census coverage error; the two-source CI excluding census undercount is [+18%, +26%])"
- [ ] Verify that sections/07-synthesis.tex Table 7.4 and the synthesis paragraph are consistent with [+15%, +29%] — they currently are correct; the abstract is the only discrepancy

**P1.3 — Add shapefile simplification subsection** (Day 2)
- [ ] Add subsection 4.5 to sections/04-polsby-popper.tex titled "Shapefile Simplification Error"
- [ ] Content: note that the empirical ΔPP = 0.003 from C.1 measures building-block resolution error (tracts vs. blocks), not Douglas-Peucker simplification error; acknowledge this as an unquantified source; bound it by noting TIGER redistricting shapefiles use a fixed tolerance applied symmetrically to both algorithmic and enacted plans, so the comparative advantage is unaffected even if absolute PP values carry simplification-induced measurement error

**P1.4 — Document PES calibration** (Days 3–4)
- [ ] In sections/03-census-sampling.tex Section 3.4, replace the current attribution to "Census Bureau's tract-level undercount estimates from the 2020 PES experimental microdata" with an accurate description of the actual product used
- [ ] If σε = 0.015 is extrapolated from published state-level PES SDs: document the extrapolation; acknowledge that this extrapolation overstates precision if undercount is spatially clustered within states
- [ ] Add a sensitivity table: show the three-source CI under σε = 0.010, 0.015, 0.020 (lower, baseline, upper). The direction should remain positive under all scenarios given the small census contribution to the joint SE.

**P1.5 — Fix Section 4.4 lower-bound description** (Day 1, concurrent with P1.1)
- [ ] In sections/04-polsby-popper.tex Section 4.4, change "The lower bound (0.438) is above the tract-level point estimate (0.441)" to "The lower bound (0.438) is below the tract-level point estimate (0.441) but above the enacted plan mean PP (0.295), confirming that the algorithmic advantage is robust to resolution measurement error."

**P1.6 — Add legal-relevance mapping** (Day 5)
- [ ] Add Table 1 to sections/01-introduction.tex mapping each uncertainty source to the class of legal challenge it addresses: seed variance → determinism challenge; census uncertainty → Wesberry population equality challenge; resolution → compactness measurement challenge
- [ ] This is documented in sections/07-synthesis.tex as prose (Section 7.5); promote it to a table in the introduction for discoverability

---

## PP4 — Fix C.0 DIA Status, Compliance Language, and VRA Property (Blocking)

**Priority**: High
**Track**: robustness
**Effort estimate**: 3–5 days
**Blocks**: Track A synthesis input, downstream legal testimony use

### Tasks

**P1.4 — DIA legal status** (Day 1)
- [ ] In sections/01-introduction.tex at first mention of the Districting Integrity Act, add footnote: "The DIA is a model statute developed within this research program as a concrete target for compliance assessment. It is not enacted federal law. References to 'DIA compliance' should be understood as compliance with the criteria specified in this model statute."
- [ ] Add the same parenthetical in the abstract immediately after "Districting Integrity Act (DIA)": "(model statute; see footnote 1)"

**P1.3 — Qualify compliance language** (Day 2)
- [ ] In sections/06-synthesis.tex Section 6.3, replace all instances of "satisfies" with "provides empirical evidence toward satisfying"
- [ ] Add after the compliance table: "Full 50-state block-level validation (C.1, currently in progress) and extension of partisan fairness analysis to all multi-district states (C.5) are required before unconditional DIA compliance can be claimed."

**P1.5 — VRA property** (Days 3–5)
Choose one option:
- [ ] **Option A (Preferred)**: Add Property 5 "VRA Stability" to the four-property framework in Section 6.1. Write a brief (3–4 paragraph) subsection in sections/06-synthesis.tex covering: (a) C.1's finding that majority-minority district counts are stable across resolutions (137 ± 1); (b) C.3's (or the data archive's) finding on MM district stability across census years; (c) cross-reference to D-track VRASection analysis for the full VRA compliance claim. Mark this as "partially validated — full 50-state analysis pending."
- [ ] **Option B (Faster)**: In sections/06-synthesis.tex Section 6.4 (limitations), add a paragraph acknowledging VRA compliance is not validated in this module. Reference C.1 (137 ± 1 MM districts across the 130× range) and D-track analysis.

**P1.6 — Temporal PP trend attribution** (Day 2, concurrent)
- [ ] In sections/03-cross-census.tex Section 3.3, change "due to refinements in tract boundary alignment" to "possibly reflecting changes in tract boundary methodology between 2000 and 2020, or population redistribution toward geographically simpler suburban areas — the mechanism requires further investigation"

---

## PP5 — Resolve C.6 Standardisation and Likert Item Validity (Pre-Publication)

**Priority**: Medium
**Track**: political-science
**Effort estimate**: 2–3 days

### Tasks

**P4-C — Standardisation procedure** (Day 1)
- [ ] In sections/04-measures.tex, add a subsection or clearly labelled paragraph specifying: (a) which unit of analysis is used (respondent-level means across two maps, or all 4,800 respondent-map pairs); (b) how the standardisation is computed (SD of what distribution); (c) confirm the published d = 0.41 corresponds to this procedure

**P4-D — Likert item 2 sensitivity** (Days 2–3)
- [ ] Check response distributions for item 2 ("the people who drew these districts...") separately for algorithmic vs. non-algorithmic conditions
- [ ] If the distributions differ systematically, add a paragraph in sections/05-results.tex reporting the sensitivity analysis: recompute the primary fairness scale score excluding item 2 in the algorithmic condition and report whether d = 0.41 is robust (expected: it is, given the scale has multiple items)
- [ ] If distributions do not differ meaningfully, add one sentence acknowledging the potential validity concern and noting that response patterns were checked and found not to differ

**P2-E — Post-treatment support measure** (Day 1, concurrent)
- [ ] In sections/05-results.tex Section 5.4, add the label "(post-treatment, non-experimental measure)" to the 71% support figure and a footnote: "This measure was collected after treatment assignment was completed and may reflect treatment contamination; it is reported descriptively and should not be interpreted as a causal treatment effect."

---

## PP6 — Add Reverse Causality Limitation Sentence to C.8 (Minor)

**Priority**: Low
**Track**: political-science
**Effort estimate**: 30 minutes

### Task
- [ ] In sections/08-conclusion.tex of C.8, add to the limitations paragraph: "Our estimates assume stable candidate entry and voter turnout patterns under algorithmic district boundaries; behavioural adaptation by candidates and parties to new boundary configurations could attenuate or amplify the competitive-district advantage reported here."

---

## Revision Sequence and Dependency Order

```
PP1 (C.3 decision)
  └─→ PP4 (C.0 can reference correct C.3 scope)
        └─→ PP3 (C.0 abstract CI must match C.7's correct CI)
              └─→ C.0 is finalised → Track A synthesis input ready

PP2 (C.1 empirical data) — independent; can proceed in parallel
PP5 (C.6 methods) — independent; can proceed in parallel
PP6 (C.8 sentence) — independent; 30 minutes
```

**Critical path**: PP1 → PP4 → PP3 → C.0 final review → Track A gate

**Parallel workstreams**:
- Stream A (robustness): PP1, PP2, PP3, PP4 (sequential within stream)
- Stream B (political-science): PP5, PP6 (independent, low effort)
- Submissions: C.2, C.5, C.6, C.8, C.9 can be submitted now (Stream C, independent)

---

## Estimated Timeline

| Week | Stream A | Stream B | Stream C |
|------|----------|----------|----------|
| 1 | PP1: C.3 decision + scope | PP5: C.6 standardisation + Likert | Submit C.5, C.6 |
| 2 | PP1: C.3 writing (if Option A) | PP6: C.8 sentence | Submit C.2, C.8, C.9 |
| 3 | PP2: C.1 revision plan + manuscript verification | — | — |
| 4–5 | PP2: C.1 block-level runs (Option A or B) | — | — |
| 6 | PP3: C.7 P1.1, P1.3, P1.4, P1.5, P1.6 | — | — |
| 7 | PP4: C.0 DIA, compliance language, VRA | — | — |
| 8 | C.0 final review + Track A gate check | — | Submit C.7 (after PP3) |

**Module readiness gate for Track A**: completion of PP1 + PP4 + PP3, yielding a finalised C.0 with correct DIA framing, resolved VRA property, and consistent CI headline.

---

## Success Criteria

The module reaches the 8.0/10 target when:

1. C.3 either exists as a manuscript (even a scoped pilot paper) or is correctly demoted in C.0 — no phantom citations
2. C.1 has revisions applied to the LaTeX manuscript and either empirical block-level data or an explicit data-status label consistent with C.2's approach
3. C.7 abstract cites [+15%, +29%] and PES calibration is documented with a sensitivity table
4. C.0 discloses DIA status, uses "provides evidence toward" language for compliance, and either has a VRA property or an explicit limitation
5. C.6 specifies standardisation procedure and Likert item 2 sensitivity
6. C.8 has the reverse causality limitation sentence
7. Five papers (C.2, C.5, C.6, C.8, C.9) are submitted to their target venues

**Projected module score after all PP items resolved**: 7.8–8.2 / 10, depending on whether C.3 is written as a full paper (higher) or demoted (lower, since the temporal stability node remains a gap).
