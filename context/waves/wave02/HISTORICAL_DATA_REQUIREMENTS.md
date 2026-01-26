# Historical Census Data Requirements

## Overview

This document outlines the requirements for processing historical census data (2010 and 2000) to ensure feature parity with the 2020 implementation.

## Critical Requirements

### 1. County-Aware Water-Based Adjacency

**IMPORTANT**: When creating adjacency graphs for island tracts, we must include water-based connections to the nearest tract **within the same county**.

#### Why This Matters
- Islands (e.g., Hawaiian islands, Alaskan islands, coastal islands) would otherwise be isolated
- Without water adjacency, METIS cannot create contiguous districts
- County-aware matching prevents connecting islands to wrong regions

#### Implementation Details

**Current 2020 Implementation**: `scripts/create_adjacency_with_water.py`

Key features:
- Identifies island tracts (tracts with no land-based neighbors)
- For each island tract:
  - Finds nearest tract **with same GEOID prefix** (county code)
  - Creates water-based adjacency link
  - Allows contiguous districts across water within counties

**Data Requirements**:
- Tract shapefiles with GEOID field (contains county code)
- Accurate geometry for spatial distance calculations

### 2. Data Pipeline for 2010 Census

#### Step 1: Download Tract Shapefiles
- **Source**: Census Bureau TIGER/Line Shapefiles 2010
- **URL Pattern**: `https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/tl_2010_{FIPS}_tract10.zip`
- **Files Needed**: All 50 states + DC
- **Format**: Shapefile (.shp, .shx, .dbf, .prj)
- **Key Fields**:
  - GEOID10 (tract identifier with county code)
  - POP10 (population)
  - ALAND10 (land area)
  - AWATER10 (water area)

#### Step 2: Create Adjacency Graphs with Water Connections
```bash
# For each state
python scripts/create_adjacency_with_water.py \
  --input data/raw/{state}_tracts_2010.parquet \
  --output data/adjacency/{state}_adjacency_2010.pkl \
  --year 2010
```

**Critical**: Script must:
- Detect island tracts (no land neighbors)
- Match islands to nearest same-county tract
- Use GEOID10 field for county matching

#### Step 3: Download Places (Cities) Data
- **Source**: Census Bureau TIGER/Line Places 2010
- **URL Pattern**: `https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2010/tl_2010_{FIPS}_place10.zip`
- **Key Fields**:
  - GEOID10
  - NAME
  - POP10

#### Step 4: Apportionment Configuration
Create state district counts for 2010:
- Based on 2010 Census apportionment
- Total: 435 districts
- Varies by state (e.g., CA had 53 in 2010, now 52 in 2020)

### 3. Data Pipeline for 2000 Census

#### Step 1: Download Tract Shapefiles
- **Source**: Census Bureau TIGER/Line Shapefiles 2000
- **URL Pattern**: `https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2000/tl_2010_{FIPS}_tract00.zip`
- **Note**: 2000 shapefiles may be in different format/structure
- **Key Fields**:
  - CTIDFP00 or GEOID (tract identifier)
  - POP2000 (population)

#### Step 2: Create Adjacency Graphs with Water Connections
```bash
# For each state
python scripts/create_adjacency_with_water.py \
  --input data/raw/{state}_tracts_2000.parquet \
  --output data/adjacency/{state}_adjacency_2000.pkl \
  --year 2000
```

**Critical**: Script must:
- Detect island tracts
- Match islands to nearest same-county tract
- Handle 2000-era GEOID format differences

#### Step 3: Download Places (Cities) Data
- **Source**: Census Bureau TIGER/Line Places 2000
- **Key Fields**:
  - GEOID
  - NAME
  - POP2000

#### Step 4: Apportionment Configuration
Create state district counts for 2000:
- Based on 2000 Census apportionment
- Total: 435 districts
- Varies by state

## Testing Requirements

When processing historical data, verify:

1. **Island Connectivity**: Check ALL states with island populations requiring water adjacency

   **Major Island States** (critical to verify):
   - **Hawaii** - Multiple major islands (Oahu, Maui, Big Island, Kauai, Molokai, Lanai) across 4 counties
   - **Alaska** - Extensive Aleutian Islands, Alexander Archipelago, coastal island communities
   - **Massachusetts** - Martha's Vineyard, Nantucket, Boston Harbor Islands, Elizabeth Islands
   - **California** - Channel Islands (Santa Catalina, San Clemente, Santa Cruz, Anacapa, Santa Rosa, San Miguel), SF Bay islands (Alameda, Treasure Island)
   - **Washington** - San Juan Islands (100+ islands), Whidbey Island, Camano Island, islands in Puget Sound
   - **New York** - Long Island, Staten Island, Fire Island, Fishers Island, Shelter Island, City Island
   - **Florida** - Florida Keys (Monroe County), barrier islands (Sanibel, Captiva, Anna Maria, etc.)
   - **Michigan** - Mackinac Island, Isle Royale, Beaver Island, Drummond Island, Sugar Island, numerous Great Lakes islands
   - **Maine** - 3,000+ coastal islands including Mount Desert Island, Vinalhaven, Isle au Haut, Matinicus

   **Additional States with Islands**:
   - **Rhode Island** - Block Island, Aquidneck Island (Newport), Conanicut Island, islands in Narragansett Bay
   - **Maryland** - Chesapeake Bay islands (Smith Island, Tangier Island accessible from MD, Kent Island)
   - **Virginia** - Eastern Shore barrier islands, Chincoteague, Tangier Island, Chesapeake Bay islands
   - **North Carolina** - Outer Banks (Hatteras, Ocracoke, Roanoke Island), barrier islands
   - **South Carolina** - Sea Islands (Hilton Head, Edisto, Johns Island, James Island, Kiawah, Seabrook)
   - **Georgia** - Sea Islands (St. Simons, Jekyll Island, Tybee, Cumberland, Sapelo)
   - **Louisiana** - Barrier islands, Grand Isle, islands in bayous and Lake Pontchartrain
   - **Texas** - Galveston Island, Padre Island, Mustang Island, Matagorda Island, barrier islands
   - **Connecticut** - Long Island Sound islands (Thimble Islands), coastal islands
   - **New Jersey** - Barrier islands (Long Beach Island, Atlantic City area, Cape May area)
   - **Delaware** - Coastal barrier areas
   - **Wisconsin** - Apostle Islands (Lake Superior), Washington Island (Door County), islands in Lake Michigan
   - **Minnesota** - Islands in Lake Superior including Grand Portage area
   - **Ohio** - Lake Erie islands (Put-in-Bay/South Bass Island, Kelley's Island, Middle Bass Island, North Bass Island)
   - **Pennsylvania** - Presque Isle (Lake Erie)
   - **Oregon** - Coastal islands and bay islands
   - **Vermont** - Islands in Lake Champlain (Grand Isle County, North Hero, South Hero, Isle La Motte)
   - **New Hampshire** - Isles of Shoals (shared with Maine), coastal islands
   - **Illinois** - Potential Mississippi River islands
   - **Tennessee** - Potential Mississippi River islands
   - **Mississippi** - Barrier islands (Ship Island, Horn Island, Petit Bois Island)
   - **Alabama** - Dauphin Island, barrier islands in Mobile Bay

   **Summary**: At least **30+ states** have island or coastal geography requiring water-based adjacency connections. This is NOT an edge case - it's a fundamental requirement for the majority of US states.

2. **County Boundaries**: Ensure water adjacencies respect county lines
   - Example: Hawaiian islands should connect within their own counties (Honolulu County, Maui County, Hawaii County, Kauai County)
   - Alaskan island communities should connect to same borough
   - Massachusetts islands: Martha's Vineyard (Dukes County) and Nantucket (Nantucket County) should not connect to each other
   - Great Lakes islands should connect to proper mainland counties
   - Barrier islands should connect to proper mainland counties

3. **Contiguity**: All districts must be contiguous (including water connections)
   - Run validation: `python scripts/validate_contiguity.py --year 2010`

## Script Updates Needed

### `create_adjacency_with_water.py`
- Add `--year` parameter to handle different GEOID field names
- Support GEOID10 (2010) and CTIDFP00/GEOID (2000)
- Maintain county-aware water adjacency logic

### `run_state_redistricting.py`
- Add `--year` parameter
- Load correct tract/adjacency files for specified year
- Use correct population field names

### `add_cities_to_districts.py`
- Add `--year` parameter
- Load correct places data for specified year
- Handle different field name conventions

## Historical Apportionment

### 2010 Apportionment (by state)
- CA: 53 districts (lost 1 in 2020)
- TX: 36 districts (gained 2 in 2020)
- NY: 27 districts (lost 1 in 2020)
- FL: 27 districts (gained 1 in 2020)
- IL: 18 districts (lost 1 in 2020)
- OH: 16 districts (lost 1 in 2020)
- ... (need full list)

### 2000 Apportionment (by state)
- CA: 53 districts
- TX: 32 districts
- NY: 29 districts
- ... (need full list)

## References

- Census Bureau TIGER/Line Shapefiles: https://www.census.gov/geographies/mapping-files/time-series/geo/tiger-line-file.html
- Apportionment Results: https://www.census.gov/data/tables/time-series/dec/apportionment-data.html
- Current 2020 water adjacency implementation: `scripts/create_adjacency_with_water.py`

## Next Steps

1. Document exact URLs for 2010 and 2000 tract/places downloads
2. Create download scripts for bulk data retrieval
3. Update `create_adjacency_with_water.py` to support multiple census years
4. Research 2000 shapefile format differences
5. Obtain official 2010 and 2000 apportionment tables
6. Create state configuration files for each census year

---

**Last Updated**: 2026-01-09
**Status**: Planning phase - 2020 complete, 2010/2000 pending
