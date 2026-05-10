---
reviewer: Jonathan Rodden
round: 2
score: 4
date: 2026-05-05
---

## Summary

The revision addresses my two most important concerns from Round 1. The partisan impact of county preservation is now directly measured and reported in Section 5.2: mean |Δseats| = 0.3 across 43 non-trivial states at alpha_c = 1.0 vs. 3.0. The constitutional criterion hierarchy concern — that county-sticky weighting applies criteria (4) and (5) simultaneously rather than hierarchically — is addressed in the new criterion-hierarchy paragraph in Section 5.2. The 34-state enumeration gap (P1.3) is partially addressed with a clarifying sentence but still lacks the full list.

## P1 Items: Response Assessment

**P1.1 (County split counts not verified against TIGER data) — Addressed.** The methodology now specifies: "County splits are counted from the TIGER/Line 2020 tract-level FIPS assignment: a county split is any (county FIPS, district) pair where the district includes tracts from that county but does not contain all tracts assigned to that county." This is a precise and reproducible definition. The 487 baseline count is now verifiable from the procedure described. The data vintage (2020 TIGER/Line) is explicitly stated. I consider this item closed.

**P1.2 (Partisan implications not discussed) — Addressed.** The new partisan-neutrality paragraph in Section 5.2 reports mean |Δseats| = 0.3 across 43 non-trivial states when comparing alpha_c = 1.0 to alpha_c = 3.0. This directly tests my concern that rural county preservation might systematically favour Republicans. The |Δseats| = 0.3 result — comparable to the RB-vs-NW result in B.5 — supports the conclusion that county preservation is geographically rather than politically determined. The mechanism is correctly identified: the algorithm eliminates avoidable splits regardless of the political lean of the county. I consider this item closed.

**P1.3 (34-state list not provided) — Partially addressed.** Section 5.2 now references the 34 states with county preservation requirements and notes the distinction between constitutional (mandatory) and statutory (directory) provisions. However, the list of states is still not provided in the paper body. A brief table or footnote listing the 34 states would complete this item. The NCSL (2021) citation is appropriate.

## Assessment of New Additions

**The criterion hierarchy paragraph is well-constructed.** The framing — "county-sticky weighting implements subdivision preservation as a soft secondary criterion after population balance, consistent with the constitutional hierarchy" — is the correct legal argument. The key insight is that population balance is enforced as a hard constraint (ufactor=5 in METIS) while county stickiness operates only within the feasible population-balanced region. This implements the "to the extent possible" language correctly.

**The partisan neutrality finding is the most legally significant addition.** |Δseats| = 0.3 across 43 non-trivial states is the right measurement and it directly addresses my concern. The parallel structure to B.5's finding (|Δseats| = 0.2 for RB vs. n-way) is appropriate and makes the pattern across papers coherent.

## Score: 4 — Accept with Minor Revisions

The partisan impact measurement and the criterion hierarchy explanation are substantive improvements. The 34-state list remains a documentation gap (P2). The paper's core contribution is well-supported and legally defensible. I recommend acceptance.
