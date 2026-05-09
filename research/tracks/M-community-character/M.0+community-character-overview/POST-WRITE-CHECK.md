# POST-WRITE CHECK: M.0 — Community Character Framework

**Date**: 2026-05-08
**Validator**: research-post-write skill (6-phase pipeline)

---

## PHASE 1 — PAPER SUMMARY

```
Paper: M.0 — Community Character Weighting: Framework and Legal Grounding
Sections found: 00-abstract, 01-introduction, 02-framework, 03-legal,
                04-spectrum, 05-bisect-integration, 06-conclusion
Spec found: YES — docs/specs/2026-05-08-m0-community-character-framework.md
Series: M.0 (framework paper, theoretical reference for entire M-track)
Target audience: algorithms + legal (dual)

Key claims:
  1. Communities of interest can be operationalized as cosine similarity of
     tract-level character vectors with no partisan data required.
     [§2, eq. (2), §6 conclusion — no empirical support, stated as framework design]
  2. The framework is legally neutral under Shaw v. Reno and Miller v. Johnson
     because no racial, partisan, or incumbency data enters the character vector.
     [§3 — legal analysis, not empirical; cited cases are real]
  3. Zone co-membership (M.6) is the most legally direct signal; the alpha-blending
     compositor (default alpha=0.5) composes character weights with any base mode.
     [§4 Table 1, §5 — described consistently]
```

---

## PHASE 2 — CONSISTENCY CHECK

| Q-ID | Quantity | Abstract | §Intro | §Framework/Spectrum | §Bisect | §Conclusion | Consistent? |
|------|----------|----------|--------|---------------------|---------|-------------|-------------|
| Q-01 | "at least thirty states" use COI criterion | YES | NO (not repeated) | — | — | — | WARN: stated only in abstract |
| Q-02 | Formula w(u,v) = w_base × sim(c_u, c_v) | YES (abstract) | YES (eq.1) | YES (eq.3) | YES (§5.2) | YES (§6) | PASS |
| Q-03 | sim ∈ [0,1] (cosine on nonneg vectors) | stated in abstract | stated in §1 | YES eq.(2) + proof | — | — | PASS |
| Q-04 | Zero-vector handling: sim=1 substitution | stated in §2.4–2.5 | NO | YES | — | NO | WARN: absent from abstract and conclusion |
| Q-05 | Alpha-blending formula (§5) | NO | NO | NO | YES (eq.5) | NO | WARN: alpha-blending introduced only in §5, not previewed in abstract or intro |
| Q-06 | Alpha default = 0.5 | NO | NO | NO | YES (§5.3 text + YAML) | NO | WARN: default value only in §5 |
| Q-07 | weight_bounded_above: w ≤ 2×w_base | — | — | YES (§2.6 invariant) | — | — | FLAG: invariant contradicted by alpha-blending formula in §5 |
| Q-08 | Seven signals M.1–M.7, M.8 composite | YES (abstract) | YES | YES (Table 1) | YES | YES | PASS |
| Q-09 | Fire district coverage ~60%, water ~40% | — | — | YES (Table 1 footnote) | — | — | PASS (consistent where stated) |
| Q-10 | TIGER UNSD label for special districts | used in §4.6 | — | YES | — | — | FLAG (see P1 below) |

### Dagger notation
None needed (M.0 is a framework paper with no single-run results).

### Algorithm complexity
No complexity claim made (appropriate for a framework paper).

### CLI flag names
`--weights-override zone-membership` and `--weights-override composite-community` appear in §5.4 CLI block. These match the CLAUDE.md documented flag `--weights-override`. PASS.

### CRITICAL INCONSISTENCY — Q-07 (P1 blocker)

The §2.6 invariant `weight_bounded_above` states:
> $w(u,v) \leq 2 \times w_{\mathrm{base}}(u,v)$ under the multiplicative formula (since $\mathrm{sim} \leq 1$).

But §5.2 (alpha-blending) defines:
> $w_{\mathrm{final}}(u,v) = \alpha \cdot w_{\mathrm{base}} + (1-\alpha) \cdot w_{\mathrm{char}}$

where $w_{\mathrm{char}} = w_{\mathrm{base}} \times \mathrm{sim}$.

Under alpha-blending, $w_{\mathrm{final}} \leq w_{\mathrm{base}}$ (since sim ≤ 1 and both terms are
weighted fractions of w_base × something ≤ w_base). The upper bound is actually $w_{\mathrm{base}}$
(achieved at sim=1 for either alpha=1 or alpha=0), NOT $2 \times w_{\mathrm{base}}$.

The bound $2 \times w_{\mathrm{base}}$ only makes sense under M.6's additive formula
`w = w_base × (1 + alpha × zone_score)`, not under the multiplicative cosine formula.
The §2.6 invariant silently copies M.6's bound into the general framework, which uses
a different formula. This is a mathematical inconsistency.

### CRITICAL INCONSISTENCY — TIGER UNSD label (P1 blocker)

§4.6 states that the M.6 zone co-membership data for fire and water districts comes from
"TIGER/Line Special District files" identified with specific codes. However, the label
"UNSD shapefile with type codes" is used in §4.6 to refer to fire/water special districts.
UNSD is the Census Bureau abbreviation for **Unified School Districts**, not for the
special district layer. The correct file name for special districts in TIGER/Line is
**SLDL/SLDU** (for legislative) or more precisely the **PLACE** and **ELSD/SCSD/UNSD**
series for school districts; fire and water special districts appear in the
**PRIMARYROAD/AREAWATER** adjacent... (actually they appear in TIGER's **SSD** — Special
District) or in a combined shapefile. This label confusion carries across to M.6 §4
data sources section. Both papers should verify the exact TIGER shapefile layer name
for special districts. [NOTE: The M.0 §4.6 summary text says "TIGER/Line Special District
files" which is the correct term; the confusion arises only in M.6 §4.4 which says
"UNSD shapefile with type codes" for fire/water. Flag on M.6 only as P1.]

```
CONSISTENCY: 3 warnings, 1 failure (Q-07 mathematical inconsistency)
P1 (fix before panel): weight_bounded_above invariant (§2.6) is wrong for the
   multiplicative cosine formula — bound is w_base, not 2×w_base. Fix by removing
   the invariant from §2.6 or constraining it to the M.6 additive variant.
P2 (should fix): Zero-vector handling (sim=1 substitution) not mentioned in abstract
   or conclusion.
P2: Alpha-blending and its default value (alpha=0.5) introduced only in §5 with no
   preview in abstract or introduction — readers of abstract don't know the final
   formula involves two composable terms.
P3 (minor): "At least thirty states" COI criterion — stated only in abstract, not
   cited anywhere in the body with a reference.
```

---

## PHASE 3 — CONTRACT CHECK (vs spec docs/specs/2026-05-08-m0-community-character-framework.md)

| Promise (from spec) | Paper section | Delivered? | Gap |
|---------------------|---------------|-----------|-----|
| Algorithm: w(u,v) = w_base × sim(char_u, char_v) | §2 eq.(3) | YES | — |
| Cosine similarity on nonneg character vectors | §2 eq.(2) + proof | YES | — |
| Legal neutrality under Shaw v. Reno | §3.2 | YES, 2 pages | — |
| Shaw and Miller citations | §3.2 | YES | — |
| CA Prop 11 criteria | §3.3, §1 | YES | — |
| CO Art. V §44 | §3.3, §1 | YES | — |
| AZ IRC criteria | §3.3, §1 | YES | — |
| WA RCW 44.05.090 | §3.3, §1 | YES | — |
| Property-tax fiscal bond doctrine | §3.4 | YES | — |
| Modular framework: any tract signal maps to a dimension | §2.1 | YES | — |
| Signal taxonomy M.1–M.7 catalogue | §4 + Table 1 | YES | — |
| Alpha-blending integration with bisect compositor | §5 | YES | — |
| L0 test invariants: symmetric, self-is-one, nonneg | §2.6 | PARTIAL | weight_bounded_above uses wrong bound (see Q-07) |
| Spec lists Thornburg v. Gingles — paper does not cite it in body | §3 | MISS | Gingles in bib but not cited in body text |
| No empirical pipeline run required | implicit | YES — paper has no results tables | — |

```
CONTRACT: PARTIAL
Promises kept: 13/15
Gaps:
  1. weight_bounded_above invariant bound is inconsistent with the multiplicative cosine formula.
  2. Thornburg v. Gingles (in bib and in spec's legal references) does not appear in the
     paper body. Not a critical gap (M.0 is not a VRA paper), but spec lists it and body omits it.
```

---

## PHASE 4 — REFEREE SIMULATION

### REFEREE 1 — Algorithms Reviewer (SODA/FOCS archetype)
**Recommendation: Minor Revision**

**SUMMARY**: The framework is clear and the cosine similarity proof is correct. The
non-negativity argument (all c_i ≥ 0 implies dot product ≥ 0) is valid. However, the
paper introduces two different formulas in different sections without reconciling them,
and the invariant list contains an error.

**MAJOR CONCERNS**:
[I-01] The §2.6 invariant `weight_bounded_above` claims $w \leq 2 \times w_{\mathrm{base}}$.
For the multiplicative formula (§2.3), the correct bound is $w \leq w_{\mathrm{base}}$
(since $\mathrm{sim} \leq 1$). The bound $2 \times w_{\mathrm{base}}$ belongs to M.6's
additive formula $w = w_{\mathrm{base}} \times (1 + \alpha \times \mathrm{zone\_score})$.
This needs to be corrected before the paper can be cited as the authoritative reference
for all M-track papers.

[I-02] §5.2 introduces a fundamentally different formula: alpha-blending replaces
the multiplicative formula $w = w_{\mathrm{base}} \times \mathrm{sim}$ with
$w_{\mathrm{final}} = \alpha \cdot w_{\mathrm{base}} + (1-\alpha) \cdot w_{\mathrm{char}}$.
These two formulas are not equivalent (one is multiplicative scaling, the other is
convex combination). The paper presents both as aspects of the same framework without
stating which is the canonical formula for METIS. This is confusing and potentially
incorrect: the alpha-blending formula in §5 should be stated as the operational formula,
and the §2 "edge weight modifier" should be clarified as a conceptual intermediate step.

[I-03] The zero-vector handling (§2.4–2.5) specifies sim=1 as the substitution but
does not prove that this is conservative in the partitioning sense (i.e., that it does
not create artificially cheap cuts that would not exist with any non-zero character vector).
A brief argument would suffice, but as stated it is an assertion.

**MINOR CONCERNS**:
- §5 mentions `--character-signals economic,zone-membership,housing` as a composite mode
  CLI flag, but the paper does not define how multiple signals compose (is it an average?
  a max? a weighted sum?). This is deferred to M.8, which should be stated explicitly.
- The paper claims the framework is the "first precise mathematical definition" of COI as
  cosine similarity (§1 Contribution 1). Altman and McDonald (2011) is cited in the
  bibliography; the paper should clarify whether they used cosine similarity or a
  different measure.

---

### REFEREE 2 — Political Science Reviewer (APSR/JOP archetype)
**Recommendation: Minor Revision**

**SUMMARY**: A framework paper is appropriate and the legal grounding is careful. The
claim of partisan neutrality is well-defended. The main concern is that the paper
makes no empirical claims but nonetheless uses language like "the framework... causes
the partitioner to be geometrically reluctant to cut through edges connecting
similar-character tracts" (§1) — this is an empirical claim about METIS behavior that
requires a citation or a run result.

**MAJOR CONCERNS**:
[I-04] The paper states (§1) that the framework "causes the partitioner to be
geometrically reluctant to cut through edges connecting similar-character tracts."
This is an empirical claim about METIS optimization behavior, not a mathematical
theorem. METIS minimizes a weighted cut, so higher edge weights reduce the probability
of cutting an edge, but do not make it "reluctant" in any precise sense. The paper
should either (a) provide a formal statement connecting higher edge weights to lower
cut probability under METIS, or (b) qualify the language as informal.

[I-05] The abstract states the framework is grounded in "Reynolds v. Sims (1964),
Shaw v. Reno (1993), and the independent redistricting commission criteria enacted in
California, Colorado, Arizona, and Washington." These are all real and correctly cited
cases and statutes. However, the abstract says nothing about the algorithmic mechanism.
A reader who encounters this in a law review search would not understand that the paper
is about METIS edge weights. The value proposition for a legal audience and an
algorithms audience need not be contradictory but the abstract currently reads as
legal framing, not algorithms framing.

**MINOR CONCERNS**:
- The paper does not discuss what happens when character signals conflict with VRA
  requirements. If a high zone co-membership score would prevent cutting through a
  majority-minority school district boundary, does the framework yield to VRA? This
  interaction is important for practitioners.
- "At least thirty states" in the abstract needs a citation.

---

### REFEREE 3 — Legal Reviewer (Law Review / Public Administration archetype)
**Recommendation: Accept (minor revisions)**

**SUMMARY**: The legal citations are accurate and the cases are real. The Shaw-Miller
analysis is correct. The California, Colorado, Arizona, and Washington statutory
citations are verified below.

**Citation verification**:
- Reynolds v. Sims, 377 U.S. 533 (1964): REAL. Correct citation.
- Shaw v. Reno, 509 U.S. 630 (1993): REAL. Correct citation.
- Miller v. Johnson, 515 U.S. 900 (1995): REAL. Correct citation.
- Milliken v. Bradley, 418 U.S. 717 (1974): REAL. Correct citation.
- CA Prop 11 (2008), Cal. Elec. Code §21601(c): REAL. Prop 11 created the Citizens
  Redistricting Commission. The section reference (§21601) is plausible; exact subsection
  (c) should be verified against current codification as the statute was renumbered
  after 2008.
- CO Art. V, §44: REAL. Colorado established its independent legislative redistricting
  commission by Amendment Z (2018), codified at Colo. Const. Art. V §44.
- AZ Const. Art. IV, Pt. 2, §1(14): REAL. Arizona's IRC was established by Prop 106
  (2000) and is codified at this provision.
- WA RCW 44.05.090: REAL. Washington's Redistricting Commission is governed by this
  statute.

**MAJOR CONCERNS**:
[I-06] Cal. Elec. Code §21601(c) — the paper cites this provision as the source of the
"favor keeping together communities of interest" language. The California Government Code
(not the Elections Code) governs the Citizens Redistricting Commission after the 2011
legislative restructuring. The correct cite is likely Govt. Code §8252(d) or similar.
The Elections Code §21601 citation should be verified; a legal reviewer challenging the
map could use an incorrect code citation to undermine the paper's credibility.

[I-07] The paper uses Milliken v. Bradley (1974) to support the proposition that
"school districts represent genuine local communities organized around the shared civic
enterprise of public education." This is a creative but somewhat strained use of Milliken.
Milliken's holding was about the scope of desegregation remedies, not about the
constitutional legitimacy of school district boundaries as such. A more direct cite for
the legitimacy of school district lines as community boundaries in redistricting would be
Abrams v. Johnson, 521 U.S. 74 (1997), or state court precedents. The Milliken cite is
not wrong but is not the strongest possible authority for this proposition.

**MINOR CONCERNS**:
- The paper mentions Rucho v. Common Cause (2019) in the bibliography but does not cite
  it in the body. Given that Rucho held that federal courts cannot review partisan
  gerrymandering claims, its relevance to a paper arguing for a neutral algorithmic
  standard is significant and should be noted.
- The "quantitative neutral defense" analogy to employment discrimination law (§3.5)
  is creative but not well-supported. The paper asserts the analogy without citing any
  case in which a court accepted such a defense in the redistricting context.

---

## PHASE 5 — ABSTRACT CHECK

**Abstract word count**: ~190 words (abstract text, counted from 00-abstract.tex)

```
ABSTRACT: ~190 words
Primary result stated: NO — the abstract describes the framework but states no
   quantitative result (appropriate for a framework paper, but the primary
   empirical claim about what happens to districts is deferred entirely to M.1–M.7)
Algorithm named: YES — "cosine similarity of tract-level character vectors" and
   "METIS graph partitioning engine" are both named
Value proposition: YES — "legal doctrine of communities of interest as articulated in
   Reynolds v. Sims (1964), Shaw v. Reno (1993)" and expert witness role in conclusion
Length: PASS (within 150-200 word target)
```

The abstract is appropriate for a framework paper. The absence of a primary quantitative
result is acceptable because M.0 is theoretical; the spec explicitly states "No empirical
pipeline run required for M.0." The abstract correctly previews the structure and claims.

One gap: the abstract does not mention zero-vector handling, which is a nontrivial
design choice that expert reviewers will probe. Consider adding one sentence.

---

## PHASE 6 — PRE-PANEL CHECKLIST

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: M.0 — Community Character Framework
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 failure (Q-07: wrong upper bound in §2.6 invariant),
                 3 warnings
  Contract:      PARTIAL — 13/15 promises kept; 2 gaps
  Referee sim:   Minor Revision (algorithmic inconsistency + legal cite check)
  Abstract:      ~190 words — PASS

P1 blockers (fix before panel review):
  [I-01/Q-07] weight_bounded_above invariant in §2.6 states w ≤ 2×w_base, but for
     the multiplicative cosine formula the correct bound is w ≤ w_base. The 2× bound
     is M.6's additive formula, not M.0's. Fix: change §2.6 invariant to
     w(u,v) ≤ w_base(u,v) and add a note that M.6's additive variant produces
     w ≤ (1+alpha)×w_base.
  [I-02] §2.3 defines w = w_base × sim (multiplicative); §5.2 defines
     w_final = alpha×w_base + (1-alpha)×w_char (alpha-blending). These are not the same
     formula. Clarify that §2.3 is the conceptual modifier and §5.2 is the operational
     compositor formula. The L0 invariants in §2.6 should refer to the §5.2 formula
     since that is what is actually implemented.
  [I-06] Cal. Elec. Code §21601(c) citation — verify whether communities of interest
     language is in the Elections Code or Government Code post-2011 reorganization.
     Fix before panel: look up current California CRC statutory codification.

P2 items (should fix):
  [I-03] Zero-vector handling: add brief argument that sim=1 substitution is
     conservative (does not create artificially cheap cuts). One sentence suffices.
  [I-04] Qualify "geometrically reluctant" language in §1 (informal claim about
     METIS behavior). Add "that is, makes cuts through such edges more costly for
     the METIS optimizer" or similar.
  [I-07] Milliken v. Bradley use is strained for the redistricting-community-boundary
     proposition. Consider adding Abrams v. Johnson (1997) or a state court cite as
     primary authority, relegating Milliken to secondary support.
  [Q-04] Zero-vector handling not mentioned in abstract or conclusion.
     Add one sentence in conclusion about the conservative fallback.
  [Q-05/Q-06] Alpha-blending and its default (0.5) not previewed in abstract or intro.
     Add one sentence to abstract: "The framework composes with any base weight mode
     via alpha-blending (alpha=0.5 default)."

P3 items (optional):
  - Add Rucho v. Common Cause (2019) cite in §3 to note that federal partisan
    gerrymandering claims are non-justiciable, reinforcing the importance of
    state-level algorithmic neutrality standards.
  - "At least thirty states" needs a citation (NCSL 2021 would suffice).
  - Altman and McDonald (2011) cited in bibliography but not compared in §1
    Contribution 1 ("first precise mathematical definition" claim).
  - The §5.4 composite mode CLI example (`--character-signals economic,...`)
    should note this is M.8 functionality and not yet implemented.

PRE-PANEL CHECKLIST:
□ [P1] weight_bounded_above invariant corrected to w ≤ w_base
□ [P1] §2.3 multiplicative formula vs §5.2 alpha-blending formula reconciled
□ [P1] Cal. Elec. Code §21601(c) citation verified or corrected
□ [P2] Zero-vector conservative argument added
□ [P2] "Geometrically reluctant" language qualified
□ [P2] Milliken use strengthened with redistricting-specific cite
□ [P2] Abstract mentions alpha-blending
□ All spec contract promises delivered (check Gingles cite if needed)
□ Court citations verified (all 4 Supreme Court cases confirmed real and correct)
□ Abstract states algorithmic mechanism (cosine similarity + METIS) — PASS
□ No single-run results (framework paper) — dagger notation N/A

VERDICT: FIXES REQUIRED
Fixes required: 3 P1, 5 P2
Next: Address P1 blockers, then run panel review using the 5-role panel.
═══════════════════════════════════════════════════════
```
