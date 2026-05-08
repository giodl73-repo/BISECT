# Raw Census Data Directory

**Last Updated**: 2026-01-18

This directory contains all raw census data used for the Congressional Redistricting project. Data is organized by census year for consistency.

## Directory Structure

```
data/
  ├─ Census 2000/  # All 2000 Census data
  ├─ Census 2010/  # All 2010 Census data
  └─ Census 2020/  # All 2020 Census data
```

## Organization Principle

**All data organized by census year**, not by source:
- Makes it easy to find all data for a specific year
- Source documented in year-specific README files
- Consistent pattern across all years

## Data Sources

### Census 2000
- **Source**: NHGIS (National Historical Geographic Information System)
- **Downloaded**: 2026-01-18
- **Contents**: National shapefiles, population CSVs, baseline congressional districts
- **Details**: See `Census 2000/README.md`

### Census 2010
- **Source**: Census Bureau PL 94-171 Redistricting Files
- **Downloaded**: 2026-01 (provided by user)
- **Contents**: State-by-state population and geography files
- **Details**: See `Census 2010/README.md`

### Census 2020
- **Source**: Census Bureau PL 94-171 Redistricting Files
- **Downloaded**: 2026-01 (provided by user)
- **Contents**: State-by-state population files
- **Details**: See `Census 2020/README.md`

## Important Notes

⚠️ **Never modify files in this directory** - These are source materials
⚠️ **Not in git** - This directory is in `.gitignore` (too large)
⚠️ **Backup recommended** - ~40GB of data, not easily re-downloadable

## Processing Flow

```
data/Census {year}/     (Raw data - this directory)
         ↓
  [Processing Scripts]
         ↓
outputs/data/           (Processed parquet files & adjacency graphs)
         ↓
  [Redistricting Pipeline]
         ↓
outputs/{version}/      (Final results - maps, CSVs, analysis)
```

## Related Documentation

- **Enhancement 47**: `context/enhancements/active/47_data_separation_restoration.md`
- **Data Formats**: `context/DATA_FORMATS.md`
- **Architecture**: `context/ARCHITECTURE.md`
- **Processed Data**: `outputs/data/README.md`
