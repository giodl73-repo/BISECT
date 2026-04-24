# VRA Compliance and Majority-Minority Districts

## Short version

The Voting Rights Act (Section 2) requires states to draw majority-minority districts where minority populations are sufficiently large and geographically concentrated. Pure compactness-based redistricting can conflict with this requirement. We tested multi-constraint METIS (2D vertex weights: population + minority VAP) across all covered states. Key finding: states with ≥42% minority population statewide can achieve all VRA targets with principled methods. Below that threshold, geography makes compliance impossible without sacrificing compactness.

---

## What a majority-minority district is

A majority-minority (MM) district is one where a racial or ethnic minority group comprises more than 50% of the voting-age population. The VRA requires these where:
1. The minority group is sufficiently large
2. It is geographically compact (so it could form a district)
3. Political processes are not equally open to participation

Courts have established targets for specific states based on litigation. Our `vra_utils.py` encodes the standard targets from enacted plans and Section 2 cases.

## The 42% threshold

Our research found a critical threshold: states where minority population exceeds roughly 42% statewide can achieve all statutory MM district targets using the multi-constraint METIS algorithm. Below ~37%, geography makes compliance impossible while maintaining contiguity and compactness.

| State | Minority % | Target MM | Result |
|-------|-----------|-----------|--------|
| Georgia | 42.4% | 5 | ✓ Achieved (70–77%) |
| Mississippi | 46.1% | 2 | ✓ Achieved (53%) |
| Louisiana | 41.6% | 2 | Partial (1 of 2) |
| Alabama | 36.9% | 2 | ✗ Best: 49.6% |
| South Carolina | 35.1% | 3 | ✗ Best: 47.2% |

The constraint is geographic, not algorithmic: minority populations in Alabama and South Carolina are too dispersed to concentrate above 50% in any contiguous, compact district.

## The metis-vra algorithm

Standard redistricting uses 1D vertex weights (population). The VRA variant (`--partition-mode metis-vra`) uses 2D vertex weights:
- Weight 1: Total population (enforces population balance)
- Weight 2: Minority VAP (non-white voting-age population)

METIS's multi-constraint partitioner (`tpwgts`) specifies target fractions for both constraints in each partition. For MM districts, we set a target minority fraction of 60% (providing a margin above the 50% threshold).

```python
# Target for an MM district in a 7-district state with 37% minority statewide:
# Population fraction: 1/7 = 14.3%
# Minority fraction: targets 60% concentration
tpwgts = [0.143, 0.232]  # [pop_fraction, minority_fraction]
```

## The compactness tradeoff

VRA compliance and compactness are in tension. Concentrating minority population into MM districts requires non-uniform geographic distribution — the algorithm must seek out and group dispersed minority tracts. This increases perimeter.

Paper D.3 (*Quantifying the VRA-Compactness Tradeoff*) measures this tradeoff precisely. The penalty exists but is modest: even Alabama's best VRA attempt achieves 49.6% minority concentration with compactness only slightly worse than the edge-weighted baseline.

## V4 pipeline run

The V4 pipeline run uses `metis-vra` mode across all 50 states (2020 Census). The dashboard shows per-district minority percentages and highlights majority-minority districts in the ⚖️ VRA tab.

States where VRA targets are not achievable (< 42% minority) still show the best achievable minority concentration — useful as a reference for policy discussions about geographic feasibility.

## Further reading

- Paper D.0: *VRA Compliance Through Edge-Weighted Graph Partitioning*
- Paper D.1: *The 42% Threshold*
- Paper D.2: *N-Way vs Recursive Bisection for VRA*
- Paper D.3: *Quantifying the VRA-Compactness Tradeoff*
- `src/apportionment/partition/vra_utils.py` — implementation
- `src/apportionment/partition/vra_targets.py` — target tree construction
