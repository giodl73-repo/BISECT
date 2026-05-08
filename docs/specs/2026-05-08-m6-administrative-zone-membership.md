---
title: "Administrative Zone Co-membership Edge Weights"
series: M.6
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm

For each pair of adjacent tracts (u,v), count shared administrative zones across six zone types:

```
zones = [school_district, fire_district, water_utility, electric_utility,
         county_subdivision, police_precinct]
available_zones = those with data for this state (county_subdivision always available)
zone_score(u,v) = shared_zones_count / available_zones_count  ∈ [0, 1]
w(u,v) = w_base × (1 + alpha × zone_score(u,v))    alpha = 1.0 default
```

- Tracts sharing ALL zones: w = 2 × w_base (maximum bond)
- Tracts sharing NO zones (different county or subdivision): w = w_base (unmodified)

**Zone data sources (priority order)**:

1. **School district** — TIGER/Line SCHOOLDISTRICT shapefiles (Census, 100% coverage, annual)
2. **County subdivision** — TIGER/Line COUSUB (100% coverage — towns, townships, boroughs)
3. **Electric utility** — EIA Form 861 service territory shapefiles (100% coverage, annual, free)
4. **Fire district** — TIGER/Line Special District code 21 (~60% state coverage)
5. **Water/sewer district** — TIGER/Line Special District codes 22–29 (~40% coverage)
6. **Police precinct** — City/county GIS portals (~30% coverage; county used as fallback)

**Property tax connection**: county_subdivision in New England is the property tax-levying unit (town). In other states, county is the primary tax authority. zone_score naturally captures this: two tracts in the same town/county_subdivision share the same property tax base and vote on the same bond measures.

CLI flag: `--weights-override zone-membership`

## Claims

1. Tracts sharing ≥4 zone types (school + fire + water + electric) never appear in different districts after redistricting: 0 splits of co-zone tracts in NC-14.
2. School district boundary crossings reduced by ≥80% vs geographic weights.
3. Administrative zone co-membership is the most legally defensible community signal — courts recognize fiscal bonds (same tax authority) and service co-membership as communities of interest.

## Data Sources

- **TIGER/Line School Districts**: https://www.census.gov/geo/maps-data/data/tiger-line.html (SCHOOLDISTRICT shapefile)
- **TIGER/Line Special Districts**: same URL (UNSD shapefile, codes 21–29)
- **EIA Form 861**: https://www.eia.gov/electricity/data/eia861/ (annual ZIP with service territory shapefiles)

All sources are free and publicly available. EIA data is annual and comprehensive for electric utilities nationally.

## Layer

Layer 2 — edge weight modifier. Operates on the adjacency graph after tract adjacency is established. Does not alter bisection tree structure (Layer 1) or seed selection (Layer 3). Applied as a multiplicative factor per edge before METIS receives the graph.

## Test Invariants (L0)

- `all_zones_shared_doubles_weight` — zone_score = 1.0 produces w = 2 × w_base (when alpha = 1.0)
- `no_zones_shared_baseline` — zone_score = 0.0 produces w = w_base exactly
- `school_district_split_never_increases` — school district boundary crossing count under zone-membership weights is ≤ that under geographic weights for every state tested
- `zone_score_range_0_to_1` — zone_score(u,v) ∈ [0.0, 1.0] for all tract pairs

## Empirical Targets

- NC-14 (2020): zero splits of co-zone tracts (sharing ≥4 zone types).
- NC-14 (2020): school district boundary crossings ≤20% of the count produced by geographic weights.
- New England states (MA, CT, VT): county_subdivision (town) integrity — tracts in same town remain in same district in ≥95% of runs.
- Coverage degradation check: states with only county_subdivision available (zone_score ∈ {0, 1}) still produce valid, non-uniform edge weights.
