---
title: "Topographic Feature Edge Weights via USGS 3DEP"
series: M.5
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm

Download USGS 3DEP 1/3 arc-second DEM (~10m resolution). For each tract centroid extract elevation (m), slope (degrees), and aspect (cardinal: N/S/E/W). Derive watershed membership via flow accumulation using the D8 algorithm.

Three rules govern edge weight derivation:

1. **Ridge crossing**: if edge (u,v) crosses a ridgeline (opposite watersheds), w(u,v) = 0.1 × w_base. Strong invitation to cut — mountains separate communities.

2. **Elevation similarity**: w(u,v) = w_base × exp(−|elev_u − elev_v| / 300m). Tracts within 300 m elevation stay together; tracts 1000 m+ apart receive very light edge weight.

3. **Valley clustering**: tracts sharing the same valley (same watershed below the ridge) receive w = 1.5 × w_base. The valley is a community — this bond is strengthened.

CLI flag: `--weights-override topographic`

## Claims

1. In NC mountain districts (western NC, k=5), ridge lines become natural cut boundaries: 0 cross-ridge districts under topographic weights vs 2–3 under geographic weights.
2. Valley communities in Appalachian states cluster correctly without manual tuning.
3. Topographic weights have minimal effect on flat states (WI, TX plains) where all tracts have similar elevation — graceful degradation to near-baseline behavior.

## Data Sources

**USGS 3DEP 1/3 arc-second national DEM**
- Download via USGS National Map API or AWS S3 (`usgs-lidar` public bucket)
- ~50 GB national; per-state tiles ~500 MB each
- Free, public domain

## Layer

Layer 2 — edge weight modifier. Does not alter the bisection tree structure (Layer 1) or seed selection (Layer 3). Applied as a multiplicative factor on each adjacency edge before METIS receives the graph.

## Test Invariants (L0)

- `same_valley_strengthened` — two tracts in the same watershed below a ridge have w ≥ 1.5 × w_base
- `opposite_watershed_weakened` — two tracts in opposite watersheds have w ≤ 0.1 × w_base
- `flat_terrain_near_baseline` — when |elev_u − elev_v| < 30 m and no ridge crossing, w ∈ [0.9 × w_base, 1.5 × w_base]
- `elevation_diff_1000m_near_zero` — exp(−1000/300) ≈ 0.036, so w < 0.06 × w_base for 1000 m elevation difference

## Empirical Targets

- NC western mountain districts (k=5, 2020 census): cross-ridge district count drops from 2–3 (geographic) to 0 (topographic).
- Flat-state check (WI, IA, TX plains tracts): mean edge weight ratio |w_topo / w_geo − 1| < 0.05 — confirms graceful degradation.
- Appalachian valley clustering (WV, VA, TN): intra-valley tract pairs remain in same district in ≥95% of runs.
