# Track E — Experimental Extensions: Panel Revision Plan

**Track**: E-experimental
**Generated**: 2026-05-07
**Based on**: REVIEW_PANEL.md (2026-05-07)
**Target module score**: 8.0 / 10 (current: 6.8 / 10)

---

## Overview

Track E's revision divides into three tiers:

1. **Complete missing content** (E.1 evaluation section, E.5 results section, E.6 missing results): These are the blocking items. Four of six empirical papers have content that doesn't exist yet.
2. **Fix factual errors** (E.2 "comparable equality" claim, E.3 beta discrepancy, E.5 EG = 0 framing): These are corrections to existing text.
3. **Strengthen framing** (speculative nature signaling across E.2, E.5; D'Hondt consistency; E.4 scope definition): These are positioning improvements.

The synthesis papers (E.0, E.7) are accepted and need only contingency footnotes and minor updates as companion papers complete.

---

## PP1 Blocking Items (Address Before Any Submission)

### PP1-E1: Write E.1 Electoral Formula Evaluation Section

**File**: `research/E.1+multi-member-districts/sections/` (new section file)
**Effort**: 2–4 weeks
**Blocks**: E.1 submission; E.7's proportionality synthesis citations

**What must be written**:

Section 4 (Electoral Formula Results) — approximately 3,000–4,000 words including:

1. **D'Hondt vs. Saint-Laguë comparison** for all 50 states:
   - Run the redistricting algorithm to generate k-member multi-member districts (k = 2, 3, 5 for each state based on delegation size)
   - Apply both D'Hondt and Saint-Laguë to the resulting districts
   - Report proportionality metrics (Gallagher index, effective threshold) for both formulas

2. **National proportionality table**:
   - Current single-member system: mean Gallagher index across 50 states
   - MMD with D'Hondt: mean Gallagher index
   - MMD with Saint-Laguë: mean Gallagher index
   - Identify states where formula choice materially changes the proportionality outcome (>1 seat difference)

3. **Case studies** (3–5 states):
   - Illustrative states where geographic sorting makes proportionality difficult even under MMD
   - States where MMD dramatically improves proportionality

4. **Limitations paragraph**:
   - Third-party and independent candidates excluded from the model
   - Static partisan shares (2020 presidential vote) used throughout
   - No accounting for strategic voting under MMD systems

**Prerequisite**: D'Hondt and Saint-Laguë implementations must be confirmed as available in the project codebase or scripts directory.

---

### PP1-E2: Correct E.2 Abstract "Comparable Equality" Claim

**File**: `research/E.2+county-representation/sections/00-abstract.tex`
**Effort**: 1 hour
**Blocks**: E.2 submission; accurate Track E synthesis

**Current text** (paraphrase): "The county-representation scheme achieves comparable population equality to the base algorithm."

**Required correction**:
> "The county-representation scheme achieves substantially worse population equality than the base algorithm (25% CV vs. ±0.5%). This tradeoff between geographic representation and population equality is the central finding of this paper."

Additionally add to the abstract:
> "Our district count estimates (197–201 multi-member districts nationally) are based on county population distribution analysis, not on running the redistricting algorithm. Full algorithmic redistricting under county-based rules is left for future work."

---

### PP1-E3: Fix E.3 Beta Discrepancy

**File**: `research/E.3+national-redistricting/sections/` (multiple)
**Effort**: 1–2 hours
**Blocks**: E.3 submission

Run `grep -r "cross-state districts" research/E.3+national-redistricting/` to find all instances of the count. Verify against the actual algorithm output. The correct number appears to be derivable from the redistricting run outputs.

Steps:
1. Locate the redistricting output files for the national redistricting run
2. Count actual cross-state districts (districts whose constituent tracts span state boundaries)
3. Replace all instances of 176 and 180 with the verified count
4. Add a footnote in the methodology section specifying exactly how "cross-state district" is defined (both constituent tracts in different states? majority of population in different states?)

---

### PP1-E5: Reframe E.5 and Write Results Section

**File**: `research/E.5+party-based-allocation/sections/`
**Effort**: 2–4 weeks
**Blocks**: E.5 submission

**Two-part fix**:

**Part 1 — Reframe the EG = 0 goal** (1–2 hours)

In the introduction, replace:
> "We seek to determine whether algorithmic redistricting can achieve partisan fairness, measured as efficiency gap = 0."

With:
> "We ask a counterfactual question: if a redistricting algorithm were explicitly designed to minimize efficiency gap (rather than edge-cut compactness), what would the resulting maps look like? This is explicitly NOT the approach taken by the base algorithm, which is designed to be neutral to partisan outcomes. Our analysis reveals the geometric constraints that limit partisan fairness even under explicit optimization — showing that EG = 0 is often geometrically infeasible due to the spatial distribution of partisan voters."

This framing change serves two purposes: (a) it positions E.5 as a counterfactual analysis of what explicit partisan optimization would produce, rather than as a recommendation to adopt partisan optimization; (b) it aligns E.5 with the program's impossibility-defense framing.

**Part 2 — Write results section** (2–3 weeks)

The results section needs:
1. For each of the 50 states: minimum achievable EG under explicit optimization vs. EG under the neutral base algorithm vs. EG under enacted plans
2. National distribution of minimum EG — showing the floor set by geographic sorting
3. Map comparison: neutral vs. EG-optimized maps for 5 illustrative states
4. Table: states where EG = 0 is geometrically feasible vs. infeasible (Rodden's 40% threshold as a proxy)

---

### PP1-E6: Complete E.6 International Results

**File**: `research/E.6+international-applications/sections/`
**Effort**: 3–6 weeks
**Blocks**: E.6 submission

The following results must be generated and written:

**UK results**:
- Run the algorithm on UK parliamentary constituencies (650 seats, single-member)
- Compare compactness to current Boundary Commission maps
- Compare across England, Scotland, Wales, Northern Ireland separately

**Canada results**:
- Run on Canadian federal electoral districts (338 seats)
- Compare to current Elections Canada boundaries

**New Zealand results**:
- NZ uses MMP (Mixed Member Proportional) — the algorithm applies to the 71 electorate seats
- Compare electorate compactness to current Representation Commission maps

**STV elevation from TODO**:
- The STV approximation for Ireland must be either implemented or removed
- If implemented: use the Gregory method for seat allocation and report results for Ireland's Dáil (160 seats, 3–5-member constituencies)
- If removed: note explicitly that STV systems require a different seat allocation methodology not currently implemented

---

## PP2 Important Items

### PP2-E2: Disclosure of Estimation Method

**File**: `research/E.2+county-representation/sections/03-methodology.tex`
**Effort**: 1 hour

Add to Section 3.1:
> "The 197–201 national district count is derived from analysis of county population distributions — specifically, by counting counties whose populations would require multi-member representation under equal-representation rules — rather than from running the redistricting algorithm with county-based constraints. Full algorithmic redistricting under county-boundary rules would require additional implementation work not performed in this paper."

---

### PP2-E5: Address Gerrymandering-by-Another-Name Concern

**File**: `research/E.5+party-based-allocation/sections/01-introduction.tex`
**Effort**: 2–3 hours

Add to the Introduction (after the counterfactual framing from PP1-E5 Part 1):
> "A reasonable objection to this analysis: any algorithm that explicitly targets a partisan outcome — even EG = 0 — is engaging in a form of partisan redistricting, which is exactly what the program argues neutral algorithmic redistricting avoids. We acknowledge this objection. Our response is that the EG-optimization exercise is valuable precisely because it quantifies the geographic constraints that prevent genuine partisan neutrality — showing that EG ≠ 0 under the neutral algorithm is not a failure of neutrality but an inevitable consequence of voter geography. The Rodden gap (C.5) is the correct framing: geographic sorting, not algorithmic bias, determines the efficiency gap floor."

---

### PP2-E6: STV Methodology Decision

**File**: `research/E.6+international-applications/sections/02-methodology.tex`
**Effort**: 2–4 hours (decision) or 3–6 weeks (implementation)

Choose one of:

**Option A (Remove Ireland/STV from scope)**: Add one sentence to Section 2: "We exclude Ireland (STV) from our international comparison because the Single Transferable Vote requires a different algorithmic framework than D'Hondt or Saint-Laguë; this extension is left for future work."

**Option B (Implement STV approximation)**: Implement Gregory method STV in scripts/ or reference the existing approximation. Document the approximation error bounds. Add Ireland results.

Option A is recommended for the current version; Option B for a future extension paper.

---

## PP3 Framing and Cross-Track Items

### PP3-E0: Add contingency footnotes for unaccepted companion papers

**File**: `research/E.0+experimental-overview/main.tex`
**Effort**: 30 minutes

For each citation of E.1–E.6 findings, add a footnote: "The [proportionality/county/federalism/etc.] analysis cited here is from a companion paper (E.X) currently under revision. The finding reported is from the current draft; final figures may change upon peer review."

---

### PP3-E7: Complete three P2 items

**File**: `research/E.7+lessons-learned/`
**Effort**: 2–4 hours

Per the E.7 revision plan:
1. Add "companion papers are under revision" note to the Introduction
2. Add D'Hondt vs. Saint-Laguë sensitivity discussion (noting that lessons from E.1 may change under Saint-Laguë)
3. Add cross-reference to G.14's practitioner comparison for the "when to use each approach" framing

---

### PP3-X2: Track-level D'Hondt policy decision

**Affects**: E.1, E.5, E.6
**Effort**: 1 day (decision + implementation)

Convene a single decision: should Track E use D'Hondt, Saint-Laguë, or both for electoral formula comparisons? 

**Recommendation**: Report both D'Hondt and Saint-Laguë in all three papers, clearly labeling which is used for the headline figure. For Electoral Studies (the target venue for E.1 and E.5), this dual reporting is standard practice. For E.6 (international applications), use the formula actually used in each country being compared.

Once decided, implement consistently across E.1, E.5, E.6.

---

### PP3-X3: E.4/E.5 Scope Delineation

**Affects**: E.4 (before writing begins)
**Effort**: 2–4 hours

Before writing E.4's main.tex, write a 1-page scope memo clarifying:
- E.4 answers: "What do geographically contiguous areas of partisan similarity look like as districts? What is the geographic structure of organic safe seats?"
- E.5 answers: "What happens when the redistricting algorithm explicitly minimizes efficiency gap? What geometric constraints prevent EG = 0?"

Key distinction: E.4 is about geographic analysis (where do partisan clusters exist?); E.5 is about optimization (can we achieve a partisan fairness target?). These are different questions and the papers should lead with different first paragraphs.

---

## Submission Sequence

| Priority | Paper | When ready | Key prerequisite |
|----------|-------|------------|-----------------|
| 1 | E.0 | Now (after PP3-E0 footnotes) | Contingency footnotes |
| 2 | E.3 | 1 week (after beta fix) | PP1-E3 |
| 3 | E.2 | 2 weeks (after abstract and methodology fixes) | PP1-E2, PP2-E2 |
| 4 | E.7 | 2 weeks (after PP3-E7) | PP3-E7 |
| 5 | E.1 | 4–6 weeks (after evaluation section written) | PP1-E1 |
| 6 | E.6 | 6–10 weeks (after UK/Canada/NZ results written) | PP1-E6 |
| 7 | E.5 | 4–6 weeks (after reframe + results section) | PP1-E5 |
| 8 | E.4 | After E.5 scope delineated; then write | PP3-X3 |

---

## Target Score Projection

After all PP1 and PP2 items resolved:

| Paper | Current | Target |
|-------|---------|--------|
| E.0 | 9.0/10 (accepted) | 9.0/10 (contingency footnotes only) |
| E.1 | 4.5/10 | 7.5/10 (after evaluation section) |
| E.2 | 5.0/10 | 7.0/10 (after factual corrections + methodology disclosure) |
| E.3 | 6.0/10 | 7.5/10 (after beta fix + compactness framing) |
| E.4 | 2.0/10 | 7.0/10 (after scope definition + writing) |
| E.5 | 3.5/10 | 7.0/10 (after reframe + results) |
| E.6 | 4.0/10 | 7.5/10 (after international results written) |
| E.7 | 9.0/10 (accepted) | 9.0/10 (after P2 items) |
| **Module** | **6.8/10** | **7.9/10** | |
