# Census 2020 PL 94-171 File Format Reference

## Overview
PL 94-171 is the official Census redistricting dataset containing block-level population data.

**Source:** https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html

## File Structure

Each state's data comes in a ZIP file containing **4 pipe-delimited (|) text files**:

1. **{state}geo2020.pl** - Geographic header file (contains GEOIDs and geographic info)
2. **{state}000012020.pl** - Segment 1: Race tables (P1, P2)
3. **{state}000022020.pl** - Segment 2: 18+ population and housing (P3, P4, H1)
4. **{state}000032020.pl** - Segment 3: Group quarters (P5)

Example for California: `cageo2020.pl`, `ca000012020.pl`, `ca000022020.pl`, `ca000032020.pl`

## Delimiter
- Files are **pipe-delimited**: `|`
- **NO HEADER ROW** - field positions are documented in tech docs

## Key Join Fields
These fields appear in ALL files and are used to join geographic data with population data:

| Field | Description |
|-------|-------------|
| FILEID | File Identification |
| STUSAB | State/US-Abbreviation (e.g., "CA") |
| CHARITER | Characteristic Iteration |
| CIFSN | Census Internal File Sequence Number |
| LOGRECNO | Logical Record Number (unique within state) |

## Geographic Header File Fields

| Field | Position | Description |
|-------|----------|-------------|
| FILEID | 1 | Always "PLST" |
| STUSAB | 2 | State abbreviation (e.g., "CA") |
| SUMLEV | 3 | Summary Level (750 = Block level) |
| GEOVAR | 4 | Geographic Variant |
| GEOCOMP | 5 | Geographic Component |
| CHARITER | 6 | Characteristic Iteration |
| CIFSN | 7 | Census Internal File Sequence Number |
| LOGRECNO | 8 | Logical Record Number |
| GEOID | 9 | Geographic Identifier (15 digits for blocks) |
| GEOCODE | 10 | Geographic Code |
| REGION | 11 | Region |
| DIVISION | 12 | Division |
| STATE | 13 | State FIPS code (2 digits) |
| STATENS | 14 | State GNIS code |
| COUNTY | 15 | County FIPS code (3 digits) |
| COUNTYCC | 16 | County FIPS Class Code |
| COUNTYNS | 17 | County GNIS code |
| COUSUB | 18 | County Subdivision |
| COUSUBCC | 19 | County Subdivision FIPS Class Code |
| COUSUBNS | 20 | County Subdivision GNIS code |
| SUBMCD | 21 | Subminor Civil Division |
| SUBMCDCC | 22 | Subminor Civil Division FIPS Class Code |
| SUBMCDNS | 23 | Subminor Civil Division GNIS code |
| ESTATE | 24 | Estate |
| ESTATECC | 25 | Estate FIPS Class Code |
| ESTATENS | 26 | Estate GNIS code |
| CONCIT | 27 | Consolidated City |
| CONCITCC | 28 | Consolidated City FIPS Class Code |
| CONCITNS | 29 | Consolidated City GNIS code |
| PLACE | 30 | Place |
| PLACECC | 31 | Place FIPS Class Code |
| PLACENS | 32 | Place GNIS code |
| TRACT | 33 | Census Tract (6 digits) |
| BLKGRP | 34 | Block Group (1 digit) |
| BLOCK | 35 | Block (4 digits) |
| AIANHH | 36 | American Indian Area/Alaska Native Area/Hawaiian Home Land |
| ... | ... | (many more geographic fields) |

## Population Data Files (Segment 1: ca000012020.pl)

**Table P1: Race**
- **P0010001** - Total Population ⭐ **THIS IS THE KEY FIELD**
- P0010002 - Population of one race
- P0010003 - White alone
- P0010004 - Black or African American alone
- P0010005 - American Indian and Alaska Native alone
- P0010006 - Asian alone
- P0010007 - Native Hawaiian and Other Pacific Islander alone
- P0010008 - Some Other Race alone
- P0010009 - Population of two or more races
- ... (continues through P0010071)

**Table P2: Hispanic or Latino, and Not Hispanic or Latino by Race**
- P0020001 - Total Population
- P0020002 - Hispanic or Latino
- P0020003 - Not Hispanic or Latino
- ... (continues)

## Block-Level Records
To get block-level data only, filter by:
- **SUMLEV = '750'** (Summary Level for Census Blocks)

## GEOID Format (15 digits)
- Positions 1-2: State FIPS (e.g., "06" for CA)
- Positions 3-5: County FIPS (e.g., "037" for Los Angeles)
- Positions 6-11: Census Tract (e.g., "980000")
- Positions 12-15: Block (e.g., "1045")

Example: `060379800001045` = California, Los Angeles County, Tract 9800.00, Block 1045

## How to Parse Files

```python
import pandas as pd
from io import StringIO

# Read files without headers (Census provides no headers)
geo_df = pd.read_csv('cageo2020.pl', delimiter='|', dtype=str, header=None, encoding='latin-1')
data_df = pd.read_csv('ca000012020.pl', delimiter='|', dtype=str, header=None, encoding='latin-1')

# Apply column names based on technical documentation
# Geographic header has ~100 columns (see tech doc page 99+)
# Data file 1 has columns for join fields + P1/P2 tables

# Filter to blocks
geo_df = geo_df[geo_df[2] == '750']  # Column 2 = SUMLEV

# Join on LOGRECNO (column varies, typically column 7 in geo, column 5 in data)
merged = geo_df.merge(data_df, on=[0,1,5,6,7], how='inner')  # Adjust column numbers

# Extract GEOID (column 8 in geo file) and P0010001 (column for total pop in data file)
```

## Alternative: Use census API or cenpy
Rather than parsing raw files, use:
- Census API with DECENNIALPL2020 dataset (requires API key)
- `cenpy` Python library (no API key required)
- Variable name in API: **P1_001N** (same as P0010001 in files)

## Download URLs
Format: `https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/{StateName}/{stateabbrev}2020.pl.zip`

Example: `https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/California/ca2020.pl.zip`

## Important Notes
1. Files have NO header row - you must apply column names manually
2. All fields are strings - convert to numeric as needed
3. Encoding is 'latin-1' (not UTF-8)
4. LOGRECNO is unique within a state but not across states
5. Files can be very large (CA is ~80MB compressed)

## References
- [Census PL 94-171 Summary Files](https://www.census.gov/programs-surveys/decennial-census/about/rdo/summary-files.html)
- [Technical Documentation](https://www2.census.gov/programs-surveys/decennial/2020/technical-documentation/complete-tech-docs/summary-file/2020Census_PL94_171Redistricting_StatesTechDoc_English.pdf)
- [Redistricting Data Hub - Field Descriptions](https://redistrictingdatahub.org/data/about-our-data/pl-94171-dataset/fields-and-descriptions/)
- [Census API Documentation](https://www.census.gov/data/developers/data-sets/decennial-census.html)
