# REVISION PLAN — F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R1 → R2
**Date**: 2026-05-05

## Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with minor revisions |
| Stephanopoulos | 3/4 | Accept with minor revisions |
| Liang | 3/4 | Accept with minor revisions |
| **Mean** | **3.0/4** | |

## Critical Issues (Must Fix Before Resubmission)

### C1 — Configuration count formula incorrect (Duchin C2)
**Problem**: Section 3.1 states "the number of distinct district configurations for a district containing m tracts grows as O(n^m/k^m) under a random graph model." This formula is incorrect — it overcounts by ignoring contiguity and population balance constraints.
**Action**: Replace with: "The number of valid configurations (contiguous, balanced subsets of m adjacent tracts in the graph) grows super-exponentially in m when m is small and plateaus as m approaches n/k. For m ≥ 20, the configuration space is large enough that the METIS edge-cut minimisation can meaningfully distinguish compact from non-compact boundaries; for m < 10, the number of feasible configurations is too small for meaningful optimisation. [cite any reference on connected subgraph counting, or remove the formula and retain only the qualitative claim]."

### C2 — Recommendation table (Section 7) absent from paper (Liang C4)
**Problem**: Abstract promises a "resolution recommendation table for all 50 states and all lower chambers." This table is referenced in Sections 4 and 7 but not shown.
**Action**: Include Table A.1 (recommendation table) in the paper. The table should list all 50 states, lower chamber, k, n_tracts, k/n, and recommended resolution. This is the paper's primary practical output and must be present.

## Important Issues (Should Fix)

### I1 — Threshold derivation vs. calibration distinction (Karypis C1, Duchin C1)
**Problem**: The three-part justification presents the 0.05 threshold as derived but it is primarily empirically calibrated. The CLT and configuration-count arguments give intuition but do not uniquely determine 0.05.
**Action**: Restructure Section 3 as: "§3.1 Motivation (CLT and optimisation richness arguments give intuition for 20 units per district); §3.2 Empirical Calibration (the 0.05 threshold is calibrated against the empirical crossover point observed in C.1 congressional analysis and the three-state comparison in Section 4); §3.3 The Rule." This separates motivation from derivation honestly.

### I2 — MAUP generalisation from single data point (Karypis C3)
**Problem**: The claim that high-k/n chambers have ±2--4 seat MAUP effect is based on one data point (WA: +2 seats). The three-tier classification (0, ±1, ±2--4 seats) has two data points for low/medium and one for high.
**Action**: Either: (a) add more high-k/n states to the MAUP analysis (e.g., ME k/n=0.42, MT k/n=0.36), or (b) qualify the claim: "For the one high-k/n chamber in our analysis (WA, k/n=0.067), the MAUP effect is +2 seats. We conjecture a range of ±2--4 seats for high-k/n chambers generally, but additional data is needed to validate this range."

### I3 — MAUP directional pattern not noted (Rodden C1)
**Problem**: Both MAUP effects that are non-zero (WA: +2D, TX: +1D) favour Democrats. The paper presents these as individual results without noting the directional consistency.
**Action**: Add paragraph: "In both states where block-group resolution changes partisan outcomes (WA and TX), the change adds Democratic seats. This is consistent with the hypothesis that block groups capture urban density more precisely than tracts, and Democratic voters are more concentrated in dense urban areas. Directional consistency across two observations is suggestive but not conclusive; additional states are needed to assess whether the MAUP effect is systematically directional."

### I4 — --resolution auto data dependency (Karypis C4)
**Problem**: --resolution auto requires knowing n_tracts to apply the k/n > 0.05 rule. It is unclear whether this requires the tract shapefile to be downloaded even for block-group-resolution runs.
**Action**: Add note: "The auto-resolution check reads n_tracts from the state configuration file (scripts/config_2020.py), not from the TIGER shapefile. Tract data downloads are not required for block-group-resolution runs; only the block-group TIGER and adjacency data are needed."

### I5 — Runtime table hardware specification missing (Liang C3)
**Problem**: Runtime figures (45s WA tract, 180s WA BG, etc.) have no hardware specification.
**Action**: Add footnote: "All runtimes measured on [CPU model, RAM, single thread vs. multithread] under [OS]. Wall-clock time for single-seed single-state run."

### I6 — Block-group-to-block MAUP not analysed (Rodden C3)
**Problem**: No MAUP analysis for the five states where k/n_bg > 0.20 that in principle should use block resolution.
**Action**: Add Section 5.5: "MAUP at block-group to block transition. For NH, WY, VT, SD, and ND, we cannot currently compute the MAUP effect from block-group to block resolution due to the 40 GB data requirement for block-level adjacency. We flag this as a direction for future work; the effect is expected to be larger than the tract-to-block-group MAUP for equivalent k/n ratios."

## Minor Issues (Can Fix in Proofing)

### M1 — Near-boundary handling for --resolution auto (Duchin C4)
**Action**: Add note: "States with k/n between 0.045 and 0.055 are near the threshold boundary. For these states, we recommend running both resolutions and selecting the map with lower edge-cut, rather than applying the threshold mechanically."

### M2 — Partisan neutrality of resolution choice (Stephanopoulos C1)
**Action**: Add paragraph in Section 5.4: "The finding that resolution choice is a substantively political decision for high-k/n chambers (±2 seats per 98) has implications for any state redistricting law or administrative rule that does not pre-specify resolution. A redistricting commission or legislature that chose resolution post-hoc, after observing partisan outcomes at each resolution, would have engaged in partisan manipulation equivalent to other forms of boundary manipulation. Pre-specification of the k/n > 0.05 rule eliminates this manipulation vector."

## Priorities for R2

1. Fix configuration count formula (C1) — mathematical error
2. Include recommendation table (C2) — missing core output
3. Restructure threshold derivation section (I1) — honesty about empirical vs. theoretical basis
4. Add more MAUP data points (I2) — single data point insufficient
5. Add directional MAUP note (I3)
6. Runtime hardware specification (I5)
