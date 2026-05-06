# Review 5 — Leah Liang (Reproducibility / Model Statute Implementation Lens)
**Paper**: D.4 — Adopting Algorithmic Congressional Redistricting: Legal Pathways, Constitutional Constraints, and Model Legislation
**Round**: R2
**Score**: 4/4 (Accept)

## Summary

As lead reviewer on Priority 1 statute deficiencies, I confirm that the two critical items I flagged are addressed. The VRA mode provision (P1-A, CRITICAL) is added as §2(e) VRA Adjustment. The compactness metric mismatch (P1-B) is resolved in the Drafting Notes. My remaining Priority 1 concerns — recertification procedure (P1-C), adjustment standing and conflict resolution (P1-D), reproducibility environment specification (P1-E), and at-large exemption (P1-F) — are not addressed in this revision. These are real statute implementation gaps but are not blocking at this stage. I am recommending acceptance.

## Response to Round 1 Issues

**P1-A (VRA mode — CRITICAL, CO-LEAD).** This was the most significant implementation gap in the original statute. The new §2(e) VRA Adjustment provision resolves it at the structural level. From an implementation standpoint, four observations:

(1) The ``minimum necessary'' standard (§2(e)(1)) is implementable: the Bureau would evaluate whether a proposed deviation produces the minimum number of district boundary changes needed to satisfy the Gingles preconditions for the identified community.

(2) The Gingles justification requirement (§2(e)(2)) creates the documentary record needed for judicial review. However, it does not specify who conducts the Gingles prongs 2 and 3 analysis (which requires election data) or what data the Bureau may use. This is a gap the Bureau's regulations would need to fill.

(3) The *Callais* reference (§2(e)(3)) is appropriate and legally current.

(4) The 15-day publication deadline (§2(e)(4)) is operationally feasible given the algorithm's computational speed.

**P1-B (Compactness metric — CO-LEAD with Duchin, Karypis).** The Drafting Notes paragraph resolves the legal litigation vulnerability by redefining ``maximum compactness'' as minimum-edge-cut partition of the TIGER adjacency graph — an objectively verifiable criterion. The note also provides the option for state legislatures to substitute Polsby-Popper, which is appropriate for state-level adoption. Resolved.

## Residual Concerns (Not Blocking)

**P1-C (Recertification — not addressed).** Section 2(b) still has no recertification procedure for subsequent census cycles. The Drafting Notes now include a paragraph noting that the statute is intended to allow recertification without statutory amendment, but no actual recertification provision was added to the statute's operative text. The provision I recommended (authorizing recertification 365 days before census data delivery) would close this gap and should be added before final publication.

**P1-D (Adjustment standing and conflict resolution — not addressed).** Section 2(d)(1) still allows ``any person'' to submit adjustments without standing requirements, submission limits, or conflict resolution procedures. The Bureau's regulations under §2(b)(3) are the appropriate vehicle for this, but the statute should at minimum specify that the Bureau's regulations must include conflict resolution procedures. A sentence added to §2(d)(1) would be sufficient.

**P1-E (Reproducibility environment — not addressed).** Section 2(b) still requires only publication of source code, without version-pinned dependencies, serialized adjacency graphs, or hardware environment specification. The Drafting Notes reference that the statute requires publication of ``underlying data and parameters'' in §2(c)(3), but this is the production step, not the certification step. The certification step should require a fully reproducible environment specification — particularly the serialized adjacency graph files, which are necessary to ensure that independent reproductions from raw census data produce identical outputs (as required by §2(a)(1)(D)).

**P1-F (At-large exemption — not addressed).** States with a single congressional district are still not addressed in the statute. The algorithm for at-large states trivially produces a single district covering the entire state, and the certification-adjustment-review process is inapplicable. An exemption provision should be added.

## Recommendation

Accept. The two most critical implementation gaps (VRA mode provision and compactness metric clarification) are resolved. The remaining gaps (recertification, standing, reproducibility environment, at-large exemption) are real but addressable in post-acceptance revision or in the Bureau's implementing regulations. The statute as revised represents a genuine and usable legislative template.
