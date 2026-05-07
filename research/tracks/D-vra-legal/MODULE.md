# Track D — VRA and Legal Implementation

**Module**: track-D
**Theme**: Does algorithmic redistricting satisfy the legal requirements of the Voting Rights Act Section 2, and what is the pathway from algorithm to enacted federal statute?
**Papers**: 6
**Author**: Giovanni Della Libera
**Created**: 2026-05-07

---

## Tracks

### Track vra-compliance

**Theme**: Can minimum-edge-cut bisection with geographic weights satisfy VRA Section 2 requirements — both the Gingles preconditions and the totality-of-circumstances standard — across states with diverse majority-minority geography?

**Chain**: `D.0+vra-compliance` → `D.1+threshold-analysis` → `D.2+nway-vs-recursive-vra` → `D.3+compactness-tradeoff`

**Arc**: D.0 establishes the VRA compliance framework: edge-weighted partitioning creates 137 majority-minority districts vs. 68 enacted (+69 surplus), demonstrating the algorithm exceeds current VRA compliance. D.1 identifies the 42% concentration threshold as the geographic limit of algorithmic VRA compliance — districts below this threshold cannot be created without explicit VRA override. D.2 shows nway bisection offers marginal VRA advantage over recursive for specific state geometries. D.3 characterizes the VRA–compactness tradeoff: real but bounded, with most VRA districts achievable within 85% of the baseline compactness score.

### Track legal-framework

**Theme**: How does algorithmic redistricting translate from a computational tool to an enacted federal statute, and what evidentiary framework do courts require for algorithmic plans?

**Chain**: `D.4+legal-implementation` → `D.5+gingles-bloc-voting-methodology`

**Arc**: D.4 provides the full 50-state adoption pathway analysis and model statute, formalizing ApportionRegions as the statutory vehicle. D.5 establishes the Gingles bloc-voting methodology as an expert witness guide — the evidentiary framework courts require when algorithmic plans face Section 2 challenge.

---

## Papers

| Paper | Tracks | Primary Number | Status | Venue |
|-------|--------|----------------|--------|-------|
| D.0+vra-compliance | vra-compliance | 137 MM districts vs. 68 enacted (+69 surplus) | draft | Yale LJ |
| D.1+threshold-analysis | vra-compliance | 42% geographic threshold for VRA compliance | draft | Yale LJ |
| D.2+nway-vs-recursive-vra | vra-compliance | nway marginal advantage for specific state geometries | draft | Yale LJ |
| D.3+compactness-tradeoff | vra-compliance | VRA–compactness tradeoff: real but bounded (85% threshold) | draft | Yale LJ |
| D.4+legal-implementation | legal-framework | 50-state adoption pathways + model statute | draft | Harvard Law |
| D.5+gingles-bloc-voting-methodology | legal-framework | Gingles bloc-voting expert witness methodology guide | draft | Yale LJ |

---

## Module Arc

Track D addresses the primary legal objection to algorithmic redistricting: that it cannot comply with the Voting Rights Act, and that even if it can, there is no legal pathway to adoption. The VRA compliance sub-track provides the empirical answer — the algorithm already exceeds current VRA performance by 69 majority-minority districts — and characterizes the geographic and compactness limits of that compliance. The legal-framework sub-track provides the institutional answer: a model statute, 50-state adoption pathways, and the Gingles expert witness methodology that courts will require. Track D depends directly on B.14 (VRASection) for the algorithmic mechanism and on C.5/C.9 for the political science validation. It is a prerequisite for A.5 (the policy brief) and for the legal viability claims in A.0 (the synthesis metapaper).
