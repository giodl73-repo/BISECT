---
reviewer: Moon Duchin
round: 1
score: 2
date: 2026-05-05
---

## Summary

B.6 is a complexity paper about the balanced k-cut redistricting problem. It attempts NP-hardness via reduction, claims an O(sqrt(log n)) approximation ratio via the ARV framework, and characterises empirical runtime as nearly linear. This is exactly the kind of theoretical grounding the redistricting literature needs. Unfortunately, the proof techniques are applied incorrectly. The NP-hardness reduction ignores the contiguity requirement (the hardest part of redistricting complexity), and the approximation ratio borrows from a result about Sparsest Cut that does not apply to Balanced k-Cut with contiguity. These are not typos — they reflect conceptual errors in the theoretical setup that require substantial revision.

## Strengths

- **The redistricting graph model is formally correct.** Problem 1 is the right formal model: weighted planar graph, integer population weights, positive edge weights, k-partition with population balance and contiguity. This is a cleaner formulation than most redistricting papers provide.
- **The empirical runtime analysis is methodologically sound.** Fitting T = a * n^b via OLS on log-transformed data with R^2 = 0.984 is convincing. The three-census-year stability (b ≈ 1.07 in all three) is strong evidence of structural near-linearity. The runtime table covers a 40x range in n, which is adequate for the log-log fit.
- **The space complexity analysis is correct and novel.** The O(n log k) space bound and the numerical verification for California (under 3 MB at tract resolution) are correct and useful. I am not aware of prior work that provides this calculation for redistricting graph partitioning.

## Weaknesses / P1 Items (Required Fixes)

- **The NP-hardness reduction fails for the contiguity requirement.** This is the central error in the paper. Planar Graph Bisection (the problem used as the source of the reduction) does not require the parts to be connected. But Problem 1 requires each P_i to induce a connected subgraph. The reduction maps a Planar Graph Bisection instance to a redistricting instance by setting all weights to 1. But a minimum bisection of the planar graph G' may produce disconnected parts — which would not be a valid solution to Problem 1 (since the redistricting problem requires connected districts). Therefore, a solution to Problem 1 with minimum edge cut is NOT equivalent to a minimum bisection of G' under disconnected parts. The proof sketch says "contiguity is equivalent to connectivity and Planar Graph Bisection already requires connected parts" — but this is factually incorrect. Garey and Johnson (1976) do not impose connectivity on the bisection parts. This invalidates the reduction as written. To fix this, the authors would need to either (a) cite a variant of Planar Graph Bisection that requires connected parts (such results exist in the literature on connected graph bisection — see Chataigner et al. 2007), or (b) construct a gadget reduction that enforces connectivity.
- **The O(sqrt(log n)) approximation ratio for Balanced k-Cut is not established by the proof sketch.** The ARV framework (Arora, Rao, Vazirani 2009) gives an O(sqrt(log n)) approximation for the Uniform Sparsest Cut problem, which is related to but distinct from Minimum Balanced Bisection. For Minimum Balanced Bisection, even on planar graphs, the best known approximation ratio is O(sqrt(log n)) via ARV, but the proof requires applying the ARV SDP to the planar case — it is not a direct consequence of the Lipton-Tarjan separator size. The paper's proof sketch claims the O(sqrt(log n)) ratio follows from "METIS's FM refinement converging to a local minimum within this bound by the ARV framework." This is not what the ARV framework says. ARV is a polynomial-time algorithm that solves an SDP relaxation; FM refinement is a local search heuristic that has no known connection to the ARV bound. The approximation ratio claim must be either correctly proved or removed.
- **The runtime theorem uses niter = O(log n) without justification.** Theorem 2 states that "with niter = O(log n), the constant factor in FM refinement is absorbed into the log n factor." But the paper's experimental setup uses niter = 100 (a constant, not O(log n)). For n up to 8,057 (California), log(8,057) ≈ 13, so niter = 100 > log(n). The theorem's runtime bound is therefore loose by a constant factor of ~7–8. This should be stated honestly: the actual bound is O(niter * n log k), with niter fixed at 100, giving O(n log k) up to a constant of 100.

## P2 Items (Suggestions)

- **Survey the connected graph partition complexity literature.** There are specific NP-hardness results for the connected balanced k-partition problem on planar graphs (Dyer and Frieze 1985 is a starting point). A correct reduction could build on these rather than attempting a new reduction from Planar Graph Bisection.
- **Consider weakening the approximation ratio claim to an empirical statement.** If proving the O(sqrt(log n)) bound for METIS is not feasible in the current paper, the approximation ratio section could be reframed as: "The best known polynomial-time algorithm (ARV) achieves O(sqrt(log n)) approximation for Minimum Bisection on planar graphs. METIS achieves comparable results empirically (2.9% above the best seed found among 10,000 trials)." This is an honest statement that does not overclaim a formal guarantee for a heuristic.

## Score: 2 — Major Revision

The NP-hardness reduction and the approximation ratio derivation both contain conceptual errors that cannot be fixed by minor revision. The paper would benefit from a collaboration with a complexity theorist or a significant reduction in its theoretical claims. The empirical contributions are sound and could stand as a shorter paper without the theoretical claims.
