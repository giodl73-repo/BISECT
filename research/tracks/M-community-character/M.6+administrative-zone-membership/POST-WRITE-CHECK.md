# POST-WRITE CHECK: M.6 — Administrative Zone Co-membership

**Date**: 2026-05-08
**Validator**: research-post-write skill (6-phase pipeline)

---

## PHASE 1 — PAPER SUMMARY

```
Paper: M.6 — Administrative Zone Co-membership Edge Weights:
       School Districts, Fire Districts, and Fiscal Bonds as
       Community Signals in Algorithmic Redistricting
Sections found: 00-abstract, 01-introduction, 02-zones-as-community,
                03-formula, 04-data-sources, 05-legal, 06-experimental-design,
                07-conclusion
Spec found: YES — docs/specs/2026-05-08-m6-administrative-zone-membership.md
Series: M.6 (M-track instantiation, inherits from M.0 framework)
Target audience: legal + practitioner (primary), algorithms (secondary)

Key claims:
  1. Zone co-membership (tracts sharing ≥4 zone types) produces zero splits
     of such co-membered tract pairs in NC-14. [DEFERRED — experimental design
     section explicitly states data not yet downloaded, implementation not yet
     deployed; claim stated in spec only]
  2. School district boundary crossings reduced by ≥80% vs geographic weights
     for NC-14. [DEFERRED — same caveat; stated in spec/§6 experimental target,
     not as achieved result]
  3. Administrative zone co-membership is the most legally defensible community
     character signal in the M-track. [§5, §7 — legal argument, not empirical]
```

---

## PHASE 2 — CONSISTENCY CHECK

| Q-ID | Quantity | Abstract | §Intro | §Formula | §Data | §Legal | §ExpDesign | §Conclusion | Consistent? |
|------|----------|----------|--------|----------|-------|--------|-----------|-------------|-------------|
| Q-01 | zone_score formula: shared/|Z| | YES (abstract) | YES (§1.1) | YES eq.(4) | — | — | — | YES eq.(4) | PASS |
| Q-02 | Edge weight formula | w_base × (1+alpha×zone_score) | YES (abstract) | YES (§1.1) | YES eq.(5) | — | — | YES CLI | YES (§7.2) | PASS |
| Q-03 | All zones shared → w = 2×w_base | YES (abstract) | — | YES eq.(6) | — | — | — | YES (§7.1) | PASS |
| Q-04 | No zones shared → w = w_base | — | — | YES eq.(7) | — | — | — | — | PASS |
| Q-05 | School district coverage 100% | — | — | — | YES (Table 1) | — | — | — | PASS |
| Q-06 | County subdivision coverage 100% | — | — | — | YES (Table 1) | — | — | — | PASS |
| Q-07 | Electric utility coverage 100% | YES ("100%") | YES | — | YES (Table 1) | YES | — | YES | PASS |
| Q-08 | Fire district coverage ~60% | YES | YES (§2.4) | YES (§3.1) | YES (Table 1) | — | — | — | PASS |
| Q-09 | Water district coverage ~40% | YES | — | — | YES (Table 1) | — | — | — | PASS |
| Q-10 | Police precinct coverage ~30% | YES | YES (§2.6) | — | YES (Table 1) | — | — | — | PASS |
| Q-11 | Minimum |Z| = 3 | — | — | YES (§3.1) | — | — | — | — | WARN: stated only in §3, not in abstract or conclusion |
| Q-12 | Maximum |Z| = 6 | — | — | YES (§3.1) | — | — | — | — | WARN: same |
| Q-13 | NC-14 zero splits of ≥4-zone tracts | NO (deferred) | NO | — | — | — | YES (§6.2) | NO | PASS (correctly deferred) |
| Q-14 | School district crossings ≤20% of geo | NO | NO | — | — | — | YES (§6.2) | NO | PASS (correctly deferred) |
| Q-15 | MA town integrity ≥95% of runs | NO | NO | — | — | — | YES (§6.5) | NO | PASS (correctly deferred) |
| Q-16 | Property tax = 36% of school revenue | — | — | — | — | YES (§5.3) | — | — | PASS (cited: NCES 2022) |
| Q-17 | 90,000 governmental units | — | YES (§2) | — | — | — | — | — | WARN: only in §2, no abstract/conclusion mention |
| Q-18 | Alpha=1.0 default | YES (abstract) | — | YES (§3.2, alpha=1 default) | — | — | YES CLI | — | PASS |

### CRITICAL INCONSISTENCY — Q-01/Q-02: Abstract formula vs. M.0 framework formula (P1 blocker)

The abstract states the edge weight formula as:
> $w(u,v) = w_{\mathrm{base}} \times (1 + \alpha \times \mathrm{zone\_score}(u,v))$

The M.0 framework (§2.3) defines the canonical formula as:
> $w(u,v) = w_{\mathrm{base}} \times \mathrm{sim}(\mathbf{c}_u, \mathbf{c}_v)$

M.6's formula is an **additive modifier** that extends beyond the M.0 multiplicative model.
When zone_score = 1 and alpha = 1, M.6 yields w = 2×w_base. The M.0 multiplicative formula
with sim = 1 yields w = w_base. These are different behaviors and the paper does not
explicitly reconcile the difference.

§1.3 (Relationship to M.0) acknowledges that M.6 "replaces cosine similarity with the zone
co-membership ratio formula" and that M.6 satisfies "symmetry, [0,1] range, graceful
degradation." But the abstract of M.6 uses a formula ($1 + \alpha \times \mathrm{zone\_score}$)
that does NOT match the M.0 pattern ($w_{\mathrm{base}} \times \mathrm{sim}$). The product
$w_{\mathrm{base}} \times \mathrm{zone\_score}$ would match M.0; the additive $(1 + \alpha \times
\mathrm{zone\_score})$ multiplier is a distinct design choice with different properties
(it makes the weight range [w_base, 2×w_base] rather than [0, w_base]).

The M.0 §4.6 summary of M.6 says: "The zone co-membership formula replaces cosine
similarity in equation (weight-modifier) and satisfies all required properties: symmetry,
[0,1] range, and graceful degradation to w_base when no zone data is available."

But under M.6's actual formula, when zone_score = 0, w = w_base × (1+0) = w_base —
this is correct, the weight degrades to w_base, NOT to zero. Under M.0's formula
w = w_base × sim, when sim = 0 the weight collapses to zero (making the cut free).
M.6's formula explicitly avoids this: tracts sharing no zones are not penalized, just
unbosted. This is a deliberate and documented design choice (§3.2), but it means M.6
does NOT satisfy the same "graceful degradation" as M.0 describes: M.0 says
"sim=0 makes the cut free"; M.6's sim=0 leaves the cut at baseline cost.

This is a real inconsistency between M.0's description of M.6 and M.6's actual formula.
Both papers need a fix: M.0 §4.6 should note that M.6 uses an additive (not multiplicative)
modifier, and M.6 §1.3 should more explicitly flag the departure from the M.0 pattern.

### CRITICAL ISSUE — TIGER UNSD label for fire/water districts (P1 blocker)

§4.4 states: "TIGER/Line Special Districts (the **UNSD shapefile** with type codes) include
fire protection districts (code 21) and various water and wastewater district types
(codes 22--29)."

**UNSD** is the Census Bureau abbreviation for **Unified School Districts**, not for
special districts. The correct TIGER/Line file for special districts is typically labeled
**SSD** (Special District) or accessed via the TIGER/Line Political Divisions layer. The
UNSD file contains school district boundaries, not fire and water district boundaries.

Using UNSD to refer to the fire/water district shapefile is technically incorrect. A data
engineer following this paper to implement the pipeline would look for the UNSD file and
find school districts, not fire districts.

The correct TIGER/Line reference for special districts is the **Governmental Units** or
**Special District** (SLDL code 5 in some vintages, or the legislative district file with
AIANNH code, or more specifically the `tl_YYYY_us_aitsn` / `tl_YYYY_us_unsd` separation).
In practice, special districts (fire, water, etc.) appear in TIGER's **Places**, **County
Subdivisions**, or as dedicated special district layers. The specific codes 21-29 referenced
in the paper are likely from a specific TIGER/Line product; the paper should cite the exact
product name rather than "UNSD shapefile."

Fix: Replace "the UNSD shapefile with type codes" with the correct TIGER/Line layer name for
special districts, or clarify the exact product/file accessed. Add the specific download URL.

### CONSISTENCY: Additional checks

- **§3.2 boundary case invariant label `all_zones_shared_doubles_weight`** states
  $\mathrm{zone\_score} = 1 \Rightarrow w = 2 \times w_{\mathrm{base}}$ when $\alpha = 1.0$.
  This is mathematically correct for the M.6 formula: $w_{\mathrm{base}} \times (1 + 1 \times 1) = 2 \times w_{\mathrm{base}}$. PASS.

- **§6.5 experimental design cites Massachusetts Constitution, "Part II, Ch. 1, §3"** as
  requiring preservation of town boundaries. The Massachusetts Constitution Part II, Chapter 1,
  Section 3 addresses the House of Representatives. The redistricting provision is in the
  Amendments to the Constitution (Article 101, enacted 1978). The citation in §6.5 is
  imprecise and a legal reviewer would flag it. (See P2.)

- **Alpha parameter description**: Abstract says alpha=1.0 default; §3.3 confirms this. PASS.

- **The spec lists 6 zone types**; the paper lists 6 zone types (school, county subdivision,
  electric utility, fire, water/sewer, police precinct). PASS.

- **Spec claim #1**: "Tracts sharing ≥4 zone types (school + fire + water + electric) never
  appear in different districts." The paper's §6.2 correctly defers this as an experimental
  target with "data not yet downloaded, implementation not yet deployed." PASS (correctly
  deferred, not claimed as achieved).

```
CONSISTENCY: 2 P1 failures (formula inconsistency with M.0, TIGER UNSD label error),
             3 warnings (Q-11/Q-12 |Z| range, Q-17 governmental units count)
P1 (fix before panel):
  - M.6 additive formula vs. M.0 multiplicative formula — explicit reconciliation needed
    in both papers (M.0 §4.6 description of M.6, M.6 §1.3 relationship to M.0)
  - TIGER UNSD label used for fire/water district shapefile is incorrect
P2 (should fix):
  - Massachusetts Constitution citation (§6.5) is imprecise — should cite Amendment 101
    rather than original Part II, Ch.1, §3
P3 (minor):
  - Min/max |Z| values (3 and 6) stated only in §3, not previewed in abstract
```

---

## PHASE 3 — CONTRACT CHECK (vs spec docs/specs/2026-05-08-m6-administrative-zone-membership.md)

| Promise (from spec) | Paper section | Delivered? | Gap |
|---------------------|---------------|-----------|-----|
| Zone score formula: shared/available ∈ [0,1] | §3.2 eq.(4) | YES | — |
| Edge weight: w_base × (1 + alpha × zone_score) | §3.2 eq.(5) | YES | — |
| All zones shared → w = 2×w_base (alpha=1) | §3.2 eq.(6) | YES | — |
| No zones shared → w = w_base | §3.2 eq.(7) | YES | — |
| Six zone types in priority order | §2, §4 Table 1 | YES | — |
| School district source: TIGER/Line SCHOOLDISTRICT | §4.1 | YES (correct label) | — |
| County subdivision source: TIGER/Line COUSUB | §4.2 | YES (correct label) | — |
| Electric utility source: EIA Form 861 + HIFLD | §4.3 | YES | — |
| Fire district source: TIGER/Line code 21 | §4.4 | PARTIAL | Wrong file label (UNSD vs special district) |
| Water district source: TIGER/Line codes 22–29 | §4.4 | PARTIAL | Same UNSD label issue |
| Police precinct fallback to county | §4.5 | YES | — |
| Coverage %: school 100%, county 100%, electric 100%, fire ~60%, water ~40%, police ~30% | Table 1 | YES | — |
| L0 invariants: all listed in spec | §3.4 | YES (5 invariants listed) | — |
| Spatial join via centroid-in-polygon | §4.6 | YES | — |
| CLI flag: --weights-override zone-membership | §3.5 | YES | — |
| YAML config: zone_alpha | §3.5 | YES | — |
| NC-14 empirical target: zero splits ≥4 zone types | §6.2 | YES (deferred) | — |
| NC-14 school crossings ≤20% of geo | §6.2 | YES (deferred) | — |
| MA town integrity ≥95% | §6.5 | YES | — |
| Legal grounding: Reynolds, Shaw, Milliken | §5.1, §5.2, §5.4 | YES | — |
| Property-tax fiscal bond doctrine | §5.3 | YES | — |
| Shaw v. Reno neutrality | §5.4 | YES | — |
| Expert witness language template | §7.1 | YES | — |

```
CONTRACT: PARTIAL
Promises kept: 18/20
Gaps:
  1. TIGER/Line fire and water district sources: correct in that the paper cites code 21 and
     22–29, but the file layer name "UNSD shapefile" is wrong (UNSD = school districts).
  2. Formula-level inconsistency with M.0 framework not acknowledged prominently enough
     in §1.3 (minor gap but creates downstream confusion for M.8 composite).
```

---

## PHASE 4 — REFEREE SIMULATION

### REFEREE 1 — Algorithms Reviewer (SODA/FOCS archetype)
**Recommendation: Major Revision**

**SUMMARY**: The formula is mathematically sound but the paper introduces an additive
modifier that departs from the M.0 framework without adequately motivating the
departure. The data source citation for fire and water districts contains a technical
error that would make the pipeline non-reproducible.

**MAJOR CONCERNS**:
[I-01] **Formula departure from M.0**: M.0 defines the canonical framework as
$w(u,v) = w_{\mathrm{base}} \times \mathrm{sim}$. M.6 uses
$w(u,v) = w_{\mathrm{base}} \times (1 + \alpha \times \mathrm{zone\_score})$.
These have qualitatively different behaviors: the M.0 formula can reduce edge weights
to zero (when sim=0), making cuts free. M.6's formula only increases edge weights —
the minimum is $w_{\mathrm{base}}$ (when zone_score=0). The choice is reasonable but the
paper buries the justification in §3.2 as a "design choice" without comparing it to
the M.0 pattern. A reader who has read M.0 first will be confused. Section §1.3 says
M.6 "satisfies the same mathematical properties" as M.0, but the key property
"graceful degradation to zero when sim=0" is NOT satisfied by M.6 (M.6 degrades to
w_base, not to zero). Fix: Explicitly state in §1.3 that M.6 uses an additive (not
multiplicative) modifier and explain why: the additive design avoids penalizing
administratively disconnected tracts (which are not inherently bad community members,
just different administrative members), whereas the multiplicative design would make
such cuts free.

[I-02] **TIGER UNSD label error**: §4.4 refers to "the UNSD shapefile with type codes"
for fire and water districts. UNSD is the Census Bureau abbreviation for Unified School
Districts. A pipeline engineer following this paper would download the UNSD file and
find school districts, not fire districts. The correct TIGER/Line product for special
districts must be identified by its actual name. This is a reproducibility error.

[I-03] **Zone assignment for split tracts**: §3.1 notes that "census tracts are designed
to respect administrative boundaries wherever possible" but gives no quantitative
estimate of how often centroid assignment fails. §4.7 acknowledges this as a limitation
but says "this case is rare." In states with frequent redistricting of school district
boundaries (California, Texas), split-tract misassignment may be material. A bound on
the error rate, even informal, would strengthen the methodology.

**MINOR CONCERNS**:
- The paper lists 6 zone types but §3.1 states |Z| ranges from 3 to 6. Only 3 zone
  types have 100% coverage; the other 3 are variable. The paper should state which 3
  are "always in" and which are optional more prominently.
- §3.5 CLI: `zone_types: [school_district, county_subdivision, electric_utility, fire_district, water_district]` lists 5 types in the YAML but the paper defines 6 (police precinct omitted). This
  is consistent with the text (police precinct is lowest priority) but the YAML omission
  should be noted in a comment.

---

### REFEREE 2 — Political Science Reviewer (APSR/JOP archetype)
**Recommendation: Minor Revision**

**SUMMARY**: The paper is well-grounded and the empirical design is reasonable. The main
weakness is that all quantitative claims are deferred — the paper makes no current
empirical contribution. Acceptable for an implementation specification paper but the
panel should be aware that the M.6 results are promises, not findings.

**MAJOR CONCERNS**:
[I-04] **All quantitative claims are deferred**: The paper explicitly states (§6) that
"these data are not yet downloaded and the implementation is not yet deployed." The
abstract's claim that the formula makes cuts "geometrically costly" is therefore a
prediction about future behavior, not a validated result. The paper would be
significantly stronger if at least one state (e.g., Massachusetts, for town integrity)
had been run. Given that TIGER/Line data is free and the bisect binary exists, the
absence of any pilot run is notable.

[I-05] **No partisan neutrality demonstration**: The paper claims zone co-membership is
"partisan-neutral by construction" (§5.4). This is true for the formula inputs, but
the paper provides no analysis of whether zone boundaries correlate with partisan
patterns. Administrative boundaries (especially school districts in the South) are
historically correlated with residential segregation, which correlates with partisan
alignment. A single-state analysis of the partisan composition of zone-co-membered
tracts would strengthen the neutrality claim.

**MINOR CONCERNS**:
- The Massachusetts Constitution citation (§6.5) to "Part II, Ch. 1, §3" is inaccurate
  for the redistricting provision. The legislative redistricting requirements were
  enacted via Amendment 101 (1978). A political science reviewer familiar with
  Massachusetts redistricting law would notice this.
- The paper claims that "expert witnesses... in proceedings across California, Colorado,
  Virginia, and North Carolina have cited [school district integrity] as evidence of
  inadequate communities of interest preservation." No citation supports this claim.
  A specific case or commission decision should be cited.

---

### REFEREE 3 — Legal Reviewer (Law Review / Public Administration archetype)
**Recommendation: Minor Revision**

**SUMMARY**: The legal analysis is the strongest part of the paper. The Supreme Court
citations are accurate. The property tax fiscal bond doctrine is well-articulated. One
citation is inaccurate (Massachusetts Constitution), and one argument relies on a
creative but indirect use of Milliken.

**Citation verification**:
- Reynolds v. Sims, 377 U.S. 533 (1964): REAL. Correct citation.
- Shaw v. Reno, 509 U.S. 630 (1993): REAL. Correct citation.
- Miller v. Johnson, 515 U.S. 900 (1995): REAL. Correct citation.
- Milliken v. Bradley, 418 U.S. 717 (1974): REAL. Correct citation.
- EIA Form 861: REAL. The Energy Information Administration publishes Form 861 Annual
  Electric Power Industry Report, which includes service territory data. URL is correct.
  The paper also correctly notes the Energy Policy Act of 2005 (§5.4) as the statutory
  authority for EIA data collection. PASS.
- HIFLD Electric Retail Service Territories: REAL. The Homeland Infrastructure
  Foundation-Level Data program maintains this dataset. URL format referenced in §4.3
  is correct (opendata.arcgis.com). PASS.
- Massachusetts Constitution, "Part II, Ch. 1, §3": INACCURATE. Part II, Chapter 1,
  Section 3 of the Massachusetts Constitution describes the original constitution of
  the House of Representatives (1780). The redistricting requirement for preservation of
  town boundaries comes from Amendment 101 to the Massachusetts Constitution (1978), which
  requires that legislative districts "not divide a city or town" except where necessary
  for population equality. This is a material error in a paper whose stated purpose
  includes legal defensibility.

**MAJOR CONCERNS**:
[I-06] **Massachusetts Constitution citation**: §6.5 states "the Massachusetts Constitution
(Part II, Ch. 1, §3) has historically been interpreted to require preservation of town
boundaries in legislative redistricting." The correct citation is Amendment 101 to the
Massachusetts Constitution (ratified 1978, effective 1980), not the original 1780
Constitution. This error would be caught by any Massachusetts redistricting practitioner
and would undermine the paper's credibility in a Massachusetts litigation context.
Fix: Replace "Part II, Ch. 1, §3" with "Amendment 101 (1978)" and cite the specific
language about not dividing municipalities.

[I-07] **The "most legally defensible" claim**: The abstract and conclusion repeatedly
describe zone co-membership as "the most legally defensible community character signal"
and "the most court-defensible community signal in the M-track." This superlative claim
is not defended against competing characterizations. A legal reviewer would ask: what
about the commuting shed (M.4), which courts in California have accepted as a proxy for
economic community of interest? The "most defensible" claim needs qualification or
comparative support.

[I-08] **Brennan Center citation (§2.1 and §5.3)**: The paper cites "Levitt 2011" for
the claim that "the Brennan Center's best-practices guide... lists school district
preservation as a concrete operationalization of the communities of interest criterion."
Justin Levitt's "A Citizen's Guide to Redistricting" (2011, Brennan Center) is a real
publication. However, the paper should verify whether the 2011 edition contains
school district preservation language specifically, as the guide has been updated and
the specific language may be in a later edition. Recommend citing the specific
passage or edition.

**MINOR CONCERNS**:
- The §5.4 Energy Policy Act of 2005 citation is accurate (the Act established
  comprehensive energy data collection requirements for EIA), but the specific
  provision for electric utility territory reporting is Section 1252 of the Act.
  A footnote with the section number would strengthen the claim.
- The paper cites Arizona's commission as having "accepted expert testimony about
  administrative zone co-membership" (§5.5). No specific case or decision supports
  this claim. Without a citation, a legal challenger could deny it.

---

## PHASE 5 — ABSTRACT CHECK

**Abstract word count**: ~195 words (00-abstract.tex, counted)

```
ABSTRACT: ~195 words
Primary result stated: PARTIAL — the abstract states the formula and design choices
   but defers all quantitative results ("Empirical validation... is deferred to
   post-implementation runs"). This is honest but leaves the abstract without a
   quantitative finding.
Algorithm named: YES — zone co-membership score formula is given explicitly in the
   abstract; METIS edge weight modifier relationship is stated.
Value proposition: YES — "most court-defensible community signal in the M-track"
   stated; property tax fiscal bond doctrine mentioned; legal grounding summarized.
Length: PASS (within 150-200 word target).
```

One concern: the abstract gives the formula with both zone_score AND the edge weight
modifier formula inline: this makes it long but information-dense. A law review
abstract would typically not contain equations; an algorithms abstract would. For the
dual audience, the current presentation is a reasonable compromise but risks losing
either audience in the first paragraph. Consider moving one formula to a note.

---

## PHASE 6 — PRE-PANEL CHECKLIST

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: M.6 — Administrative Zone Co-membership
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   2 failures (additive formula departure from M.0, TIGER UNSD label);
                 3 warnings
  Contract:      PARTIAL — 18/20 promises kept; 2 gaps (both TIGER file label)
  Referee sim:   Major Revision (algorithmic: TIGER label error + formula departure;
                 legal: Massachusetts Constitution citation error)
  Abstract:      ~195 words — PASS; no primary quantitative result (correctly deferred)

P1 blockers (fix before panel review):
  [I-01/Q-01] M.6 additive formula w_base×(1+alpha×zone_score) departs from M.0's
     multiplicative formula w_base×sim. §1.3 says M.6 satisfies "same mathematical
     properties" as M.0, but this is false for the "sim=0 makes cut free" property.
     Fix: Revise §1.3 to explicitly state that M.6 uses an additive (boosting) modifier
     while M.0 uses multiplicative scaling, and explain the rationale: M.6 boosts
     co-membered pairs rather than penalizing non-co-membered pairs.
     Also coordinate with M.0 §4.6 to update its description of M.6.
  [I-02] TIGER UNSD label used for fire and water district shapefiles (§4.4) is
     incorrect. UNSD = Unified School Districts. The correct TIGER/Line product for
     special districts should be identified. Fix: Determine the actual TIGER/Line
     layer name for special districts (likely the "Political Divisions > Special
     Purpose Districts" layer or the AREAWATER-adjacent political feature) and correct
     §4.4 with the exact file name and download path.
  [I-06] Massachusetts Constitution citation in §6.5 — "Part II, Ch. 1, §3" is the
     1780 original constitution, not the redistricting provision. The correct citation
     is Amendment 101 (1978). Fix: Replace citation with "Amendment 101 to the
     Massachusetts Constitution (ratified 1978)" with specific language about
     not dividing municipalities.

P2 items (should fix):
  [I-03] Add at least informal estimate of split-tract misassignment rate for centroid
     zone assignment, or cite any study of tract-boundary alignment with administrative
     boundaries.
  [I-04] At minimum, document a pilot run plan or rationale for why no pilot run was
     performed. Consider running Massachusetts for town integrity as a quick validation
     (TIGER COUSUB data requires no special download).
  [I-07] Qualify "most legally defensible community signal" claim. Add one sentence
     acknowledging that commuting shed (M.4) is also well-established in California
     commission proceedings, and defend why zone co-membership is stronger.
  [I-08] Verify Levitt 2011 Brennan Center guide contains the school district
     preservation language as cited, or update to specific edition/passage.
  [Ref2-minor] Add citation for expert witness proceedings where school district
     integrity was cited as COI evidence (California 2011 or 2021 CRC proceedings
     have published transcripts).

P3 items (optional):
  - Add §5.4 footnote with Energy Policy Act of 2005, Section 1252 reference.
  - Arizona IRC expert testimony claim (§5.5) needs a specific case or decision cite.
  - §3.5 YAML: add comment noting police precinct is excluded from default zone_types list.
  - §6.5 New England metric: clarify that the 95% target is for adjacent same-town pairs,
     not for all towns (small towns at district boundaries will always split sometimes).
  - Consider adding Rucho v. Common Cause (2019) in §5 to note that federal courts
     cannot hear partisan gerrymandering claims, reinforcing the importance of
     state-level administrative neutrality standards.

PRE-PANEL CHECKLIST:
□ [P1] M.6 additive formula vs. M.0 multiplicative formula — §1.3 revised with
       explicit statement of departure and rationale
□ [P1] M.0 §4.6 description of M.6 updated to reflect additive formula
□ [P1] TIGER fire/water district shapefile layer name corrected in §4.4
□ [P1] Massachusetts Constitution citation corrected to Amendment 101 (1978)
□ [P2] Centroid misassignment rate characterized or bounded
□ [P2] "Most legally defensible" claim qualified
□ [P2] Expert witness proceeding citation added for school district COI recognition
□ All empirical targets correctly labeled as deferred (no achieved results claimed)
□ Court citations verified: Reynolds, Shaw, Miller, Milliken — all REAL and correct
□ EIA Form 861 data source verified as real and accurately described — PASS
□ TIGER/Line school district (SCHOOLDISTRICT) and COUSUB labels verified — PASS
□ Abstract gives formula and defers results — acceptable for spec paper

VERDICT: FIXES REQUIRED
Fixes required: 3 P1, 5 P2
Priority: P1-TIGER label fix is a reproducibility blocker; P1-Massachusetts cite is a
legal credibility risk; P1-formula reconciliation is a cross-paper consistency issue.
Next: Fix P1 blockers, coordinate M.0 §4.6 update, then run panel review.
═══════════════════════════════════════════════════════
```
