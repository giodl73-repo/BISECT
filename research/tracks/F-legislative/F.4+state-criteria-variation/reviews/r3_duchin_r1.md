# Review 3 — Moon Duchin
**Paper**: F.4: Satisfying 50 Different Rule Sets — State Constitutional Redistricting Criteria and Algorithmic Adaptation
**Round**: R1
**Score**: 3/4

## Summary

F.4 is the most legally oriented paper in the F track. From a mathematical perspective, the paper's main claims are: (1) the algorithm's compactness output is equivalent to perimeter minimisation when edges are weighted by shared boundary length; (2) county preservation maps to a multiplicative edge weight; and (3) partisan neutrality is structural. My review focuses on these claims and on the ensemble context that is missing.

## Strengths

The compactness-as-edge-cut equivalence is correctly stated: edge weights proportional to shared boundary length mean that minimising total edge cut is approximately equivalent to minimising total district boundary length, which is the geometric definition of compactness. The paper correctly notes that "edge-cut minimisation is equivalent, in expectation, to perimeter minimisation when edges are weighted by shared boundary length" — the "in expectation" qualifier is appropriately cautious (exact equivalence would require continuous geometry, not discrete graph cuts).

The five-type taxonomy's separation of constitutional from statutory criteria (Section 5) is useful and correctly applied to the key examples (California's priority ordering is constitutional, Iowa's compactness requirement is statutory but uses mandatory language).

## Concerns

**C1 — Single-map results cannot satisfy all criteria simultaneously.** The paper's central argument is that parameterization through YAML allows the algorithm to satisfy different states' criteria. But the paper generates only one map per state (seed 42). For states with multiple competing criteria (e.g., California: population equality, VRA compliance, compactness, community of interest, county preservation), the algorithm must satisfy all criteria simultaneously with a single parameter vector. The paper does not demonstrate that any single YAML configuration simultaneously satisfies all applicable criteria for a given state — only that each criterion maps to some parameter. Whether the parameters can be jointly optimised to satisfy all criteria is not shown.

**C2 — No ensemble analysis for legal compliance.** For legal purposes, the relevant question is not whether one algorithmic map satisfies the criteria, but whether the algorithm's output space systematically satisfies them. If an algorithm produces one map that happens to be compact, that does not establish that the algorithm produces compact maps in general. The paper's claims would be strengthened substantially by showing that across multiple seeds (or across the plan ensemble), maps generated with the appropriate YAML parameters satisfy the relevant criteria with high probability.

**C3 — Arizona mean margin calculation.** The paper reports that METIS maps produce "a mean margin of 7.2 percentage points in competitive congressional districts (margin < 15 points), compared to a mean margin of 9.4 in the 2022 enacted Arizona map." These specific figures appear without a source or derivation. What is the data source for the 2022 enacted Arizona map's competitive margin? How was the METIS map's competitive margin computed — using what election data interpolated to what resolution? This claim needs documentation.

**C4 — Discretionary criteria and the algorithm's implicit optimisation.** Section 5.3 states that "for discretionary criteria, the algorithmic approach has a structural advantage: it will naturally minimize county splits and maximize compactness within the bounds of population balance." This is not quite right: the algorithm minimises edge cut, which correlates with compactness and county splits but is not identical to either. The paper should be precise: the algorithm does not maximise Polsby-Popper (which it does not compute during redistricting), it minimises total boundary length (edge cut weighted by boundary length). This is approximately equivalent for the criteria discussed, but the distinction matters for legal claims about what "the algorithm does."

## Recommendation

Accept with minor revisions. C1 (joint satisfaction of multiple criteria) and C4 (edge cut vs. PP equivalence) are the most important mathematical concerns. The legal analysis is strong and the taxonomy is useful.
