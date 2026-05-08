---
title: "B.27 — Economic Character Edge Weights (LODES WAC)"
series: B.27
status: Accepted 3.5/4
date: 2026-05-08
track: B-algorithm
layer: Structure (Layer 2 — edge weight modifier)
target_audience: algorithms / practitioner
---

## Algorithm

Download LODES WAC (Workplace Area Characteristics) per state from the Census
Bureau's LEHD program. Aggregate block-level job counts to census tract by
taking the first 11 characters of the 15-character block GEOID.

For each tract `t` compute a 3-component economic character vector:
```
commercial_intensity(t) = (CNS07 + CNS09 + CNS10 + CNS11) / C000
industrial_fraction(t)  = (CNS01 + CNS02 + CNS05 + CNS08) / C000
jobs_per_resident(t)    = C000 / tract_population
```

where CNS fields are NAICS-sector job counts and C000 is total jobs.
Tracts with C000=0 (purely residential, no workplace records) get
character vector [0, 0, 0] — they are pure residential and should
cluster with other pure residential tracts.

Edge weight between adjacent tracts `u` and `v`:
```
w_econ(u, v) = w_base × cosine_similarity(char(u), char(v))
```

Combined with existing geographic boundary weight:
```
w(u, v) = alpha × w_boundary(u, v) + (1 - alpha) × w_econ(u, v)
```
where `alpha = 0.5` is the default blend factor (CLI: `--econ-alpha`).

CLI flag: `--weights-override economic-character`
Data requirement: `data/{year}/lodes/{state}_wac_tract.csv` (produced by
`bisect fetch --type lodes --year 2020 --state north_carolina`)

## Claims

1. Economic character edge weights reduce mean within-district economic
   variance (std dev of jobs_per_resident across tracts) by ≥15% vs
   standard geographic weights on NC-14, WI-8, TX-38.

2. Commercial corridors and industrial zones become natural district cut
   boundaries: in NC-14, the Research Triangle Park (high jobs_per_resident)
   forms a distinct cluster that is preserved rather than split.

3. Proportionality gap for NC changes from −6.5pp (standard) toward ±5pp
   with economic weights, because commercial/industrial tracts that were
   bundled with partisan residential tracts are separated at economic seams.

4. The weight computation is O(n) after LODES aggregation and adds <50ms
   overhead on a 2000-tract state graph.

## Data Sources

Primary:
  LODES 8 WAC (Workplace Area Characteristics), all jobs, all job types:
  URL: https://lehd.ces.census.gov/data/lodes/LODES8/{state_abbr}/wac/
       {state_abbr}_wac_S000_JT00_{year}.csv.gz
  Format: CSV (gzip), one row per census block, columns w_geocode + C000 + CNS01–CNS20
  Update: Annual (2002–2021 available as of 2024)
  License: Public domain (U.S. Government)

Aggregation: w_geocode[:11] → tract GEOID (matches demographics CSV GEOID format)

Population denominator: data/{year}/demographics/{state}_demographics_{year}.csv
  column: total_pop (already in pipeline)

## Layer

Layer 2 — edge/vertex weight modifier. Feeds into the METIS graph before
bisection. Does NOT change the bisection algorithm (Layer 1) or seed
strategy (Layer 3). Composable with all existing weight modes.

New CLI value for existing `--weights-override` flag:
  `economic-character` (requires LODES data downloaded)

## Implementation Location

`crates/bisect-cli/src/vertex_weights.rs` — new `EconomicCharacter` variant
in `WeightsOverride` enum alongside existing `Geographic`, `County`, `VraAligned`.

LODES aggregation: new module `crates/bisect-cli/src/lodes.rs`
  - `load_lodes_wac(state, year) -> HashMap<TractGeoid, EconChar>`
  - `aggregate_blocks_to_tracts(block_records) -> HashMap<TractGeoid, EconChar>`

Download: extend `crates/bisect-cli/src/fetch.rs` with `FetchType::Lodes`

## Test Invariants (L0)

- `lodes_aggregation_preserves_job_counts`: sum of block-level C000 ==
  sum of tract-level C000 after aggregation for NC test fixture
- `cosine_similarity_pure_residential_tracts`: two tracts with char=[0,0,0]
  get similarity=1.0 (both residential → keep together)
- `cosine_similarity_residential_vs_industrial`: char=[0,0,0] vs
  char=[0,0.9,5.0] gives similarity < 0.3 (invite cut)
- `economic_weight_blend_range`: w(u,v) ∈ [0, w_base] for all tract pairs
- `zero_jobs_tract_handled`: tracts with C000=0 don't produce NaN weights

## Empirical Targets

States: NC (k=14, 2195 tracts), WI (k=8, 1406 tracts), TX (k=38, 5265 tracts)
Year: 2020 LODES

Metrics to report:
- Mean within-district std(jobs_per_resident): economic-character vs geographic
- Proportionality gap (pp): economic-character vs −6.5pp (NC baseline)
- Research Triangle Park district integrity: is RTP kept in one district?
- Mean Polsby-Popper: does economic weighting hurt compactness?

Comparison algorithms:
- standard-bisect + geographic weights (baseline)
- standard-bisect + county weights (B.10)
- standard-bisect + economic-character weights (this paper)
