# REVIEW_PANEL.md — Track A: Portfolio Synthesis

**Module**: track-A
**Review Type**: Module-level panel review (synthesis/capstone track)
**Date**: 2026-05-07
**Reviewers**: 7-member panel
**Review Round**: 1

---

## Panel Composition

| # | Reviewer | Affiliation | Primary Expertise |
|---|----------|-------------|-------------------|
| R1 | Moon Duchin | Tufts / MGGG | Mathematical redistricting; synthesis accuracy |
| R2 | Jonathan Rodden | Stanford | Political geography; partisan findings communication |
| R3 | Nicholas Stephanopoulos | Harvard Law | Election law; policy brief quality |
| R4 | Suzanne Mettler | Cornell Government | Policy communication; legislative staff audiences |
| R5 | Victoria Stodden | UIUC iSchool | Replication standards; AEA reproducibility |
| R6 | John Sides | Vanderbilt | Political science communication; public-facing writing |
| R7 | Thomas Brunell | UT Dallas | Redistricting practice; commissioner/practitioner perspective |

---

## Module Overview

Track A is the capstone of the program, not a research track in its own right. Its papers do not generate new empirical findings — they synthesize findings from Tracks B through H and translate them across audiences. The quality of Track A depends entirely on the quality of the underlying tracks: A.0 is only as strong as the 60+ papers it integrates, and A.5 is only as credible as the legal and empirical work in Tracks C, D, and G.

The panel assessed each paper individually and rendered a module-level verdict. Because Track A is a synthesis track, the panel gave particular weight to:
1. **Synthesis accuracy**: Does Track A faithfully represent what Tracks B–H actually show?
2. **Translation effectiveness**: Do the individual papers serve their intended audiences?

---

## Paper-by-Paper Assessment

---

### A.0 — Synthesis Metapaper
**Target Venue**: *Science*
**Status**: Draft with R1 revision cycle complete (mean 3.0/4.0)
**Scope**: Full national demonstration — 50 states × 3 census decades × 6 headline findings

#### Synthesis Accuracy Assessment

The panel cross-checked A.0's six headline findings against the Track B–H MODULE.md primary claims.

| A.0 Claim | Source Track | Verdict |
|-----------|--------------|---------|
| +22% compactness vs. enacted maps | B.1 | Accurate; note B.2 adds further +22% to +44% total; inter-document inconsistency with A.1/A.2 citing "+56%" requires reconciliation |
| 137 MM districts vs. 68 enacted (+69 surplus) | D.0 | Accurate and consistent |
| 42% demographic threshold | D.1 | Accurate; Gingles qualification now present |
| 62% partisan bias reduction; EG -3.2% / +5.1% | C.5 | Accurate |
| 80% temporal stability; 14 pp recursive advantage | C.3/B.15 | Accurate and consistent |
| 3.2× variance decomposition (geography > time) | C.2 | Directionally consistent |

#### Material Omissions

**Omission 1 — Track G ensemble context (G.1)**: The bisection plan sits at the 0.1–0.7th compactness percentile in the feasible-plan ensemble for WI, GA, PA, CA — meaning it is near the compactness extremum, not the partisan extremum, of all valid plans. This finding is absent from A.0. For a *Science* submission, reviewers familiar with the MGGG/GerryChain literature will immediately ask where the bisection plan sits in ensemble distributions.

**Omission 2 — Track B structure variants**: A.0 writes as if recursive bisection has one mode. Track B MODULE.md documents 10+ structure modes: GeoSection, ApportionRegions (NC 7D/7R, 223D/209R national), ProportionalSection (sigma→0 paradox), NestSection, VRASection, StabilitySection. A.0 does not acknowledge that the "algorithm" is a parameterized family with materially different outputs.

**Omission 3 — Track H search strategies**: PercentileSweep (statutory legal posture selection), BisectionEnsemble, and the redist-ensemble Rust implementation (2500× speed) are unmentioned. These are directly relevant to implementability claims in A.0's policy implications section.

**Omission 4 — Track F state legislative**: A.0 focuses entirely on 435 congressional districts. Track F covers all 50 state house chambers, bicameral nesting, and high-k chambers. Given that A.0's implications section argues for adoption by redistricting commissions that draw state maps, the absence of Track F evidence leaves the generalizability claim unsubstantiated.

#### Reviewer Assessments

**R1 (Duchin)**: The ensemble context omission (PP1.3) is the most significant accuracy concern. The +22%/+56% terminology inconsistency across the synthesis documents needs resolution. The "cannot gerrymander" language propagation to A.1/A.2 is a downstream concern.

**R2 (Rodden)**: The partisan findings are now evaluable with proper metrics after the efficiency gap addition. The absence of C.8 (30% more swing districts) from the implications section is a missed opportunity.

**R3 (Stephanopoulos)**: The justiciability section (5.4) is the strongest new contribution from the R1 revision cycle. The analysis that algorithmic methods relocate rather than resolve Rucho's standards problem is careful and accurate.

**R6 (Sides)**: The *Science* framing and Huntington-Hill precedent analogy remain the paper's strongest structural assets. The abstract is well-calibrated for a broad scientific audience.

**Paper Score**: **7.2 / 10**

---

### A.1 — Portfolio Guide
**Target Venue**: Internal / researcher-facing
**Status**: Draft; no panel review history

#### Accuracy Assessment

A.1 has a fundamental structural mismatch: it describes an "11-paper portfolio" organized as Papers 01–11, but the actual program has 75 papers organized into Tracks B–H. A.1 was written at an early portfolio snapshot and has not been updated.

Specific accuracy failures:
- "All 11 papers are complete and ready for submission" — materially false
- Paper 01–11 numbering does not correspond to current track/paper identifiers
- Tracks C, E, F, G, H are entirely absent
- "+56% compactness improvement" presented without baseline disambiguation
- Contact email "giovanni.deluca@example.com" is a placeholder never updated

The four reading-path tables are genuinely useful as a navigation format and should be preserved in any rewrite. The flat 11-paper structure is not salvageable as a guide to the current program.

**Paper Score**: **4.1 / 10**

---

### A.2 — Portfolio Summary
**Target Venue**: Internal / researcher/funder-facing
**Status**: Draft; no panel review history

#### Accuracy Assessment

A.2 has the same foundational scope problem as A.1 (11 papers vs. 75), but the per-paper descriptions are accurate and well-written for the papers they cover.

**What is accurate**: Per-paper descriptions for the 11 covered papers are detailed and internally consistent. The "Four Core Contributions" framework is a sound synthesis scaffold. The "Impact and Implications" subsections are well-calibrated.

**What is inaccurate or missing**: "All 11 papers are complete and ready for submission" — same staleness problem. Tracks C, E, F, G, H entirely absent. Replication repository URL is a placeholder. "150 redistricting scenarios (50 states × 3 census decades)" omits Track F.

**R2 (Rodden)**: The "Portfolio-Level Themes" section is well-structured. Extending it to cover Track C's full scope would partially address the omission without full rewrite.

**R6 (Sides)**: The "Four Core Contributions" bullet structure is the most communications-effective passage in the full synthesis track and should be used as the lead scaffold.

**Paper Score**: **5.8 / 10**

---

### A.3 — Portfolio Visualization Guide
**Target Venue**: Non-technical audiences — judges, journalists, legislators
**Status**: R2 complete (avg R1: 3.12/4.0; avg R2: 3.48/4.0; passes target ≥ 3.3/4.0)

#### Accuracy Assessment

A.3 is the most review-mature synthesis document. Two rounds produced meaningful improvements. The GeoSection counterexample was added (R2, P1-B): "same data under GeoSection gives 5D/9R, illustrating that algorithm choice — not the data — determines partisan outcomes." The Gingles qualification was added to the 42% VRA threshold.

**Residual issues** (deferred from R2):
- P2-A: Edge-weight clarification not yet added
- P2-B: "Cannot gerrymander" language still unqualified (C.5's -3.2% geographic efficiency gap not mentioned)
- P2-C: Repository URL missing
- P2-D: Track G scope qualifier needed ("statistically indistinguishable from random draws" needs qualification for specific states)

The panel assesses P2-A and P2-B as higher urgency than their P2 classification suggests. A.3 is the primary non-technical entry point for judges and legislative staff — the audiences least likely to read A.0's more nuanced qualifications.

**Track coverage note**: Abstract references "more than forty papers organized into seven tracks (A–G)." Program has 75 papers across eight tracks (A–H). Track H is absent from A.3's track summary section.

**Paper Score**: **7.6 / 10**

---

### A.4 — Replication Materials
**Target Venue**: Zenodo / Harvard Dataverse / Open ICPSR
**Status**: Planned — plan.md only; no main.tex; package not executed

#### Assessment

The A.4 plan is technically sound and AEA-aware. The three-level replication design (quick verify / selective / complete) matches AEA Data and Code Availability Policy structure.

**Critical blocking issue — Python vs. Rust tool inconsistency (PP1.1)**:

The plan throughout describes a Python codebase and Python-based pipeline:
- "Complete Python codebase (src/, scripts/, tests/)"
- "Python 3.13+" as primary software requirement
- `scripts/pipeline/run_state_redistricting.py` as the primary entry point

The production system described in CLAUDE.md is the `redist` Rust binary — "~213× faster than the archived Python pipeline." The Python pipeline is archived (`archive/python-pipeline-final/`) and described as "sealed forensic reference — do not touch."

This is not a minor discrepancy. A reviewer attempting to replicate following A.4's instructions would fail. The plan must be rewritten to describe the Rust CLI workflow (`redist build`, `redist label-analyze`, etc.) as the primary replication entry point.

**Secondary issues**: Plan scopes to "23 empirical papers (B.1–B.5, C.1–C.5, D.0–D.3, E.1–E.5)" — omits Track F (7 papers), Track G (15 papers), Track H (4 papers). Repository URL and Zenodo DOI are placeholders.

**R5 (Stodden)**: The plan architecture is genuinely strong. The blocking issue is entirely the Python/Rust inconsistency. Resolve that and execute; the plan's structure does not need redesign.

**Paper Score**: **3.9 / 10**

---

### A.5 — Policy Brief
**Target Venue**: State legislators, redistricting commissioners, legislative staff
**Status**: Planned — plan.md only; no main.tex

#### Assessment

The plan is well-conceived for its target audience. The three adoption models (legislative mandate, commission guidelines, court-ordered remedy) are realistic. Cost estimates ($50K–$200K vs. $1M+ redistricting litigation) are the most audience-relevant comparison and should be prominent.

**Audience calibration**: Correct. R4 (Mettler) and R3 (Stephanopoulos) both assessed the planned structure as appropriate.

**Finding selection accuracy**: The five planned findings are consistent with their source tracks. However, the finding set is incomplete for the target audience.

**Missing Track F — state legislative evidence (PP2.2)**:
The brief targets state legislators and redistricting commissions. These audiences primarily oversee state chamber redistricting, not congressional redistricting. Track F documents that the algorithm works for all 50 state house chambers (F.1), scales to high-k chambers up to 400 districts (F.3), and extends VRA compliance analysis to state legislative maps (F.6). A policy brief for state legislators that does not demonstrate the algorithm works for state maps will face the immediate objection "does this work for our state house?" and will have no answer.

**Missing C.8 — competitive elections finding (PP2.3)**:
C.8 (30% more swing districts vs. enacted plans) is more politically salient to state legislators than temporal boundary stability (Finding 4). Competitive elections are a bipartisan concern.

**Note on "+20% compactness"**: The plan correctly cites "+20% more compact" (B.2 improvement over enacted maps at 0.367 vs. 0.305 Polsby-Popper). This is accurate and avoids the "+56% vs. unweighted baseline" figure that would be misleading for a policy audience.

**R4 (Mettler)**: The brief's FAQ ("No catch — only losers are politicians who benefit from gerrymandering") is too adversarial for legislative distribution.

**R7 (Brunell)**: The absence of Track F evidence is a critical gap for this audience. Every redistricting commissioner asked to consider this approach will immediately ask whether it works for state maps.

**Paper Score**: **5.2 / 10**

---

## Module-Level Assessment

### Capstone Chain Quality

| Link | Document Status | Accuracy Status | Chain Status |
|------|----------------|-----------------|--------------|
| A.0 Synthesis Metapaper | Draft, R1 revision complete | Accurate in claims; material omissions | Functional |
| A.1 Portfolio Guide | Draft, no review | Wrong portfolio scope (11 vs. 75 papers) | Broken |
| A.2 Portfolio Summary | Draft, no review | Accurate for B/D; C/E/F/G/H absent | Partial |
| A.3 Visual Guide | R2 complete (3.48/4.0) | Four P2 items outstanding; two higher urgency | Functional |
| A.4 Replication Materials | Plan only | Tool description inconsistent with production system | Blocked |
| A.5 Policy Brief | Plan only | Finding set incomplete (missing Track F, C.8) | Unexecuted |

### Track Coverage Summary

| Track | A.0 | A.1 | A.2 | A.3 | A.5 plan |
|-------|-----|-----|-----|-----|----------|
| B Algorithm (25 papers) | Foundations only | Partial (11 papers) | Partial (11 papers) | Summary | Partial |
| C Validation (10 papers) | Yes (5 of 10) | Partial | Partial | Yes | Partial |
| D VRA/Legal (6 papers) | Yes | Yes | Yes | Yes | Yes |
| E Experimental (8 papers) | No | No | No | No | No |
| F State Legislative (7 papers) | No | No | No | No | No |
| G Ensemble (15 papers) | No | No | No | No (scope caveat only) | No |
| H Search (4 papers) | No | No | No | No | No |

Five of seven underlying tracks are absent or materially underrepresented. Track A synthesizes roughly 40% of the program (Tracks B/C/D) and ignores 60% (Tracks E/F/G/H).

### Module Score

| Paper | Score | Tier |
|-------|-------|------|
| A.0 Synthesis Metapaper | 7.2 / 10 | Publishable with targeted revisions |
| A.1 Portfolio Guide | 4.1 / 10 | Requires substantial revision |
| A.2 Portfolio Summary | 5.8 / 10 | Updatable; scope expansion needed |
| A.3 Portfolio Visualization | 7.6 / 10 | Near-ready (sentence-level fixes) |
| A.4 Replication Materials | 3.9 / 10 | Blocked (unexecuted; tool inconsistency) |
| A.5 Policy Brief | 5.2 / 10 | Unexecuted (plan quality score) |

**Module Score**: **5.6 / 10**
**Module Tier**: Below target — path to 8.5+ is well-defined and requires no new research.

---

## PP1 / PP2 / PP3 Items

### PP1 — Priority 1 (Blocking)

**PP1.1 — A.4 tool description inconsistency [A.4]**
The replication plan describes a Python pipeline that does not match the production `redist` Rust binary. An independent researcher following A.4's instructions would fail to replicate results. Resolution: rewrite the plan to describe the Rust CLI workflow (`redist build`, `redist label-analyze`, `redist label-verify`) as the primary replication entry point.

**PP1.2 — A.1 portfolio scope mismatch [A.1]**
A.1 describes an 11-paper portfolio. The actual program has 75 papers across Tracks B–H. A.1 requires a structural rewrite organized around tracks (A–H), not numbered papers. The reading-path table format is worth preserving; all paper content must be replaced or substantially updated.

**PP1.3 — A.0 ensemble context absent [A.0]**
A.0 makes no mention of Track G's finding that bisection plans sit at the 0.1–0.7th compactness percentile in the feasible-plan ensemble for WI/GA/PA/CA, and at the 50th percentile in NC (G.1). For a *Science* submission, reviewers with knowledge of the GerryChain/MGGG ensemble literature will treat this omission as a methodological gap. A paragraph in Section 4 contextualizing the compactness results against the ensemble distribution is required before resubmission.

**PP1.4 — "Cannot gerrymander" overstatement propagation [A.1, A.2; see also A.3 P2-B]**
A.0 now qualifies the "impossibility defense" as procedural (transparent, reproducible) rather than substantive (immune from challenge), and acknowledges the -3.2% algorithmic efficiency gap. A.1 and A.2 do not carry these qualifications. A.3 partially addresses this via the GeoSection counterexample but the "cannot gerrymander" language itself remains unqualified. All four documents must consistently convey: (a) the algorithm cannot be instructed to gerrymander; (b) compact neutral maps still reflect geographic voter concentration (C.5's -3.2% finding); (c) the defense is procedural, not a substantive shield.

### PP2 — Priority 2 (Strongly Recommended)

**PP2.1 — A.2 scope expansion to full program [A.2]**
Adding track-level summaries for C, E, F, G, H using the same paragraph format would bring the document to full-program scope with manageable effort.

**PP2.2 — A.5 Track F state legislative evidence [A.5 plan, before writing]**
State legislators primarily oversee state chamber redistricting. A sixth finding should be added: "Works for state legislative chambers — tested on all 50 state houses, from Delaware (41 districts) to New Hampshire (400 districts)."

**PP2.3 — A.5 C.8 competitive elections finding [A.5 plan, before writing]**
C.8 (30% more swing districts) should replace or supplement the temporal boundary stability finding. Competitive elections are more politically salient to the primary distribution targets.

**PP2.4 — A.3 P2-A and P2-B sentence fixes [A.3]**
Two P2 items deferred from A.3 Round 2 are higher urgency than classified:
- P2-A: Add edge-weight clarification sentence
- P2-B: Change "It cannot gerrymander" to "It cannot be instructed to gerrymander" and add reference to C.5's -3.2% geographic efficiency gap

**PP2.5 — A.0 Track B structure variant disclosure [A.0]**
A.0 presents recursive bisection as if it has one parameterization. A single paragraph in Section 3 (Method) covering the structure-mode design space — GeoSection, ApportionRegions (NC 7D/7R), ProportionalSection (sigma→0 paradox) — would suffice.

**PP2.6 — Compactness headline reconciliation [A.0, A.1, A.2, A.3]**
Four documents use inconsistent compactness improvement figures (+22%, +56%, +44%, +20%). A single canonical reference table — algorithm variant, baseline, improvement percentage, source paper — should be created and referenced consistently across all synthesis documents.

### PP3 — Priority 3 (Further Strengthening)

**PP3.1 — A.0 Track C user study and competitive elections [A.0]**
C.6 (public rates algorithmic maps as fairer) and C.8 (30% more swing districts) should appear in Section 5 (Implications).

**PP3.2 — A.0 Track H implementability context [A.0]**
H.2's 2500× speed advantage and H.0's PercentileSweep are directly relevant to the implementability arguments in Section 5.3.

**PP3.3 — A.4 execution [A.4]**: After resolving PP1.1, execute the plan (8–12 weeks).

**PP3.4 — A.5 execution [A.5]**: Write the main.tex incorporating PP2.2 and PP2.3 updates.

**PP3.5 — A.3 Track G scope qualifier and Track H addition [A.3]**: Update abstract from "seven tracks (A–G)" to "eight tracks (A–H)" and add Track H to track summaries.

**PP3.6 — A.5 "no catch" FAQ language [A.5 plan]**: Revise "No catch — only losers are politicians who benefit from gerrymandering" to neutral language: "No catch — the only parties who face greater competition are those currently benefiting from drawn district lines."

---

*Review conducted by 7-member module panel, 2026-05-07.*
