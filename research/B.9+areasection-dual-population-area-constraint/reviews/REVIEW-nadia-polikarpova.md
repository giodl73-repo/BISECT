# Review R-3: Nadia Polikarpova
**Paper**: AreaSection: Simultaneous Population and Land-Area Balance in Minimum-Edge-Cut Redistricting
**Date**: 2026-05-03
**Score**: 2.5 / 4

---

## Summary

AreaSection adds ncon=2 METIS constraints (population + area) to redistricting bisection. The algorithm uses dual vertex weights (population interleaved with hectare-scaled ALAND), guided by a Lorenz curve pre-filter that prunes ratios for which area balance is geometrically impossible. Subsequent recursive levels revert to ncon=1. The Lorenz Proposition characterises necessary conditions under which a ratio is feasible in the non-contiguous relaxation.

The implementation specification is unusually precise. The algorithm is clearly structured. However, the paper makes three claims that exceed what the algorithm actually delivers: it "enforces" area balance when the constraint is soft; the Lorenz condition "identifies feasible ratios" when it identifies only the non-contiguous relaxation; and results are "geographically balanced" without scoping that claim to the first bisection level only.

---

## Strengths

**S1. Implementation specification is unusually precise.** The paper pins exact parameter values: ubvec [1.001, 1.10], area scaling to hectares (÷10,000 m²) with max(1) clamping, isoperimetric normalisation by sqrt(min(left_k, right_k)), the right_w = 1.0 - left_w identity to guarantee floating-point sum-to-one. This level of reproducibility detail is rare in redistricting papers.

**S2. The Lorenz curve analysis is well-motivated.** The pre-filter logic — checking whether [lorenz_min_area(p), 1 - lorenz_min_area(1-p)] overlaps the allowed area window — is computationally cheap and correctly positioned before the expensive multi-seed METIS search.

**S3. The ncon=2 first-level / ncon=1 recursive design is architecturally clean.** Maintaining a strict boundary between the initial geographic balance pass and subsequent compactness optimization avoids cascading constraint violations.

**S4. The METIS API path is well-documented.** The paper clearly distinguishes the multi-constraint path and explains why certain flags are omitted in the vendored FFI context.

---

## Weaknesses

**W1. Proposition 3.1 is stated as a necessary-and-sufficient feasibility characterisation, but the proof establishes necessity only under the non-contiguous relaxation.**

The Lorenz pre-filter operates on the idealized problem: "can we select *any* set of tracts with this population fraction such that their total area falls within the target window?" This relaxation ignores contiguity. The actual problem METIS solves requires connected subregions. A ratio that passes the Lorenz filter may be infeasible in the contiguous problem (geographic chokepoints, elongated states). The paper should state explicitly that Proposition 3.1 provides a necessary condition only. The abstract and §3 introduction refer to the filter as identifying "feasible ratios," which implies sufficiency.

**W2. The claim that AreaSection "enforces" area balance is imprecise.**

Section 4 uses "enforces area balance" and "guarantees 10% area swing." This is not what ubvec does. METIS returns METIS_OK even when ubvec is violated when population and area constraints conflict. Under the constraint hierarchy implicit in the design — population ubvec [1.001] is much tighter than area ubvec [1.10] — population balance will routinely override area balance. The correct language is "targets" or "penalises deviations from" area balance, not "enforces."

**W3. The recursive scope of geographic balance is undefined.**

The paper asserts AreaSection produces "geographically balanced" districts. But the dual constraint applies only at the first bisection level; all recursive calls revert to ncon=1. For California (k=52, bisection depth 6), geographic balance is only constrained at the root split. The 5 subsequent recursive levels are free to produce any area distribution. There is no formal statement of what "geographic balance" means for the full bisection tree.

**W4. The contiguity guarantee is not formally established.**

The implementation omits the -contig flag in the bisection paths. The paper mentions contiguity repair was implemented then removed when -contig was restored, but does not specify under what conditions -contig is active in the final code. The paper should state clearly: contiguity is not enforced by METIS in the ncon=2 path; it is relied on empirically for planar graphs but is not proved.

**W5. The isoperimetric normalisation denominator lacks a derivation.**

The ratio selection criterion divides raw edge-cut by sqrt(min(left_k, right_k)). An isoperimetric inequality on graphs would give a specific functional form for how boundary length scales with region size; sqrt is one possibility but not the only one. Without a derivation, the normalisation is an empirically-tuned heuristic presented as if it were principled.

---

## P1 Items (Required Before Accept)

**P1-I. Proposition 3.1: fix scope claim.**
Replace every instance of "feasible ratio" in §3.2 and §4 with "Lorenz-feasible ratio," and add a remark immediately after Proposition 3.1 stating: "Lorenz feasibility is a necessary condition for contiguous feasibility; it is not sufficient. A ratio may pass the Lorenz filter and still be infeasible for the connected subgraph problem solved by METIS."

**P1-II. Replace "enforces" with precise language throughout.**
Audit all instances of "enforces area balance," "guarantees area balance." Replace with language that reflects the soft-constraint reality: METIS *targets* area balance subject to the tighter population constraint taking priority when they conflict.

**P1-III. State the contiguity guarantee (or lack thereof) explicitly.**
Add a paragraph stating: the ncon=2 and ncon=1 recursive paths do not set the METIS Contig option; contiguity is not algorithmically guaranteed; for planar census tract graphs contiguity is typically observed empirically but is not proved.

---

## P2 Items (Recommended)

**P2-I.** Formalise or disclaim the tree-level geographic balance claim — either prove a bound on the area distribution achieved at the leaf level, or scope the geographic balance claim to the root split only.

**P2-II.** Derive or explicitly label the isoperimetric normalisation — either provide a formal justification for the sqrt(min_k) denominator, or label it as an empirically-motivated heuristic and ablate over alternatives.

**P2-III.** Clarify the part_kway vs part_recursive selection criterion for ncon=1 vs ncon=2 paths.

**P2-IV.** Add an empirical area balance verification table reporting actual left/right area fractions achieved after AreaSection completes for representative states, compared to the ubvec target window.

---

## Verdict

The algorithm is well-specified and the implementation is precise about the METIS parameter surface. The Lorenz pre-filter is a clean and computationally efficient idea. However, the paper makes three precision failures: (1) "enforces" area balance when the constraint is soft; (2) implies the Lorenz condition is sufficient when it is only necessary; (3) claims "geographic balance" for the full bisection tree when it only applies to the first level.

These are precision failures of the kind that matter in an algorithms paper. They require rewording, not redesign. I would accept with mandatory P1 revisions.

**Score: 2.5 / 4** — Major revision required; borderline accept after revision.
