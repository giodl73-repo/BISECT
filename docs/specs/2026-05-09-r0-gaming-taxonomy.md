---
title: "A Taxonomy of Gaming Vectors in Algorithmic Redistricting"
series: R.0
status: Planned
date: 2026-05-09
track: R-adversarial-robustness
---

## Claims
1. Five gaming vectors exist for algorithmic redistricting: (1) parameter manipulation, (2) input data manipulation, (3) geographic definition gaming, (4) selective result presentation, and (5) algorithm version selection. Only vectors 1-3 affect the plan itself; 4-5 affect the perception of the plan.
2. For bisect specifically: vectors 1 (parameter tuning) and 3 (resolution choice) are bounded by the DIA's parameter pre-registration requirement. Vector 2 (input data) is bounded by the SHA-256 audit chain. Vector 4 is bounded by the audit chain's manifest disclosure.
3. The maximum achievable partisan shift under any gaming vector, given the DIA's constraints, is estimated at 0.5 D-seats nationally — within the sampling noise established by B.7 (CV < 2% for 96% of states).
4. The gaming-resistance analysis converts the "algorithm can be gamed" objection from an abstract concern to a quantified, bounded, auditable claim: ≤0.5 seats nationally, detectable via SHA-256 audit.
