# Track H — Ensemble Search Strategies: Panel Review

**Track**: H-search
**Module**: track-H
**Papers reviewed**: H.0 (PercentileSweep), H.1 (BisectionEnsemble), H.2 (redist-ensemble), H.3 (resolution-aware)
**Panel date**: 2026-05-07
**Panel chair**: Track H Board, Apportionment Research Group

---

## Panel Composition

| Role | Expertise | Papers of Primary Concern |
|------|-----------|--------------------------|
| MCMC-A | Redistricting MCMC, ReCom, mixing | H.1, H.2, H.3 |
| MCMC-B | Chain diagnostics, convergence, SMC | H.2, H.3 |
| SYS-A | Rust performance, parallel computation | H.2 |
| SYS-B | HPC, ensemble scale, benchmark methodology | H.2, H.3 |
| POLSCI | Political science methodology, statutory use of algorithms | H.0, H.1 |
| STATS | Calibration, chain diagnostics, SMC | H.0, H.1, H.2 |
| PRACT | Real-world usability, litigation readiness | H.0, H.1, H.2 |

---

## Module Overview Assessment

Track H is the most applied track in the portfolio, translating Track G's ensemble science into
deployable statutory tools. All four papers are accepted at the journal level (avg 3.35/4), making
this the most review-mature applied track. The four papers cover orthogonal aspects of the search
problem: legal posture (H.0), structural integration with bisection (H.1), computational scale
(H.2), and data resolution (H.3).

The module arc is coherent: H.0 defines what it means to "choose" a plan from a family; H.1 embeds
that choice locally inside the bisection tree; H.2 provides the Rust engine that makes it practical;
H.3 adds a new dimension of choice (resolution). Together they implement the `--search` flag
ecosystem in the `redist` binary.

**Module score estimate**: 3.35/4 (accepted). Not yet at 3.75+ threshold for board-level
distinction. The gap is reproducibility (H.1, H.2) and the absence of peer review files for H.3.

---

## Paper-by-Paper Reviews

---

### H.0: PercentileSweep — Statutory Choice of Legal Posture

**Venue target**: Political Analysis
**Current score**: 3.4/4 (R2 conditional accept, avg of 5 reviewers)
**Round history**: R1 avg 2.75/4 → R2 avg 3.4/4 (+0.65). Duchin blocking concern (statute box bisection/ensemble conflation) resolved.
**Status**: In R2 revision; R3 required to clear 6 remaining P1 items and 3 P2 items.

#### Panel Assessment by Role

**POLSCI (Primary)**

The paper's core contribution — making the compactness percentile a visible statutory parameter
with a legal posture taxonomy — is sound and genuinely novel for the redistricting methodology
literature. The three-posture framework (compactness doctrine at p=0.0, representativeness
doctrine at p=0.5, full ensemble representativeness via TargetedSweep) provides useful statutory
vocabulary. The insensitivity finding (partisan outcomes largely invariant to p across the bisection
family) is the key empirical result and the paper's primary defense against the manipulation concern.

The manipulation concern deserves direct engagement: if partisan outcomes are insensitive to p
across the bisection family, then the percentile choice is a compactness decision, not a partisan
one. This argument is present in §5.1 but structured poorly — the section leads with the zero-sum
argument ("compactness is not a zero-sum quantity") before reaching the insensitivity result.
Duchin's D-P1-B (reorder §5.1: insensitivity before zero-sum argument) is correct and should be
treated as a structural fix, not cosmetic. The insensitivity argument is stronger; leading with it
forecloses the manipulation objection before the zero-sum argument is even needed.

The Karcher analogy (S-P1-A, Stephanopoulos) is the paper's most significant legal vulnerability.
Karcher v. Daggett is a population-equality case, not a compactness case. The paper's §5.1 and the
compactness posture statute box present "under Karcher's logic" as an established doctrinal
framework, not as an advocacy argument by structural parallel. Experienced opposing counsel will
exploit this mischaracterization. Stephanopoulos's S-P1-A is correct and blocking for Political
Analysis: the statute box language must qualify the Karcher invocation as "under the Karcher
framework by analogy" rather than implying direct doctrinal support.

**STATS (Primary)**

The central claim — partisan insensitivity across p — rests on four confirmed states (NC, WI, GA,
PA) and two extrapolated states (TX, CA) from B.11 single-run data. The extrapolation is clearly
labeled in R2 (abstract states "TX and CA show at most 0.5 seats variation (extrapolated from B.11
single-run data)"), which is a genuine improvement. However, the adversarial bar argument in §5.1
("the challenger must produce a plan more compact than 99.8% of all valid plans") depends on G.1
ensemble percentiles that are point estimates without confidence bounds. Duchin's D-P1-C is a
genuine intellectual integrity concern: if G.1's sample is not large enough to pin the 0.1–0.7th
percentile with precision, the adversarial bar collapses. The G.1 qualification must be added to
§5.1.

The T=601 sensitivity check (K-P1-C, Karypis) is appropriately deferred to H.2. The forward
reference (W16) must be added to §1.3 or §3.

**PRACT (Primary)**

The VRA interaction omission (D-P1-D / S-P1-C) is the most practically serious gap. For Georgia —
one of the paper's key states — any plan must satisfy VRA requirements for majority-minority
districts. A three-posture taxonomy that omits VRA-constrained compactness as a fourth posture is
incomplete for practitioners in covered jurisdictions. The §6.3 future-work mention of "VrASection"
is insufficient. At minimum, §5 needs a paragraph acknowledging that VRA compliance operates
independently of or in interaction with percentile choice, and that a fourth posture (VRA-constrained
at p=0.0) may be legally required in covered jurisdictions. The Callais cross-reference requested by
Duchin and Stephanopoulos belongs in this paragraph.

#### Key Open Items (H.0)

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| W3b | P1 | §5.1 paragraph reorder: insensitivity argument before zero-sum argument | Duchin |
| W7 | P1 | Geographic-sorting acknowledgment paragraph + Rodden (2019) citation | Rodden |
| W9 | P1 | G.1 ensemble-percentile point-estimate qualification added to §5.1 | Duchin |
| W10 | P1 | Structural-bias challenge route paragraph + B.0/B.17 cross-ref in §5 | Stephanopoulos |
| W11 | P1 | Qualify Karcher invocation as advocacy-by-parallel, not established precedent | Stephanopoulos |
| W13 | P1 | VRA interaction paragraph in §5 (GA specifically; Callais cross-ref) | Duchin, Stephanopoulos |
| W15 | P2 | Fix §1.3 body: still says "at most 0.5 seats" (inconsistent with abstract) | Karypis, Liang |
| W16 | P2 | Add forward reference to H.2 for T=601 sensitivity check | Karypis, Liang |
| W8 | P2 | Bootstrap/jackknife CI for p=1.0 partisan outcome (GA, NC) | Rodden (softened) |

#### PP1 Flags (H.0)

| Flag | Description |
|------|-------------|
| PP1-H0-KARCHER | Karcher analogy presented as established precedent rather than advocacy-by-parallel in §5.1 statute box. Blocking for Political Analysis; will be exploited by opposing experts. |
| PP1-H0-VRA | VRA posture absent from three-posture taxonomy. Blocking for any statutory use in covered jurisdictions (GA, TX). |

#### Panel Verdict (H.0)

**Accept with R3 required.** Six P1 items remain open, all prose-level. The paper cannot be
submitted to Political Analysis in current form. The Karcher analogy qualification (W11) and
structural-bias challenge route (W10) are the most consequential for the venue. The VRA interaction
(W13) blocks deployment as a statutory framework for any covered jurisdiction. The insensitivity
argument reorder (W3b) is a structural issue that weakens the paper's primary defense. R3 is a
prose revision; no new experiments required. Target score after R3: 3.7–3.8/4.

---

### H.1: BisectionEnsemble — Local ReCom at Each Bisection Node

**Venue target**: ICML
**Current score**: 3.0/4 (R2 conditional accept; 5 reviewers at 2.5–3.5/4)
**Round history**: R1 major revisions → R2 avg 3.0/4. All critical fixes applied or deferred with acknowledgment.
**Status**: R2 synthesis issued; short R3 needed to clear 5 remaining items before ICML submission.

#### Cross-Track Dependency Assessment (B.11 / PP1 Question)

H.1's relationship to B.11 (ApportionRegions) is correctly handled. The related-work section (§2) explicitly cites both B.1 and B.11, describes the prime-factorization-guided k-way splits, and correctly explains BisectionEnsemble's compatibility with ApportionRegions: it applies at 2-way nodes; p-way nodes with p>2 continue to use METIS. The introduction cites ApportionRegions and GeoSection in the opening paragraph.

**Finding**: H.1 does not reinvent B.11 without acknowledgment. No PP1 flag on the B.11 cross-track dependency.

#### Panel Assessment by Role

**MCMC-A (Primary)**

The algorithm is sound. Embedding 2-way ReCom at each bisection node cleanly avoids bipartition
failure for large prime k while preserving the bisection tree guarantees from B.1/B.11. The causal
account of GerryChain's TX failure was corrected in R2. Theorem 1 was substantially rewritten in R2
(three-part structure: existence, positive probability, finite expected steps).

Remaining precision issue: Theorem 1's statement says "O(|V_H|) steps in expectation" but the
proof establishes "O(log(1/delta)) resamples with probability >= 1-delta." This is a mismatch
between the statement's expected-steps form and the proof's probabilistic bound form. Either update
the theorem statement to match the probabilistic bound, or add a sentence converting the bound to
expected steps.

**STATS (Primary)**

All empirical results in Tables 2–4 are single-run observations. The R2 synthesis explicitly
acknowledges this as future work. The WI one-seat shift at p=0.5 (Table 3) may be within single-run
variance — this is a liability without multi-run confirmation.

**PRACT (Primary)**

The GA/PA factual error (R2 synthesis F-1) is a must-fix before any submission. Section 4.4 line
144 reads "Partisan outcomes are stable across percentile levels p for NC, GA, PA (zero seat
change)." GA and PA data are not in the paper. This is a factual assertion without supporting data.

The GerryChain configuration disclosure (F-3) is a litigation readiness requirement. Table 1's
acceptance rates are not reproducible without GerryChain version, tolerance epsilon, ReCom variant,
and random seed for the TX experiment.

#### Key Open Items (H.1)

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| F-1 | Must fix (P0) | GA/PA factual error in §4.4: stability claimed with no GA/PA data | Rodden, Liang |
| F-2 | Should fix (P1) | Theorem 1 statement/proof mismatch: expected steps vs. probabilistic bound | Karypis |
| F-3 | Should fix (P1) | GerryChain config for Table 1 unspecified (version, epsilon, variant, seed) | Duchin, Stephanopoulos, Liang |
| F-4 | Should fix (P1) | Hardware/software spec for Table 4 | Liang |
| F-5 | Minor (P2) | Formal definition of "bipartition failure" missing from §1 or §3 | Karypis, Duchin |
| Ack-1 | Required | Explicit statement in §4.1 that Tables 2–4 are single-run observations | Liang |
| Ack-2 | Required | Paragraph in §4.2 noting MultiSeedMETIS comparison is deferred to future work | Rodden, Liang |

#### PP1 Flags (H.1)

| Flag | Description |
|------|-------------|
| PP1-H1-FACTUAL | GA/PA cited as stability-confirmed states in §4.4 with no supporting data in the paper. Must fix before any submission. |

#### Panel Verdict (H.1)

**Accept with R3 required.** The GA/PA factual error (F-1) is a must-fix before any submission.
F-2 (Theorem 1 precision) and F-3 (GerryChain config) are should-fixes that will trigger ICML
reviewer rejection if absent. Total revision effort is low: three targeted prose fixes, two
acknowledgment paragraphs, one hardware spec sentence. The B.11 cross-track dependency is properly
handled; no PP1 reinvention flag required. Target score after R3: 3.3–3.5/4.

---

### H.2: redist-ensemble — High-Performance Rust ReCom

**Venue target**: USENIX ATC
**Current score**: 3.2/4 (R2 conditional accept, avg 3.2/4 across 5 reviewers)
**Round history**: R1 18 issues identified → R2 all 18 addressed (FIXED or DEFERRED with dagger notation). PDF compiled clean at 25 pages.
**Status**: Accepted at R2 level pending one missed fix (R04 planarity) and three Phase 2 benchmarks. Essentially ready for USENIX submission.

#### Panel Assessment by Role

**SYS-A (Primary)**

The Rust implementation architecture (spanning.rs / recom.rs / chain.rs) is well-described and
correct. The Wilson's algorithm complexity attribution is now accurate: Aldous's planar bound
(O(|V|log|V|)) is explicitly separated from Wilson's cover-time result. The SHA-256 seed encoding
is precisely specified. The serde output schema is versioned. The SmallRng PractRand correction is applied.

One R2 item was not resolved: R04 (planarity of census-tract subgraphs). Karypis's R2 review
explicitly notes that the planarity sentence was not added to §3.2 despite being in the revision
plan. This is a one-sentence addition required before USENIX submission.

**SYS-B (Primary)**

The 2,300–2,500x speedup claim is a theoretical estimate pending criterion.rs validation. The paper
correctly marks all throughput figures with dagger notation and commits to a falsifiable Phase 2
benchmark specifying hardware (Intel i7-11800H, 2.3 GHz base, 4.6 GHz boost, 24 MB L3, Windows 11),
GerryChain version (0.3.2, NumPy 1.24, Python 3.10), and an explicit acceptance criterion (>30,000
steps/sec for NC confirms the estimate; below 30,000 triggers revision of abstract and Table 1).
This is the correct scientific posture for a pre-validation implementation paper.

**MCMC-A (Primary)**

R07 (R-hat convergence semantics) was correctly addressed: the paper now states that R-hat
diagnostics on summary statistics certify convergence of marginal distributions of those statistics,
not convergence of the full plan-space distribution. The Autry et al. (2021) multiscale mixing
reference is cited in §6.1.

#### Key Open Items (H.2)

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| R04 | Should fix (one sentence) | Planarity sentence missing from §3.2: TIGER non-crossing polygons establish planarity of census-tract subgraphs | Karypis R2 |
| R02 | Phase 2 | Stationarity of pair reselection: compound-event gap in detailed-balance argument; formal proof deferred to Phase 2 | Karypis |
| R06 | Phase 2 | criterion.rs benchmarks required to confirm 50,000 steps/sec estimate | Liang |
| R13 | Phase 2 | CA vs. PA per-step cost ordering: deferred to Phase 2 empirical benchmarks | Duchin |

#### Panel Verdict (H.2)

**Accept with one minor fix.** R04 (planarity sentence in §3.2) is a one-sentence addition and
must be fixed before USENIX submission. The three Phase 2 dependencies are appropriately scoped
and do not block acceptance. The 2,300x speedup framing — theoretical estimate with falsifiable
benchmark commitment, sensitivity range table, and dagger notation throughout — is the correct
scientific posture. This is the strongest paper in the track on systems grounds.

---

### H.3: Resolution-Aware Redistricting

**Venue target**: GIS (journal)
**Current score**: 3.8/4 (per PAPERS.md and MODULE.md)
**Review status**: No reviews/ directory exists. No REVISION-PLAN.md. No _panel.yaml.
**Status**: Score asserted but unverifiable. Review trail must be generated before board review.

#### Panel Assessment by Role

**MCMC-A (Primary)**

The derive_partition function (GEOID prefix-matching to derive fine-to-coarse partitions) is
mathematically clean and the proof that the county adjacency criterion correctly propagates
geographic adjacency is accepted. The criterion is constructive, efficient (O(|E_T|) time), and
requires no additional data files for Option B (tract→county).

The key empirical claim — Option B reduces lag-100 Hamming autocorrelation by approximately 27%
on TX k=38 relative to single-scale ReCom — is estimated from a single 2,000-step run (seed s=42).
This is explicitly labeled with dagger notation and cross-referenced to a Phase 2 validation plan.
The Phase 2 multi-run validation is necessary to establish the autocorrelation claim.

**MCMC-B (Primary)**

The autocorrelation metric (lag-100 Hamming autocorrelation averaged over 8 parallel chains) is
appropriate and consistent with G.4. However, 2,000 steps is short for a 50,000-step benchmark
comparison: the reported autocorrelation may not represent the chain's steady-state behavior.

#### Missing Infrastructure (H.3)

H.3 is the only paper in Track H with no reviews/ directory and no REVISION-PLAN.md. The 3.8/4
score reported in MODULE.md and PAPERS.md cannot be verified against reviewer text. This is a
gap in the research record:

- No individual reviewer files exist (no r1_*.md, r2_*.md, r_synthesis.md)
- No _panel.yaml tracking paper review status
- No REVISION-PLAN.md documenting issue history and resolution

#### Key Open Items (H.3)

| ID | Severity | Description | Source |
|----|----------|-------------|--------|
| INFRA-1 | Track-level (must fix) | No reviews/ directory — 3.8/4 score is unverifiable | Panel |
| INFRA-2 | Track-level (must fix) | No _panel.yaml tracking review status | Panel |
| INFRA-3 | Track-level (must fix) | No REVISION-PLAN.md documenting issue history | Panel |
| EMP-1 | Paper-level | 27% autocorrelation reduction from single 2,000-step run (seed s=42) — Phase 2 validation required | MCMC-A |
| EMP-2 | Paper-level | 3–8% PP improvement (Option A) is projection by analogy, not empirical result | STATS |
| THEORY-1 | Paper-level | No formal MH correction at county level (Forest ReCom deferred); stationary distribution implications not stated | MCMC-B |

#### Panel Verdict (H.3)

**Provisionally accepted; review trail required before board submission.** The paper itself is
well-constructed: the GEOID derivation proof is clean, the county adjacency criterion is correct
and efficient, and the manifest system is a genuine practitioner contribution. The 3.8/4 score is
plausible. However, the absence of any review files means this score cannot be verified against
actual reviewer feedback.

---

## Cross-Track Dependencies

### H.1 → B.11 (ApportionRegions): CLEAR

H.1 explicitly cites B.11 in §2, describes the prime-factorization-guided k-way splits, and
correctly scopes BisectionEnsemble's coverage (2-way nodes only; p-way nodes with p>2 use METIS).
No PP1 flag required.

### H.0 → G.1 (ensemble baseline): OPEN

H.0's adversarial bar argument depends on G.1 ensemble percentiles that are point estimates without
confidence bounds. D-P1-C (Duchin) is an open P1 item: H.0 cannot be submitted to Political
Analysis without qualifying that the 0.1–0.7th percentile figure from G.1 is a point estimate
contingent on G.1's sampling adequacy.

### H.2 → G.1 (stationarity validation): DEFERRED

H.2's pair reselection stationarity conjecture is to be settled empirically by comparing
redist-ensemble and GerryChain ensemble distributions on the six G-track states. This is a Phase 2
dependency that is appropriately scoped and does not block USENIX submission.

---

## Track-Level Panel Findings

### Track Strengths

1. **Coherent arc**: H.0 through H.3 form a clean research program. Each paper adds one orthogonal dimension to the search problem (posture, structure, speed, resolution) without overlap.
2. **Litigation readiness**: H.0, H.1, and H.2 each contain explicit statutory disclosure frameworks, pre-commitment protocols, and audit log requirements.
3. **Implementation depth**: H.2 is the most implementation-detailed paper in the portfolio. The falsifiable Phase 2 benchmark commitment is the correct scientific posture.
4. **B.11 cross-track handled correctly**: H.1 builds on B.11 without reinventing it. No PP1 flag.

### Track Weaknesses

1. **Reproducibility gap across H.0–H.2**: All three reviewed papers have single-run or pre-validation empirical claims.
2. **H.3 review absence**: The highest-scoring paper in the track (3.8/4) has no review files. This is a track-level hygiene failure that must be remedied before board submission of Track H as a module.
3. **VRA integration absent across H.0 and H.1**: Both papers omit VRA as a legal posture dimension.
4. **TX/CA empirical coverage thin**: The two states with the most complex redistricting problems have the weakest empirical coverage in H.0 and H.1.

### Module PP1 Flags

| Flag | Paper | Description | Priority |
|------|-------|-------------|----------|
| PP1-H0-KARCHER | H.0 | Karcher analogy in §5.1 statute box presented as established precedent rather than advocacy-by-parallel | P1 — blocking for Political Analysis |
| PP1-H0-VRA | H.0 | VRA posture absent from three-posture taxonomy; GA and TX are covered jurisdictions | P1 — blocking for statutory use |
| PP1-H1-FACTUAL | H.1 | GA/PA stability claimed in §4.4 with no supporting data in the paper | P0 — must fix before any submission |
| PP1-H3-NOREVIEWS | H.3 | 3.8/4 score with no review files — score cannot be verified against reviewer text | Track-level — must fix before board |

---

## Track Score Summary

| Paper | Venue | Current Score | Round | Path to Board |
|-------|-------|--------------|-------|---------------|
| H.0 | Political Analysis | 3.4/4 | R2 → R3 required | R3 clears 6 P1 items (all prose); target 3.7–3.8/4 |
| H.1 | ICML | 3.0/4 | R2 → R3 required | R3 fixes GA/PA factual error + 4 items; target 3.3–3.5/4 |
| H.2 | USENIX ATC | 3.2/4 | Accepted | Add R04 planarity sentence; then submit |
| H.3 | GIS | 3.8/4 (unverified) | Unknown | Generate review round; verify score; then submit |
| **Track avg** | | **3.35/4** | | |

*Panel convened 2026-05-07. Track H — four papers across ensemble search strategies.*
