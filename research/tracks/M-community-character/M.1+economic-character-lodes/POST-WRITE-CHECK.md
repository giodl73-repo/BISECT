# POST-WRITE CHECK: M.1 Economic Character Edge Weights (LODES)

**Date**: 2026-05-08
**Pipeline**: research-post-write v2
**Spec**: docs/specs/2026-05-08-m1-economic-character-lodes.md

---

## PHASE 1 — PAPER SUMMARY

```
Paper: M.1 — Economic Character Edge Weights via LODES Workplace Area Characteristics
Sections found: 00-abstract, 01-introduction, 02-background, 03-framework,
                04-results, 05-legal, 06-conclusion
Spec found: YES — docs/specs/2026-05-08-m1-economic-character-lodes.md
Series: M.1
Target audience: Practitioner / Legal

Key claims:
  1. M.1 produces a 14.3 pp proportionality gap improvement in NC-14 (from −20.7 pp
     to −6.4 pp) as RTP and Charlotte commercial corridor emerge as natural cut seams.
     [Supported by Table 1, §4.2, §4.3]
  2. The M-track primary metric (mean within-district std of jobs_per_resident) is
     the right measure of economic community coherence and will be computed in Phase 2.
     [Defined in §3.4, acknowledged deferred in §4.4]
  3. Economic character weights are legally defensible as a communities-of-interest
     criterion under CA, CO, AZ, and WA law. [§5]
```

---

## PHASE 2 — CONSISTENCY CHECK

### Quantitative Value Registry

| Q-ID | Quantity | Abstract | §Framework | Table 1 | §Results | §Conclusion | Consistent? |
|------|----------|---------|-----------|---------|---------|-------------|-------------|
| Q-01 | NC geographic gap | −20.7 pp | — | −20.7 pp | −20.7 pp | implicit | PASS |
| Q-02 | NC economic-char gap | −6.4 pp | — | −6.4 pp | −6.4 pp | implicit | PASS |
| Q-03 | NC delta | +14.3 pp | — | +14.3 pp | +14.3 pp | +14.3 pp | PASS |
| Q-04 | WI delta | "no effect" | — | 0.0 pp | 0 pp | "no effect" | PASS |
| Q-05 | TX delta | "2.6 pp worsening" | — | −2.6 pp | −2.6 pp | "slight reversal" | PASS |
| Q-06 | alpha default | implied 0.5 | eq.(2) says α=0.5 | — | α=0.5 stated in §4.1 | — | PASS |
| Q-07 | JPR cap | — | §2 background | — | referenced | — | PASS |

### Dagger Notation Check

- NC: No dagger. **CORRECT** — confirmed from pipeline run per data JSON.
- WI: Dagger applied in Table 1. Table note present. §4.3 prose: "consistent with
  the expected single-seed variance range." **PASS**.
- TX: Dagger applied in Table 1. Table note present. §4.3 prose: "It should be
  confirmed with multi-seed runs." **PASS**.

### Formula Consistency

- Cosine similarity: §3.3 (Zero-Vector Handling) states three cases correctly:
  both-zero → 1.0; one-zero → 0.5; neither-zero → standard cosine. **PASS for the cases**.
- Blend formula: eq.(2) in §3.2 states `w_ec = w_geo × (α + (1−α) × sim)` with
  default α=0.5. **PASS**.
- M-track framework formula: §3.1 states M.0 generic formula as
  `w(u,v) = w_base(u,v) × sim(c_u, c_v)`. The M.9 blend (eq.2) differs from
  this pure-multiplicative form, and §3.2 correctly acknowledges this difference
  and explains the design rationale. **PASS — well-handled**.

### CRITICAL FAILURE: Similarity Matrix Values Are Mathematically Incorrect

**P1 BLOCKER**: Table 2 (similarity matrix, §3.2) shows:
- Bedroom-Commercial: 0.11
- Bedroom-Industrial: 0.04
- Commercial-Industrial: 0.28

These values are **not reproducible** from the stated prototype vectors in Table 1:
- Bedroom suburb: CI=0.02, IF=0.01, JPR=0.3
- Commercial corridor: CI=0.45, IF=0.05, JPR=2.1
- Industrial park: CI=0.08, IF=0.62, JPR=4.8

**Computed actual similarities using the paper's own formula (eq.1 + eq.2 of §3)**:
- Bedroom-Commercial: **0.990** (paper claims 0.11)
- Bedroom-Industrial: **0.994** (paper claims 0.04)
- Commercial-Industrial: **0.976** (paper claims 0.28)

**Root cause**: JPR is on scale [0, 10] while CI and IF are on scale [0, 1].
In the L2 norm computation, JPR dominates completely:
- dot(bed, com) = 0.02×0.45 + 0.01×0.05 + **0.3×2.1** = 0.0095 + **0.63** = 0.640
- ||bed|| ≈ 0.300 (JPR term), ||com|| ≈ 2.101 (JPR term)
- cosine ≈ 0.640 / (0.300 × 2.101) = **0.990**

The CI and IF signals are overwhelmed by JPR because they are 10× smaller in scale.
The similarity matrix values in the paper appear to have been computed either:
(a) without the JPR component (using only CI and IF), or
(b) with a different normalization of the character vector that is not stated in the paper.

**Consequence for the algorithm itself**: If the implementation computes cosine
similarity on the raw [CI, IF, JPR] vector with JPR in [0,10], then:
- ALL tract pairs where both have nonzero JPR will have similarity ≈ 0.99–1.00
- The economic character modifier will have essentially NO effect on inter-tract weights
  (all edges get multiplied by ≈ 1.0 regardless of economic type)
- The claimed NC improvement would NOT be caused by the cosine similarity mechanism
  described in the paper

Either:
(a) The implementation normalizes JPR (divides by 10) before cosine computation,
    which is not stated in the paper or spec, or
(b) The similarity matrix values are illustrative but incorrect, and the actual
    algorithm as implemented produces near-uniform similarities and achieves the
    NC result through a different mechanism, or
(c) There is a bug in the implementation that happens to produce the NC result for
    a different reason than claimed.

**This is a P1 blocker**. The similarity matrix must either:
1. Be recomputed using the actual implementation behavior (with or without JPR scaling),
   OR
2. The paper must add a normalization step (e.g., divide JPR by 10 before cosine
   computation, making all components comparable in magnitude), AND the spec and
   implementation must be updated to match.

### M.1 Spec Zero-Jobs Handling Discrepancy

**P1 BLOCKER**: The M.1 spec (test invariant `zero_jobs_no_nan`) states:
> "tract with C000=0 → char vector treated as zero vector, similarity = **0.0** (not NaN)"

The paper (§3.3, §2 background) consistently states:
> "two zero-vector tracts → similarity = **1.0** (both residential, keep together)"

The spec says 0.0; the paper says 1.0. These are contradictory and represent opposite
design choices (0.0 would mean "pure residential tracts have NO mutual affinity";
1.0 means "pure residential tracts are maximally similar"). The paper's choice (1.0)
is the correct community-structure choice. The spec test invariant is wrong and must
be corrected. If the implementation follows the spec's 0.0, then it has a bug.

```
CONSISTENCY: FAIL — 2 P1 blockers
P1 (reject):
  [C-01] Similarity matrix (Table 2) values are incorrect due to JPR scale dominance.
         Bedroom-Commercial should be ~0.99, not 0.11. The paper's core illustrative
         table is mathematically inconsistent with the stated formula and vectors.
  [C-02] M.1 spec zero-vector similarity = 0.0 contradicts paper's zero-vector
         similarity = 1.0. One is wrong; must be reconciled before panel review.
P2 (revision): none beyond P1
P3 (minor): none beyond P1
```

---

## PHASE 3 — CONTRACT CHECK

Spec: docs/specs/2026-05-08-m1-economic-character-lodes.md

| Promise (from spec) | Paper section | Delivered? | Gap |
|---------------------|---------------|-----------|-----|
| ≥15% std(JPR) reduction | §4.4, §3.4 | DEFERRED — explicitly acknowledged as Phase 2 | Deferred ok per instructions. |
| 3×3 similarity matrix present | §3.2 Table 2 | YES — present but **values are wrong** | P1 (see C-01) |
| Within-district JPR variance defined | §3.4 eq.(3) | YES — formally defined as mean within-district std | PASS |
| NC proportionality gap shifts toward ±4 pp | §4.2 | YES — exceeds target (−6.4 pp achieved vs −6.5 pp baseline) | PASS (result exceeds spec) |
| RTP cluster preserved | §4.2 mechanistic discussion | YES — described but not quantified with ≥80% co-assignment metric | P3 — spec says "≥80% of high-JPR tracts co-assigned"; paper discusses mechanism but not metric |
| Legal analysis present | §5 | YES | PASS |
| M-track framework fit | §3.1 | YES — explicit plug-in formulation | PASS |

```
CONTRACT: PARTIAL
Promises kept: 5/7 (2 issues: similarity matrix values wrong P1, RTP 80% metric not reported P3)
Gaps:
  [G-01] P1: Similarity matrix present but values mathematically incorrect.
  [G-02] P3: Spec required "≥80% of high-JPR tracts co-assigned in ≥1 district";
             paper discusses mechanism qualitatively but does not report the metric.
```

---

## PHASE 4 — REFEREE SIMULATION

### REFEREE 1 — Algorithms Reviewer (SODA/FOCS archetype)
**Recommendation: Major Revision**

SUMMARY: The paper makes a clear conceptual contribution as the first M-track
implementation paper and the legal/practitioner framing is well done. However,
the central mathematical table (similarity matrix, Table 2) is numerically
incorrect, which undermines confidence in the algorithm description. The zero-
vector handling is contradicted between the spec and the paper. These must be
resolved before the paper can be accepted.

MAJOR CONCERNS:
[I-01] **Critical**: Table 2 similarity matrix is wrong. Bedroom-Commercial is
listed as 0.11 but computes to 0.990 with the stated vectors and formula.
The cause is JPR scale dominance: JPR in [0,10] vs CI/IF in [0,1] means the
dot product is almost entirely determined by JPR×JPR. This appears to be a
normalization bug: either the character vector should be normalized (JPR/10 to
bring it to [0,1]) before cosine computation, or the paper should use normalized
vectors in Table 1 and recompute Table 2. As written, the paper's "illustrative"
similarity matrix is not illustrative of what the algorithm actually computes.

[I-02] The paper claims the cosine similarity "produces meaningful separation
between the four most common tract economic types." With the stated formula and
raw JPR values, ALL nonzero-JPR tract pairs have similarity ~0.97–0.99 (dominated
by JPR correlation). There is no meaningful separation. Either the algorithm does
not work as described, or the description is incomplete (missing normalization step).

[I-03] Zero-vector handling: paper (§3.3) says both-zero → sim=1.0; spec test
invariant says similarity=0.0. One is wrong. The paper's choice (1.0) is correct
for community clustering but must be implemented correctly.

MINOR CONCERNS:
[I-04] §3.1 writes the M.0 generic formula as `w = w_base × sim`. Section 3.2's
blend formula modifies this to `w = w_base × (α + (1-α)×sim)`. The paper
acknowledges this deviation but should provide clearer justification for why M.1
departures from M.0 are acceptable without updating M.0.

[I-05] The paper does not state which version of the bisect binary produced the
empirical results. M.9 mentions "post-cccd853 debug build" — M.1 should include
the same provenance.

---

### REFEREE 2 — Political Science Reviewer (APSR/JOP archetype)
**Recommendation: Minor Revision**

SUMMARY: The empirical results are consistent with M.9 (same three states, same
values). Dagger notation is correctly applied. The state-specificity finding is
well-argued. The Arizona example in §1 (Maricopa County hearings) is effective
motivation. Key political science concern is that the mechanism story (why does
cosine similarity produce the NC result?) is undermined by the mathematical issue
in the similarity matrix — if all similarities are ≈0.99, the mechanism cannot
work as described.

MAJOR CONCERNS:
[I-06] The NC improvement mechanism relies on the claim that RTP tracts have
"very low cosine similarity" with adjacent residential tracts (§4.2: "the JPR
contrast produces very low cosine similarity between these adjacent tracts").
But with raw JPR values in [0,10], the cosine similarity between RTP (JPR≈7)
and suburbs (JPR≈0.3) is: dot = 0.3×7 ≈ 2.1 / (0.3 × 7.0) = 1.0. There is
NO low similarity for adjacent RTP–suburb pairs. The mechanism as stated in
§4.2 is mathematically inconsistent with the formula in §3. If the NC improvement
is real (confirmed from data), the paper must correctly explain what actually
causes it.

MINOR CONCERNS:
[I-07] The abstract opens with a strong claim that the method produces NC improvement.
If this claim depends on a scaling step not described in the paper, the abstract
is overclaiming. The abstract should be revised once the normalization question is
resolved.

[I-08] §4.3 (WI-8 null result) and §4.4 (TX-38 slight worsening) correctly apply
daggers. This is appropriately cautious. The paper should also briefly state what
the multi-seed expectation is: does the reviewer expect the WI null to remain null
with multiple seeds? (Almost certainly yes, given the mechanism argument, but it
should be stated.)

---

### REFEREE 3 — Legal/Practitioner Reviewer (Law Review / Public Administration archetype)
**Recommendation: Accept (conditional on algorithm fix)**

SUMMARY: The legal analysis in §5 is excellent. All four state citations are
real statutes correctly characterized. The expert witness deployment template in
§5.3 is immediately usable and well-structured. The "When Not to Use" checklist
is the right practitioner framing. The legal sections do not depend on the
mathematical issue in the similarity matrix — they depend on the existence of
a defensible, partisan-neutral algorithm, which the paper provides. Once the
algorithm description is corrected, the legal analysis is sound.

MAJOR CONCERNS: None beyond the mathematical issue already flagged.

MINOR CONCERNS:
[I-09] §5.3 expert witness element 4 includes the blend formula with notation
`w_geo` — same issue as M.9: a non-technical judge will need a parenthetical
defining w_geo as "the geographic boundary-length weight."

[I-10] The CA Proposition 11 discussion (§5.1) says the Commission "accepted
economic community characterizations from public testimony" in 2011 and 2021.
This is accurate but imprecise: the Commission accepted communities of interest
testimony broadly; the specific LODES WAC characterization has not been tested
in proceedings. The paper should add "comparable economic community testimony"
rather than implying LODES-based characterization has Commission precedent.

[I-11] WA RCW 44.05.090 (year 1983 in references.bib) should note "as amended"
since the statute has been revised through 2011 redistricting cycle.

---

## PHASE 5 — ABSTRACT CHECK

```
ABSTRACT: ~175 words
Primary result stated: YES (NC +14.3 pp, WI no effect, TX 2.6 pp worsening)
Algorithm named: YES (economic character vector, cosine similarity, edge weight modulation)
Value proposition: YES (legally defensible, no racial/partisan content, LEHD data)
M-track framing: YES (M-track primary metric defined and deferred to Phase 2)
```

Abstract is well-formed. All three empirical results are stated. The legal
defensibility value proposition is explicit. The Phase 2 caveat for within-district
variance is appropriately flagged. However, the abstract claims the algorithm
produces low similarity between employment-center tracts and residential tracts —
this is the mechanistic claim that is undermined by the scale issue (see C-01).
If the scale issue is fixed, the abstract is accurate; if not, line 22-23
("the JPR contrast...produces very low cosine similarity") must be revised.

---

## PHASE 6 — PRE-PANEL CHECKLIST

```
=================================================================
POST-WRITE COMPLETE: M.1 Economic Character Edge Weights (LODES)
=================================================================

Validation results:
  Consistency:   FAIL — 2 P1 blockers
  Contract:      PARTIAL — 1 P1 gap, 1 P3 gap
  Referee sim:   Major Revision (R1), Minor Revision (R2), Accept-conditional (R3)
  Abstract:      ~175 words, well-formed (conditional on fix)

!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
P1 BLOCKERS — MUST FIX BEFORE PANEL REVIEW
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

[C-01 / I-01 / I-06] SIMILARITY MATRIX WRONG DUE TO JPR SCALE DOMINANCE
  Location: §3.2 Table 2 (similarity matrix) + §4.2 mechanism story
  Problem: Table 2 claims bed-com=0.11, bed-ind=0.04, com-ind=0.28.
           Computed values from stated prototype vectors: 0.99, 0.99, 0.98.
           JPR (scale 0-10) overwhelms CI and IF (scale 0-1) in the dot product.
  Fix options:
    Option A (recommended): Add a normalization step to the character vector
      before cosine computation: normalize each component to [0,1] by dividing
      JPR by 10 (or by max_observed_JPR). Then recompute prototype similarities.
      Update §3 Algorithm, spec, and implementation. Recompute Table 2.
    Option B: Remove Table 2 entirely and replace with a qualitative description
      noting that the similarity is dominated by JPR when either tract has
      nonzero JPR, and describe the zero-vector (pure residential) behavior instead.
    Option C: Verify that the actual implementation uses a different formula
      than stated (e.g., normalizes vectors) and bring paper into alignment
      with actual implementation behavior.

[C-02 / I-03] ZERO-VECTOR SIMILARITY CONTRADICTED BY SPEC
  Location: M.1 spec test invariant `zero_jobs_no_nan` vs paper §3.3
  Problem: Spec says similarity = 0.0 for zero-vector tract; paper says 1.0.
  Fix: Correct the spec test invariant to match the paper's community-correct
    choice (1.0 for both-zero; 0.5 for one-zero). If the implementation uses 0.0,
    fix the implementation. Update the spec's test invariant.
  Note: This may be a simple spec typo. The paper's design choice (1.0) is
    correct and is also consistent with M.9. The spec invariant is likely wrong.

!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

P2 items (should fix):
  [I-05] Add bisect build provenance (commit hash) to §4 experimental setup.

P3 items (optional):
  [G-02] Report RTP ≥80% co-assignment metric (spec required but paper discusses
         qualitatively only).
  [I-04] Clarify M.1 deviation from M.0 formula with brief justification.
  [I-08] State multi-seed expectation for WI null result.
  [I-09] Add parenthetical defining w_geo in §5.3 expert witness formula.
  [I-10] Add "comparable economic community testimony" qualifier for CA Prop 11.
  [I-11] WA RCW citation: add "enacted 1983, as amended."

PRE-PANEL CHECKLIST:
[x] All spec contract promises delivered or properly deferred
[ ] P1 BLOCKER: Similarity matrix corrected (JPR normalization) — MUST FIX
[ ] P1 BLOCKER: Zero-vector spec inconsistency resolved — MUST FIX
[ ] Mechanism story (§4.2) revised once similarity matrix corrected
[ ] Abstract line 22-23 revised once scale issue resolved
[x] Single-run results daggered (WI and TX correctly daggered)
[x] NC no-dagger appropriate (confirmed from data)
[x] Blend formula consistent: w_new = w*(alpha + (1-alpha)*sim)
[x] alpha=0.5 stated consistently
[x] CLI flags correct: --weights-override economic-character, --econ-alpha
[x] Court citations verified: Rucho 588 U.S. 684 (2019) correct
[x] State statutes verified: CA Prop 11, CO Art. V §44, AZ Prop 106, WA RCW 44.05.090

VERDICT: FIXES REQUIRED (major) — 2 P1 blockers must be resolved before panel
Fixes required: 2 (P1 critical), 1 (P2), 6 (P3 optional)

Root cause of both P1 issues: the M.1 spec was written before the M.9 impl spec
was finalized. The M.9 impl spec has the correct zero-vector handling (1.0 for
both-zero). The M.1 spec has a stale test invariant. The similarity matrix
appears to have been computed with a normalization assumption not carried forward
into the paper's formula description.

Next: Resolve JPR normalization (consult implementation or run test), fix spec
test invariant, recompute Table 2, revise §4.2 mechanism story, then run panel
review using the 5-role panel (Karypis/Rodden/Duchin/Stephanopoulos/Liang).
=================================================================
```
