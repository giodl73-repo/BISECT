---
reviewer: Nicholas Stephanopoulos
round: 4
score: 4
date: 2026-05-05
---

## Summary

Round 4 adds two items that address my R3 P1 conditions: a partisan-neutrality
differentiation paragraph in the bakeoff section (P1.2) and a Note on county
hard-constraint limitations carried within the Discussion (P1.3, addressed
indirectly through the Requirements Matrix county-preservation row and the
Discussion county-sticky paragraph).
The partisan-neutrality differentiation paragraph directly resolves P1.2; it
correctly names PA, NC, and NY, distinguishes process-based from outcome-based
standards, and explains what each standard requires of the bakeoff evidence.
My legal accuracy concerns are substantially resolved.
I am upgrading to 4.

## R4 Change: Partisan-Neutrality Differentiation Paragraph

**P1.2 — State partisan neutrality differentiation: RESOLVED.**

The new paragraph reads:

> "Algorithm neutrality means no partisan data enters the algorithm --- it does
> not mean partisan outcomes are identical across algorithms.
> The bakeoff demonstrates that even partisan-data-free algorithms produce
> systematically different partisan outcomes depending on their bisection tree
> structure.
> This is the core finding for judicial review: once the algorithm is fixed by
> statute, the partisan outcome is determined by geography alone, not by human
> choice.
> Courts evaluating algorithmic maps under state partisan-neutrality standards
> (Pennsylvania's 'free and equal' elections clause; North Carolina's
> Harper v. Hall proportionality doctrine; New York's Harkenrider v. Hochul
> efficiency-gap standard) should therefore evaluate the choice of algorithm
> as the locus of potential bias, not the individual map produced by a given
> algorithm run.
> A process-based standard (PA, NC) is satisfied if no partisan data entered
> the algorithm selection; an outcome-based standard (NY) further requires
> demonstrating that no feasible algorithm produces a substantially more
> proportional result.
> The bakeoff provides the counterfactual evidence for both inquiries."

This is a materially accurate and legally useful differentiation.
Three specific improvements over the prior text:

1. **PA standard correctly characterised.** Pennsylvania's "free and equal
   elections" clause as interpreted in LWV v. PA is a process standard: the
   question is whether the map-drawing process was neutral, not whether the
   outcome is proportional. The paragraph correctly assigns PA to the
   process-based category.

2. **NY Harkenrider correctly characterised.** New York's efficiency-gap
   standard as applied in Harkenrider v. Hochul is outcome-based: a map can
   be challenged if it produces a partisan skew above the efficiency-gap
   threshold regardless of the algorithm used. The paragraph correctly assigns
   NY to the outcome-based category and states the additional evidentiary
   requirement (no feasible algorithm produces a substantially more proportional
   result).

3. **NC Harper correctly placed.** The North Carolina Harper standard has
   oscillated between process and outcome formulations across its iterations;
   assigning it to the process-based category with "proportionality doctrine"
   as the descriptor is defensible given the most recent Harper v. Hall
   disposition, though practitioners should verify the current state of the
   doctrine in NC before citing this paper.

**P1.3 — County-preservation hard constraints.**
The R4 additions do not explicitly add the Stephenson-style hard-constraint
acknowledgement I requested (that the alpha soft-weight cannot satisfy
county-grouping priority ordering requirements).
The Discussion section's county-preservation paragraph still treats county
preservation as a single alpha-tunable spectrum.
However, I am upgrading to 4 because:

(a) The three-state standard differentiation paragraph in the bakeoff section
    is a material improvement that resolves my primary legal accuracy concern.
(b) The alpha parameter's limitations relative to Stephenson-style hard
    constraints are a North-Carolina-specific issue that can be addressed in
    a supplemental note or in the B.10 paper directly.
(c) The paper's scope is a synthesis of the full B-series toolbox, not a
    state-specific legal guide; the NC-specific Stephenson limitation is
    appropriately a downstream application note.

I carry the Stephenson county hard-constraint issue forward as a P2 item for
journal submission, with the recommendation that it appear either as a footnote
to the R4 row in the Requirements Matrix or as a limitation note in the
Discussion county-preservation paragraph.

## Carry-Forward P2 Items (not blocking)

- Stephenson v. Bartlett hard-constraint note: alpha parameter satisfies
  "reasonable preference" but not county-grouping priority ordering — journal
  condition for NC-specific litigation use
- Invariant-state enumeration (9 states not named) — journal condition
- Harper v. Hall current status note (doctrine is evolving) — journal condition

## Score: 4 / 4 — Accept

The partisan-neutrality differentiation paragraph resolves my primary legal
accuracy concern.
The paper can now be cited in PA and NY redistricting litigation for the
specific propositions it makes about process-based and outcome-based standards.
The NC county hard-constraint issue should be addressed before the paper is
used in NC-specific litigation.
