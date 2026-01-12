# Data Formats

Comprehensive reference for all data formats used in the congressional redistricting project.

## Overview

This project works with multiple data sources and formats:

1. **Census Tract Geometries** - TIGER/Line Shapefiles (2020, 2010, 2000)
2. **Population Data** - PL 94-171 Redistricting Files
3. **Demographic Data** - Census DHC API
4. **Election Data** - MIT Election Lab CSV files
5. **Adjacency Graphs** - NetworkX pickled graphs
6. **District Assignments** - Python dictionaries (pickled)

## Census TIGER/Line Shapefiles

### Format
- **Type**: ESRI Shapefile (.shp, .shx, .dbf, .prj)
- **Source**: Census Bureau TIGER/Line
- **Resolution**: Census tract level (~84K tracts nationwide)
- **Projection**: NAD83 (EPSG:4269)

### 2020 Tract Files

**URL Pattern**:
```
https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_{FIPS}_tract20.zip
```

**Key Fields**:
- `GEOID20`: 11-digit tract identifier (string)
- `NAME20`: Tract name
- `ALAND20`: Land area (sq meters)
- `AWATER20`: Water area (sq meters)
- `geometry`: Polygon geometry

**GEOID Format** (11 digits):
```
SSCCCTTTTTT
│  │  └─────── Tract code (6 digits, implied decimal: 123456 = 1234.56)
│  └────────── County FIPS (3 digits)
└──────────── State FIPS (2 digits)
```

**Example**: `06037980000` = California (06), Los Angeles County (037), Tract 9800.00

**Note**: Population is NOT included in TIGER/Line files - must be joined from PL 94-171 files.

### 2010 Tract Files

**URL Pattern**:
```
https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/tl_2010_{FIPS}_tract10.zip
```

**Key Fields**:
- `GEOID10`: 11-digit tract identifier (string)
- `NAME10`: Tract name
- `ALAND10`: Land area (sq meters)
- `AWATER10`: Water area (sq meters)
- `POP10`: Population (if available, but usually missing)
- `geometry`: Polygon geometry

### 2000 Tract Files

**URL Pattern**:
```
https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2000/tl_2010_{FIPS}_tract00.zip
```

**Key Fields**:
- `CTIDFP00` or `GEOID`: Tract identifier (format may vary)
- `NAME`: Tract name
- `POP2000`: Population (if available)
- `geometry`: Polygon geometry

**Note**: 2000 shapefiles have less standardized field names - check documentation for specific fields.

### Places (Cities) Files

**Purpose**: Point geometries for city centers (used for map labeling)

**2020 URL Pattern**:
```
https://www2.census.gov/geo/tiger/TIGER2020/PLACE/tl_2020_{FIPS}_place.zip
```

**2010 URL Pattern**:
```
https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2010/tl_2010_{FIPS}_place10.zip
```

**Key Fields**:
- `GEOID`: Place identifier
- `NAME`: City name
- `geometry`: Point or Polygon geometry (centroids used for labels)

**Usage**: Only places with population > 50,000 typically used for map labeling.

## Census PL 94-171 Redistricting Files

### Overview

Official Census redistricting dataset containing block-level population and demographic data.

**Source**: https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html

**Format**: Pipe-delimited text files (|) with NO HEADER ROW

### File Structure

Each state's data comes in a ZIP file containing 4 files:

1. **{state}geo2020.pl** - Geographic header (GEOIDs, geographic info)
2. **{state}000012020.pl** - Segment 1: Race tables (P1, P2)
3. **{state}000022020.pl** - Segment 2: 18+ population and housing (P3, P4, H1)
4. **{state}000032020.pl** - Segment 3: Group quarters (P5)

**Example for California**: `cageo2020.pl`, `ca000012020.pl`, `ca000022020.pl`, `ca000032020.pl`

### Geographic Header File (geo2020.pl)

**Key Fields** (no header - position-based):

| Position | Field | Description |
|----------|-------|-------------|
| 1 | FILEID | Always "PLST" |
| 2 | STUSAB | State abbreviation (e.g., "CA") |
| 3 | SUMLEV | Summary Level (750 = Block, 140 = Tract) |
| 6 | CHARITER | Characteristic Iteration |
| 7 | CIFSN | Census Internal File Sequence Number |
| 8 | LOGRECNO | Logical Record Number (join key) |
| 9 | GEOID | Geographic Identifier |
| 13 | STATE | State FIPS code (2 digits) |
| 15 | COUNTY | County FIPS code (3 digits) |
| 33 | TRACT | Census Tract (6 digits) |
| 34 | BLKGRP | Block Group (1 digit) |
| 35 | BLOCK | Block (4 digits) |

### Population Data File (000012020.pl)

**Table P1: Race**

| Field | Description |
|-------|-------------|
| **P0010001** | **Total Population** ⭐ KEY FIELD |
| P0010002 | Population of one race |
| P0010003 | White alone |
| P0010004 | Black or African American alone |
| P0010005 | American Indian and Alaska Native alone |
| P0010006 | Asian alone |
| P0010007 | Native Hawaiian and Other Pacific Islander alone |
| P0010008 | Some Other Race alone |
| P0010009 | Population of two or more races |

**Table P2: Hispanic or Latino by Race**

| Field | Description |
|-------|-------------|
| P0020001 | Total Population |
| P0020002 | Hispanic or Latino |
| P0020003 | Not Hispanic or Latino |

### Block-Level vs Tract-Level

**Filter by Summary Level (SUMLEV)**:
- `140` = Census Tract level
- `750` = Census Block level

**Block GEOID Format** (15 digits):
```
SSCCCTTTTTTBBBB
│  │  │      └────── Block (4 digits)
│  │  └─────────────── Tract (6 digits)
│  └────────────────── County (3 digits)
└───────────────────── State (2 digits)
```

**Example**: `060379800001045` = CA (06), Los Angeles (037), Tract 9800.00, Block 1045

**Tract GEOID Format** (11 digits):
```
SSCCCTTTTTT
```

Truncate block GEOID to first 11 digits to get tract GEOID.

### Parsing PL 94-171 Files

```python
import pandas as pd

# Read files (no headers, pipe-delimited, latin-1 encoding)
geo_df = pd.read_csv('cageo2020.pl', delimiter='|', dtype=str,
                      header=None, encoding='latin-1')
data_df = pd.read_csv('ca000012020.pl', delimiter='|', dtype=str,
                       header=None, encoding='latin-1')

# Filter to tract level (SUMLEV = 140)
geo_df = geo_df[geo_df[2] == '140']

# Join on LOGRECNO (position 7 in geo, position 4 in data)
merged = geo_df.merge(data_df, left_on=7, right_on=4, how='inner')

# Extract GEOID (position 8) and total population (P0010001)
# Position varies - check technical documentation
```

### Download URLs

**Pattern**:
```
https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/{StateName}/{stateabbrev}2020.pl.zip
```

**Example**:
```
https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/California/ca2020.pl.zip
```

### Important Notes

1. **No Headers**: Files have NO header row - must apply column names manually
2. **String Fields**: All fields are strings - convert to numeric as needed
3. **Encoding**: Use `latin-1` encoding (not UTF-8)
4. **LOGRECNO**: Unique within a state, not across states
5. **Large Files**: CA is ~80MB compressed

### Alternative: Census API

Instead of parsing raw files, use Census API:

**Dataset**: `DECENNIALPL2020`
**Variable**: `P1_001N` (equivalent to P0010001)
**Requires**: Census API key (free from census.gov)

```python
import requests

url = "https://api.census.gov/data/2020/dec/pl"
params = {
    'get': 'P1_001N',  # Total population
    'for': 'tract:*',
    'in': 'state:06',  # California
    'key': 'YOUR_API_KEY'
}
response = requests.get(url, params=params)
data = response.json()
```

## Demographic Data (DHC)

### Source

Census Demographic and Housing Characteristics (DHC) Dataset

**API Endpoint**: https://api.census.gov/data/2020/dec/dhc

### Available Variables

**Sex**:
- `P12_001N`: Total population
- `P12_002N`: Male
- `P12_026N`: Female

**Race/Ethnicity (Non-Hispanic)**:
- `P3_003N`: White alone, not Hispanic
- `P3_004N`: Black alone, not Hispanic
- `P3_006N`: Asian alone, not Hispanic

**Hispanic**:
- `P4_002N`: Hispanic or Latino (any race)

### API Query Example

```python
import requests

url = "https://api.census.gov/data/2020/dec/dhc"
params = {
    'get': 'P12_001N,P12_002N,P12_026N,P3_003N,P3_004N,P3_006N,P4_002N',
    'for': 'tract:*',
    'in': 'state:06',  # California
    'key': 'YOUR_API_KEY'
}
response = requests.get(url, params=params)
data = response.json()
```

### Output Format

**Processed Parquet**: `data/processed/demographics/2020_demographics_tract.parquet`

**Columns**:
- `GEOID` (string, 11 digits, zero-padded)
- `state` (string, 2 digits)
- `county` (string, 3 digits)
- `tract` (string, 6 digits)
- `total_pop` (int)
- `male`, `female` (int)
- `white_non_hispanic`, `black_non_hispanic`, `asian_non_hispanic`, `hispanic`, `other` (int)
- `male_pct`, `female_pct` (float, 0-100)
- `white_pct`, `black_pct`, `asian_pct`, `hispanic_pct`, `other_pct` (float, 0-100)

## Election Data (MIT Election Lab)

### Source

MIT Election Data + Science Lab
https://electionlab.mit.edu/data

### Format

**Raw**: CSV files per state
**Processed**: Parquet file aggregated by census tract

### Raw CSV Format

**File**: `data/raw/elections/{state}_precinct_{year}.csv`

**Columns**:
- `precinct`: Precinct identifier
- `county`: County name
- `candidate`: Candidate name
- `party`: Party affiliation
- `votes`: Vote count

### Processed Parquet Format

**File**: `data/processed/elections/2020_president_tract.parquet`

**Columns**:
- `GEOID` (string, 11 digits, zero-padded)
- `total_votes` (int)
- `biden_votes` (int)
- `trump_votes` (int)
- `other_votes` (int)

**Note**: Geocoded from precincts to tracts (approximation - precinct boundaries don't perfectly align with census tracts)

### Coverage

- **2020**: 48 states (missing AK, HI tract-level data)
- **2016**: Similar coverage

## Adjacency Graphs

### Format

**Type**: NetworkX Graph (pickled)
**File**: `data/adjacency/{state}_adjacency_{year}.pkl`

### Graph Structure

**Nodes**: One per census tract

**Node Attributes**:
- `GEOID`: 11-digit tract identifier (string)
- `population`: Tract population (int)
- `geometry`: Shapely polygon (optional)

**Edges**: Tract adjacency (queen contiguity - shared edge or corner)

**Edge Attributes**: None (unweighted graph)

### Special Handling: Water-Based Adjacency

**Problem**: Island tracts with no land-based neighbors

**Solution**: Add water-based adjacency to nearest tract in same county

**Implementation**:
1. Identify island tracts (degree 0 in land-only graph)
2. For each island:
   - Find nearest tract with same county GEOID prefix
   - Add edge if distance < threshold (e.g., 5 km)
3. Verify all tracts have at least one neighbor

**Critical for**:
- Hawaii (multiple islands across 4 counties)
- Alaska (Aleutian Islands, Alexander Archipelago)
- Massachusetts (Martha's Vineyard, Nantucket)
- Washington (San Juan Islands)
- Michigan (Great Lakes islands)
- Many coastal states (30+ states affected)

**County-Aware Matching**:
```python
# Extract county code from GEOID (first 5 digits)
island_county = island_geoid[:5]  # e.g., "06037" for LA County

# Find nearest tract with same county prefix
for tract in candidate_tracts:
    if tract['GEOID'][:5] == island_county:
        if distance(island, tract) < 5000:  # 5 km threshold
            graph.add_edge(island, tract)
```

### Loading Adjacency Graphs

```python
import pickle
import networkx as nx

# Load graph
with open('data/adjacency/ca_adjacency_2020.pkl', 'rb') as f:
    graph = pickle.load(f)

# Get node attributes
populations = nx.get_node_attributes(graph, 'population')
geoids = nx.get_node_attributes(graph, 'GEOID')

# Check connectivity
assert nx.is_connected(graph), "Graph must be connected"
```

## District Assignments

### Format

**Type**: Python dictionary (pickled)
**File**: `{state}/final_assignments.pkl`

### Structure

```python
{
    'GEOID_1': district_number,
    'GEOID_2': district_number,
    ...
}
```

**Example**:
```python
{
    '06001400100': 1,  # Tract in district 1
    '06001400200': 1,  # Tract in district 1
    '06001400300': 2,  # Tract in district 2
    ...
}
```

**Keys**: Census tract GEOID (string, 11 digits, zero-padded)
**Values**: District number (int, 1 to N)

### Loading Assignments

```python
import pickle

# Load assignments
with open('california/final_assignments.pkl', 'rb') as f:
    assignments = pickle.load(f)

# Count districts
num_districts = max(assignments.values())

# Get tracts in district 5
district_5_tracts = [geoid for geoid, dist in assignments.items() if dist == 5]
```

## Parquet Files

### Why Parquet?

- **Fast**: Columnar storage enables efficient filtering and aggregation
- **Compressed**: 5-10x smaller than CSV
- **Typed**: Preserves data types (no GEOID int/str confusion)
- **Pandas-native**: Direct read/write
- **Widely supported**: Spark, Dask, Arrow

### Reading/Writing Parquet

```python
import pandas as pd

# Write
df.to_parquet('data.parquet', compression='snappy', index=False)

# Read
df = pd.read_parquet('data.parquet')

# Read specific columns only (fast!)
df = pd.read_parquet('data.parquet', columns=['GEOID', 'population'])
```

### Compression

- **Default**: Snappy (fast compression/decompression)
- **Alternative**: gzip (better compression ratio, slower)
- **Not recommended**: Uncompressed (10x larger files)

## GEOID Handling (CRITICAL)

### The Problem

Census tract GEOIDs are 11-digit identifiers with leading zeros:
- California: `06001400100` (starts with 06)
- Alabama: `01001020100` (starts with 01)

When stored as integers, leading zeros are lost:
- `06001400100` → `6001400100` (10 digits, wrong!)
- `01001020100` → `1001020100` (10 digits, wrong!)

### The Solution

**ALWAYS use this pattern before merging DataFrames**:

```python
# Convert both dataframes to string with zero-padding
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)

# Now safe to merge
result = df1.merge(df2, on='GEOID')
```

### Why zfill(11)?

- Census tract GEOIDs are exactly 11 digits
- State (2) + County (3) + Tract (6) = 11
- `zfill(11)` pads with leading zeros if needed
- `'6001400100'.zfill(11)` → `'06001400100'`

### When to Apply

- After reading from CSV (often stored as int)
- After reading from shapefile (dtype varies)
- Before any merge/join operations
- When creating new GEOIDs from components

## File Naming Conventions

### State-Level Data

```
{state_code}_{datatype}_{year}.{ext}
```

**Examples**:
- `ca_tracts_2020.parquet`
- `ny_places_2020.parquet`
- `tx_adjacency_2020.pkl`
- `fl_demographics_2020.csv`

### National Aggregates

```
{year}_{datatype}_tract.parquet
```

**Examples**:
- `2020_president_tract.parquet`
- `2020_demographics_tract.parquet`

### District Assignments

```
{state}/final_assignments.pkl
```

**Example**: `california/final_assignments.pkl`

### Adjacency Graphs

```
{state_code}_adjacency_{year}.pkl
```

**Example**: `ca_adjacency_2020.pkl`

## Data Sizes

### Raw Data (data/raw/)

| Dataset | Per State | All States |
|---------|-----------|------------|
| Tracts (parquet) | 50-500 MB | ~10 GB |
| Places (parquet) | 1-5 MB | ~200 MB |
| Demographics (CSV) | 1-5 MB | ~250 MB |
| Elections (CSV) | 5-20 MB | ~500 MB |

### Processed Data (data/processed/)

| Dataset | Size |
|---------|------|
| 2020 Demographics (parquet) | 3-5 MB |
| 2020 Elections (parquet) | 10-15 MB |

### Adjacency Graphs (data/adjacency/)

| Per State | All States |
|-----------|------------|
| 100KB-5MB | ~200 MB |

**Total Data Footprint**: ~12-15 GB for complete dataset (all 50 states, 2020)

## References

### Census Bureau
- TIGER/Line Shapefiles: https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html
- PL 94-171 Summary Files: https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html
- PL 94-171 Technical Documentation: https://www2.census.gov/programs-surveys/decennial/2020/technical-documentation/complete-tech-docs/summary-file/2020Census_PL94_171Redistricting_StatesTechDoc_English.pdf
- Census API: https://www.census.gov/data/developers/data-sets/decennial-census.html

### Election Data
- MIT Election Data + Science Lab: https://electionlab.mit.edu/data

### Additional Resources
- Redistricting Data Hub: https://redistrictingdatahub.org/
- Census Bureau Apportionment Data: https://www.census.gov/data/tables/time-series/dec/apportionment-data.html
