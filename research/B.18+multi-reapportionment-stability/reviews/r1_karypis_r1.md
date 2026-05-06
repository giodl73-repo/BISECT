# Review 1 — Reviewer: George Karypis (METIS / Graph Partitioning)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 1
**Score:** 3/4

## Summary

This paper characterizes how the ApportionRegions bisection tree changes when congressional seat counts change at decadal reapportionment, and measures the resulting tract-level boundary disruption. The prime factorization analysis is technically correct and represents a genuine contribution to understanding the algorithmic behavior of the system across reapportionment cycles. The simulation methodology is appropriate given the unavailability of 2030 census data.

## Strengths

The prime factorization analysis is the paper's strongest contribution. The observation that the tree depth is determined by the factorization of k, and that prime k produces a flat single-level tree while composite k produces a hierarchical tree, is correct. The Texas case (38 = 2×19 → 41 prime) is the cleanest and most dramatic illustration: the transition from a two-level tree to a flat 41-way split is qualitatively different from any within-tree parameter variation, and the paper correctly predicts large boundary disruption (d_Ham = 0.23).

The GerryChain baseline comparison is well-conceived. Using the ReCom step size (d_Ham ≈ 1/k per step) as the unit of measurement for boundary disruption provides an intuitive scale: the 2030 Texas reapportionment is a "9-step disruption." This is exactly the kind of interpretable scale reference that makes the paper accessible to non-technical audiences.

The factorization table (Table 2) is complete and accurate. The characterization of Illinois, Michigan, and Pennsylvania as "prime to composite" transitions (the reverse of Texas) is an important observation that is easily missed: these states will gain hierarchical structure in 2030, potentially improving compactness but at the cost of boundary disruption.

## Weaknesses and Concerns

The seat projection claims require more careful sourcing. The abstract and introduction state: "TX: 38→41, CA: 52→50, FL: 28→29, NY: 26→25." The California claim (52→50) deserves scrutiny. California currently has 52 seats (118th Congress, after the 2020 census). A projection of 50 seats would represent a loss of 2 — which is on the high end of current apportionment projections. Many apportionment models project California at 51 or 52 seats for 2030 (depending on whether migration trends continue). The paper should report the uncertainty range from the Census Bureau's own projection models and note whether the main conclusions hold under alternative seat count projections.

Similarly, the Florida projection (28→29) also seems potentially low — some models project Florida gaining 2 seats (28→30), which would produce a different factorization analysis (30 = 2 × 3 × 5, three-level tree, not the composite-to-prime transition described). The paper's structural conclusions are highly sensitive to whether FL lands at 29 (prime) or 30 (composite), and this uncertainty is not adequately addressed.

The simulation methodology has a known limitation that is acknowledged but not sufficiently quantified: "We are measuring the boundary disruption from the seat count change alone, not from the combination of seat change and population redistribution. The actual 2030 disruption will be larger." The paper should provide an estimate of how much larger. B.18's own simulation framework can do this: run the 2020-seat algorithm on the 2020 graph, then run the 2030-seat algorithm on a projected 2030 graph (derived by scaling tract populations proportionally to Census Bureau state-level projections). This would give a more realistic estimate of total 2030 disruption.

## Minor Issues

- Table 4 shows that the Florida reapportionment produces d_Ham = 0.19, which the paper describes as "also dramatically affected." But the tree change analysis (28 = 2²×7 → 29 prime, both producing flat-tree transitions) does make this prediction reasonable. The "composite to prime" framing for Florida is correct — but only if the seat count lands at 29.
- The "prime frequency problem" discussed in Section 7.3 is interesting and correctly invokes the prime number theorem. The note that ~27% of seat counts near 40 are prime is correct asymptotically. For the relevant range [1,60], the exact prime density could be computed: there are 18 primes in [1,60] out of 60 numbers, giving 30% density — slightly higher than the asymptotic estimate.
- The note about Illinois/PA going from prime=17 to composite=2^4=16 should clarify that 2^4 produces a four-level tree (depth=4, each level bisecting): the paper says "Four-level (2^4)" in Table 3 but should explain this means depth=4, which is deeper than any current state configuration. This maximum-depth case deserves more analysis.

## Recommendation

Accept with moderate revisions. The seat projections need uncertainty ranges, and the simulation should include a projected-population sensitivity analysis for at least the Texas and California cases. The core analysis is technically correct and valuable.
