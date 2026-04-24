# Census Data Structure

## Short version

We use three Census data sources: TIGER/Line shapefiles (tract geometries), PL 94-171 redistricting files (population counts), and ACS demographics (race/ethnicity). All data lives in `data/{year}/`. The `download_orchestrator.py` script fetches everything directly from Census Bureau servers.

---

## The three sources

### 1. TIGER/Line shapefiles (`data/{year}/tiger/`)

Geographic boundaries for every census tract in every state. A census tract is the fundamental unit — small enough to capture neighborhood-level population variation, large enough to have stable boundaries across decades.

Each state's tract file is a shapefile with columns:
- `GEOID`: 11-digit tract identifier (`{state_fips}{county_fips}{tract_code}`)
- `geometry`: polygon boundary
- Population is NOT in the shapefile — it comes from PL 94-171

Typical state: 500–8,000 tracts. California has ~8,000; Wyoming has ~130.

### 2. PL 94-171 redistricting files (`data/{year}/redistricting/`)

The Census Bureau publishes special redistricting files after each decennial census specifically for state redistricting purposes. These contain:
- Total population per tract
- Race/ethnicity counts (for VRA analysis)
- Voting-age population counts

These are tab-separated files with cryptic column names defined in the PL 94-171 technical documentation. The `scripts/data/census/` scripts parse and merge them.

### 3. ACS demographics (`data/{year}/demographics/`)

American Community Survey data for race/ethnicity percentages per tract. Used in the VRA variant and demographic analysis. Much smaller than PL 94-171 (pre-processed to ~5MB per year).

## Processed outputs

After running the census data processing pipeline, cleaned files land in `outputs/data/{year}/`:

- `adjacency/{state}_adjacency_{year}.pkl` — tract adjacency graph with edge weights (the key computed asset, ~7MB for all 50 states at tract level)
- `demographics/{state}_demographics_{year}.csv` — race/ethnicity per tract
- `elections/{year}_president_tract.csv` — 2020 election results merged to tracts (2020 only)

## Year differences

| Year | Redistricting files | TIGER source | Notes |
|------|--------------------|--------------|----|
| 2020 | PL 94-171 (2020) | TIGER 2020 | Primary run year |
| 2010 | PL 94-171 (2010) | TIGER 2010 | Cross-census validation |
| 2000 | PL 94-171 (2000) | TIGER 2000 | Historical baseline |

The 2000 data has a different directory structure (`data/2000/tracts/`, `data/2000/population/`) because the Census Bureau changed their file format between cycles. The download scripts handle these differences automatically.

## Downloading

```bash
# Check what's missing
python scripts/data/download_orchestrator.py --stages redistricting --year 2020 --check-only

# Download missing data (4 parallel workers)
python scripts/data/download_orchestrator.py --stages redistricting demographics --year 2020 --workers 4
```

The orchestrator skips files that already exist (cache-aware). A fresh 2020 download takes 1–2 hours on a decent connection.

## GEOID format

Every tract has an 11-digit GEOID: `SSCCCTTTTTT` where:
- `SS` = 2-digit state FIPS code
- `CCC` = 3-digit county FIPS code
- `TTTTTT` = 6-digit tract code

Example: `17031980000` = Illinois (17) + Cook County (031) + tract 9800.00

GEOIDs are the join key between geometry (TIGER) and population (PL 94-171). Leading zeros matter — always store as strings.

## Further reading

- `docs/CENSUS_DATA_PROCESSING.md` — detailed processing walkthrough
- `docs/DATA_DICTIONARY.md` — all column definitions
- `scripts/data/download_orchestrator.py --help`
