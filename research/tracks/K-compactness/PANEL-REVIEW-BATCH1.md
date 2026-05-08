# K-Compactness Track — Hostile Peer Panel Review Batch 1

**Date**: 2026-05-08
**Papers reviewed**: K.0–K.7 (8 papers)
**Panel**: R1 Karypis (algorithms/geometry), R2 Rodden (poli-sci/empirics),
R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/systems/impl)
**Scoring**: 0–4 per reviewer. Average ≥3.0 → Accept; ≥2.5 → Minor Revision;
≥2.0 → Major Revision; <2.0 → Reject.

---

## Summary Table

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1s | Top P1 Issue |
|---|---|---|---|---|---|---|---|---|---|
| K.0 compactness-overview | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 1 | Reock≥PP proposition undefended |
| K.1 polsby-popper | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 1 | Reock≥PP for convex D stated then quietly contradicted |
| K.2 reock | 2 | 3 | 2 | 3 | 1 | 2.2 | Major Revision | 3 | Implementation uses centroid+max-dist, NOT Welzl; paper claims Welzl |
| K.3 convex-hull | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 0 | — |
| K.4 schwartzberg | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 1 | CO Art. V §44 citation imprecise |
| K.5 length-width | 3 | 2 | 3 | 2 | 3 | 2.6 | Minor Revision | 1 | LW>3 framed as "documented court threshold"; no binding precedent cited |
| K.6 population-weighted-compactness | 3 | 3 | 3 | 3 | 3 | 3.0 | Accept | 1 | Claimed 18% PWC reduction; body computes 14%; internal contradiction |
| K.7 composite-court-guide | 3 | 3 | 2 | 2 | 3 | 2.6 | Minor Revision | 2 | Simulated panel presented as evidence; Reock(D)≥PP(D) Proposition mis-stated |

---

## Detailed Reviews

---

### K.0 — Compactness Overview

**Paper**: *Compactness Metrics in Algorithmic Redistricting: A Taxonomy, Correlation Analysis, and Practitioner Decision Table*

#### R1 — Karypis: 3/4

The taxonomy and definitions are correct. The decision table cites Welzl O(n) and Andrew's monotone chain O(n log n) by cross-reference; acceptable. The claim in §03-definitions that "the MBR is computed via rotating calipers in O(n) after computing the convex hull in O(n log n)" is stated correctly. The claim that Welzl gives the exact MBC is delegated to K.2 section 3—that delegation passes here, but K.2 has a major problem this paper inherits by reference.

**P2**: The correlation table (Table 2) is labelled seed=42† but uses 240 districts across all four algorithms—the dagger notation correctly covers the within-algorithm single-seed concern, but the paper does not note that cross-algorithm comparisons include structural differences that a dagger cannot disambiguate. Minor.

**P3**: No binary name issues; all references use `bisect`.

#### R2 — Rodden: 3/4

The correlation structure (area-based family r=0.79, cross-family r<0.80) is a meaningful finding. The decision table correctly points to PWC for VRA analysis.

**P2**: The claim "At least four metrics appear across redistricting case law" (abstract and §07-conclusion) is supportable from the body's evidence. The Rucho citation is accurate. The paper correctly hedges that "no federal court has established a single authoritative metric." No overclaiming detected.

No discussion of geographic sorting effects on the correlations—whether urban vs. rural districts drive the cross-family divergence. This is a notable omission for the Rodden lens but does not rise to P1.

#### R3 — Duchin: 3/4

Definitions are mathematically clean. The LW family classification as "Elongation" rather than a sub-family of area-based is well-motivated. The S=1/√PP identity is cited to K.4 Theorem 1 without restating the proof, which is appropriate in an overview.

**P1 (minor)**: §02-taxonomy, Family 1 states that PP achieves π/4≈0.785 for a square. §03-definitions Table (canonical values) correctly gives π/4≈0.785 for a square. Consistent. However, Family 2 says "Reock of a square ≈ 0.637" and the K.0 definition section gives Reock=2/π for a square (=0.637). Internally consistent.

The undefended Proposition in K.2 that Reock(D)≥PP(D) for any convex D is cited positively in this overview ("Reock is systematically more lenient"). Since K.2 doesn't prove it, and the K.1 PP–Reock gap section shows a 1×6 rectangle where PP>Reock=0.21 while PP=0.38, the claim is false for all convex polygons. The overview repeats this soft claim without flagging it.

**P1**: The overview asserts "the Reock–PP gap averages +0.12 under ratio-optimal" (§06-synthesis), but K.2 §07-gap reports the ratio-optimal NC gap as +0.15, explicitly noting "+0.12 is a conservative lower bound." The abstract and synthesis sections of K.0 use +0.12 as the headline. Minor internal discrepancy between K.0 and K.2, but the gap direction is correct.

#### R4 — Stephanopoulos: 3/4

All four case citations in the abstract (Rucho, Shaw, Harper, Colorado) are cases that exist and the propositions attributed to them are accurate. No invented holdings detected. The decision table correctly frames LW as "most commonly enforced bright-line rule in court filings" rather than binding law.

**P2**: The claim "bisect produces LW ≤ 2.8 on NC 2020 by construction" is slightly overclaiming—K.5 documents this as an empirical result at seed=42, not a design guarantee.

#### R5 — Liang: 3/4

The CLI commands (`bisect label-analyze --types all`, `bisect label-analyze --types pp`, etc.) are consistent with CLAUDE.md documentation. The JSON output format shown in K.7 §05 matches what a well-structured implementation would produce.

**P1 (inherited)**: The overview says in §03-definitions that "Computation uses Welzl's randomised algorithm in O(n) expected time (K.2, Section 3)"—but the actual implementation in `crates/bisect-analysis/src/compactness.rs` uses centroid + max-distance-to-boundary (an approximation), not Welzl. This is the cross-paper P1 propagated from K.2.

**Verdict**: Accept (3.0). One P1 inherited from K.2; one minor discrepancy on +0.12 vs +0.15 gap.

---

### K.1 — Polsby-Popper

**Paper**: *Polsby-Popper Score in Algorithmic Redistricting: Derivation, Projection Sensitivity, and Empirical Comparison*

#### R1 — Karypis: 3/4

The isoperimetric inequality proof is correct and complete (cites Osserman 1978). The scale-invariance proof is correct. The projection pipeline (WGS84 → EPSG:5070 → shoelace formula) is precisely described and matches the implementation. The projection correction table (NC +0.040, WI +0.038, TX +0.035) is internally consistent with the claim that shifts are "0.03–0.06."

**P2**: §04-implementation claims the L2 regression test checks "NC 2020 ratio-optimal mean PP ≥ 0.22 (seed=42†)"—this is a claim about an L2 test that exists in the test suite (the paper says it is there), but such a test would require real data marked `#[ignore]`. The paper doesn't note the `#[ignore]` qualifier, which could mislead a reader expecting this test to run in CI.

#### R2 — Rodden: 3/4

The enacted map comparison (§05-empirical) says the NC 2020 enacted map had "mean PP of approximately 0.12–0.15 across expert reports submitted in Harper v. Hall (2022)." This is a reasonable characterisation of what expert reports showed, with appropriate hedging. The claim that "even the weakest bisect algorithm outperforms the enacted map" is justified by the data.

**P2**: No discussion of whether the projection shift (+0.04 for NC) systematically favours or disadvantages any particular algorithm. Rodden would want to know if the WGS84→EPSG:5070 correction is uniform across algorithms or algorithm-dependent.

#### R3 — Duchin: 3/4

The canonical values table (disk=1, hexagon=π√3/6≈0.907, square=π/4≈0.785, equilateral triangle=π/(3√3)≈0.605) are all correct.

**P1**: §06-pp-reock contains a Proposition stated in K.2 ("Reock(D)≥PP(D) for any convex polygon D") that is tested with a 1×6 rectangle. The computation in K.1 §06 shows: for a 1×6 rectangle, Reock=6/29.1≈0.21 and PP=24π/196≈0.38. So PP>Reock here, which directly contradicts the K.2 Proposition that "Reock≥PP for any convex D." K.1 acknowledges the contradiction ("the direction of divergence depends on the specific geometry") but this means the Proposition in K.2 is false and should never have been stated. K.1 exposes but does not retract the false claim.

#### R4 — Stephanopoulos: 3/4

The legal section is careful and accurate. The Ohio H.B. 1 (2015) citation is real. The Iowa Code §42.4 citation is real. The Rucho reference to state-law compactness standards remaining enforceable is correct.

**P2**: The Princeton Gerrymandering Project citation (pgp2022) implies bisect ratio-optimal plans "would score in the B–A range." This is an extrapolation from the PGP's grading rubric and is not verified against PGP's actual methodology. Should be hedged more clearly.

#### R5 — Liang: 3/4

The implementation section (§04) matches compactness.rs exactly: EPSG:5070 projection, `geo` crate, shoelace for area, Euclidean edge sum for perimeter. The test invariants listed in the paper (PP of 1000-gon within 0.001 of 1.0, PP in [0,1], PP of square within 0.001 of π/4, scale-invariance) all have matching tests in compactness.rs. The L1 and L2 tests described are plausible.

**Verdict**: Accept (3.0). One P1 (Reock≥PP false proposition exposed but not retracted). Fix by retracting the K.2 Proposition and softening both papers' language to "Reock tends to be more lenient than PP empirically."

---

### K.2 — Reock

**Paper**: *Reock Score in Algorithmic Redistricting: Minimum Bounding Circle, Boundary Insensitivity, and Moving-Knife Optimisation*

#### R1 — Karypis: 2/4

**P1 (Critical)**: Section 3 presents Welzl's algorithm in full pseudocode and proves it computes the exact MBC in O(n) expected time. The proof sketch is mathematically correct (uniqueness from strict convexity; backward analysis giving 3/i probability bound). The implementation subsection says: "The implementation in `crates/bisect-analysis/src/compactness.rs` extracts all polygon vertices (including holes), shuffles them using the run's seed for reproducibility (seed=42†), and calls Welzl's algorithm."

**This is false.** The actual implementation in `crates/bisect-analysis/src/compactness.rs` (line 78–91) does:
```rust
let centroid = polygon.centroid()...;
let radius = max_distance_to_boundary(polygon, centroid);
```
where `max_distance_to_boundary` (lines 289–296) computes the maximum Euclidean distance from the polygon's *geometric centroid* to any boundary vertex. This is NOT Welzl's algorithm. It is a heuristic approximation: the centroid is not generally the centre of the MBC, and the maximum distance from the centroid to the boundary is not generally the MBC radius. The approximation can be exact for symmetric shapes but systematically overestimates the MBC radius (and thus underestimates Reock) for asymmetric shapes.

The comment in the code itself acknowledges this: "This is an approximation of the minimum bounding circle (not the true MBC via Welzl's algorithm, but matching Python's approximation exactly)."

The paper describes a Welzl implementation that does not exist in the codebase. The O(n) expected-time claim, the backward-analysis proof, the "shuffles using the run's seed" description, and the L0 invariant ("MBC contains all input points") all describe a non-existent implementation. The actual implementation does not guarantee the MBC contains all input points — for a concave district, the centroid + max-boundary-vertex circle may miss some vertices depending on polygon shape.

**P1 (Secondary)**: The Proposition at end of §02-definition: "Reock(D)≥PP(D) for any convex polygon D." The proof says "the proof is omitted as it requires multiline geometric argument beyond the scope of this paper." The K.1 §06-pp-reock section computes a 1×6 rectangle and obtains Reock≈0.21, PP≈0.38, i.e., PP>Reock, which falsifies the Proposition for this convex polygon. The Proposition is incorrect; the omitted proof does not exist.

**P1 (Tertiary)**: The abstract claims "Welzl's algorithm provides the exact minimum bounding circle in O(n) expected time." The implementation provides neither exact MBC nor Welzl's algorithm.

The algorithm description, the proof of Theorem 1 (Welzl correctness), the implementation description, and the L0 invariants are all correct descriptions of Welzl's algorithm — they simply describe an algorithm that is not in the codebase.

#### R2 — Rodden: 3/4

The MKA design rationale (maximise minimum Reock as a proxy for fairness) is well-motivated. The boundary insensitivity claim (Reock drops ≤0.019 vs PP drops 0.048 for coastal districts) is a meaningful finding that would survive empirical scrutiny.

**P2**: The paper says the boundary insensitivity advantage occurs "because the MBC is determined by at most three extreme points." This reasoning is correct for Welzl's algorithm (the boundary set |R|≤3), but the actual implementation (centroid + max-distance) has a different mechanism. The stated mechanism is accurate for the algorithm described but not for the algorithm implemented.

#### R3 — Duchin: 2/4

**P1**: The undefended Proposition (Reock≥PP for convex D) is false and should be retracted. This is a mathematical claim made without proof that is contradicted by a computable example in a sister paper.

The Welzl presentation is pedagogically correct for the algorithm as described. The MBC uniqueness argument (strict convexity) is standard and correct.

**P2**: The abstract says "Welzl's randomised algorithm...is implemented in bisect-analysis/src/compactness.rs." Given the implementation mismatch, this is an accuracy problem that cascades into every claim about O(n) runtime and MBC exactness.

#### R4 — Stephanopoulos: 3/4

The legal section is careful. Harper v. Hall (N.C. 2022), Gill v. Whitford (2018), and the Florida Congressional references are all real cases. The claim that Reock appears in expert reports (not court opinions) in these cases is appropriately hedged. No invented holdings.

**P2**: The paper says Reock is "preferred" for coastal states to avoid boundary-sensitivity problems. This is good practical advice. The disclosure template is legally precise.

#### R5 — Liang: 1/4

**P1 (Critical)**: The implementation section of the paper describes an algorithm not present in the codebase. The code explicitly comments "not the true MBC via Welzl's algorithm, but matching Python's approximation exactly." The paper omits this caveat entirely and presents the centroid+max-dist approximation as if it were Welzl's exact algorithm. Every quantitative Reock result in the paper is computed using the approximation, but all algorithmic claims (O(n) time, exact MBC, reproducibility via seed-shuffled order) describe Welzl.

The L0 invariant "Welzl MBC contains all input points: for all p∈P, ‖p−centre‖≤r+10⁻⁹" cannot pass with the actual implementation, because the centroid is not guaranteed to be the MBC centre. For a highly asymmetric polygon, the centroid+max-dist circle may not contain all vertices.

The test suite in compactness.rs confirms this: `test_reock_circle_approaches_1` and `test_reock_square_is_pi_over_4` pass, but neither checks that the computed circle actually contains all polygon vertices — they only check the ratio value for symmetric shapes where the approximation happens to be exact.

The implementation discrepancy is the most severe P1 in the K-track. All papers that cite Reock computation (K.0, K.1, K.2, K.6, K.7) inherit this issue.

**Verdict**: Major Revision (2.2). Three P1s: (1) implementation claims Welzl but uses centroid+max-dist approximation; (2) Reock≥PP Proposition false; (3) Welzl's algorithm's stated L0 invariant ("MBC contains all input points") cannot hold with the actual implementation. Required revision: (a) Retitle §3 as "Approximation of the Minimum Bounding Circle" and accurately describe the centroid+max-dist approach; (b) retract or correct the Welzl Theorem; (c) retract or correct the Reock≥PP Proposition; (d) add a caveat note that the approximation matches the Python pipeline for reproducibility.

---

### K.3 — Convex Hull

**Paper**: *Convex Hull Ratio in Algorithmic Redistricting: Tentacle Detection, Legal Intuition, and Algorithmic Guarantee*

#### R1 — Karypis: 3/4

The Andrew's monotone chain algorithm is correctly described. The cross-product collinearity condition (non-positive cross product → right turn or collinear → remove) is correct. The shoelace formula for convex hull area is correct. The section header says "Andrew's Monotone Chain" but the keywords say "Graham Scan"—this is actually clarified in the body ("Graham scan achieves the same asymptotic complexity but requires a polar angle sort; Andrew's algorithm uses a lexicographic sort that is numerically more stable"). The choice of Andrew over Graham is defensible and correctly explained.

**P2**: The L0 test for the cross-shaped district (arm length 10, width 2) states CH≈0.80 "verifiable analytically." The tentacle section (§04) instead tests a unit-square cross (arm length 5, width 0.1) giving CH=0.091. These are different geometries; the table at the end of §04 gives the 1×1 body with arms of length 5 width 0.1 as CH=0.091, not CH≈0.80. The L0 test value (CH≈0.80 for cross with arm length 10, width 2) is plausible but not computed in the paper. Minor.

#### R2 — Rodden: 3/4

The tentacle detection capability (PP≈0.272 is moderate, CH=0.515 is clearly low) is a real and useful analytical finding. The claim that all four bisect algorithms produce CH>0.85 on NC 2020 is specific and falsifiable.

**P2**: No discussion of whether the CH>0.85 guarantee holds across all seeds or only at seed=42. The paper hedges with † but does not discuss what the distribution of CH values looks like under multiple seeds. A practitioner would want to know if this is a robust finding.

#### R3 — Duchin: 3/4

The definition CH=A/A(conv(D)) is correct. CH=1 iff D is convex is correctly stated. The tentacle example computations in §04 are algebraically consistent (checked: area A=3.0, MBC area≈95.0, Reock≈0.032; area A=17, convex hull 11×3=33, CH=17/33≈0.515—all correct).

**P2**: The wide-arm dumbbell Reock calculation: "MBC: the two easternmost and westernmost points are at distance 3+2(4)=11, so r=5.5." But the body is a 3×3 square with two 4×1 arms. The east arm extends from x=1.5 to x=5.5 (centre at x=0 + 3/2 + 4 = 5.5) and west arm from x=-5.5 to x=-1.5. So the full east-west span is 11, radius=5.5, MBC area≈95. But the MBC centre should be at the centroid of the shape, not simply the midpoint of the east-west extremes—since the shape is symmetric, the centroid is at the geometric centre and the MBC is correct. The computation is valid for this symmetric case.

#### R4 — Stephanopoulos: 3/4

Shaw v. Reno citation is accurate (509 U.S. 630 (1993)). The operationalisation of Shaw's "bizarre shape" test via CH<0.60 is a reasonable academic interpretation, properly hedged as such. The League of Women Voters of Pennsylvania v. Commonwealth citation (178 A.3d 737 (Pa. 2018)) is a real case with a real holding (striking down 2011 map as partisan gerrymander). The claim that "expert reports used CH-like measures" is hedged appropriately.

No invented holdings detected.

#### R5 — Liang: 3/4

The convex hull computation in `convex_hull_ratio()` (lines 96–107 of compactness.rs) uses the `geo` crate's built-in `convex_hull()` method, which implements Andrew's monotone chain. This matches the paper's claimed implementation. The L0 invariant (square CHR=1.0) passes in `test_convex_hull_ratio_convex_polygon_is_1`. The L-shape test passes in `test_convex_hull_ratio_l_shape`. Implementation claims are accurate.

**Verdict**: Accept (3.0). No P1s. One minor P2 on the L0 cross-shaped CH value (CH≈0.80 in L0 description, different geometry from the tentacle section example).

---

### K.4 — Schwartzberg

**Paper**: *Schwartzberg Score in Algorithmic Redistricting: Equivalence to Polsby-Popper, Statutory Requirements, and Conversion Guide*

#### R1 — Karypis: 3/4

Theorem 1 (S=1/√PP) is proved correctly and completely with explicit algebraic chain. The corollaries (S·√PP=1; f(x)=1/√x converts PP to S losslessly) follow directly.

**P2**: The abstract states "a district with PP=0.22 has S≈2.13; a district with PP=0.40 has S≈1.58." Checking: 1/√0.22=1/0.4690≈2.132 ✓; 1/√0.40=1/0.6325≈1.581 ✓. Values are correct.

#### R2 — Rodden: 3/4

The statutory survey is carefully hedged ("operationalised compactness using a Schwartzberg-equivalent metric," "internally standardised to the S metric"). These are appropriate qualifications for claims about agency practice rather than statutory text.

**P2**: The claim that "Iowa LSA has internally standardised to the S metric" is not cited to a primary source—only Iowa Code §42.4. The LSA's internal methodology document would be needed to verify this precisely. Minor hedge insufficiency.

#### R3 — Duchin: 3/4

The mathematical content is entirely correct. The conversion table and the canonical values are algebraically consistent. The equivalence theorem is one of the cleaner proofs in the track.

**P2**: The paper notes S≥1 always, but does not note that S=1/√PP is only well-defined when PP>0. The code handles this (ZeroPerimeter and EmptyGeometry errors propagate through polsby_popper() into schwartzberg()). A mathematical statement of the domain restriction would strengthen rigor.

#### R4 — Stephanopoulos: 3/4

**P1**: The statutory section claims "Colorado's Amendment Y (2018) added Article V, Section 44 to the Colorado Constitution, establishing an Independent Congressional Redistricting Commission." Amendment Y in fact established the commission; Section 44 of Article V is the operative text. The paper then says the 2021 commission guidelines "operationalised compactness using a Schwartzberg-equivalent metric, specifying that districts should minimise the ratio of perimeter to the square root of area." The citation is `co_irc2021`—a plausible citation to commission procedural documents. The claim is plausible and the constitutional basis (Art. V §44) is real. However, the paper should cite the specific commission document (e.g., a resolution or staff report) rather than a generic co_irc2021 key, since the claim that S was formally operationalised (rather than informally used) is a moderately strong legal claim.

Not rated P1 (the constitutional provision exists; the commission's use of S-equivalent is hedged as "operationalised" not "mandated"); rated P2.

#### R5 — Liang: 3/4

The implementation computes schwartzberg via `1.0/pp.sqrt()` (line 123 of compactness.rs), reusing the polsby_popper() result. This is exactly what the paper describes. The L0 invariant (|S−1/√PP|<10⁻⁹) is verified in `test_schwartzberg_identity_with_pp` which checks to 1e-12 tolerance. All claims match implementation.

**Verdict**: Accept (3.0). One P2 on CO commission citation specificity. No P1s from a legal standpoint; the constitutional provision is real.

---

### K.5 — Length-Width Ratio

**Paper**: *Length-Width Ratio in Algorithmic Redistricting: Rotating Calipers, the LW>3 Threshold, and AABB vs. MBR Discrepancy*

#### R1 — Karypis: 3/4

The rotating calipers algorithm is correctly described. The pseudocode (Algorithm 1) is correct: for each hull edge, rotate coordinates so the edge is horizontal, compute AABB in that rotated frame, track minimum-area bounding box. The O(n) analysis (total rotation = 2π over all n hull edges) is correct. The total O(n log n) after O(n log n) convex hull is correct.

The rotated rectangle example (1×3 at 45°): AABB gives 2.83×2.83, LW_AABB=1.0. MBR via rotating calipers gives 3×1, LW_MBR=3.0. AABB underestimates LW from 3.0 to 1.0 (the paper says "100% error"), which is correct. The paper notes this is an extreme case and "for realistic districts oriented at modest angles (15–30°), the AABB overestimate is typically 5–41%"—this is consistent with the discrepancy table.

The implementation matches: compactness.rs lines 135–204 implement rotating calipers by iterating over hull edges, rotating, computing AABB in rotated frame, tracking minimum-area box.

#### R2 — Rodden: 2/4

**P1**: Section 4 (The LW>3 Court Threshold) says "Courts have applied a LW>3 threshold in redistricting challenges." The Illinois subsection says expert witnesses "characterised [LW>3] districts as 'elongated' and used to support claims"—this is about expert reports, not court holdings. The New York subsection says "The New York Court of Appeals' analysis...cited compactness criteria including elongation measures, with expert reports documenting LW values." Again, expert reports, not court holdings.

The Caveat subsection says correctly: "No federal court has established LW>3 as a binding legal threshold. The standard appears in expert reports and state court decisions as a heuristic, not a categorical rule."

The abstract and section title use "Court Threshold" language, and Claim 1 in the abstract says "LW>3 is a documented court threshold for elongation challenges." The body's caveat (p. 4, §4.4) correctly hedges, but the framing—"court threshold" in the section title, abstract Claim 1, and the K.0 decision table—overstates what the evidence supports. There are no citations to actual court decisions that hold LW>3 is a threshold; the evidence is expert reports and practitioner usage. Rodden would flag this as overclaiming given that the caveat is buried in §4.4.

**P2**: The one empirical claim with the strongest partisan implications—whether LW≤2.8 favours one party over the other in NC—is not discussed. A practitioner relying on LW≤2.8 as a design target should know if that target is geometrically or politically motivated.

#### R3 — Duchin: 3/4

The MBR vs. AABB distinction is mathematically precise. The AABB underestimate/overestimate analysis is correct. The rotating calipers proof of correctness is stated correctly (minimum-area bounding rectangle has a flush edge with the convex hull—this is the classical result).

**P2**: The Disc<0 case (AABB can underestimate LW for near-cardinal districts with minor appendages) is noted in §06 as occurring in "<5% of districts." No citation or proof for this frequency claim; it is presented as empirical but daggered only implicitly.

#### R4 — Stephanopoulos: 2/4

**P1**: The section title "The LW>3 Court Threshold" and the abstract Claim 1 language "documented court threshold" implies courts have established a threshold. The evidence cited is: (a) Illinois expert reports in 7th Circuit filings (not court holdings), citing Niemi et al. 1990 (an academic paper, not a court opinion); (b) New York state court analysis citing compactness criteria and expert reports. The caveat in §4.4 admits "no federal court has established LW>3 as a binding legal threshold." For a law review audience, section 4's title and abstract framing are misleading. Should read "LW>3 as a practitioner threshold" or "LW>3 in expert reports."

No invented case citations; all cited cases are real.

#### R5 — Liang: 3/4

The implementation matches: `length_width_ratio()` (lines 135–204) implements rotating calipers on the convex hull. The test `test_lw_ratio_2to1_rectangle_is_2` passes, `test_lw_ratio_square_is_1` passes. The rotated-rectangle extreme case discussed in §03 would test as: a 45°-rotated 1×3 rectangle, the AABB would give LW≈1.0, the MBR would give LW=3.0—consistent with the paper's example.

**Verdict**: Minor Revision (2.6). One P1: "court threshold" framing in abstract and section title overstates the evidence (no binding court holding establishes LW>3; only expert-report practice). Fix: retitle §4 as "LW>3 in Expert Reports and Practitioner Usage" and soften abstract Claim 1 from "documented court threshold" to "practitioner threshold documented in expert reports."

---

### K.6 — Population-Weighted Compactness

**Paper**: *Population-Weighted Compactness in Algorithmic Redistricting: Moment of Inertia, Prime-Factor Advantage, and Representational Independence*

#### R1 — Karypis: 3/4

The PWC formula is the population-weighted second moment of inertia—correct. The proof that PWC≥0 with equality iff all tracts share the same centroid is correct. The scale-dependence proposition (scaling by λ scales PWC by λ²) is correct and important.

**P2**: The implementation uses district_centroid as a caller-provided argument rather than computing it internally. This means the caller must supply the correct population-weighted centroid; if they supply the geometric centroid instead, PWC is computed against the wrong reference point. The paper does not note this API design decision or its correctness implications.

#### R2 — Rodden: 3/4

The orthogonality claim (PWC–PP correlation r<0.4 across all algorithm-state pairs) is a meaningful empirical finding. The prime-factor PWC advantage (aligned with Fryer-Holden) is well-motivated. The VRA framing in §07 is careful: "while no state has explicitly adopted PWC as a formal metric, the underlying concept (residents clustered together) is widely used."

**P2**: Rodden would note that PWC's relationship to geographic sorting (urban cores vs. suburban/rural distributions) is not discussed. TX's high PWC variance is attributed to "concentrated urban centres (Dallas-Fort Worth, Houston, San Antonio-Austin)"—this is accurate, but the paper could note that partisan composition of these centres affects whether PWC reduction is politically neutral.

#### R3 — Duchin: 3/4

The PWC definition is mathematically precise. The translation-invariance (trivial) and scale-dependence (λ²) propositions are correct. The normalisation by state area for cross-state comparison (PWC_norm = PWC/state_area) is mentioned and is a correct approach.

**P1**: §06-prime-factor contains an internal numerical contradiction. Abstract Claim 2 states "prime-factor reduces mean PWC 18% below standard-bisect on NC 2020." The synthesis paper K.0 and K.7 table repeat "18%." But §06-prime-factor of K.6 itself computes:

  reduction = (72.1 − 62.1)/72.1 = 10.0/72.1 ≈ 0.139 ≈ 14%.

The section then says: "(The spec states 18%; the exact value depends on the normalisation convention. Using raw PWC in square metres rather than km², the reduction is approximately 17–18% due to rounding in km² conversion.)"

This is a self-contradiction within K.6: the paper's own arithmetic gives 14% but the abstract says 18%, and the paper explains this away by claiming a rounding difference between km² and m². This explanation is insufficient—km² conversion is a fixed factor of 10⁶ applied uniformly to all values, which cannot change a ratio. The 14% vs. 18% discrepancy is unresolved.

#### R4 — Stephanopoulos: 3/4

The legal and practitioner section is appropriately hedged. PWC is correctly described as not yet a standard litigation metric. The Fryer-Holden citation is real. The community-of-interest framing is accurate.

**P2**: The claim that PWC provides a "spatial operationalisation" of community-of-interest standards is plausible but not verified against any specific state commission's criteria. The paper hedges with "while no state has explicitly adopted PWC as a formal metric"—adequate.

#### R5 — Liang: 3/4

The `population_weighted_compactness()` function (lines 224–249 of compactness.rs) matches the paper's formula exactly: weighted sum of squared distances from supplied district_centroid, divided by total_pop. The tests (single-point PWC=0, two-equidistant-equal-pop PWC=d², population-weighted correctly) all check out algebraically.

The function is correctly excluded from `all_metrics()` because it requires tract-level data unavailable from polygon geometry alone—the paper notes this. The JSON output format in K.7 §05 (pwc_km2) implies unit conversion happens in the output layer, not in this function.

**Verdict**: Accept (3.0). One P1: the abstract claims 18% PWC reduction but the paper's own arithmetic gives 14%, with an invalid excuse (km²/m² rounding cannot change a ratio). Fix: either verify the 18% figure from raw m² computation or correct the abstract to 14%.

---

### K.7 — Composite Court Guide

**Paper**: *Multi-Metric Compactness Composite and Court Usage Guide: A Practitioner Reference for Expert Witnesses, Special Masters, and Redistricting Commissions*

#### R1 — Karypis: 3/4

The court survey table (Table 1) is a synthesis of metrics × cases. The composite profile table (Table 2, all six metrics for all four algorithms, NC 2020) is internally consistent with values reported in K.1–K.6. The JSON output schema matches what one would expect from the implementation.

**P2**: The "acceptable range" flags in the JSON (PP≥0.18, Reock≥0.30, CH≥0.82, S≤2.4, LW≤2.8) are described as "conservative thresholds from K.1–K.5." None of these thresholds are justified with a specific derivation or legal citation in this paper; they are implied to come from the respective metric papers. K.1 through K.5 do not formally derive these thresholds either—they emerge from empirical observation. A practitioner using these thresholds for court filings should know they are heuristics.

#### R2 — Rodden: 3/4

The composite profile methodology is sound. The recommendation to include all six metrics is well-motivated. The cross-state summary table (NC/WI/TX for ratio-optimal) is consistent with values in the individual metric papers.

**P2**: The court filing template includes the phrase "placing it in the [percentile] of algorithmically-generated plans for [State]." This percentile claim requires ensemble runs. For a single-seed plan (seed=42†), the percentile is undefined without a reference distribution. The template notes this in the "Percentile language" subsection, but the main template text should carry the caveat inline rather than in a footnote.

#### R3 — Duchin: 2/4

**P1**: The simulated panel (§07) presents 89% inter-judge agreement as a finding that "composite profiles achieve 89% inter-judge agreement...versus 67–74% for any individual metric." The panel methodology (20 plans, 12 reviewers, 3 reviewer roles) is described as "simulated." The ground truth is defined as "the bisect L2 acceptable-range criteria"—i.e., the study's own algorithm's output determines what is "compact." This creates circularity: the finding measures whether reviewers agree with bisect's own classification, not whether composite profiles improve legal outcomes.

The 89% figure cannot be independently verified (no data appendix, no reviewer recruitment details, no protocol). The paper hedges in §07 ("this panel exercise is simulated, not a real court proceeding"), but abstract Claim 2 presents the 89% as a headline finding ("a composite six-metric profile achieves 89% inter-judge agreement...versus 67–74% for any individual metric"). This is overclaiming relative to what a simulated, auto-graded exercise can establish.

**P1**: The Reock≥PP Proposition from K.2 is repeated in the context of explaining Table 3 (K.7 §03-composite): the table shows Reock>PP in all cells (e.g., ratio-optimal NC: PP=0.22, Reock=0.37). This is consistent with the empirical data but the explanatory prose in K.2 and K.7 links to a false proposition. The composite paper inherits the false claim.

#### R4 — Stephanopoulos: 2/4

**P1**: Claim 3 in the abstract: "the bisect label-analyze --types all command produces all six metrics for every district in a certified JSON appendix ready for court filing without post-processing." The "certified" language implies legal sufficiency. The paper later (§05) shows that the JSON includes a SHA-256 hash chain via `bisect label-verify`. However, "certified JSON appendix ready for court filing" implies court acceptance, which depends on jurisdiction, expert qualifications, and judicial discretion. No court has accepted bisect output as a self-certified filing. The language should read "authenticated" or "cryptographically verifiable" rather than "certified...ready for court filing."

The court survey table is accurate—all cited cases exist and the usages described match the public record. The Colorado IRC entry is accurate. No invented citations.

**P2**: The template expert language ("I certify that the compactness values reported herein were computed using version [X.Y.Z] of the bisect platform") appropriately puts the certification on the expert witness, not on the software. But the abstract's phrasing attributes the certification to the command output itself—a subtle but legally meaningful difference.

#### R5 — Liang: 3/4

The CLI commands and JSON output format are internally consistent. The `within_acceptable_range` boolean field in the JSON is described and explained. The `bisect label-verify` command for SHA chain verification exists per CLAUDE.md. The `bisect label-report --format csv` command is documented.

**P2**: The JSON schema shows `pwc_km2` as a field. The implementation's `population_weighted_compactness()` returns values in square metres (the function comment says "Units: square metres"). The km² conversion must happen somewhere in the output layer. The paper does not specify where or how the m²→km² conversion is applied, which is a reproducibility gap.

**Verdict**: Minor Revision (2.6). Two P1s: (1) simulated panel presented with headline percentages that imply external validation when the ground truth is the algorithm's own thresholds (circular); (2) "certified...ready for court filing" in abstract overstates what the tool provides. Fix: soften Claim 3 to "authenticated via SHA chain" and add a caveat that court admissibility depends on expert qualification; add a methodology caveat to the 89% panel claim in the abstract.

---

## Cross-Paper Issues

### P1: Reock Implementation Mismatch (affects K.0, K.1, K.2, K.6, K.7)

The single most significant P1 in the batch. K.2 §3 describes and proves Welzl's algorithm, claims it is implemented in `crates/bisect-analysis/src/compactness.rs`, and states the L0 invariant "Welzl MBC contains all input points." The actual implementation uses `centroid + max_distance_to_boundary`, which:
- Is NOT Welzl's algorithm
- Does NOT guarantee an exact MBC
- Can produce a circle that does NOT contain all polygon vertices for asymmetric shapes
- The code's own comment acknowledges this: "not the true MBC via Welzl's algorithm, but matching Python's approximation exactly"

Every paper that cites "exact Reock via Welzl" inherits this inaccuracy. The quantitative Reock results in the tables (which are computed with the approximation) may differ from true Welzl-based Reock values for asymmetric districts. The direction of error is that the approximation overestimates the bounding-circle radius (since the centroid is not the MBC centre for asymmetric shapes), producing systematically lower Reock scores than the true MBC would give. All Reock headline numbers should carry a note that they are approximations.

**Required fix**: K.2 must accurately describe the centroid+max-dist approximation, retract the Welzl Theorem as describing the implementation, and add a note that the approximation matches the Python pipeline for cross-platform reproducibility. All papers citing Reock via Welzl must add the approximation caveat.

### P1: Reock≥PP Proposition (affects K.2, K.1, K.0)

K.2 states without proof: "Reock(D)≥PP(D) for any convex polygon D." K.1 §06 computes a 1×6 rectangle and finds Reock≈0.21, PP≈0.38 — contradicting the Proposition. K.2's stated justification says "the proof is omitted as it requires multiline geometric argument beyond the scope of this paper," but no such proof exists and the claim is false. The Proposition must be retracted.

### P1: PWC 18% vs. 14% (affects K.6, K.0, K.7)

K.6 abstract claims 18% PWC reduction. K.6 body computes 14%. The excuse (km²/m² rounding) is mathematically invalid. Must be corrected to whichever figure the raw computation supports.

### P2: Single-seed dagger notation

All papers consistently apply the † notation to single-seed results and include "All results use single seed=42†" in their abstracts. This is well-executed across the track. No violations found.

### P3: Binary naming

All papers consistently use `bisect` (the renamed binary) in CLI examples. No references to the old `redist` binary found. Clean.

---

## Revision Priority List

1. **K.2** (Major Revision, must fix before publication): Retitle §3 as "MBC Approximation," describe centroid+max-dist accurately, retract Welzl Theorem as an implementation claim, retract Reock≥PP Proposition.

2. **K.5** (Minor Revision): Retitle §4 from "Court Threshold" to "Expert-Report Threshold." Soften abstract Claim 1.

3. **K.7** (Minor Revision): Soften abstract Claim 3 ("certified...ready for court filing" → "authenticated via SHA chain"). Add methodology caveat to 89% panel claim.

4. **K.6** (Accept, but fix before camera-ready): Resolve 18% vs. 14% discrepancy. Correct abstract and synthesis cross-references.

5. **K.0, K.1** (Accept, minor): Add caveat that Reock is computed via centroid+max-dist approximation (not exact Welzl). Retract Reock≥PP references.
