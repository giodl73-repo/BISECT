# M — Community Character Weights

**Theme**: Any signal that makes census tracts similar or dissimilar translates
into an edge weight modifier for METIS bisection. This track operationalizes
the redistricting criterion "preserve communities of interest" through
measurable, verifiable, non-partisan tract-level signals.

**Core framework**:
```
w(u,v) = w_base × similarity(char_u, char_v)
```
Similar tracts → heavy edge (METIS keeps together)
Dissimilar tracts → light edge (METIS invited to cut between them)

**Legal grounding**: Communities of interest doctrine; *Shaw v. Reno* (1993);
property tax fiscal bond; administrative service co-membership as proxy for
community; *Thornburg v. Gingles* (1986).

## Track Chain

M.0 (framework) → M.1 (economic) → M.2 (land use) → M.3 (housing)
              → M.4 (commuting) → M.5 (topographic) → M.6 (administrative)
              → M.7 (transit) → M.8 (composite)

M.1–M.7 are independent signals; M.8 synthesises all seven.

## Papers

| Paper | Title | Data Source | Stage |
|-------|-------|-------------|-------|
| M.0 | Community Character Weighting: Framework and Legal Grounding | — | planned |
| M.1 | Economic Character Edge Weights (LODES WAC) | LEHD LODES | planned |
| M.2 | Land Use Edge Weights (NLCD) | USGS NLCD | planned |
| M.3 | Housing Character Edge Weights (ACS) | Census ACS | planned |
| M.4 | Commuting Shed Similarity (LODES OD) | LEHD LODES OD | planned |
| M.5 | Topographic Feature Edge Weights (SRTM/3DEP) | USGS 3DEP | planned |
| M.6 | Administrative Zone Co-membership | TIGER/Line, EIA, USFA | planned |
| M.7 | Transit Accessibility Edge Weights (GTFS) | Regional GTFS | planned |
| M.8 | Composite Community Character Index | All M.1–M.7 | planned |

## Zone Co-membership (M.6) — Data Sources

| Zone Type | Data Source | Coverage | URL |
|-----------|-------------|----------|-----|
| School district | TIGER/Line SCHOOLDISTRICT | 100% | census.gov/geo/maps-data/data/tiger |
| Fire district | TIGER/Line Special Districts | ~60% | census.gov/geo/maps-data/data/tiger |
| Water/utility | TIGER/Line Special Districts | ~40% | census.gov/geo/maps-data/data/tiger |
| Electric utility | EIA Form 861 | 100% | eia.gov/electricity/data/eia861 |
| Police precinct | City/county GIS (variable) | ~30% | varies by jurisdiction |
| County subdivision | TIGER/Line | 100% | census.gov (always available as baseline) |
| Property tax jurisdiction | County assessor / TIGER | 100% (county proxy) | varies |

## Contracts

Every M.1–M.7 paper must:
- Define the similarity metric mathematically with proven range [0,1]
- Document the data source (URL, format, update frequency, license)
- Show empirical effect on NC/WI/TX district composition vs standard-bisect
- Measure proportionality gap change vs −6.5pp NC baseline
- Include §5 Legal Usage citing communities of interest case law
- Implement as new `--weights-override` value or modifier flag

## Quantification

Primary: mean within-district economic variance reduction (%) vs standard-bisect
Secondary: proportionality gap change for NC-14
Legal: named court precedent supporting each signal type as community evidence
