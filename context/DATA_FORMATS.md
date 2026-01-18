# Data Formats

**Updated**: 2026-01-17

## Data Sources

1. **Census TIGER/Line** - Tract geometries (shapefiles)
2. **Census PL 94-171** - Population data (pipe-delimited text)
3. **Census DHC API** - Demographics (sex, race/ethnicity)
4. **MIT Election Lab** - Presidential results (CSV)
5. **NetworkX Graphs** - Adjacency (pickled)
6. **District Assignments** - Tract→District mapping (pickled dict)

## Census TIGER/Line Shapefiles

### Format
- **Type**: ESRI Shapefile (.shp/.shx/.dbf/.prj)
- **Source**: Census Bureau TIGER/Line
- **Resolution**: Census tracts (~84K nationwide)
- **Projection**: NAD83 (EPSG:4269)

### URLs

**2020 Tracts**: `https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_{FIPS}_tract20.zip`
**2010 Tracts**: `https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/tl_2010_{FIPS}_tract10.zip`
**2000 Tracts**: `https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2000/tl_2010_{FIPS}_tract00.zip`
**2020 Places**: `https://www2.census.gov/geo/tiger/TIGER2020/PLACE/tl_2020_{FIPS}_place.zip`

### Fields

**2020**: `GEOID20` (11-digit str), `NAME20`, `ALAND20`, `AWATER20`, `geometry`
**2010**: `GEOID10` (11-digit str), `NAME10`, `ALAND10`, `AWATER10`, `geometry`
**2000**: `CTIDFP00` or `GEOID` (varies), `NAME`, `POP2000` (if present), `geometry`
**Places**: `GEOID`, `NAME`, `geometry` (use pop > 50K for labeling)

### GEOID Format (11 digits)

```
SSCCCTTTTTT
││ │  └─ Tract (6 digits: 123456 = 1234.56)
││ └──── County FIPS (3 digits)
│└───── State FIPS (2 digits)

Example: 06037980000 = CA(06), LA County(037), Tract 9800.00
```

⚠️ **Population NOT in TIGER/Line** - must join from PL 94-171

## Census PL 94-171 Redistricting Files

### Overview
- **Format**: Pipe-delimited (|), **NO HEADERS**, `latin-1` encoding
- **Source**: https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html
- **URL Pattern**: `https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/{StateName}/{state}2020.pl.zip`

### Files (per state ZIP)

1. `{state}geo2020.pl` - Geographic header (GEOIDs)
2. `{state}000012020.pl` - Segment 1: Race (P1, P2)
3. `{state}000022020.pl` - Segment 2: 18+ pop, housing (P3, P4, H1)
4. `{state}000032020.pl` - Segment 3: Group quarters (P5)

### Key Fields

**Geographic Header** (position-based, no header):
- Pos 2: `STUSAB` - State code (CA, TX, ...)
- Pos 3: `SUMLEV` - Summary level (140=Tract, 750=Block)
- Pos 8: `LOGRECNO` - Join key
- Pos 9: `GEOID` - Geographic ID

**Population Data** (join on LOGRECNO):
- `P0010001` - **Total Population** ⭐ KEY FIELD
- `P0010003` - White alone
- `P0010004` - Black/African American alone
- `P0010006` - Asian alone
- `P0020002` - Hispanic/Latino

### Parsing

```python
# Read (no headers, pipe-delim, latin-1)
geo = pd.read_csv('cageo2020.pl', delimiter='|', dtype=str, header=None, encoding='latin-1')
data = pd.read_csv('ca000012020.pl', delimiter='|', dtype=str, header=None, encoding='latin-1')

# Filter tracts (SUMLEV=140), join on LOGRECNO
geo = geo[geo[2] == '140']
merged = geo.merge(data, left_on=7, right_on=4)

# Extract GEOID (pos 8), population (check tech doc for exact position)
```

### GEOID Formats

**Tract** (11 digits): `SSCCCTTTTTT`
**Block** (15 digits): `SSCCCTTTTTTBBBB` (truncate to 11 for tract)

### Alternative: Census API

**2020**: `https://api.census.gov/data/2020/dec/pl`
**Variable**: `P1_001N` (total pop)
**Requires**: API key (free from census.gov)

```python
params = {'get': 'P1_001N', 'for': 'tract:*', 'in': 'state:06', 'key': 'YOUR_KEY'}
```

## Demographic Data (DHC API)

**Endpoint**: `https://api.census.gov/data/2020/dec/dhc`

**Variables**:
- `P12_001N` - Total pop
- `P12_002N` - Male
- `P12_026N` - Female
- `P3_003N` - White (non-Hispanic)
- `P3_004N` - Black (non-Hispanic)
- `P3_006N` - Asian (non-Hispanic)
- `P4_002N` - Hispanic/Latino (any race)

**Output**: `data/processed/demographics/{year}_demographics_tract.parquet`
**Columns**: `GEOID` (str, 11 digits), `state`, `county`, `tract`, `total_pop`, `male`, `female`, `white_nh`, `black_nh`, `asian_nh`, `hispanic`

## Election Data (MIT Election Lab)

**Source**: https://electionlab.mit.edu/data
**Dataset**: County Presidential Election Returns 2000-2020

**Raw**: `data/elections/countypres_2000-2020.csv`
**Processed**: `data/processed/elections/{year}_president_tract.parquet`

**Geocoding**: County results → precinct estimates → tract assignment

**Output Columns**:
- `GEOID` (str, 11 digits)
- `state`, `county`, `tract`
- `dem_votes`, `rep_votes`, `total_votes`
- `dem_pct`, `rep_pct`
- `margin` (dem_pct - rep_pct)
- `winner` ('D'/'R'/'TIE')

⚠️ **2020 only** - Not available for 2010/2000

## Adjacency Graphs

**Format**: NetworkX Graph (pickled)
**Location**: `data/adjacency/{year}/{state}_adjacency_{year}.pkl`

**Structure**:
```python
# Adjacency list (zero-indexed tract IDs)
adjacency = {
    0: [1, 2, 5],      # Tract 0 neighbors: 1, 2, 5
    1: [0, 3, 4],      # Tract 1 neighbors: 0, 3, 4
    ...
}

# Load
import pickle
with open(file, 'rb') as f:
    adjacency = pickle.load(f)
```

**Edge Weights** (optional - for edge-weighted mode):
```python
edge_weights = {
    (0, 1): 1234.56,   # Boundary length in meters
    (0, 2): 789.12,
    ...
}
```

**Generation**: `scripts/data/geography/build_adjacency.py`
**Validation**: `scripts/data/geography/check_graph_connectivity.py`

## District Assignments

**Format**: Python dict (pickled)
**Location**: `outputs/us_{year}_{version}/states/{state}/data/final_assignments.pkl`

**Structure**:
```python
assignments = {
    0: 1,    # Tract 0 → District 1
    1: 1,    # Tract 1 → District 1
    2: 2,    # Tract 2 → District 2
    ...
}
```

**CSV Export**: `district_summary.csv` (district metrics)

## Parquet Files

**Why Parquet?**
- ✅ Fast (columnar storage)
- ✅ Compressed (5-10x smaller than CSV)
- ✅ Typed (preserves dtypes, no int/str confusion)
- ✅ Pandas-native

**Pattern**:
```python
# Write
tracts.to_parquet('tracts.parquet', engine='pyarrow', compression='snappy')

# Read
tracts = gpd.read_parquet('tracts.parquet')
```

**Use CSV only for**: Human-readable outputs, Excel compatibility

## GEOID Handling (CRITICAL)

⚠️ **Problem**: GEOIDs read as int → lose leading zeros → merge fails

**Solution**: Force string + zero-pad to 11 chars

```python
# ✅ ALWAYS do this
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)

# Before ANY: loading, merging, filtering, joining
```

**Example**:
```python
# Wrong
tracts['GEOID']  # int64: 6001400100

# Right
tracts['GEOID'] = tracts['GEOID'].astype(str).str.zfill(11)
tracts['GEOID']  # str: '06001400100'
```

## File Naming Conventions

### Input Data
```
data/tracts/{year}/{state}_tracts_{year}.parquet
data/tracts/{year}/{state}_places_{year}.parquet
data/adjacency/{year}/{state}_adjacency_{year}.pkl
data/processed/demographics/{year}_demographics_tract.parquet
data/processed/elections/{year}_president_tract.parquet
```

### Output Files
```
outputs/us_{year}_{version}/
├─ states/{state_name}/
│  ├─ data/
│  │  ├─ final_assignments.pkl
│  │  ├─ district_summary.csv
│  │  ├─ district_cities.csv
│  │  └─ rounds_hierarchy.csv
│  ├─ maps/
│  │  ├─ {state}_{N}_districts.png
│  │  ├─ districts/district_{dd}.png
│  │  └─ rounds/round_{dd}.png
│  ├─ political/district_political.csv, maps/
│  ├─ demographic/district_demographics.csv, maps/
│  └─ compactness/district_compactness.csv, maps/
├─ data/
│  ├─ us_all_districts.csv
│  ├─ us_district_summary.csv
│  └─ us_rounds_hierarchy.csv
└─ maps/us_*.png
```

## Data Sizes

**Input** (per year):
- Tracts (all states): ~2GB compressed
- Adjacency graphs: ~500MB
- Demographics: ~100MB
- Elections: ~50MB (2020 only)

**Output** (per run):
- Maps (150 DPI): ~5GB
- Maps (300 DPI): ~20GB
- CSV/data: ~500MB

**Total**: ~40GB input data, ~20GB output per run

## Output Directory Structure

**Pattern**: `outputs/us_{year}_{version}/`

**Year**: 2020, 2010, 2000
**Version**: User-specified (v1, v2, test, ...)

**Example**: `outputs/us_2020_v1/`

**State naming**: lowercase_underscores (`california`, `new_york`)

## CSV Schemas

### district_summary.csv

```
district,population,area_sq_km,perimeter_km,polsby_popper,reock,num_tracts,num_cities
1,761169,4523.45,412.3,0.334,0.612,142,8
2,761234,3891.12,389.7,0.322,0.589,128,6
```

### district_cities.csv

```
district,city_name,city_population,city_geoid
1,Los Angeles,3898747,0644000
1,Long Beach,462628,0643000
2,San Diego,1386932,0666000
```

### district_political.csv (2020 only)

```
district,dem_votes,rep_votes,total_votes,dem_pct,rep_pct,margin,winner
1,345123,234567,589012,58.6,39.8,18.8,D
2,287654,298765,598234,48.1,49.9,-1.8,R
```

### district_demographics.csv

```
district,total_pop,male,female,white_nh,black_nh,asian_nh,hispanic
1,761169,378234,382935,234567,123456,178901,198765
```

### rounds_hierarchy.csv

```
round,district_ids,population,parent_district
0,1,761169,
1,2-27,20517500,1
1,28-52,20517500,1
2,2-14,10258750,2-27
2,15-27,10258750,2-27
```

## References

- **Census TIGER/Line**: https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html
- **PL 94-171 Files**: https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html
- **Census API**: https://www.census.gov/data/developers/data-sets.html
- **MIT Election Lab**: https://electionlab.mit.edu/data
- **Parquet Format**: https://parquet.apache.org/
