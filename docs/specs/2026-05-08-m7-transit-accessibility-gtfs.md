---
title: "Transit Accessibility Edge Weights via GTFS"
series: M.7
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm

Download GTFS feeds for metropolitan transit agencies covering the state. Compute a per-tract transit score:

```
transit_score(t) = Σ_routes_within_400m (frequency_per_hour × route_speed_factor)
```

- High score: dense transit (subway, frequent bus)
- Low score: car-dependent
- Zero: no transit within 400 m of centroid

Similarity between adjacent tracts:

```
transit_sim(u,v) = 1 − |transit_score(u) − transit_score(v)| / max(transit_score(u), transit_score(v))
Special case: if both scores = 0 (both car-dependent rural), similarity = 1.0
```

Edge weight:

```
w(u,v) = w_base × transit_sim(u,v)
```

**Effect by tract-pair type**:

| Pair type | Similarity | Result |
|-----------|-----------|--------|
| Transit-rich urban ↔ transit-rich urban | high | heavy edge — stay together |
| Rural ↔ rural (both zero) | 1.0 | heavy edge — stay together |
| Urban core ↔ car-dependent suburb | low | light edge — invite cut at fringe |
| Car-dependent suburb ↔ rural exurb | low | light edge — invite cut |

CLI flag: `--weights-override transit`

## Claims

1. Transit-rich tracts (score > 10 routes/hr within 400 m) cluster together in NC metro areas (Charlotte, Raleigh) under transit weights.
2. The urban-rural fringe becomes a natural cut seam — fringe edges receive systematically lighter weight than intra-urban or intra-rural edges.
3. In states without GTFS coverage (most rural states), transit weights gracefully degrade to geographic weights: all scores = 0, all similarities = 1.0, all edge weights = w_base.

## Data Sources

**GTFS feeds from transit agencies**
- Primary source: Mobility Database (https://database.mobilitydata.org/) — aggregates feeds from agencies worldwide
- Secondary: state DOT GTFS compilations where available
- Coverage: all major metros (Charlotte, Raleigh, Charlotte, Atlanta, Chicago, etc.)
- Rural areas: sparse or absent — graceful degradation applies
- Free where available; most US agencies publish GTFS publicly

## Layer

Layer 2 — edge weight modifier. GTFS data is preprocessed into per-tract transit scores once per state/year; the similarity function is applied per adjacency edge at graph construction time. Does not alter bisection tree structure (Layer 1) or seed selection (Layer 3).

## Test Invariants (L0)

- `rural_rural_similarity_one` — two tracts both with transit_score = 0 have transit_sim = 1.0
- `urban_rural_similarity_low` — a tract with score > 10 adjacent to a tract with score = 0 has transit_sim < 0.1 (max denominator clamp)
- `no_gtfs_data_degrades_to_baseline` — when all tract scores = 0 for a state, all w(u,v) = w_base
- `transit_score_nonneg` — transit_score(t) ≥ 0.0 for all tracts

## Empirical Targets

- NC Charlotte metro (2020): transit-rich tracts (score > 10) form contiguous clusters in ≥90% of METIS runs under transit weights.
- Urban-rural fringe edge weight: mean w_fringe / w_base < 0.4 in NC metro counties.
- Rural-state degradation check (ND, WY, MT): |mean w_transit / w_geo − 1| < 0.01 — confirms flat baseline when no GTFS data is present.
- No numerical instability: similarity computation handles max = 0 edge case without division by zero.
