# Track H — Ensemble Search Strategies: Panel Revision Plan

**Track**: H-search
**Module**: track-H
**Generated**: 2026-05-07
**Based on**: REVIEW_PANEL.md, individual paper REVISION-PLAN.md files, R2 review files

---

## Executive Summary

Track H has four papers at 3.0–3.8/4, all accepted at the journal level. The track is coherent and
the B.11 cross-track dependency is correctly handled (no PP1 reinvention flag). The revision work
divides into three categories:

- **H.2**: One sentence to add (R04 planarity). Ready for USENIX submission after that fix.
- **H.0 and H.1**: Short prose-only R3 revisions. H.0 has 6 P1 items (all legal/structural prose); H.1 has 1 must-fix factual error plus 4 should-fix items. No new experiments required for either.
- **H.3**: Infrastructure gap — review round must be generated before any board submission.

Total effort to bring the track to board-ready: approximately 2–3 focused writing sessions for H.0 R3, one short revision pass for H.1, one sentence for H.2, and one review generation session for H.3.

---

## H.2: redist-ensemble — Action Required Before Submission

**Priority**: Immediate (one sentence; unblocks USENIX submission)

### R2-H2-01: Add planarity sentence to §3.2

**Status**: NOT DONE despite being in revision plan (Karypis R2 review confirmed absent)
**Required text** (add after Wilson's theorem in §3.2):

> In practice, the subgraph $H = G[V_i \cup V_j]$ is a planar graph: it is induced from the
> census-tract adjacency graph, which is derived from TIGER shapefiles defining non-crossing
> polygonal boundaries. Small non-planar artifacts (tri-point boundaries, state-line adjacencies)
> affect at most a small fraction of steps and do not alter the asymptotic cover-time bound.

**Effort**: 2 minutes.
**Blocking for**: USENIX submission (Karypis will re-raise if absent in final submission).

After this fix, H.2 is ready for USENIX ATC submission. No other changes required.

---

## H.1: BisectionEnsemble — R3 Revision

**Priority**: High (F-1 is must-fix; F-2 through F-5 are should-fix for ICML)
**Estimated effort**: 1–2 hours

### R3-H1-F1: Fix GA/PA factual error in §4.4 (MUST FIX — P0)

**Location**: §4.4, line 144
**Current text**: "Partisan outcomes are stable across percentile levels p for NC, GA, PA (zero seat change)."
**Problem**: GA and PA data do not appear in Table 3 or anywhere in the paper.
**Fix**: Remove "GA, PA" and restrict the stability claim to NC only:

> "Partisan outcomes are stable across percentile levels p for NC (zero seat change across all five p values in Table 3). Extension to GA and PA is deferred to Phase 2."

**Effort**: 5 minutes.
**Blocking for**: All venue submissions and any litigation-facing use.

### R3-H1-F2: Align Theorem 1 statement with proof (should fix)

**Location**: §3.4
**Problem**: Theorem 1 states "O(|V_H|) steps in expectation" but the proof establishes "O(log(1/delta)) resamples with probability >= 1-delta."
**Fix option A** (simpler): Update the theorem statement to the probabilistic form:

> Theorem 1 (Tractability). For any connected subgraph H and balance tolerance epsilon > 0, the BisectionEnsemble chain accepts a balanced bisection within O(log(1/delta)) ReCom steps with probability at least 1 - delta.

**Effort**: 20 minutes.
**Blocking for**: ICML acceptance.

### R3-H1-F3: Add GerryChain configuration footnote for Table 1 (should fix)

**Location**: §4 (Table 1 footnote or §4.1 setup paragraph)
**Required content**:
- GerryChain version (pip-installed tag or commit hash)
- Population balance tolerance epsilon
- ReCom variant (reversible or non-reversible)
- Initial plan: cold start from seed-0 random valid plan
- Number of runs
- Random seed for the TX acceptance rate measurement

**Effort**: 15 minutes.
**Blocking for**: ICML acceptance and litigation use of Table 1.

### R3-H1-F4: Add hardware/software specification to §4.1 (should fix)

**Location**: §4.1 (one sentence)
**Required content**: CPU model, clock speed, Rust compiler version (rustc --version), METIS version.
**Effort**: 5 minutes.

### R3-H1-F5: Add formal definition of "bipartition failure" (minor)

**Location**: §1 or §3
**Required text**:

> We say a ReCom step fails if no balanced cut edge exists in the sampled spanning tree; a chain stalls if the acceptance rate falls below a threshold alpha (we use alpha = 0.01 in experiments).

**Effort**: 5 minutes.

### R3-H1-Ack1: Disclose single-run status of Tables 2–4 (required)

**Location**: §4.1 setup paragraph
**Required text**:

> All results in Tables 2–4 are single-run observations from one BisectionEnsemble run per (state, p) combination. Distribution of results across independent runs is deferred to Phase 2.

**Effort**: 3 minutes.

### R3-H1-Ack2: MultiSeedMETIS comparison deferred paragraph (required)

**Location**: §4.2
**Required text** (add after Table 3):

> A natural comparison for BisectionEnsemble is MultiSeedMETIS(N=100): running METIS 100 times with different seeds at each node and selecting rank floor(p × 100). This comparison is deferred to Phase 2; the absence of this baseline means the current results do not establish that the ReCom chain provides benefit beyond seed diversity alone.

**Effort**: 10 minutes.

### R3-H1 Revision Checklist

- [ ] F-1: Remove GA/PA from §4.4 stability claim (P0 — must fix)
- [ ] F-2: Align Theorem 1 statement with probabilistic proof bound (P1)
- [ ] F-3: Add GerryChain config footnote for Table 1 (P1)
- [ ] F-4: Add hardware/software spec to §4.1 (P1)
- [ ] F-5: Add formal bipartition failure definition to §1 or §3 (P2)
- [ ] Ack-1: Add single-run disclosure sentence to §4.1 (required)
- [ ] Ack-2: Add MultiSeedMETIS deferred paragraph to §4.2 (required)

**Target score after R3**: 3.3–3.5/4.

---

## H.0: PercentileSweep — R3 Revision

**Priority**: High (6 P1 items, all prose; no new experiments)
**Estimated effort**: 3–4 hours

### R3-H0-W11: Qualify Karcher invocation in §5.1 and statute box (P1 — PP1 flag)

**Location**: §5.1 prose and Compactness Posture statute box
**Fix**: In the statute box and §5.1 prose, change all "under Karcher's logic" or "under Karcher" to "under the Karcher framework by analogy." Add one sentence in §5.1:

> The Karcher analogy is a structural parallel — the paper argues that the same logic that requires justification for population deviations should require justification for compactness deviations — not an established federal holding. This is an advocacy position for practitioners to assert, not settled doctrine.

**Effort**: 30 minutes.
**Blocking for**: Political Analysis submission.

### R3-H0-W10: Add structural-bias challenge route paragraph to §5 (P1)

**Location**: §5 (new paragraph after §5.1 recommendation)
**Required content**:

> A separate challenge route argues that structural bias — if the bisection tree structure systematically favors one party across all seeds — makes percentile selection irrelevant: every percentile produces the same partisan outcome because the structure determines it. This track's response to structural-bias challenges is in B.0 and B.17: geographic determinism, not algorithmic bias, explains the observed partisan outcomes. See B.17 for the seat-variance decomposition attributing >90% of variance to structure vs. <10% to seed selection.

Add cross-references to B.0 and B.17 in §5.
**Effort**: 30 minutes.

### R3-H0-W13: Add VRA interaction paragraph to §5 (P1 — PP1 flag)

**Location**: §5 (new §5.5 or appended to §5.4)
**Required content**:

> The three postures in this section address compactness, population balance, and contiguity only. In states subject to VRA Section 2 requirements (notably GA, TX, NC, and others with majority-minority district obligations), a fourth posture applies: VRA-constrained compactness, where p=0.0 is applied subject to the constraint that majority-minority districts are preserved. See Louisiana v. Callais for a recent example of compactness and VRA interacting in congressional redistricting litigation. Practitioners in covered jurisdictions should treat the posture taxonomy here as a starting point, not a complete framework.

**Effort**: 30 minutes.

### R3-H0-W3b: Reorder §5.1 paragraphs (P1)

**Location**: §5.1
**Fix**: Reorder so that the insensitivity finding appears before the zero-sum argument. The rhetorical order should be: (1) insensitivity — the choice of p has no partisan consequence; (2) zero-sum — compactness improvement comes at no cost to any other criterion; (3) adversarial bar — under the Karcher framework by analogy, challengers face a high bar.
**Effort**: 20 minutes.

### R3-H0-W7: Add geographic-sorting paragraph + Rodden (2019) citation (P1)

**Location**: §5.1 or §4
**Required content**: Add a paragraph noting that the partisan insensitivity finding is consistent with the geographic-sorting literature (Rodden 2019): because Democratic voters are geographically concentrated, compact districts tend to produce similar partisan outcomes regardless of compactness level. Cite Rodden (2019) "Why Cities Lose."
**Effort**: 20 minutes.

### R3-H0-W9: Add G.1 point-estimate qualification to §5.1 (P1)

**Location**: §5.1, adversarial bar paragraph
**Fix**: After the adversarial bar statement, add:

> This bar is computed from the G.1 GerryChain ensemble. The percentile figures (0.1–0.7th percentile for most states) are point estimates from a single ensemble run; their precision depends on G.1's sample size and mixing time. A more conservative statement is that the p=0.0 plan is more compact than approximately 99%+ of G.1 ensemble plans, with the exact percentile subject to sampling uncertainty.

**Effort**: 15 minutes.

### R3-H0 Revision Checklist

- [ ] W11: Qualify Karcher invocation as advocacy-by-parallel in §5.1 and statute box (P1 — PP1)
- [ ] W10: Add structural-bias challenge route paragraph to §5 with B.0/B.17 cross-refs (P1)
- [ ] W13: Add VRA interaction paragraph (§5.5) with Callais cross-ref (P1 — PP1)
- [ ] W3b: Reorder §5.1: insensitivity → zero-sum → adversarial bar (P1)
- [ ] W7: Add geographic-sorting paragraph + Rodden (2019) citation (P1)
- [ ] W9: Add G.1 point-estimate qualification to §5.1 adversarial bar paragraph (P1)
- [ ] W15: Fix §1.3 body to match abstract four-state/two-state language (P2)
- [ ] W16: Add forward reference to H.2 for T=601 sensitivity check (P2)

**Target score after R3**: 3.7–3.8/4.

---

## H.3: Resolution-Aware — Review Round Generation Required

**Priority**: Track-level infrastructure (must fix before board submission of Track H)
**Estimated effort**: Full review generation session (one working day)

### Infrastructure Fixes (Required Before Board Submission)

#### INFRA-H3-01: Create reviews/ directory and generate review files

H.3 has no `reviews/` directory. Before board submission of Track H as a module, the following must be created:

```
research/H.3+resolution-aware/reviews/
  r1_karypis.md    — graph partitioning, performance
  r1_rodden.md     — political geography
  r1_duchin.md     — ReCom/metric geometry
  r1_stephanopoulos.md — election law
  r1_liang.md      — reproducibility / GIS systems
```

The review round should be generated using the panel's H.3 assessment in REVIEW_PANEL.md as the basis for the reviewer perspectives.

#### INFRA-H3-02: Create _panel.yaml

Create `research/H.3+resolution-aware/_panel.yaml` with at minimum:

```yaml
paper: H.3+resolution-aware
title: "Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter"
venue: GIS
status: under-review
score: 3.8
score_basis: panel-assessed  # distinguish from reviewer-confirmed
round: 1-pending
reviewers: [karypis, rodden, duchin, stephanopoulos, liang]
```

#### INFRA-H3-03: Create REVISION-PLAN.md after R1 reviews are generated

Once r1_*.md files are generated, create `research/H.3+resolution-aware/REVISION-PLAN.md` following the format of H.0 and H.2 REVISION-PLAN.md files.

### Paper-Level Items to Address in H.3 R1/R2

| ID | Description | Suggested reviewer | Section |
|----|-------------|-------------------|---------|
| EMP-1 | 27% autocorrelation reduction is a single 2,000-step estimate; multi-run validation required | MCMC-A | §5.1 |
| EMP-2 | 3–8% PP improvement (Option A) is a projection, not a result; must be clearly labeled | STATS | §5.3 |
| THEORY-1 | County-level MH approximation: paper should state explicitly this is the same class of approximation as standard ReCom, not a novel departure | MCMC-B | §3.3 |
| LEGAL-1 | Whether resolution choice (tract vs. BG vs. county) interacts with VRA majority-minority district boundary precision requirements | Stephanopoulos | §5b |

---

## Submission Sequencing

| Step | Action | Paper | Blocking dependency |
|------|--------|-------|---------------------|
| 1 | Add R04 planarity sentence to §3.2 | H.2 | None — do immediately |
| 2 | Submit H.2 to USENIX ATC | H.2 | Step 1 |
| 3 | Complete R3 revision (6 P1 items) | H.0 | None — prose only |
| 4 | Complete R3 revision (F-1 through F-5) | H.1 | None — prose only |
| 5 | Generate H.3 review round (r1_*.md files) | H.3 | None — can run in parallel with 3/4 |
| 6 | Submit H.0 to Political Analysis | H.0 | Step 3 |
| 7 | Submit H.1 to ICML | H.1 | Step 4 |
| 8 | Complete H.3 R1 cycle (REVISION-PLAN.md) | H.3 | Step 5 |
| 9 | Submit H.3 to GIS | H.3 | Step 8 |
| 10 | Board module submission for Track H | All | Steps 2, 6, 7, 9 |

Steps 1–2 can be done today. Steps 3, 4, and 5 can run in parallel.

---

## Phase 2 Dependencies Across Track H

| ID | Paper | Description |
|----|-------|-------------|
| PH2-H0 | H.0 | Actual PercentileSweep runs on TX and CA (currently B.11 extrapolations) |
| PH2-H0 | H.0 | T=601 sensitivity check for NC and GA |
| PH2-H1 | H.1 | Multi-run statistics for Tables 2–4 (10 independent runs per state/p) |
| PH2-H1 | H.1 | MultiSeedMETIS(N=100) baseline comparison |
| PH2-H2 | H.2 | criterion.rs benchmarks to confirm 50,000 steps/sec estimate |
| PH2-H2 | H.2 | Stationarity of pair reselection: G-track empirical comparison |
| PH2-H3 | H.3 | Multi-run autocorrelation validation for TX Option B (27% claim) |
| PH2-H3 | H.3 | BG-level (Option A) Polsby-Popper improvement (3–8% projection) |
| PH2-H3 | H.3 | Forest ReCom exact MH correction at county level |
