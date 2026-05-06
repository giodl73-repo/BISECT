# Review 4 — Reviewer: Nicholas Stephanopoulos (Election Law / Partisan Gerrymandering)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 1
**Score:** 3/4

## Summary

This paper addresses an important aspect of the DIA framework's temporal dimension: what happens at each decadal reapportionment, and how much disruption does the algorithmic system produce? The legal and statutory framing in Section 6 is strong and directly useful for DIA drafting. My concerns are primarily about the accuracy of the seat projections and about the constitutional framing of the disruption analysis.

## Strengths

The statutory note (Section 6) is the paper's most legally valuable contribution. The three DIA drafting implications — no grandfathering clause, no transition period, prime factorization predictable in advance — are concrete and well-reasoned. The argument that the AR algorithm "already produces reapportionment-stable maps by construction without an explicit continuity bonus" is important: it answers the legislative question of whether a stability-preservation mandate is needed.

The political neutrality of the fresh-recomputation design (Section 6.2) is well-stated. The constitutional basis in *Wesberry v. Sanders* for fresh recomputation — responding to population redistribution by resetting to equal population — is correct and provides a firm statutory foundation.

The note on the 2030 Texas case (Section 6.4) is particularly valuable. The suggestion that "states and legislators should understand this before lobbying for a k=40=2³×5 apportionment" — implying that the arithmetic of prime factorization can motivate lobbying for specific seat counts — is a sophisticated political insight that deserves more development.

## Weaknesses and Concerns

The seat projection claims are the paper's main vulnerability from a legal evidentiary standpoint. The abstract states "TX: 38→41, CA: 52→50, FL: 28→29, NY: 26→25" and attributes these to "Census Bureau projections." The accuracy of these projections matters because the paper's entire structural analysis — which tree changes are dramatic vs. minor, which states are "most affected" — depends on the specific seat counts.

For California, current Census Bureau population estimates show slower-than-projected migration out of California (as of 2024-2025 data), making a 52→50 projection possible but far from certain. Some apportionment models produce 52→51 (loss of one) rather than two. For Florida, 28→30 (gain of two) is more likely than 28→29 (gain of one) under many current projections. The paper needs to show that its conclusions are robust to ±1 seat variation in the projected counts.

Specifically: if Florida ends up at 30 rather than 29, it is NOT a prime, and the paper's "composite to prime" characterization is wrong. 30 = 2×3×5 has a three-level tree structure, not a flat prime structure. This would substantially change the disruption estimate for Florida. The paper must include a sensitivity analysis: for each projected seat count, show what happens if the projection is off by ±1.

The constitutional framing in Section 6.2 correctly cites *Wesberry v. Sanders* but omits *Karcher v. Daggett* (1983), which held that states must make "a good faith effort to achieve precise mathematical equality" and must justify any population deviation. The fresh-recomputation design satisfies *Wesberry*/*Karcher*, but this should be stated explicitly.

The paper notes that "the stability result is reassurance, not a target" (Section 6.3) — the DIA optimizes for compactness, and stability is a byproduct. This is legally important because it immunizes the DIA from challenges arguing that stability should be weighted against compactness. The paper should make this argument more explicitly: by not targeting stability, the DIA cannot be accused of prioritizing stability over population equality.

## Minor Issues

- The "incumbent protection analysis" (Section 5.4) is legally relevant — incumbent protection has been a basis for redistricting challenges in some state courts — but the paper's treatment is too brief to be useful for that purpose.
- The open question about "optimal transition strategies" (Section 7.2) should note that any transition-minimizing variant of the AR algorithm would introduce a path dependency (the current map influences the next map) that could be exploited for partisan continuity. This tension between transition stability and political neutrality should be acknowledged.

## Recommendation

Accept with moderate revisions. Add ±1 seat sensitivity analysis for all projected seat counts (particularly Florida and California), add *Karcher v. Daggett* to the constitutional citations, and expand the incumbent protection analysis.
