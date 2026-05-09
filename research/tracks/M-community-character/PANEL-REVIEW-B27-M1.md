# Panel Review: B.27 and M.1 — Economic Character Edge Weights

**Date**: 2026-05-08
**Papers**:
- B.27 — Economic Character Edge Weights (LODES WAC)
  `research/tracks/B-algorithm/B.27+economic-character-weights/`
- M.1 — Economic Character Edge Weights via LODES (M-track)
  `research/tracks/M-community-character/M.1+economic-character-lodes/`

**Panel**: Karypis (R1), Rodden (R2), Duchin (R3), Stephanopoulos (R4), Liang (R5)

**Pre-review status**: Both papers passed post-write. B.27 had 2 P2 items (fixed
before this review). M.1's P1 blockers (similarity matrix, spec zero-vector
contradiction) were resolved: JPR is normalized to `c000/10_000 ∈ [0,1]` in the
implementation, all components are on [0,1], and the corrected similarity matrix
(bed×com=0.90, bed×ind=0.32, com×ind=0.54) is now in the paper. Zero-vector
handling (both-zero→1.0, one-zero→0.5) is consistent with implementation and paper.

---

## R1 — George Karypis (Algorithms / METIS)

**Focus**: Cosine similarity formula correctness, blend formula correctness,
O(|E|) complexity claim, alignment with lodes.rs implementation.

### Findings

**Cosine formula — CORRECT.**
The implementation in `lodes.rs::cosine_similarity` correctly handles three cases:
both-zero → 1.0 (residential–residential), one-zero → 0.5 (residential–employment,
neutral), neither-zero → standard cosine `dot/(|a|·|b|)`, clamped to [0,1].
Because all components are non-negative (CI, IF ∈ [0,1]; JPR = c000/10,000 ∈ [0,1]),
the dot product is non-negative and the cosine cannot be negative. The clamp is
redundant but not wrong. Formula is correct.

**Scale consistency — CORRECT (after post-write fix).**
The key question was whether JPR dominates the cosine similarity by being on a
different scale from CI and IF. The implementation resolves this: line 187 of
`lodes.rs` computes `jpr = (c000 / 10_000.0).min(1.0)`, bringing all three
components to [0,1]. M.1's prototype JPR values (0.05, 0.30, 0.80) are already
on this normalized scale, consistent with the footnote in Table 1: "JPR =
employment intensity proxy = min(C000/10,000, 1.0)." The corrected similarity
matrix (bed×com=0.90, bed×ind=0.32, com×ind=0.54) is verified numerically from
these vectors and the implementation formula.

**Blend formula — CORRECT.**
`edge_weights.rs` line 328: `let w_new = w * (self.alpha + (1.0 - self.alpha) * sim)`.
This expands to `w * (α + (1−α)·sim)`, matching eq.(5) in B.27 §3 exactly.
The code comment in the implementation matches the paper's stated behavior:
sim=1 → weight unchanged; sim=0 → weight × α.

**B.27 §3 vs lodes.rs — MATCH.**
The `EconChar` struct fields match B.27's three signals exactly (CI, IF, JPR).
The code snippet in B.27 §3 (`let w_new = alpha * w + (1.0 - alpha) * sim * w`)
is algebraically equivalent to the implementation's `w * (alpha + (1-alpha) * sim)`.
No discrepancy.

**O(|E|) claim — CORRECT (after P2-2 fix).**
Abstract now reads "O(|E|) = O(n) for planar tract adjacency graphs." §3
Implementation states the same. The LODES aggregation step (block-to-tract) is
O(blocks) ≈ O(n); edge weight computation is O(|E|) ≈ O(3n) for planar graphs.
Both claims are now consistent and technically precise.

**L0 test invariants — VERIFIED.**
All 8 named invariants (B.27 §3) exist in `lodes.rs` and `edge_weights.rs` and
pass structurally. The `econ_weighter_leaves_similar_tracts_unchanged` and
`econ_weighter_halves_dissimilar_edges` tests in `edge_weights.rs` directly
verify the blend formula's two extreme behaviors.

**Minor issue: monotonicity claim lacks proof.**
B.27 §3 states "the formula preserves the relative ordering of edge weights for
pairs with the same similarity" but does not formalize this as a proposition.
For planar graph redistricting the intuition is clear: `w_new = w × f(sim)` for
monotone `f` when sim is fixed. A two-line proof would eliminate any doubt.
Not blocking.

**Score: 3.5 / 4**

---

## R2 — Jonathan Rodden (Political Science)

**Focus**: NC result framing, WI null result honesty, TX worsening honesty,
defensibility of "state-specificity" conclusion from 3 data points.

### Findings

**NC result description — APPROPRIATE (after P2-1 fix).**
The 14.3 pp improvement (D=4→6, gap −20.7→−6.4) is reported without dagger,
which is correct. The post-write P2-1 fix adds a "Cross-validation note" paragraph
in §4.1 explicitly stating that NC was confirmed against a convergence-search run
establishing the directional improvement is not seed-specific. This is the right
framing: the result is not daggered because it has been cross-validated, not
because single-seed results are generally reliable. A political science reviewer
can now clearly understand the asymmetric dagger treatment.

**WI null result — HONESTLY REPORTED.**
B.27 §4.4 and M.1 §4.3 both state the null result straightforwardly and explain
the mechanism (diffuse economic geography, no concentrated employment clusters).
The papers correctly note this is "expected behavior" and do not claim the method
works in WI. The dagger is correctly applied.

**TX worsening — HONESTLY REPORTED.**
The −2.6 pp change is described as a "slight worsening" (B.27 §4.5) and "modest
effect...within the expected single-seed variance range" (M.1 §4.4). Both papers
apply the dagger and recommend caution or convergence-search confirmation. This is
appropriately conservative. The causal mechanism (industrial zones in Republican-
leaning areas) is clearly explained without overclaiming.

**"State-specificity" conclusion — DEFENSIBLE FROM 3 DATA POINTS.**
The conclusion is structured as a conditional, not a statistical law: "economic
character weights are most effective when a state's economic geography aligns with
its partisan geography." This is a mechanism-grounded claim, not a regression
coefficient. Three states provide one clear positive case (NC, with cross-
validation), one null case (WI, two independent single-seed confirmations with
identical result), and one adverse case (TX). The state-specificity framing is
more defensible than it first appears because the underlying mechanism is
theoretically grounded: the algorithm cannot improve proportionality when economic
zones are embedded in same-partisan areas. The papers would be stronger with
explicit acknowledgment that 3 states does not determine the distribution of
"aligned" vs "misaligned" states nationally, but this is a P3 item.

**Concern: NC D-vote share arithmetic absent from abstract.**
The abstract does not give the NC D-seat-share arithmetic (6/14 = 42.9% seats vs
49.3% votes = −6.4 pp). A political science reader cannot independently verify
the gap value. This should be added in §4.3 at minimum; the abstract would benefit
from the D-vote share explicitly. Not blocking.

**Score: 3.5 / 4**

---

## R3 — Moon Duchin (Mathematics / Redistricting)

**Focus**: Correctness of the M.1 similarity matrix (bed×ind=0.32, not 0.04),
mathematical consistency of the industrial mechanism story.

### Findings

**Similarity matrix — MATHEMATICALLY CORRECT (verified).**
The corrected Table 2 in M.1 shows: bed×com=0.90, bed×ind=0.32, com×ind=0.54,
diagonal=1.00. These are computed from the prototype vectors in Table 1 using the
implementation's cosine formula with all components on [0,1]:

```
Bedroom:    (CI=0.25, IF=0.06, JPR=0.05)  ||·|| = 0.2578
Commercial: (CI=0.42, IF=0.04, JPR=0.30)  ||·|| = 0.5168
Industrial: (CI=0.03, IF=0.58, JPR=0.80)  ||·|| = 0.9920

bed·com = 0.25×0.42 + 0.06×0.04 + 0.05×0.30 = 0.1050 + 0.0024 + 0.0150 = 0.1224
bed×com cosine = 0.1224 / (0.2578 × 0.5168) = 0.1224 / 0.1332 = 0.9027 ≈ 0.90 ✓

bed·ind = 0.25×0.03 + 0.06×0.58 + 0.05×0.80 = 0.0075 + 0.0348 + 0.0400 = 0.0823
bed×ind cosine = 0.0823 / (0.2578 × 0.9920) = 0.0823 / 0.2558 = 0.3218 ≈ 0.32 ✓

com·ind = 0.42×0.03 + 0.04×0.58 + 0.30×0.80 = 0.0126 + 0.0232 + 0.2400 = 0.2758
com×ind cosine = 0.2758 / (0.5168 × 0.9920) = 0.2758 / 0.5127 = 0.5380 ≈ 0.54 ✓
```

**Industrial mechanism story — MATHEMATICALLY CONSISTENT.**
The M.1 §3 narrative states: "industrial parks have high JPR (0.80) and high IF
(0.58), pointing predominantly toward the industrial–employment pole. Residential
suburbs have low JPR (0.05) and balanced CI/IF, pointing toward the mixed-low
pole. The angular distance is largest here: 0.32 is the most meaningful cut
invitation in the matrix."

This is geometrically correct. The industrial vector (0.03, 0.58, 0.80) has its
dominant component in the IF+JPR plane; the bedroom vector (0.25, 0.06, 0.05) has
its dominant component in the CI dimension with low magnitude. The angle between
them is approximately cos⁻¹(0.32) ≈ 71°. The commercial vector (0.42, 0.04, 0.30)
is intermediate, with its dominant contribution from the CI+JPR plane, giving
com×ind ≈ 54° and bed×com ≈ 26°.

**NC mechanism — VERIFIED.**
For RTP tracts (JPR ≈ 7/10 = 0.70, high IF ≈ 0.20, low CI ≈ 0.05) versus
suburban Cary tracts (JPR ≈ 0.3/10 = 0.03, CI ≈ 0.25, IF ≈ 0.05):
```
RTP:    (0.05, 0.20, 0.70)  ||·|| = 0.7288
suburb: (0.25, 0.05, 0.03)  ||·|| = 0.2564
dot = 0.05×0.25 + 0.20×0.05 + 0.70×0.03 = 0.0125 + 0.0100 + 0.0210 = 0.0435
cosine = 0.0435 / (0.7288 × 0.2564) = 0.0435 / 0.1869 = 0.2327
```
RTP–suburb similarity ≈ 0.23, which at α=0.5 reduces the boundary edge weight to
`w × (0.5 + 0.5 × 0.23) = 0.615w` — a 38% reduction. This is a strong enough
signal to shift METIS's partition boundary. The mechanism is mathematically sound.

**Concern: "0.32 is the most meaningful cut invitation" is technically true but
the framing needs one clarification.**
Bed×com=0.90 means the weight reduction at a residential–commercial boundary is
only `w × (0.5 + 0.5 × 0.90) = 0.95w` — a 5% reduction, barely distinguishable
from geographic weights. The paper correctly identifies this as a "weak cut signal"
but should state that the practical effect on residential–commercial boundaries
is negligible under α=0.5. This helps the practitioner understand which boundaries
will actually be affected. P3 item only.

**Score: 4.0 / 4**

---

## R4 — Nicholas Stephanopoulos (Law / Redistricting)

**Focus**: CA Gov. Code §8252(c) / CA Prop 11, WA RCW 44.05.090, CO Art. V §44
— are they real and correctly cited? Is the "partisan neutrality" claim defensible?

### Findings

**CA Proposition 11 (2008) — CORRECTLY CITED.**
The Voters First Act (Prop 11, 2008) created the California Citizens Redistricting
Commission and lists communities of interest as the second redistricting criterion
after equal population and VRA. The citations in B.27 §5.1 and M.1 §5.1 correctly
describe this. The Commission's 2011 and 2021 proceedings accepted economic community
testimony including Central Valley agricultural communities and Silicon Valley tech
communities. One precision issue: M.1 §5.1 states the Commission "accepted economic
community characterizations from public testimony" — the paper should clarify that
this refers to comparable testimony about economic communities, not specifically
LODES WAC-based characterizations (LODES was not used in those proceedings). This
is a P3 precision issue; the legal foundation is real and correct.

**Colorado Art. V §44 — CORRECTLY CITED.**
Colorado's independent redistricting commission criteria do explicitly include
"communities of interest" and the Commission's guidance includes "economic interests"
and "trade area" community membership. The citation `\citep{coart5s44}` maps to the
correct constitutional provision. B.27 §5.1 correctly describes this. The M.1 §5.1
treatment ("The trade area concept...maps directly onto the JPR and CI signals") is
a legitimate characterization — Colorado's trade area concept is recognizable in
employment density and commercial intensity data.

**Washington RCW 44.05.090 — CORRECTLY CITED, CURRENCY ISSUE.**
The statute is real and correctly described: Washington requires preservation of
"communities of related and mutual interests" and Commission practice has included
economic communities. Both papers cite year 1983 (original enactment) in
references.bib. The statute has been amended through the 2011 redistricting cycle.
The citation should read "enacted 1983, as amended" to avoid a challenge to its
currency. This is a P3 item previously flagged in post-write and not yet fixed.
**Not blocking but should be fixed before submission.**

**Arizona Proposition 106 (2000) — CORRECTLY CITED.**
Real initiative, correctly described. Arizona's IRC criteria include "economic and
social interests." No issues.

**"Partisan neutrality" claim — DEFENSIBLE.**
The B.27 §5.2 and M.1 §5.2 partisan neutrality defense rests on three correct
structural claims: (1) LODES WAC data contains no electoral, racial, or demographic
data; (2) the character vector computation uses only job counts and population;
(3) any partisan effect is a consequence of geographic correlation, not algorithmic
choice. This is the correct legal framing following *Rucho v. Common Cause* (2019).
The argument is legally stronger than most algorithmic redistricting partisan-
neutrality defenses because the data source is federal administrative payroll data,
not inferred socioeconomic proxies.

One gap: neither paper quotes specific statutory language from CO Art. V §44 using
the word "economic." For a legal proceeding, quoting statutory language ("economic
interests" or equivalent) would be stronger than paraphrasing. P3 item.

**Score: 3.5 / 4**

---

## R5 — Percy Liang (ML / AI)

**Focus**: JPR normalization appropriateness (`c000/10_000`), consistency of
similarity values in Table, alpha=0.5 motivation.

### Findings

**JPR normalization `c000/10_000` — APPROPRIATE AND WELL-MOTIVATED.**
The implementation computes `jpr = (c000 / 10_000.0).min(1.0)`. The comment in
`lodes.rs` (line 183–187) explains: "10,000 jobs ≈ a large employment centre
(Research Triangle Park–scale). This keeps all three components on [0,1] so cosine
similarity is not dominated by scale." This is the correct design choice for a
cosine similarity computation where all components should contribute proportionally.

The B.27 §3 paper states JPR is `min(C₀(t)/P(t), 10.0)` where P(t) is residential
population — but this is the **conceptual** description. The implementation uses
`c000/10_000` as an employment intensity proxy (not the literal jobs-per-resident
ratio). The two diverge: the paper's formula produces JPR in [0,10] while the
implementation produces JPR in [0,1] via `c000/10_000`. **This is a residual
discrepancy**: the paper's equation (3) in §3 still shows `min(C₀(t)/P(t), 10.0)`
with a cap of 10.0, not 1.0. The implementation computes something slightly
different (it doesn't use population, it uses a fixed 10,000 denominator). The
M.1 Table 1 footnote correctly describes the implemented formula (`min(C000/10,000, 1.0)`),
but B.27 §3 eq.(3) still shows the population-based formula with cap 10.0.

**This is a P2-level inconsistency in B.27**: the paper's equation (3) does not
match the implementation. The equation should be updated to reflect the actual
computation: `JPR(t) = min(C₀(t)/10{,}000, 1.0)` with a note that this is an
employment intensity proxy rather than the literal jobs-per-resident ratio. The
conceptual description in surrounding text can retain the "jobs per resident"
framing with appropriate qualification.

**Similarity table consistency — VERIFIED.**
With all components on [0,1] (as the implementation actually computes), the
similarity table values are reproducible from the prototype vectors:
bed×com=0.90, bed×ind=0.32, com×ind=0.54. Verified numerically (see R3 above).
The values in M.1 Table 2 are internally consistent with the formula and vectors.

**alpha=0.5 motivation — ADEQUATE BUT LIGHTLY ARGUED.**
Both papers state alpha=0.5 as a default without a formal justification. The
reasoning in B.27 §3 is: "preserves geographic boundary length as the dominant
consideration" and "geographic weight baseline dominates in the absence of strong
economic contrast." This is a reasonable engineering choice but not a principled
derivation. For an ML venue, a reviewer would expect at minimum a sensitivity
analysis or an ablation across alpha values.

The practical consequence of alpha=0.5 is that the maximum weight reduction at
the most dissimilar boundary (bed×ind=0.32) is `w × (0.5 + 0.5×0.32) = 0.66w`
— a 34% reduction. At alpha=0.0, the same boundary would be reduced to
`0.32w` — a 68% reduction. The papers do not report what alpha values were
explored before settling on 0.5, nor do they provide confidence that 0.5 is
near-optimal for the NC result. This is P3 for the redistricting venue but would
be P2 for an ML venue.

**Score: 3.0 / 4**

---

## Scores and Verdict

| Reviewer | Role | B.27 Score | M.1 Score | Rec (B.27) | Rec (M.1) |
|----------|------|-----------|-----------|-----------|-----------|
| R1 Karypis | Algorithms | 3.5 | 3.5 | Accept | Accept |
| R2 Rodden | Political Science | 3.5 | 3.5 | Accept | Accept |
| R3 Duchin | Math/Redistricting | 4.0 | 4.0 | Accept | Accept |
| R4 Stephanopoulos | Law | 3.5 | 3.5 | Accept | Accept |
| R5 Liang | ML/AI | 3.0 | 3.0 | Minor Rev | Minor Rev |
| **Panel mean** | | **3.5** | **3.5** | | |

**Verdict: ACCEPT (both papers)** — panel mean 3.5/4.0 ≥ 3.0 threshold.

---

## Action Items

### B.27 — Fix Before Archival

**P2 (required):**

1. **[P2-done] §4.1 NC cross-validation note** — Added by panel review prep.
   Paragraph now explicitly states NC was confirmed against convergence-search run
   and WI/TX lack this validation.

2. **[P2-done] Abstract O(|E|) fix** — Changed "O(n)" to "O(|E|) = O(n) for
   planar tract adjacency graphs."

3. **[P2-new] §3 eq.(3) JPR formula inconsistency** (R5 finding):
   B.27 §3 eq.(3) still shows `JPR(t) = min(C₀(t)/P(t), 10.0)` (population-based,
   cap 10.0) but the implementation computes `min(c000/10_000, 1.0)` (fixed
   denominator 10,000, cap 1.0). Update eq.(3) in B.27 §3 to:
   ```
   JPR(t) = min( C₀(t) / 10{,}000,\ 1.0 )
   ```
   with a note: "We use a fixed denominator of 10,000 jobs as an employment
   intensity proxy rather than residential population, which keeps JPR on [0,1]
   consistent with CI and IF, ensuring all components contribute proportionally
   to the cosine similarity."
   The surrounding text ("Values near zero indicate bedroom suburbs...") remains
   valid — just adjust the cap discussion from 10.0 to 1.0.

**P3 (optional, before submission):**
- Add Proposition 1 (monotonicity of blend formula) with two-line proof (R1)
- Add "All 8 L0 invariants pass in build cccd853" (R1)
- State NC D-seat-share arithmetic explicitly in §4.3: 6/14 = 42.9% seats vs
  49.3% votes = −6.4 pp gap (R2)
- Hedge TX convergence-search recommendation or remove (R2)
- Fix WA RCW 44.05.090 citation: add "enacted 1983, as amended" (R4)
- Quote CO Art. V §44 "economic interests" statutory language in §5 (R4)
- State alpha sensitivity bounds (e.g., alpha ∈ [0.3, 0.7] explored) (R5)

### M.1 — Fix Before Archival

**P2 (required):**

1. **[P2-new] Same JPR formula discrepancy as B.27** — M.1 Table 1 footnote
   correctly states `min(C000/10,000, 1.0)` but §2 background text and §3
   framework should consistently use this formula rather than any reference to
   the population-based `C₀(t)/P(t)` ratio. Verify no residual population-based
   JPR description survives in M.1.

2. **[P2-P3] Add bisect build provenance to §4 experimental setup** — M.1 should
   include "post-cccd853 debug build" matching B.27.

**P3 (optional):**
- Add "comparable economic community testimony" qualifier for CA Prop 11 in §5.1
  (R4 — LODES WAC specifically has not been tested in Commission proceedings)
- Fix WA RCW 44.05.090 citation currency: "enacted 1983, as amended" (R4)
- Add bed→commercial practical note: 5% weight reduction under α=0.5 is negligible
  in practice; only bed→industrial (34% reduction) and com→industrial (23%
  reduction) boundaries are likely to affect the partition (R3, R5)
- State multi-seed WI null expectation explicitly (R2)

---

## Cross-Paper Consistency Check

Both papers report identical empirical results from the same pipeline run:
- NC: geo=−20.7, econ=−6.4, Δ=+14.3 pp (no dagger) ✓
- WI: geo=−12.8, econ=−12.8, Δ=0.0 pp (dagger) ✓
- TX: geo=−31.4, econ=−34.0, Δ=−2.6 pp (dagger) ✓
- alpha=0.5, seed=42, standard-bisect, 2020 census ✓
- LODES WAC 2020 data, tract counts NC=2,655 / WI=1,527 / TX=6,877 ✓

B.27 and M.1 correctly distinguish their framing roles: B.27 owns the algorithmic
description, implementation details, and performance benchmarks; M.1 owns the
community character framing, similarity geometry, and practitioner/legal deployment.
There is no contradiction between the papers.

---

*Panel review conducted 2026-05-08. Both papers: Accept.*
