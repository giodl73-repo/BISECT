---
title: "Commuting Shed Similarity via LODES Origin-Destination Data"
series: M.4
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm
LODES OD (Origin-Destination) data: for each home tract h, aggregate job flows from all resident blocks to all work tracts w. Produce per-tract commuting destination distribution D(h) = {w: count(h→w)}, normalized so Σ_w D(h,w) = 1.

Similarity between two home tracts = weighted Jaccard similarity of their destination distributions:

  sim(h1, h2) = Σ_w min(D(h1,w), D(h2,w)) / Σ_w max(D(h1,w), D(h2,w))

Edge weight = w_base × sim(u,v).

Blend: w(u,v) = alpha × w_boundary + (1-alpha) × w_commute, alpha = 0.5 default.

CLI flag: `--weights-override commuting-shed`

## Claims
1. Bedroom suburb tracts commuting to the same employment center cluster together: WI Milwaukee suburbs sharing a primary destination (downtown Milwaukee or Brookfield employment corridor) have sim > 0.5 and co-assign to the same district at ≥70% rate.
2. Commuting shed similarity better predicts district cohesion than geographic proximity for polycentric metro areas: NC Raleigh suburbs and Charlotte suburbs remain in separate districts despite geographic overlap in Piedmont corridor under commuting-shed weights.
3. Commuting shed weights are computable in O(n × avg_destinations) time where avg_destinations ≈ 15–30 per tract, making 50-state computation feasible in under 10 minutes on 8 workers.

## Data Sources
https://lehd.ces.census.gov/data/lodes/LODES8/{state}/od/{state}_od_main_JT00_2020.csv.gz
Format: CSV (gzipped), one row per (home block, work block, job count) triple, LEHD LODES v8.
License: Public domain, U.S. Census Bureau / BLS annual release.
Same LEHD source infrastructure as M.1 (WAC) — same fetch mechanism, different file suffix (od vs wac).

## Layer
Plug-in similarity function for the M-track framework (M.0). Commuting shed captures revealed economic community — where people actually go — rather than where they work (M.1) or land use zoning (M.2). Complementary to M.1: M.1 weights employment centers, M.4 weights residential catchment areas sharing the same center.

## Test Invariants (L0)
- same_tract_similarity_is_one: sim(h, h) == 1.0 for any tract h with nonzero commute flows
- disjoint_destinations_similarity_is_zero: two tracts with no shared destination tracts → sim == 0.0
- jaccard_symmetric: sim(h1, h2) == sim(h2, h1) for all tract pairs

## Empirical Targets
States: WI (8 congressional districts, 2020) and NC (14 congressional districts, 2020).

WI target — suburban clustering:
- Milwaukee suburb tracts (Waukesha, Ozaukee, Washington counties) sharing primary destination: co-assignment rate ≥70% vs ≤50% under geographic weights.

NC target — polycentric separation:
- Raleigh metro tracts (Wake + Durham counties) vs Charlotte metro tracts (Mecklenburg + Union counties): cross-metro co-assignment rate < 5% under commuting-shed weights (vs up to 15% under geographic weights in competitive Piedmont zone).

Performance target:
- Weighted Jaccard computation for all NC tract pairs: wall time < 60 seconds on 8 workers.

Comparison baseline: `--weights-override geographic` on same WI-8 and NC-14 plan configs.
