# Pipeline Output Inventory

Every file the pipeline writes, per state and nationally. Use this as the ground truth when porting to Rust or adding new analysis steps.

## Per-State Outputs

Output root: `outputs/{VERSION}/{YEAR}/states/{state_name}/`

### Always generated (core redistricting)

| Script | Output | Description |
|--------|--------|-------------|
| `run_state_redistricting.py` | `data/final_assignments.pkl` | District assignment per tract (binary) |
| `run_state_redistricting.py` | `intermediate/depth_NN/` | Intermediate round assignments + metadata JSON |
| `run_state_redistricting.py` | `maps/all_districts.png` | Final district map |
| `create_final_district_summary.py` | `data/district_summary.csv` | Per-district: pop, deviation%, tracts, area, perimeter, Polsby-Popper, Reock, largest city |
| `create_final_district_summary.py` | `data/rounds_hierarchy.csv` | Hierarchical bisection round summary |
| `add_cities_to_districts.py` | `data/district_cities.csv` | Largest city per district (spatial join) |
| `add_cities_to_districts.py` | `maps/all_districts_with_cities.png` | District map with city labels |
| `visualize_all_rounds.py` | `maps/rounds/round_00.png` … `round_NN.png` | One PNG per bisection round |
| `visualize_individual_districts.py` | `maps/districts/district_01.png` … `district_NN.png` | One PNG per district |

### Generated with `--run-analysis`

| Script | Output | Condition |
|--------|--------|-----------|
| `visualize_metro_areas.py` | `maps/metros/{metro_name}_districts.png` | State has major metros |
| `analyze_district_compactness.py` | `compactness/district_compactness.csv` | Always (if flag set) |
| `visualize_compactness.py` | `compactness/maps/polsby_popper.png` | Always |
| `visualize_compactness.py` | `compactness/maps/reock.png` | Always |
| `analyze_district_demographics.py` | `demographic/district_demographics.csv` | Demographic data exists for year |
| `visualize_district_demographics.py` | `demographic/maps/diversity_index.png` | Demographic data exists |
| `visualize_district_demographics.py` | `demographic/maps/gender_balance.png` | Demographic data exists |
| `visualize_district_demographics.py` | `demographic/maps/majority_race.png` | Demographic data exists |
| `analyze_districts.py` | `political/district_political.csv` | 2020 only, election data exists |
| `analyze_districts.py` | `political/rounds_political.csv` | 2020 only, election data exists |
| `visualize_partisan_lean.py` | `political/maps/partisan_lean.png` | 2020 only, election data exists |

---

## National / Aggregate Outputs

Output root: `outputs/{VERSION}/{YEAR}/`

| Script | Output | Description |
|--------|--------|-------------|
| `create_us_aggregate.py` | `data/us_all_districts.csv` | All 435 districts concatenated |
| `create_us_aggregate.py` | `data/us_district_summary.csv` | National summary stats |
| `create_us_aggregate.py` | `maps/us_national_political_2020.png` | National partisan lean map (2020 only) |
| `create_us_aggregate.py` | `maps/us_national_demographic_2020.png` | National demographic map (2020 only) |
| `create_us_rounds_hierarchy.py` | `data/us_rounds_hierarchy.csv` | National bisection round hierarchy |
| `visualize_national_districts.py` | `maps/us_all_districts.png` | National district map |
| `visualize_national_rounds.py` | `maps/rounds/round_00.png` … `round_06.png` | National round progression (7 rounds) |
| `generate_dashboard.py` | `index.html` | Static interactive dashboard |

---

## Data Processing Outputs (Census pipeline)

Output root: `outputs/{VERSION}/{YEAR}/data/` and `outputs/data/{YEAR}/`

| Script | Output | Description |
|--------|--------|-------------|
| `process_census_data.py` | `outputs/data/{YEAR}/adjacency/{state}_adjacency_{YEAR}.pkl` | Tract adjacency graph per state |
| `process_census_data.py` | `.tract_tracts_complete` etc. | Stage completion markers |
| `process_census_data.py` | `outputs/data/{YEAR}/adjacency/{state}_block_adjacency_{YEAR}.pkl` | Block-level adjacency (MAUP analysis) |
| `process_census_data.py` | `outputs/data/{YEAR}/adjacency/{state}_block_group_adjacency_{YEAR}.pkl` | Block-group adjacency (MAUP analysis) |

---

## VRA-Specific Outputs (V4 / `--partition-mode metis-vra`)

Additional outputs when running VRA mode:

| Script | Output | Description |
|--------|--------|-------------|
| `run_state_redistricting.py` | `data/vra_analysis.pkl` | Per-district minority percentages, MM district flags |

---

## Completion Markers

The pipeline writes hidden marker files to enable resuming interrupted runs:

| Marker | Meaning |
|--------|---------|
| `.tract_tracts_complete` | Tract shapefiles merged |
| `.tract_merge_complete` | Tract-population merge done |
| `.tract_adjacency_complete` | Adjacency graphs built |
| `.tract_demographics_complete` | Demographics processed |
| `.tract_elections_complete` | Election data merged |
| `.census_complete` | All data stages done |
| `.states_complete` | Lists completed state codes (one per line) |

---

## Notes for Rust Port

- `final_assignments.pkl` — replace with `bincode` or `flatbuffers` binary format
- `intermediate/depth_NN/` — same binary format, one file per round
- All CSVs — keep as CSV (universal, no format migration needed)
- All PNGs — generate via a thin Python wrapper or port to `plotters` crate
- `vra_analysis.pkl` — replace with JSON or bincode
- Dashboard HTML — keep as Python (not performance-critical)
