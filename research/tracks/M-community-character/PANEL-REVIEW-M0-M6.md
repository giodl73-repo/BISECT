# Panel Review: M.0 and M.6
**Date**: 2026-05-08
**Panel**: 5-member hostile review (Karypis, Rodden, Duchin, Stephanopoulos, Liang)
**Post-write status**: Both papers P1-fixed before this review

---

## Summary Table

| Paper | Avg Score | Verdict | P1 Count | Top Issues |
|-------|-----------|---------|----------|------------|
| M.0 — Community Character Framework | 2.7 | Minor Revision | 1 | Multiplicative formula vs alpha-blend not reconciled in body; CA code cite; "first definition" over-claim |
| M.6 — Administrative Zone Co-membership | 2.6 | Minor Revision | 1 | Additive/multiplicative divergence not explicit enough in §1.3; MA Constitution fix confirmed but §1.3 language still incomplete; all quantitative claims deferred |

---

## M.0 — Community Character Weighting: Framework and Legal Grounding

### R1 — Karypis (Algorithms) — Score: 3 / 4

**Summary**: The cosine similarity formula is mathematically correct. The non-negativity
proof (all c_i ≥ 0 implies dot product ≥ 0) is valid. The graceful degradation property
is correctly stated. The zero-vector handling (sim = 1 substitution) is documented in
§2.4–2.5 and is conservative in the right direction. The §2.6 invariant list no longer
claims the erroneous 2×w_base upper bound for the multiplicative formula; it correctly
notes that the 2× bound belongs to M.6's additive variant. These are the P1 fixes and
they are present.

**Remaining concerns (minor revision)**:

[R1-M0-A] The paper still presents two operationally different formulas without designating
one as canonical. Section 2.3 defines the weight modifier as
$w(u,v) = w_{\mathrm{base}} \times \mathrm{sim}(\mathbf{c}_u, \mathbf{c}_v)$
(multiplicative, range $[0, w_{\mathrm{base}}]$). Section 5.2 then defines the operational
formula as
$w_{\mathrm{final}}(u,v) = \alpha \cdot w_{\mathrm{base}} + (1-\alpha) \cdot w_{\mathrm{char}}$
(alpha-blend, range $[w_{\mathrm{base}} \cdot \min(\mathrm{sim}), w_{\mathrm{base}}]$).
These are not equivalent. The §2.6 invariants list ("weight_bounded_above: w ≤ w_base
under the multiplicative formula") is now technically accurate, but the invariants are
stated as L0 tests "checked on every build." Which formula is the implementation testing —
the §2.3 multiplicative formula or the §5.2 alpha-blend formula? The paper does not say.
A future implementer reading §2.6 to write a unit test will not know which formula to
exercise. The fix is one sentence: "The §5.2 alpha-blending formula is the operational
formula; §2.3 is a conceptual intermediate that the alpha-blend formula generalizes."

[R1-M0-B] The zero-vector claim "sim = 1 substitution is conservative" (§2.4–2.5) is
asserted but not argued. Conservative here means it does not create artificially cheap cuts.
For the multiplicative formula this is immediate (sim = 1 means the cut cost equals baseline,
which is at least as high as any real similarity would produce if the zero vector were replaced
by a unit vector in any direction). A single sentence proof would close this gap.

[R1-M0-C] The §1 Contribution 1 claim ("the first precise mathematical definition of
community of interest as cosine similarity") is not defended against Altman and McDonald
(2011), which is in the bibliography. Either compare or weaken the superlative.

**Score rationale**: Formula correctness is sound. Two structural issues remain (formula
identity ambiguity, zero-vector argument gap) but neither is a mathematical error. Score 3.

---

### R2 — Rodden (Political Science) — Score: 2 / 4

**Summary**: The partisan neutrality claim is the paper's most important policy assertion
and it is poorly substantiated for a political science audience.

**Major concerns**:

[R2-M0-A] The paper states (§1, §3, §6) that the framework is "partisan-neutral" because
no electoral data enters the character vector. This is a claim about inputs, not outputs.
No evidence is presented that the resulting edge weights do not systematically favor one
party. Community character signals are geographically correlated with partisan alignment:
occupational mix (M.1) correlates with education and partisan sorting; land use (M.2)
correlates strongly with urban/rural partisan patterns; school district structure (M.6)
correlates with residential segregation, which correlates with partisanship. The claim
that a formula is neutral because it does not ingest electoral data is a claim that courts
and political scientists would immediately challenge. The paper needs either (a) an
acknowledgment that input-neutrality does not guarantee output-neutrality, with M.8 or
empirical M.1–M.7 papers addressing this gap, or (b) a brief theoretical argument for
why correlational effects average out across signal types.

[R2-M0-B] The paper uses the phrase "the partitioner to be geometrically reluctant to cut"
(§1). This is informal language for an empirical claim about METIS optimization behavior.
METIS minimizes a weighted cut objective; higher edge weights increase the cost of cutting
an edge, but in a combinatorial optimizer with population-balance constraints, "reluctance"
is not a precise property. The paper should say "makes cuts through high-similarity edges
more costly for the METIS optimizer" and not imply a behavioral reluctance.

[R2-M0-C] The framework defers all empirical validation to M.1–M.7. This is stated
explicitly and is appropriate for a framework paper. However, the conclusion's claim
about "expert witness testimony" relies on the framework being empirically effective, not
merely mathematically defined. A framework paper that claims court-ready expert witness
utility while providing zero empirical validation of its effectiveness overpromises.

[R2-M0-D] The abstract states "at least thirty states" use the COI criterion without a
citation. NCSL (2021) is in the bibliography and would serve. This is a minor but easy fix.

**Score rationale**: The neutrality claim is the paper's center of gravity for policy
audiences, and it is defended only at the input level. Output-level neutrality is
unaddressed. Score 2.

---

### R3 — Duchin (Math/Law) — Score: 3 / 4

**Summary**: The mathematical framework is correct. The legal analysis is careful. The
additive/multiplicative distinction between M.0 and M.6 is now handled — §2.6 correctly
flags the M.6 variant — but the description of M.6 in §4.6 still states that zone
co-membership "satisfies all required properties: symmetry, [0,1] range, and graceful
degradation to w_base when no zone data is available." The phrase "graceful degradation to
w_base" is correct for M.6 (zone_score = 0 gives w = w_base), but M.0 defines graceful
degradation differently in §2.3: "when sim = 0, the edge weight collapses to zero." The
§4.6 summary of M.6 does not acknowledge this definitional difference, and a reader who
has absorbed §2.3's definition of graceful degradation will think M.6 degrades to zero
when no zones are shared. It does not.

**Concerns**:

[R3-M0-A] Section 4.6 summary of M.6 uses "graceful degradation" in a meaning
inconsistent with §2.3. Fix: Add one parenthetical to §4.6: "(degradation here means
the weight returns to w_base rather than zero, because M.6 uses an additive boost rather
than multiplicative attenuation; see M.6 §3.2 for the distinction)."

[R3-M0-B] The symmetry property proof (§2.4) relies on "the dot product is symmetric."
This is correct for cosine similarity on any vectors, not just non-negative ones. The
non-negativity requirement is load-bearing only for the [0,1] range claim. The paper
correctly identifies this but could be clearer about which assumptions are needed for
which properties.

[R3-M0-C] The alpha-blending formula in §5.2 substitutes $w_{\mathrm{char}} = w_{\mathrm{base}} \times \mathrm{sim}$.
Expanding:
$w_{\mathrm{final}} = \alpha w_{\mathrm{base}} + (1-\alpha) w_{\mathrm{base}} \mathrm{sim}
= w_{\mathrm{base}} [\alpha + (1-\alpha)\mathrm{sim}]$.
The range of $w_{\mathrm{final}}$ is $[w_{\mathrm{base}} \cdot \alpha, w_{\mathrm{base}}]$
(minimum achieved at sim=0, maximum at sim=1). For $\alpha = 0.5$, the minimum is
$0.5 \cdot w_{\mathrm{base}}$. The paper claims graceful degradation means the weight
stays at $w_{\mathrm{base}}$ when all tracts are identical (sim=1), which is correct.
But it does not state the lower bound, which matters for METIS: with alpha=0.5 and
sim=0, the minimum edge weight is half the base weight, not zero. This is different
from the §2.3 multiplicative formula where the lower bound is zero. The paper should
state the lower bound explicitly.

**Score rationale**: Framework properties are correctly stated after P1 fixes. Remaining
issues are precision gaps (graceful degradation terminology in §4.6, lower bound of
alpha-blend formula) rather than errors. Score 3.

---

### R4 — Stephanopoulos (Law) — Score: 3 / 4

**Summary**: Citation audit complete. All Supreme Court citations are real and correct.
State statutory citations are correct or have been corrected. One remaining issue.

**Citation audit**:
- Reynolds v. Sims, 377 U.S. 533 (1964): REAL. Holding quoted accurately ("maintain
  the integrity of various political subdivisions"). PASS.
- Shaw v. Reno, 509 U.S. 630 (1993): REAL. Characterization accurate (race cannot be
  predominant factor). PASS.
- Miller v. Johnson, 515 U.S. 900 (1995): REAL. "Overriding, predominant force" language
  is the correct Miller formulation. PASS.
- Milliken v. Bradley, 418 U.S. 717 (1974): REAL. Used to support school district
  community legitimacy. The Stewart concurrence quote ("No single tradition in public
  education is more deeply rooted than local control") appears in the opinion. This use
  of Milliken is creative but not incorrect; the paper correctly acknowledges it is not a
  redistricting case. PASS with caveat (see [R4-M0-A] below).
- Cal. Gov. Code §8252(c): CORRECTED from earlier draft's "Cal. Elec. Code §21601(c)."
  The Government Code citation is correct post-2011 codification. The paper now cites
  "Cal. Gov. Code §8252(c), as enacted by Proposition 11 (2008)." PASS.
- CO Art. V §44: REAL. Amendment Z (2018) codified at this provision. PASS.
- AZ Const. Art. IV, Pt. 2, §1(14): REAL. Prop 106 (2000) establishes the IRC at this
  provision. PASS.
- WA RCW 44.05.090: REAL. Washington's Redistricting Commission governed by this
  statute. PASS.

**Remaining concerns**:

[R4-M0-A] The use of Milliken v. Bradley to support school district boundary legitimacy
in a redistricting context remains the weakest cite in the paper. Milliken's holding was
about the scope of cross-district desegregation remedies; the proposition being supported
("school districts represent genuine local communities") requires a redistricting cite.
Abrams v. Johnson, 521 U.S. 74 (1997), accepted school district preservation as a
legitimate state interest in the Georgia redistricting context and would be a stronger
primary authority. The current Milliken cite is not wrong but is not best practice.
Recommend: elevate Abrams v. Johnson as the primary cite for this proposition and
retain Milliken as secondary support for the broader civic significance of local school
governance.

[R4-M0-B] The "quantitative neutral defense" analogy to employment discrimination law
(§3.5) is creative and interesting but cites no redistricting case in which a court
accepted this defense. The paper presents this as presumptively lawful based on analogy.
A footnote acknowledging the analogy is not yet tested in redistricting litigation would
improve credibility.

[R4-M0-C] Rucho v. Common Cause (2019) is in the bibliography but not cited in the body.
The paper's argument for algorithmic neutrality is strengthened by the observation that
after Rucho, federal courts cannot review partisan gerrymandering claims, making
state-level algorithmic neutrality standards the primary protection. One sentence in §3
citing Rucho for this proposition would strengthen the legal argument.

**Score rationale**: Core legal citations corrected (CA code fix confirmed). All four
Supreme Court cases verified as real and cited for accurate holdings. Remaining issues
are strengthening opportunities, not errors. Score 3.

---

### R5 — Liang (ML/AI) — Score: 2 / 4

**Summary**: The cosine similarity formula is standard and correct. The zero-vector
edge case is handled. However, the paper is a specification document for a system that
has not been implemented or tested empirically, and several ML-relevant design choices
are not validated.

**Concerns**:

[R5-M0-A] The paper defines character vectors as $\mathbf{c}(t) \in \mathbb{R}^k_{\geq 0}$
where $k$ depends on which signals are active. When multiple signals are active (M.1–M.7
combined), the vector space has variable dimension and the signals are on different scales
(occupational shares from LODES are in $[0,1]$; topographic zone codes from M.5 are
categorical; transit isochrone overlap from M.7 is in $[0,1]$). Concatenating these into
a single cosine similarity computation requires that all dimensions be on comparable scales
or that the cosine similarity be insensitive to scaling differences. The paper does not
address scale normalization. For a single-signal M.0 application (M.1 alone, M.6 alone),
this is not an issue. For the M.8 composite, it is a serious concern.

[R5-M0-B] The zero-vector handling specifies sim = 1 as the substitution for tracts
where $\mathbf{c}(t) = \mathbf{0}$. This is described as "conservative." It is conservative
in the sense that it does not create cheap cuts, but it is not conservative in the sense
of being unbiased: it treats a zero-vector tract as perfectly similar to all its neighbors,
which means cuts adjacent to such tracts will have full base weight rather than reduced
weight. In a state with many zero-vector tracts (e.g., entirely commercial tracts for
an M.1 occupational signal), this substitution would systematically treat an entire
neighborhood as high-community-similarity regardless of its actual relationship to
neighbors. The paper should quantify the expected frequency of zero-vector tracts for
each signal type or argue that it is negligible.

[R5-M0-C] Cosine similarity is a reasonable choice for compositional data (proportions),
where orientation matters more than magnitude. However, for some signals (e.g., binary
zone co-membership in M.6), cosine similarity is less appropriate than set-theoretic
measures (Jaccard, ratio). The paper correctly uses ratio similarity for M.6 and Jaccard
for M.4, which is appropriate. But the abstract says the framework uses "cosine similarity
of tract-level character vectors" without qualification. A reader who applies cosine
similarity to the M.6 zone co-membership signal (which is binary categorical, not
continuous compositional) would get a different result than the ratio formula. The abstract
should qualify: "cosine similarity for continuous compositional signals; ratio- and set-based
measures for categorical signals."

[R5-M0-D] The claim of "graceful degradation to the unweighted baseline when all tracts
are identical" (§2.4, §2.6) is correct for the multiplicative formula. It is also correct
for the alpha-blend formula: if all tracts are identical, sim = 1 everywhere, so
$w_{\mathrm{final}} = \alpha w_{\mathrm{base}} + (1-\alpha) w_{\mathrm{base}} = w_{\mathrm{base}}$.
This is fine. But the degradation condition "all tracts identical" is a very strong
condition. A more practically relevant degradation would be "when the signal has no
geographic variation" (i.e., all edge similarities are equal, though not necessarily 1).
Under that condition, all edge weights scale uniformly and the METIS partition is unchanged
relative to baseline (because METIS minimizes cut cost, and uniform scaling preserves the
relative costs). The paper should clarify whether this stronger form of graceful degradation
holds, as it is more relevant to the use case.

**Score rationale**: The formulas are correct and the overall ML design is sound, but
the framework leaves too many design choices unspecified (scale normalization, zero-vector
frequency, similarity metric selection for categorical signals) for an ML reviewer to
consider it implementation-ready. Score 2.

---

## M.0 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| R1 Karypis | 3 | Minor Revision |
| R2 Rodden | 2 | Major Revision |
| R3 Duchin | 3 | Minor Revision |
| R4 Stephanopoulos | 3 | Minor Revision |
| R5 Liang | 2 | Major Revision |
| **Average** | **2.6** | **Minor Revision** |

**P1 for revision**: One remaining structural issue after P1 fixes — the duality of
§2.3 formula vs §5.2 formula is flagged by three reviewers (R1, R3, R5) and needs a
single disambiguating sentence designating the §5.2 alpha-blend as the operational
formula.

---

---

## M.6 — Administrative Zone Co-membership Edge Weights

### R1 — Karypis (Algorithms) — Score: 3 / 4

**Summary**: The zone co-membership formula is mathematically correct. The additive
boost structure ($w = w_{\mathrm{base}} \times (1 + \alpha \times \mathrm{zone\_score})$)
is well-motivated and the boundary cases (score=0 and score=1) are derived correctly.
The §3.2 Remark correctly identifies the departure from M.0 and gives a rationale.
The TIGER SDSD / UNSD distinction is fixed in §4.4. The Massachusetts Constitution
citation is corrected to Amendment 101 (1978). These P1 fixes are confirmed.

**Remaining concerns (minor revision)**:

[R1-M6-A] The §1.3 (Relationship to M.0) still states that M.6 "satisfies the same
mathematical properties (symmetry, [0,1] range, graceful degradation) but is more
directly interpretable for legal purposes." The phrase "graceful degradation" is used
with a different meaning here than in M.0 §2.3. M.0 §2.3 defines graceful degradation
as "when all tracts are identical, w = w_base." M.6's graceful degradation means "when
no zone data is available, |Z| is reduced gracefully." These are different degradation
properties. The statement "satisfies the same mathematical properties" is misleading
for the sim=0 / zone_score=0 behavior: under M.0's multiplicative formula, sim=0 makes
cuts free (w=0); under M.6's additive formula, zone_score=0 leaves w = w_base (not free).
The §1.3 statement should add: "Note that M.6 does not satisfy M.0's free-cut property
(sim=0 makes w=0); instead, M.6's minimum weight is w_base, not zero."

[R1-M6-B] The SDSD download URL is given as
`https://www2.census.gov/geo/tiger/TIGER{year}/SDSD/`.
This URL pattern should be verified as active for recent TIGER vintages. The TIGER/Line
URL structure has changed across years; some vintages use `TIGER{year}` and others use
`TigerYear{year}`. A note that the exact URL should be confirmed from the TIGER/Line
documentation page (census.gov/geo) would prevent a reproducibility failure if the URL
pattern changes.

[R1-M6-C] The invariant `school_district_split_never_increases` (§3.4) is stated as an
L0 unit test but it is not actually an L0 (per-build) testable invariant without running
redistricting. It is an L1 (integration) test: it requires running bisect on a state with
both geographic and zone-membership weights and comparing cut counts. The paper should
correct the test level from L0 to L1.

[R1-M6-D] The §3.5 YAML example lists `zone_types: [school_district, county_subdivision, electric_utility, fire_district, water_district]` (5 types) but the paper defines 6 zone types.
Police precinct is the omitted type. The YAML should include a comment: `# police_precinct excluded from default; add if city GIS data is available`.

**Score rationale**: Formula is correct. TIGER fix confirmed. Remaining issues are
documentation precision, not formula errors. Score 3.

---

### R2 — Rodden (Political Science) — Score: 2 / 4

**Summary**: The political science concern is that all quantitative claims are deferred
and the partisan neutrality claim is not demonstrated for this signal, which has known
correlations with racial and partisan residential patterns.

**Major concerns**:

[R2-M6-A] All three headline quantitative claims are explicitly deferred:
(1) NC-14 zero splits of co-membered tract pairs, (2) school district boundary crossings
≤20% of geographic baseline, (3) Massachusetts town integrity ≥95%. The paper is an
implementation specification, not a results paper. This is clearly labeled. However, the
abstract and conclusion both use present-tense language ("making cuts through administratively
co-membered tract pairs geometrically costly") that implies the system works as described.
Until at least one pilot run exists, these should use future-tense or conditional language.

[R2-M6-B] School districts in the United States — particularly unified school districts
in the South — were historically drawn along racially segregated lines. In states such as
North Carolina (the paper's primary validation target), school district boundaries reflect
historical segregation patterns that correlate strongly with both racial composition and
partisan alignment. A formula that treats school district co-membership as partisan-neutral
"by construction" does not account for the correlation between school district structure
and partisan sorting. The paper needs at least an acknowledgment of this issue, with a
pointer to how M.0's Shaw-Miller analysis handles the incidental-correlation case.

[R2-M6-C] The claim that zone co-membership is "the most legally defensible community
character signal in the M-track" (abstract, §7) is unsupported comparatively. M.4
(commuting shed) and M.3 (housing character) have also been cited in redistricting
commission proceedings. Without a comparative analysis, the superlative is undefended.

[R2-M6-D] The paper cites "expert witnesses... in proceedings across California, Colorado,
Virginia, and North Carolina" as having cited school district integrity as a COI criterion,
but provides no specific case or proceeding citation. The California CRC 2011 and 2021
proceedings have public transcripts; a specific cite would eliminate this gap.

**Score rationale**: Strong legal framing but all empirical claims deferred and the
partisan neutrality assertion is not defended for signals with known partisan correlations.
Score 2.

---

### R3 — Duchin (Math/Law) — Score: 3 / 4

**Summary**: The additive/multiplicative reconciliation is handled in §3.2 Remark. The
formula properties are correctly proven. The Massachusetts Constitution citation is
corrected. One structural issue remains in §1.3.

**Concerns**:

[R3-M6-A] Section 1.3 says M.6 satisfies "the same mathematical properties (symmetry,
[0,1] range, graceful degradation) independently." The Remark in §3.2 correctly explains
the additive/multiplicative distinction. But §1.3 comes before §3.2, and a reader reading
linearly will reach §1.3 first. At §1.3 they read that M.6 satisfies "the same properties"
as M.0 without knowing yet that M.6 uses a different formula. By §3.2 they learn the
formula is additive. The structural problem is that the correction comes after the
potentially misleading claim. Fix: add a forward reference in §1.3: "M.6 uses an additive
boost modifier ($w = w_{\mathrm{base}} \times (1 + \alpha \times \mathrm{zone\_score})$,
detailed in §3.2) rather than M.0's multiplicative scaling; the key difference is that
M.6's minimum weight is $w_{\mathrm{base}}$ (not zero)."

[R3-M6-B] The [0,1] range claim for zone_score is trivially proven by the formula
(ratio of non-negative integers with denominator ≥ numerator). This is fine. The paper
should note explicitly that |Z| ≥ 1 is guaranteed by the county fallback, so the
denominator is never zero. This is mentioned in §3.4 ("always including county
co-membership as a fallback") but should also appear in the proof of [0,1] range.

[R3-M6-C] The paper's §5 legal analysis cites Reynolds v. Sims for the "integrity of
various political subdivisions" language and correctly notes this as the constitutional
anchor. The analysis is technically correct. However, Reynolds addressed state legislative
apportionment; the primary validation target is NC-14 (congressional districts). The
legal standard for congressional districts comes from Wesberry v. Sanders (1964), not
Reynolds. While the communities of interest doctrine is largely developed at the state
legislative level and imported to congressional redistricting by analogy, a paper targeting
a legal audience should acknowledge this gap and explain why the Reynolds doctrine applies
to congressional redistricting (it does, by analogy, but the connection should be explicit).

**Score rationale**: Formula and property proofs are correct. §1.3 forward-reference
gap is a minor structural issue. Reynolds/Wesberry gap is a legal precision point.
Score 3.

---

### R4 — Stephanopoulos (Law) — Score: 3 / 4

**Summary**: Citation audit complete. Supreme Court citations are all real and accurate.
The Massachusetts Constitutional citation is corrected. EIA and TIGER citations verified.

**Citation audit**:
- Reynolds v. Sims, 377 U.S. 533 (1964): REAL. Quote from §5.1 verified as in opinion.
  PASS.
- Shaw v. Reno, 509 U.S. 630 (1993): REAL. PASS.
- Miller v. Johnson, 515 U.S. 900 (1995): REAL. PASS.
- Milliken v. Bradley, 418 U.S. 717 (1974): REAL. Stewart concurrence quote verified.
  PASS.
- Rucho v. Common Cause, 588 U.S. 684 (2019): In bibliography but not cited in body.
  (Not a legal error, just an unused resource. See [R4-M6-B].)
- Mass. Const. Amend. Art. 101 (1978): CORRECTED from "Part II, Ch. 1, §3" to
  "Amendment 101 (1978)" in §6.5. The paper now cites "Mass. Const. Amend. Art. 101
  (1978)" with language about "not dividing a city or town" except where necessary for
  population equality. This is accurate. PASS.
- EIA Form 861: REAL. Annual Electric Power Industry Report. The §5.4 reference to the
  Energy Policy Act of 2005 is accurate. PASS.
- TIGER/Line SCHOOLDISTRICT, COUSUB, SDSD: These are the correct TIGER/Line layer
  names. SDSD fix confirmed. PASS.

**Remaining concerns**:

[R4-M6-A] The Milliken use (§5.2) to support school district boundary legitimacy carries
the same issue as in M.0: it is not a redistricting case. Abrams v. Johnson, 521 U.S.
74 (1997), would be more direct. This is a strengthen-the-cite issue, not an error.

[R4-M6-B] The paper does not cite Rucho v. Common Cause (2019) in the body despite
having it in the bibliography. In the context of §5 (legal analysis), noting that federal
courts cannot review partisan gerrymandering claims after Rucho reinforces why a
state-level algorithmic standard with quantitative defensibility is valuable. One sentence
would improve the legal narrative.

[R4-M6-C] The §5.5 claim that Arizona's IRC "has accepted expert testimony about
administrative zone co-membership as evidence that communities of interest have been
preserved" has no supporting citation. The Arizona IRC proceedings (2001, 2011, 2021)
have public records. Either cite a specific proceeding or change "has accepted" to
"is empowered to accept."

[R4-M6-D] The Levitt (2011) Brennan Center citation (§2.1) for school district
preservation as a COI operationalization is real. However, the 2011 guide may have been
superseded by later Brennan Center publications. The paper should confirm which edition
and page contains the specific school district language, or cite the passage directly.

**Score rationale**: Core citations corrected or verified. Remaining issues are
strengthen-the-cite opportunities and one unsupported commission claim. Score 3.

---

### R5 — Liang (ML/AI) — Score: 2 / 4

**Summary**: The zone co-membership formula is simple, correct, and appropriate for
categorical zone membership data. The formula's properties are proven correctly. The ML
design concerns center on edge case handling, consistency with M.0's similarity interface,
and the unverified empirical claims.

**Concerns**:

[R5-M6-A] The zone_score formula
$\mathrm{zone\_score}(u,v) = |\{z \in Z : \mathrm{zone}_z(u) = \mathrm{zone}_z(v)\}| / |Z|$
is a simple Hamming similarity (proportion of matching categorical features). This is
appropriate for the application. However, the formula treats all zone types equally
(school district co-membership has the same weight as police precinct co-membership).
For the planned NC-14 validation, school district co-membership is arguably much more
legally significant than police precinct co-membership. The equal-weight assumption should
be explicitly motivated or a weighted variant should be mentioned for future work.

[R5-M6-B] The prototypical example values referenced in the §3.2 Remark and experimental
design — specifically the claim that bedroom communities sharing all zones would score
~0.98 and bedroom×commercial pairs would score ~0.11 — do not appear in the current paper
text. The panel note for this review instructed verification of "bedroom=0.98,
bedroom×commercial=0.11" values. These values are NOT in the M.6 paper. If these were
examples from an earlier draft, they were removed. If they are meant to appear as
worked examples (which would strengthen the paper), they should be added as a
Proposition or Example in §3.2. As written, no prototypical numerical examples appear,
which is a gap for an ML audience that expects worked examples to verify the formula.

[R5-M6-C] The centroid-in-polygon assignment (§3.1, §4.6) does not address the case
where a tract centroid falls outside all zone polygons (e.g., a water body tract or
an island tract). The paper notes that point adjacencies (zero-length boundaries) are
excluded from the edge set in M.0, but does not address tracts whose centroids fall
outside all administrative zone polygons. For coastal tracts or island tracts, this is
a non-trivial edge case. The implementation should return |Z| computed only over zones
where the centroid falls within a polygon; tracts with no valid zone assignment for a
given zone type should treat that zone type as missing (|Z| reduced by 1).

[R5-M6-D] The additive boost formula range for arbitrary alpha:
$w(u,v) \in [w_{\mathrm{base}}, (1 + \alpha) w_{\mathrm{base}}]$.
The paper states this correctly for alpha=1 (range $[w_{\mathrm{base}}, 2 w_{\mathrm{base}}]$)
and alpha=2 (range $[w_{\mathrm{base}}, 3 w_{\mathrm{base}}]$). The paper does not state
the general formula. For METIS, the ratio of maximum to minimum edge weight determines
how much the zone signal can influence partitioning relative to geographic variation in
base weights. For the geographic base mode (weights proportional to boundary length),
this ratio is bounded by (1+alpha) from the zone signal, but the baseline ratio of max
to min boundary length across edges can be much larger. A comparison of the zone
signal's dynamic range to the baseline weight dynamic range would help practitioners
set alpha.

**Score rationale**: Formula is correct and appropriate. Two significant gaps remain:
no worked numerical examples (expected by ML audience), and prototypical bedroom/commercial
values referenced in the panel brief are absent from the paper. Score 2.

---

## M.6 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| R1 Karypis | 3 | Minor Revision |
| R2 Rodden | 2 | Major Revision |
| R3 Duchin | 3 | Minor Revision |
| R4 Stephanopoulos | 3 | Minor Revision |
| R5 Liang | 2 | Major Revision |
| **Average** | **2.6** | **Minor Revision** |

**P1 for revision**: One remaining cross-paper issue — §1.3 needs a forward reference
to the additive formula and explicit statement that M.6's minimum weight is w_base not
zero. All three P1 items from post-write confirmed resolved (TIGER SDSD label, MA
Constitution citation, additive formula noted in §3.2 Remark). The §1.3 gap is new
(identified at panel level).

---

## Cross-Paper Issues (M.0 × M.6)

**Issue X-1 (Minor, affects both)**:
M.0 §4.6 describes M.6's graceful degradation as "degrades to w_base when no zone data
is available." M.6 §1.3 says it satisfies "the same mathematical properties" as M.0.
M.0 §2.3 defines graceful degradation as "when sim=0, edge weight collapses to zero."
These are three inconsistent uses of "graceful degradation" across two papers. A
consistent definition should be chosen: either (a) degrade to w_base (M.6's behavior)
and relabel M.0 §2.3 as "collapse to zero" rather than "graceful degradation," or (b)
distinguish "full graceful degradation to zero" (M.0 multiplicative) from "partial
degradation to baseline" (M.6 additive). This is a terminology decision that should be
made once in M.0 §2.4 or §4.6 and propagated.

**Issue X-2 (Minor, affects M.0)**:
M.0 §5.2 alpha-blending formula and M.0 §2.3 multiplicative formula are both presented
as "the" edge weight formula without designating one as the implementation formula. R1
(Karypis), R3 (Duchin), and R5 (Liang) all independently flagged this in M.0. The fix
is one sentence in §2.3 or §5.2.

**Issue X-3 (Already fixed, confirm in both)**:
Cal. Gov. Code §8252(c) — confirmed corrected in M.0 §3.3. M.6 does not separately cite
this provision but defers to M.0 for the California statutory analysis. No action needed
in M.6.

---

## Revision Action List

### M.0 — Required for resubmission

**P1 (must fix before accept)**:
- [R1-M0-A / R3-M0-C / R5] Add one sentence to §2.3 or §5.2 designating which formula
  is the implementation formula: §5.2 alpha-blend is operational; §2.3 is the conceptual
  modifier generalized by §5.2.

**P2 (should fix)**:
- [R1-M0-B] Add one-sentence proof that sim=1 substitution for zero vectors is
  conservative (does not reduce edge weight below baseline).
- [R1-M0-C] Compare to Altman and McDonald (2011) in §1 Contribution 1, or soften
  "first precise mathematical definition" claim.
- [R2-M0-A] Add acknowledgment that input-neutrality does not guarantee output-neutrality;
  forward-reference M.1–M.7 empirical papers for output-level partisan analysis.
- [R2-M0-B] Replace "geometrically reluctant" with "makes cuts more costly for the
  METIS optimizer."
- [R3-M0-A] Fix §4.6 "graceful degradation" terminology: add parenthetical that M.6
  degrades to w_base (not zero) because of its additive formula.
- [R3-M0-C] Add explicit lower bound of alpha-blend formula ($w_{\mathrm{final}} \geq
  \alpha \cdot w_{\mathrm{base}}$) in §5.2.
- [R4-M0-A] Add Abrams v. Johnson (1997) as primary cite for school district boundary
  legitimacy; retain Milliken as secondary.
- [R4-M0-B] Add footnote in §3.5 acknowledging "quantitative neutral defense" analogy is
  untested in redistricting litigation.
- [R5-M0-A] Add note in §2 that composite multi-signal vectors require scale normalization
  before cosine similarity; defer details to M.8.
- [R5-M0-C] Qualify abstract sentence: "cosine similarity for continuous compositional
  signals; ratio/set-based measures for categorical signals."
- [R2-M0-D] Add NCSL (2021) citation for "at least thirty states" COI criterion in abstract.

**P3 (optional)**:
- [R4-M0-C] Cite Rucho v. Common Cause (2019) in §3 for non-justiciability of federal
  partisan gerrymandering claims.
- [R5-M0-D] Discuss whether "uniform similarity" (all edge similarities equal but not 1)
  is also a graceful degradation condition.

---

### M.6 — Required for resubmission

**P1 (must fix before accept)**:
- [R1-M6-A / R3-M6-A] Add forward reference in §1.3 stating M.6 uses an additive
  boost modifier (not multiplicative) and that M.6's minimum weight is w_base not zero.
  One sentence with a forward reference to §3.2.

**P2 (should fix)**:
- [R1-M6-B] Add note that the SDSD URL pattern should be verified from census.gov/geo
  for each TIGER vintage, as URL structure changes across years.
- [R1-M6-C] Correct `school_district_split_never_increases` from L0 to L1 test level
  in §3.4 (requires a redistricting run, not a unit test).
- [R1-M6-D] Add comment in §3.5 YAML: `# police_precinct excluded from default; add
  if city GIS data is available`.
- [R2-M6-B] Add acknowledgment in §5.4 that school district boundaries in Southern
  states correlate historically with residential segregation and thus with partisan
  alignment; note that the formula is input-neutral and that output effects are addressed
  in the experimental design's proportionality gap metric.
- [R2-M6-C] Qualify "most legally defensible" claim: add one sentence comparing to M.4
  (commuting shed) and defending why administrative co-membership is stronger than
  economic similarity for legal defensibility.
- [R2-M6-D] Add specific citation for California or Colorado commission proceedings
  where school district integrity was cited as COI evidence.
- [R3-M6-B] Add explicit note that |Z| ≥ 1 is guaranteed by county fallback, so
  zone_score denominator is never zero.
- [R3-M6-C] Acknowledge in §5.1 that Reynolds addresses state legislative apportionment
  and that its communities-of-interest doctrine extends to congressional redistricting
  by analogy (citing Wesberry v. Sanders, 376 U.S. 1 (1964), for the congressional
  equal-population standard).
- [R4-M6-C] Replace "has accepted" with "is empowered to accept" for Arizona IRC expert
  testimony on zone co-membership (§5.5), or add a specific proceeding citation.
- [R5-M6-B] Add worked numerical examples in §3.2 illustrating zone_score values for
  representative tract pairs (e.g., same school district and utility but different fire
  district: score = 2/3 with |Z|=3). The bedroom=0.98 / bedroom×commercial=0.11
  figures do not currently appear in the paper and should be added or the panel brief
  corrected.
- [R5-M6-C] Add handling note for water body / island tracts whose centroids fall
  outside all administrative zone polygons.

**P3 (optional)**:
- [R4-M6-A] Add Abrams v. Johnson (1997) as primary cite for school district boundary
  legitimacy in redistricting context.
- [R4-M6-B] Cite Rucho v. Common Cause (2019) in §5.4 for non-justiciability of
  federal partisan gerrymandering claims.
- [R4-M6-D] Verify Levitt (2011) Brennan Center guide edition for school district
  preservation language.
- [R5-M6-A] Add future-work note about weighted zone types (school district weighted
  more heavily than police precinct).
- [R5-M6-D] Add note comparing zone signal dynamic range to base weight dynamic range
  for practitioners setting alpha.
