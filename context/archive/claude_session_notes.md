# Claude Code Session Notes - Apportionment Project

## Project Overview
Redistricting all 50 US states using recursive bisection with METIS graph partitioning.
- Goal: Create fair congressional districts based on 2020 Census data
- Method: Recursive bisection using gpmetis for graph splitting
- Test state: California (52 districts)

## What's Been Done
1. ✅ Downloaded Census block-level data for all 50 states
2. ✅ Built adjacency graphs from shapefiles (including water-based adjacency)
3. ✅ Found working version of gpmetis for Windows
4. ✅ Got gpmetis working with a simple test
5. ⚠️ Attempted full California run - **POPULATION NUMBERS WRONG**

## Current Issue - RESOLVED
**Root Cause Found:** The shapefiles downloaded via pygris don't include population data!
- All 519,723 CA blocks have population = 0
- The adjacency graph was built with these zeros
- This made the algorithm think each block has ~1 person

**Solution In Progress:**
- Created `add_population_cenpy.py` script
- Uses cenpy library (no API key required)
- Currently downloading population data for all 58 CA counties
- Will merge with existing shapefile data

## Data Files Present
- `data/raw/ca_blocks_2020.parquet` - 519,723 blocks, but population all zeros
- `data/raw/ca_blocks_2020_003.parquet` - County 003 subset (sample data)
- `data/processed/ca_blocks_graph.pkl` - Built with zero population (needs rebuild)
- `data/processed/ca_blocks_graph_003.pkl` - County 003 graph

## Next Steps
1. ✅ Identify population data issue
2. ✅ Download real population data using cenpy (39,538,223 total - correct!)
3. ✅ Document PL 94-171 file format (see PL94171_FORMAT.md)
4. ✅ Rebuild adjacency graph with correct population weights (519K blocks, 1.7M edges)
5. ✅ Modified algorithm to save intermediate results by round
6. ✅ Added hierarchical binary naming (CA0, CA1 → CA00, CA01, CA10, CA11, etc.)
7. ✅ Created tract-level download/adjacency scripts
8. ✅ Downloaded proper tract data from Census (9,129 tracts, 39.5M population)
9. ✅ Created overview maps of CA tracts (verified shapes look correct!)
10. ✅ Built tract adjacency graph (9,129 tracts, 29,284 edges)
11. ✅ Connected 3 disconnected components (added 2 edges to make fully connected)
12. ✅ Fixed gpmetis contiguity issue (removed -ptype=rb to enable -contig)
13. ✅ Tested first split at tract level - **1.95% max deviation, FULLY CONTIGUOUS!**
14. ✅ Verified both regions (CA0, CA1) are internally connected
15. ✅ Generated map of actual first split (outputs/splits/ca_first_split_tracts.png)
16. ⏳ Downloading tract data for all 50 states in background (AL, AK, AZ done, AR in progress)

## Round-by-Round Testing
Created new capability to run and visualize each round separately:
- **Round 1:** 2 regions (CA0, CA1) - 26 districts each
- **Round 2:** 4 regions (CA00, CA01, CA10, CA11) - 13 districts each
- **Round 3:** 8 regions - 6-7 districts each
- **Rounds 4-6:** Continue until 52 final districts

Usage: `python scripts/test_rounds.py --state CA --num-districts 52 --round 1`

## Population Data
- **Correct total:** 39,538,223 (2020 Census)
- **Per district target (52 districts):** ~760,350 people
- **First split should be:** 26/26 districts → ~19,769,112 per side

## First Split Results (Tract Level) - FINAL
**Fixed gpmetis to enforce contiguity by removing `-ptype=rb` flag**

- **Left (CA0):** 20,154,342 people (4,631 tracts) → 775,167 per district (+1.95%)
- **Right (CA1):** 19,383,881 people (4,498 tracts) → 745,534 per district (-1.95%)
- **Max deviation:** 1.95% - Good balance
- **Connectivity:** Both regions are FULLY CONTIGUOUS ✓
- **Map:** outputs/splits/ca_first_split_tracts.png
- **Assignments:** outputs/splits/ca_first_split_tracts_assignments.pkl

**Key Fix:** gpmetis's `-ptype=rb` doesn't support `-contig`. Since we implement recursive bisection at a higher level, we just need gpmetis to do contiguous k-way partitioning for each 2-way split.

## Important Notes
- User wants to see the first split visualized before proceeding
- Don't rush ahead without user approval
- The first split numbers matter because errors compound in recursive splits

## California Full Redistricting - Results

### Target
- 52 congressional districts
- ~760,350 people per district (39,538,223 / 52)
- First split: 52 → 26/26 districts

### Strategy Insights
**Dynamic ufactor approach:**
- Depth 1 (2 regions): ufactor=1.001 (0.1% tolerance) - Super tight
- Depth 2 (4 regions): ufactor=1.002 (0.2% tolerance)
- Depth 3 (8 regions): ufactor=1.003 (0.3% tolerance)
- Depth 4+ (16+ regions): ufactor=1.005 (0.5% tolerance)

**Problem with 52 districts:**
- 52 is not a power of 2, causing uneven splits (7/6, 4/3, 2/1)
- Each split is balanced, but uneven district counts create compounding imbalances
- Round 1-5 deviations: 0.00%, 0.02%, 0.11%, 0.18%, 0.30% - Excellent!
- Round 6 (52 final): 62.87% deviation - too high due to recursive splitting artifacts
- Population range: 615k - 1.24M (ideal: 760k)

**Visualization improvements:**
- Labels now use white text with black outline (no background box)
- More visible against all region colors
- Versioned output directories with timestamps
- Need smaller labels for dense regions (LA, Bay Area)
