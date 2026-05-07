# Data Acquisition Scripts

Downloads and processes Census, election, and demographic data required for redistricting.

## Overview

Organized into subdirectories by data type:
- **census/**: TIGER/Line tract and place geometries
- **elections/**: Presidential election results by tract
- **demographics/**: 2020 Census demographic characteristics
- **geography/**: Adjacency graphs and connectivity data

## Directory Structure

```
scripts/data/
├── census/          # Census tract and place downloads
├── elections/       # Election data downloads and processing
├── demographics/    # Demographic data downloads and processing
└── geography/       # Adjacency graphs, connectivity analysis
```

## Census Data (census/)

### download_all_states_tracts.py

**Purpose**: Download tract geometries and population for all 50 states

**Source**: Census TIGER/Line Shapefiles

**Usage**:
```bash
# Download all states for 2020
python scripts/data/census/download_all_states_tracts.py --year 2020

# Single state
python scripts/data/census/download_all_states_tracts.py --year 2020 --states CA
```

**Output**: `data/raw/{state}_tracts_{year}.parquet`

**Columns**:
- GEOID: 11-digit tract identifier
- NAME: Tract name
- geometry: Polygon geometry
- population: Tract population from PL 94-171 file

**File Size**: ~50-500 MB per state (compressed parquet)

### download_all_states_places.py

**Purpose**: Download place (city) points for major city labeling

**Source**: Census TIGER/Line Places

**Usage**:
```bash
python scripts/data/census/download_all_states_places.py --year 2020
```

**Output**: `data/raw/{state}_places_{year}.parquet`

**Columns**:
- NAME: City name
- geometry: Point geometry (city center)
- population: City population

**Note**: Only places with population > 50,000 are typically used for map labeling

### fix_2010_population.py

**Purpose**: Backfill 2010 tract population from PL 94-171 files

**Why**: TIGER/Line shapefiles don't include population; must merge with Census redistricting files

**Usage**:
```bash
python scripts/data/census/fix_2010_population.py --state CA
```

## Election Data (elections/)

### download_election_data.py

**Purpose**: Download presidential election results

**Source**: MIT Election Data + Science Lab

**Usage**:
```bash
# Download 2020 election data
python scripts/data/elections/download_election_data.py --year 2020

# Download 2016 election data
python scripts/data/elections/download_election_data.py --year 2016
```

**Output**: `data/raw/elections/{state}_precinct_{year}.csv`

**Columns**:
```
precinct,county,candidate,party,votes
```

**Coverage**:
- 2020: 48 states (missing AK, HI at tract level)
- 2016: Similar coverage

### process_election_data.py

**Purpose**: Geocode precinct-level results to census tracts

**Challenge**: Precincts don't align with census tracts

**Approach**:
1. Geocode precincts to approximate locations
2. Assign to nearest census tract
3. Aggregate to tract level

**Usage**:
```bash
python scripts/data/elections/process_election_data.py --year 2020
```

**Output**: `data/processed/elections/{year}_president_tract.parquet`

**Columns**:
```
GEOID,total_votes,biden_votes,trump_votes,other_votes
```

**Note**: This is an approximation - precinct boundaries don't perfectly align with tracts

## Demographic Data (demographics/)

### download_demographic_data_robust.py

**Purpose**: Download 2020 Census DHC (Demographic and Housing Characteristics) data

**Source**: Census API (DHC dataset)

**Usage**:
```bash
python scripts/data/demographics/download_demographic_data_robust.py --year 2020
```

**Features**:
- Exponential backoff retry logic (handles API rate limits)
- Resumable (skips already-downloaded states)
- Robust error handling

**Variables Retrieved**:
- Total population
- Male population
- Female population
- White (non-Hispanic)
- Black (non-Hispanic)
- Asian (non-Hispanic)
- Hispanic (any race)
- Other races

**Output**: `data/raw/demographics/{state}_demographics_{year}.csv`

**Rate Limiting**:
```python
# Exponential backoff pattern
for attempt in range(max_retries):
    response = requests.get(url)
    if response.status_code == 429:  # Rate limited
        delay = base_delay * (2 ** attempt)  # 1s, 2s, 4s, 8s, 16s
        time.sleep(delay)
        continue
```

### process_demographic_data.py

**Purpose**: Consolidate state CSVs into single national parquet file

**Usage**:
```bash
python scripts/data/demographics/process_demographic_data.py --year 2020
```

**Processing**:
1. Load all state CSVs
2. Calculate percentages for each demographic group
3. Combine into single DataFrame
4. Save as compressed parquet

**Output**: `data/processed/demographics/{year}_demographics_tract.parquet`

**Columns**:
```
GEOID,state,county,tract,total_pop,male,female,
white_non_hispanic,black_non_hispanic,asian_non_hispanic,hispanic,other,
male_pct,female_pct,white_pct,black_pct,asian_pct,hispanic_pct,other_pct
```

**File Size**: ~3-5 MB (compressed parquet) for all 84K+ tracts

## Geography Data (geography/)

### build_adjacency_graphs.py

**Purpose**: Create tract adjacency graphs for graph partitioning

**Algorithm**: Queen contiguity (shared edge or corner)

**Usage**:
```bash
python scripts/data/geography/build_adjacency_graphs.py --state CA --year 2020
```

**Output**: `data/adjacency/{state}_adjacency_{year}.pkl` (NetworkX Graph)

**Node Attributes**:
- index: Tract index
- GEOID: 11-digit tract ID
- population: Tract population

**Edge Attributes**: None (unweighted graph)

**Special Handling**:
- Water-based adjacency for coastal/island areas
- County-aware for cross-county connections

### water_adjacency_implementation.py

**Purpose**: Add water-based adjacency for coastal states

**Why**: Tracts separated by narrow water bodies should be connected

**Examples**:
- San Francisco Bay Area
- Puget Sound
- Chesapeake Bay
- Florida Keys

**Pattern**:
```python
# Add edge if:
# 1. Tracts in same county (or adjacent counties)
# 2. Distance < threshold (e.g., 5 km)
# 3. Not already connected by land
if distance < 5000 and same_county(tract1, tract2):
    graph.add_edge(tract1, tract2)
```

## Data Flow

```
┌──────────────────────────────────────┐
│ EXTERNAL SOURCES                     │
│ - Census TIGER/Line API              │
│ - Census DHC API                     │
│ - MIT Election Lab                   │
└────────────┬─────────────────────────┘
             │
    ┌────────▼────────┐
    │  DOWNLOAD       │
    │  (raw/)         │
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │  PROCESS        │
    │  (processed/)   │
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │  BUILD GRAPHS   │
    │  (adjacency/)   │
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │  REDISTRICTING  │
    │  (outputs/)     │
    └─────────────────┘
```

## File Naming Conventions

### State-Level Data
```
{state_code}_{datatype}_{year}.{ext}

Examples:
ca_tracts_2020.parquet
ny_places_2020.parquet
tx_demographics_2020.csv
```

### National Aggregates
```
{year}_{datatype}_tract.parquet

Examples:
2020_president_tract.parquet
2020_demographics_tract.parquet
```

### Graphs
```
{state_code}_adjacency_{year}.pkl

Examples:
ca_adjacency_2020.pkl
```

## Key Patterns

### Robust Download Pattern

**All download scripts should use exponential backoff**:

```python
def download_with_retry(url, max_retries=5, base_delay=1.0):
    """Download with exponential backoff."""
    for attempt in range(max_retries):
        try:
            response = requests.get(url, timeout=30)

            if response.status_code == 200:
                return response

            elif response.status_code == 429:  # Rate limited
                if attempt < max_retries - 1:
                    delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
                    print(f"  Rate limited. Retry {attempt+1}/{max_retries} after {delay:.1f}s...")
                    time.sleep(delay)
                    continue

            else:
                print(f"  HTTP {response.status_code}: {response.text[:100]}")
                return None

        except requests.exceptions.Timeout:
            print(f"  Timeout on attempt {attempt+1}")
            if attempt < max_retries - 1:
                time.sleep(base_delay * (2 ** attempt))
                continue

    return None
```

**Why**: Census API can be rate-limited or temporarily unavailable

### Resume Logic

**All download scripts should skip existing files**:

```python
output_file = Path(f'data/raw/{state}_tracts_2020.parquet')

if output_file.exists():
    print(f"Already downloaded: {state} - skipping")
    continue

# Download...
```

**Why**: Enables resumable downloads after failures or interruptions

### File Format Choice

**Use Parquet for large tabular data**:
- ✅ Fast: Columnar storage, efficient compression
- ✅ Typed: Preserves dtypes (no GEOID int/str confusion)
- ✅ Small: 5-10x smaller than CSV
- ✅ Pandas-native: Direct read/write

**Use CSV for**:
- Human inspection
- Excel compatibility
- Small files (<1000 rows)

**Use Pickle for**:
- Python objects (graphs, dicts)
- Not portable to other languages

## Data Sizes

### Raw Data (data/raw/)

| Dataset | Per State | All States |
|---------|-----------|------------|
| Tracts | 50-500 MB | ~10 GB |
| Places | 1-5 MB | ~200 MB |
| Demographics (CSV) | 1-5 MB | ~250 MB |
| Elections (CSV) | 5-20 MB | ~500 MB |

### Processed Data (data/processed/)

| Dataset | Size |
|---------|------|
| 2020 Demographics | 3-5 MB |
| 2020 Elections | 10-15 MB |

### Adjacency Graphs (data/adjacency/)

| Per State | All States |
|-----------|------------|
| 100KB-5MB | ~200 MB |

**Total Data Footprint**: ~12-15 GB for full dataset

## Common Issues

### Issue: Census API Rate Limiting

**Symptom**: "429 Too Many Requests"

**Solution**: Script automatically retries with exponential backoff (1s, 2s, 4s, 8s, 16s)

**Manual Fix**: Wait a few minutes, re-run (script resumes)

### Issue: Missing 2010 Population

**Symptom**: 2010 tracts have no population data

**Cause**: TIGER/Line shapefiles don't include population for 2010

**Solution**: Use `fix_2010_population.py` to merge with PL 94-171 files

### Issue: GEOID Leading Zeros

**Symptom**: California GEOIDs start with 06, but stored as 6

**Solution**: Always use `.astype(str).str.zfill(11)`

```python
# Convert GEOID to properly formatted string
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)
```

### Issue: Incomplete Election Data

**Symptom**: Alaska, Hawaii election analysis fails

**Cause**: No tract-level election data available for these states

**Solution**: This is expected limitation - skip political analysis for AK, HI

### Issue: Download Interrupted

**Symptom**: Script failed halfway through

**Solution**: Just re-run - script skips already-downloaded files

## Performance

### Download Times

| Operation | Time |
|-----------|------|
| Single state tracts | 30s - 2 min |
| All 50 state tracts | 30-60 min |
| Single state demographics | 5-30 seconds |
| All 50 state demographics | 5-10 min |

**Tip**: Run overnight for first-time setup

### Processing Times

| Operation | Time |
|-----------|------|
| Process demographics | 1-2 min |
| Process elections | 5-10 min |
| Build single adjacency graph | 10-60 sec |
| Build all adjacency graphs | 15-30 min |

## Initialization Checklist

**Before running redistricting pipeline**:

1. ✅ Download 2020 tracts (all states)
   ```bash
   python scripts/data/census/download_all_states_tracts.py --year 2020
   ```

2. ✅ Download 2020 places (all states)
   ```bash
   python scripts/data/census/download_all_states_places.py --year 2020
   ```

3. ✅ Build adjacency graphs (all states)
   ```bash
   python scripts/data/geography/build_adjacency_graphs.py --year 2020
   ```

4. ✅ (Optional) Download & process election data
   ```bash
   python scripts/data/elections/download_election_data.py --year 2020
   python scripts/data/elections/process_election_data.py --year 2020
   ```

5. ✅ (Optional) Download & process demographic data
   ```bash
   python scripts/data/demographics/download_demographic_data_robust.py --year 2020
   python scripts/data/demographics/process_demographic_data.py --year 2020
   ```

**Total time**: 1-2 hours (with parallel downloads where possible)

## See Also

- `CODING_PATTERNS.md` - File naming, error handling, GEOID patterns
- `ARCHITECTURE.md` - Data flow, system design
- `DATA_FORMATS.md` - Detailed format specifications
- `DEPENDENCIES.md` - Required packages and setup
