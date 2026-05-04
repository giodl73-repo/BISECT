# B.10 — Subdivision-Respecting Redistricting: Governmental Hierarchy as Edge-Weight Signal

**Paper Type**: Algorithm Design + Empirical Evaluation  
**Status**: Planning (2026-05-02)  
**Series**: B (Algorithm Design)  
**Depends on**: B.2 (edge-weighted bisection infrastructure), B.7 (partisan outcome baseline)  

---

## Core Idea

Most state redistricting statutes list "preserving political subdivisions" as an
explicit criterion — but no algorithmic implementation operationalizes this formally.

We add a **governmental stickiness weight** to every adjacency edge: two census
tracts that share a common governmental jurisdiction at level L are harder to
separate. When the algorithm must split a county (as federal ±0.5% balance often
requires), it finds the cleanest seam. When it doesn't need to split, it keeps
the jurisdiction whole.

```
w(u,v) = boundary_length(u,v) × (1 + Σ_L α_L × 1[same_jurisdiction_L(u,v)])
```

- `α_L = 0` everywhere → standard edge-weighted bisection (B.2)  
- `α_county → ∞` → county splits only when population balance forces it  
- Independent α per level → tune the hierarchy  

---

## Governmental Hierarchy

The US subdivision hierarchy varies by state. All levels are in Census TIGER:

| Level | TIGER file | Description | State variants |
|-------|-----------|-------------|----------------|
| **County** | GEOID chars 3–5 | Available from tract GEOID | LA=parish, AK=borough, VA=independent city |
| **County subdivision (MCD)** | `tl_YYYY_SS_cousub` | Civil divisions | Townships (MN/WI/NE/KS), magisterial districts (VA) |
| **Incorporated place** | `tl_YYYY_SS_place` | Cities, towns, villages, boroughs | Borough (PA/NJ), township (MI/OH) |
| **Voting tabulation district** | `tl_YYYY_SS_vtd20` | Precincts, wards | State-specific: LA precincts, NY wards, IL townships |

### State-Specific Hierarchy Notes

- **Louisiana**: Parishes instead of counties; precincts within parishes  
- **Alaska**: Boroughs (only 19, cover ~55% of area); remainder = Unorganized Borough  
- **Virginia**: 39 independent cities at county level; no county-city nesting  
- **New England (CT/RI/MA/VT/NH/ME)**: Towns are primary unit, counties less meaningful  
- **Minnesota/Wisconsin/Nebraska**: Townships as strong MCDs (many residents identify with township)  
- **New York**: Boroughs (Manhattan, Brooklyn, etc.) within NYC at county level  

---

## Algorithm

```
SubdivisionBisect(G, k, α_county, α_mcd, α_place, α_vtd):

  1. Load governmental affiliations per tract:
     - county_fips: from GEOID (free, always available)
     - cousub_fips: from tl_YYYY_SS_cousub spatial join
     - place_fips:  from tl_YYYY_SS_place spatial join
     - vtd_fips:    from tl_YYYY_SS_vtd20 spatial join

  2. For each edge (u,v):
     bonus = α_county × 1[county(u)==county(v)]
           + α_mcd    × 1[cousub(u)==cousub(v)]
           + α_place  × 1[place(u)==place(v)]
           + α_vtd    × 1[vtd(u)==vtd(v)]
     w'(u,v) = boundary_length(u,v) × (1 + bonus)

  3. Run standard recursive bisection with weights w'.
```

---

## Key Research Questions

1. **α sweep**: For each level, what α value reduces splits by 50%? 90%? At what
   edge-cut cost? This traces the Pareto frontier for each state.

2. **Partisan neutrality**: Does governmental-line stickiness change D/R outcomes?
   Geographic sorting should still dominate — but does respecting cities (which are
   heavily Democratic) systematically affect seat counts?

3. **Hierarchy dominance**: In township states (MN/WI), does α_mcd matter more than
   α_county? In New England, does α_place dominate?

4. **Interaction with population balance**: Congressional ±0.5% is tight. Large
   counties (LA, Cook IL, Harris TX) must be split regardless of α. What is the
   maximum α that still finds a valid plan?

5. **VTD/precinct level**: Since precincts are the fundamental voting unit, preserving
   precinct integrity is directly analogous to preserving community of interest. Does
   precinct stickiness reduce "cracked precincts" (a major gerrymandering complaint)?

---

## Data Requirements

### Already available
- County affiliation: GEOID[2:5] — immediate, no download

### Need download (Census TIGER, ~2GB total for all 50 states)
- `tl_2020_SS_cousub` — county subdivision / MCD files (51 states × ~3MB)
- `tl_2020_SS_place`  — incorporated place files (51 states × ~5MB)
- `tl_2020_SS_vtd20`  — voting tabulation districts (51 states × ~10MB)

### Spatial join strategy
Tract → governmental unit assignment by **majority area overlap**: the tract belongs
to the governmental unit that covers the largest fraction of its area. For split
tracts (a tract spanning two cities), assign to the majority unit and record the
fraction for uncertainty analysis.

For precincts: since 2020 Census tracts were often drawn to align with precinct
boundaries, many tract-to-VTD assignments are 1:1 (no split). Record split fraction.

---

## Implementation Plan

### Phase 1: County-only (immediate)
1. Extract county FIPS from GEOID in adjacency builder
2. Add `county_fips: Vec<Option<String>>` to `AdjacencyGraph`
3. Add `--subdivision-alpha-county f64` flag to `StateArgs`
4. In edge-weight computation: multiply by `(1 + α_county)` for same-county edges
5. Run 50-state sweep at α ∈ {0, 1, 5, 10, 50} — measure county splits and EC

### Phase 2: MCD + Place (after TIGER download)
1. Download `cousub` and `place` files via `redist fetch`
2. Implement spatial join (tract centroid → cousub/place lookup, or bbox overlap)
3. Add `cousub_fips`, `place_fips` to adjacency data
4. Add `--subdivision-alpha-mcd` and `--subdivision-alpha-place` flags

### Phase 3: Precinct (VTD)
1. Download `vtd20` files
2. Spatial join (precinct boundaries tend to align with tract boundaries in 2020)
3. Add `--subdivision-alpha-vtd` flag
4. Study precinct-split reduction as a proxy for community-of-interest preservation

---

## Metrics

For each state and α configuration:

| Metric | Definition |
|--------|-----------|
| **county_splits** | Number of counties with tracts in ≥2 districts |
| **mcd_splits** | Number of MCDs split between districts |
| **place_splits** | Number of incorporated places split |
| **vtd_splits** | Number of precincts split |
| **ec_km** | Total edge cut (boundary length) |
| **d_seats, gap_pp** | Partisan outcome |
| **polsby_popper** | Mean district compactness |

Primary comparison baseline: B.2 edge-weighted (α=0 everywhere).

---

## Legal Hook

"Preserving political subdivisions" appears in:
- U.S. Supreme Court: *Rucho v. Common Cause* (2019) — partisan gerrymandering
  claims nonjusticiable but traditional criteria (including county preservation)
  remain valid state-court grounds
- Most state constitutions and statutes (CA, TX, FL, PA, etc.) list it explicitly
- *Harper v. Hall* (NC 2022): NC Supreme Court required county grouping compliance

This paper provides the first algorithmic formalization of this statutory criterion
as a tunable edge-weight parameter, with empirical characterization of the
splits/efficiency tradeoff.

---

## Series Placement

B.10 — extends B.2 (edge-weighted bisection) to use governmental hierarchy as the
weight signal. The α parameter is analogous to the boundary-length weight in B.2:
a single published number that fully determines the algorithm's behavior.
