# Review 1 — George Karypis (Algorithmic / Technical Accuracy Lens)
**Paper**: D.4 — Adopting Algorithmic Congressional Redistricting: Legal Pathways, Constitutional Constraints, and Model Legislation
**Round**: R2
**Score**: 4/4 (Accept)

## Summary

The revision addresses the two Priority 1 technical concerns I co-raised in Round 1: the compactness metric mismatch (P1-B, lead with Duchin) and the absence of a VRA mode provision in the model statute (P1-A). The compactness note in the Drafting Notes section correctly describes the relationship between METIS edge-cut minimization and Polsby-Popper, and proposes a coherent resolution. The new §2(e) VRA Adjustment provision closes the statutory gap. I am recommending acceptance.

## Response to Round 1 Issues

**P1-B (Compactness metric mismatch — CO-LEAD).** The Drafting Notes now include a paragraph titled ``Note on compactness measurement'' that accurately characterizes the distinction: METIS minimizes edge cut in the census-tract adjacency graph; the Polsby-Popper metric minimizes perimeter-to-area ratio in continuous space; these are correlated but distinct. The paragraph correctly notes that courts would evaluate whether the algorithm was run correctly on the published adjacency graph, not whether the Polsby-Popper score exceeds a threshold. This resolves the litigation vulnerability I identified. The statutory text still uses the Polsby-Popper formulation in Section 2(a)(1)(C)(iii), but the Drafting Notes now provide the doctrinal bridge. The alternative formulation for state legislatures preferring Polsby-Popper is a useful addition. Resolved.

**P1-A (VRA mode not in statute — CO-LEAD).** The new §2(e) VRA Adjustment is a well-drafted addition. It correctly: (1) limits deviations to the minimum necessary; (2) requires written Gingles justification; (3) explicitly cites *Callais* on the prohibition of partisan data in VRA determinations; and (4) imposes a publication deadline for adjusted maps and justifications. The provision is cleanly drafted in U.S. Code style and integrates correctly with the existing Section 2(b)–(d) structure. Resolved.

## Residual Observations (Not Blocking)

**Open-source specification.** My Round 1 concern about the underspecified open-source requirement (no license type, no version control requirement, no publication timing constraint) is not addressed. Section 2(a)(1)(E) still states only ``source code that is published and publicly accessible.'' The Bureau's regulations under Section 2(b)(3) would presumably fill this gap, but the statute should specify the minimum requirements for reproducibility at the statutory level rather than delegating them entirely to regulation. A sentence specifying that the Bureau must publish version-pinned software dependencies alongside the source code would improve the statute's reproducibility guarantee.

**METIS-versus-equivalents.** The argument that the paper's constitutional claims apply to the class of recursive graph bisection methods, not to METIS specifically, is not addressed. This is a minor point — the statute correctly specifies properties rather than implementation — but the paper's legal discussion occasionally refers to ``the METIS algorithm'' in ways that suggest METIS-specificity. Cleaning up the language to consistently refer to ``a certified redistricting algorithm satisfying the requirements of Section 2(a)(1)'' rather than ``METIS'' would improve precision.

## Recommendation

Accept. The two critical P1 technical issues are resolved. The compactness clarification and the VRA mode statutory provision are both well-executed additions to the paper.
