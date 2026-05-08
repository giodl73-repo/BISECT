---
title: "Land Use Edge Weights via NLCD 2021"
series: M.2
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm
Download NLCD 2021 GeoTIFF (30m resolution). Zonal statistics per census tract polygon → compute fractions:
- pct_residential = (class 22 + class 23) / total pixels
- pct_commercial = class 24 / total pixels
- pct_open = class 21 / total pixels
- pct_natural = (class 31 + class 41–95) / total pixels
- pct_water = class 11 / total pixels

Similarity = cosine_similarity([pct_residential, pct_commercial, pct_open, pct_natural]).
Note: pct_water excluded from cosine vector; used only for hard-cut rule.

Special rule: if min(pct_water_u, pct_water_v) > 0.5 → w(u,v) = 0 (hard water boundary — no cross-water districts without bridge adjacency).

Edge weight = w_base × similarity(u,v), subject to hard-cut override.

CLI flag: `--weights-override land-use`

## Claims
1. Residential-to-commercial transitions become natural cut seams: tracts ≥50% NLCD-24 adjacent to tracts ≥50% NLCD-22 get edge weight < 0.3 × baseline.
2. Water boundaries (class 11 majority in both tracts) become hard cuts with w(u,v) = 0.
3. NLCD weights reduce the number of districts crossing natural land-use boundaries by ≥20% vs geographic weights on NC (14-district plan).

## Data Sources
https://www.mrlc.gov/data/nlcd-2021-land-cover-conus
Format: GeoTIFF (~2 GB), 30m resolution, CONUS coverage.
License: Public domain, USGS / Multi-Resolution Land Characteristics Consortium (MRLC).

## Layer
Plug-in similarity function for the M-track framework (M.0). Implements a hard-cut rule on top of the standard alpha-blend interface: water-majority adjacencies are zeroed before blending, ensuring the compositor cannot assign cross-water neighbors to the same district unless explicit bridge adjacency is present in the graph.

## Test Invariants (L0)
- water_majority_gives_zero_weight: tract pair where both pct_water > 0.5 → w(u,v) == 0.0
- identical_landuse_gives_max_weight: two tracts with identical NLCD fraction vectors → similarity == 1.0 and w(u,v) == w_base
- residential_vs_commercial_weight_low: tract with pct_commercial=0.9 adjacent to tract with pct_residential=0.9 → w(u,v) < 0.3 × w_base

## Empirical Targets
State: NC (14 congressional districts, 2020).
Metrics:
- Cross-boundary district count: number of districts spanning a land-use category transition (residential→commercial or natural→developed) must decrease ≥20% vs `--weights-override geographic` baseline.
- Hard-cut validation: zero districts in final plan cross a majority-water edge (Pamlico Sound tracts, Outer Banks).
- Compactness: Polsby-Popper mean must not regress vs geographic baseline (land-use weights should not fragment compact districts).

Comparison baseline: `--weights-override geographic` on same NC-14 plan config.
