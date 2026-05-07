# Review 3 — Moon Duchin (Mathematics / Algorithm-Law Interface Lens)
**Paper**: D.4 — Adopting Algorithmic Congressional Redistricting: Legal Pathways, Constitutional Constraints, and Model Legislation
**Round**: R2
**Score**: 4/4 (Accept)

## Summary

The revision resolves both Priority 1 issues I co-flagged: the compactness metric mismatch (P1-B, lead with Karypis) and the missing VRA mode statute provision (P1-A). The Miller v. Johnson predominant-factor analysis (P2-C) is now addressed in Section 6, which was the Priority 2 issue I flagged in Round 1. All my blocking concerns are resolved. I am recommending acceptance.

## Response to Round 1 Issues

**P1-B (Compactness mismatch — CO-LEAD).** The Drafting Notes paragraph is mathematically accurate. The critical distinction — that METIS minimizes edge cut in a graph (number of inter-tract adjacencies severed) while Polsby-Popper measures perimeter-to-area ratio in continuous space — is correctly drawn. The resolution is also mathematically coherent: define ``maximum compactness'' as minimum-edge-cut partition of the published TIGER adjacency graph. This definition is objectively verifiable from the algorithm's output and the published adjacency graph, which is exactly what legal enforceability requires. The note that Polsby-Popper scores are sensitive to coastline complexity is accurate and important — it explains why graph edge-cut is actually a more stable legal standard than Polsby-Popper for states with complex coastlines (Florida, Louisiana, Alaska). Resolved.

**P1-A (VRA mode statute provision — CO-LEAD with Liang, Stephanopoulos).** The new §2(e) VRA Adjustment is appropriately narrow. It authorizes state deviations subject to four conditions: minimum-necessary deviation, written Gingles justification, no partisan data consistent with *Callais*, and timely publication. The mathematical structure of the provision is correct: it authorizes the deviation at the output level (district map) rather than in the algorithm's parameter space, which means states can implement VRA adjustments without modifying the certified algorithm's code. This is the right architectural choice — it preserves the certified algorithm's integrity while creating a documented record of VRA-motivated modifications. Resolved.

**P2-C (Miller v. Johnson predominant-factor — addressed).** The Miller paragraph in Section 6.4 is the most mathematically important addition to the paper. It correctly applies the predominant-factor test to the VRA mode's structure. The key argument is architecturally sound: the base partition is produced without any demographic data; the VRA adjustment modifies specific district boundaries with documented Gingles justification; the two operations are analytically separable. Under this architecture, race does not ``predominate'' in the base partition (no demographic data entered it), and in the VRA adjustment, race is explicitly constrained to the minimum necessary for Section 2 compliance. The analogy to population equality as a constraint rather than a ``predominant factor'' is apt and should be persuasive to courts. Resolved.

## Residual Concerns (Not Blocking)

**The 2% deviation tail.** My Round 1 concern about the 2% of districts exceeding ±0.5% deviation and its relationship to *Karcher v. Daggett*'s ``as nearly as practicable'' standard is not addressed. The statute still reads as a hard cap (no exceptions) in §2(a)(1)(C)(i), which conflicts with the empirical 98% figure. A sentence in the Drafting Notes explaining that the 2% tail results from geographic constraints and is Karcher-compliant because the algorithm minimizes deviations subject to contiguity and census tract indivisibility requirements would resolve this.

**Shaw/bizarreness test still analytically primary.** The Section 6.4 Shaw analysis now correctly includes the Miller v. Johnson predominant-factor standard, but the section still leads with the bizarreness/shape analysis before engaging Miller. For legally sophisticated readers, Miller is the more demanding test and should probably be addressed first. This is a minor structural suggestion, not a substantive concern.

## Recommendation

Accept. The P1 and P2 issues are resolved. The paper makes a technically rigorous contribution to the algorithm-law interface literature.
