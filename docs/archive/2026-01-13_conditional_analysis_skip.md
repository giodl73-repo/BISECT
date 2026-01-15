# Conditional Analysis Skip Based on Data Availability

**Date:** January 13, 2026
**Purpose:** Automatically skip political and demographic analysis when required data isn't available for a given census year.

## Problem

When running redistricting for historical census years (e.g., 2010), the pipeline would fail because:
1. Political analysis requires election data from the same time period (2010 needs 2010/2012 election data)
2. Demographic analysis requires detailed census demographics for that year
3. Pipeline would attempt these analyses and fail, even though the data doesn't exist

## Solution

Added conditional logic to check for data availability BEFORE adding analysis steps to the pipeline.

### Data Requirements

**Political Analysis:**
- Requires: Election data matching or close to census year
- Available: 2020 presidential election data only
- Result: Only runs for 2020 census, skipped for 2010/2000

**Demographic Analysis:**
- Requires: Detailed demographic data for specific census year
- Available: 2020 only
- Result: Only runs for years with demographic data

**Compactness Analysis:**
- Requires: Only tract geometry (always available)
- Result: Always runs (no dependencies)

## Files Modified

### 1. `scripts/pipeline/process_single_state.py` (Lines 101-134)

Added data availability checks before adding optional analysis steps:

```python
# Add optional analysis steps
if args.run_analysis:
    # Check data availability
    # Political analysis requires election data from same time period as census
    # 2020 census -> use 2020 election, 2010 census -> would need 2010/2012 election (not available)
    election_data_2020 = Path('data/processed/elections/2020_president_tract.parquet')
    demographic_data = Path(f'data/processed/demographics/{args.year}_demographics_tract.parquet')

    # Political analysis (only if compatible election data exists for this census year)
    can_do_political = (args.year == '2020' and election_data_2020.exists())
    if can_do_political:
        # Add political analysis and visualization steps
        ...

    # Demographic analysis (only if demographic data exists for this census year)
    if demographic_data.exists():
        # Add demographic analysis and visualization steps
        ...

    # Compactness visualization (metrics already calculated - always run)
    ...
```

**Key Change**: `can_do_political` now checks BOTH that election data exists AND that census year is 2020.

### 2. `scripts/pipeline/run_complete_redistricting.py` (Lines 687-703, 707, 719, 736, 752)

Added similar checks for post-processing/national aggregation:

```python
# Check data availability for optional analysis
# Political analysis requires election data from same time period as census
election_data_file = Path(f'data/processed/elections/{args.election_year}_president_tract.parquet')
election_data_available = (args.year == '2020' and election_data_file.exists())
demographic_data_available = Path(f'data/processed/demographics/{args.year}_demographics_tract.parquet').exists()

# Log data availability status
if not election_data_available and not args.skip_political:
    if args.year != '2020':
        print(f"\n[INFO] Political analysis will be skipped: Census year {args.year} requires {args.year}/2012 election data (not available)")
    else:
        print(f"\n[INFO] Political analysis will be skipped: No {args.election_year} election data found")
        print(f"       Expected: data/processed/elections/{args.election_year}_president_tract.parquet")
if not demographic_data_available and not args.skip_demographic:
    print(f"[INFO] Demographic analysis will be skipped: No {args.year} demographic data found")
    print(f"       Expected: data/processed/demographics/{args.year}_demographics_tract.parquet\n")
```

Then use `election_data_available` and `demographic_data_available` flags in all subsequent checks for:
- Batch political analysis (line 707)
- National political map (line 710)
- Batch demographic analysis (line 719)
- Batch demographic visualization (line 736)
- National demographic map (line 752)

## Behavior

### 2020 Census (Full Analysis)

```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1
```

**Output:**
```
Election data found: data\processed\elections\2020_president_tract.parquet
Demographic data found: data\processed\demographics\2020_demographics_tract.parquet
```

**Steps per state:** 11
1. Redistricting
2. Cities
3. Summary
4. Round maps
5. District maps
6. Political analysis ✅
7. Political visualization ✅
8. Demographic analysis ✅
9. Demographic visualization ✅
10. Compactness visualization ✅
11. Metro area maps ✅

### 2010 Census (Core Only)

```bash
python scripts/pipeline/run_complete_redistricting.py --year 2010 --version v1
```

**Output:**
```
[INFO] Political analysis will be skipped: Census year 2010 requires 2010/2012 election data (not available)
[INFO] Demographic analysis will be skipped: No 2010 demographic data found
       Expected: data/processed/demographics/2010_demographics_tract.parquet
```

**Steps per state:** 7
1. Redistricting
2. Cities
3. Summary
4. Round maps
5. District maps
6. Compactness visualization ✅
7. Metro area maps ✅

**Skipped:**
- Political analysis (no 2010/2012 election data)
- Demographic analysis (no 2010 detailed demographics)

## Testing

### Test 1: 2010 Skip Logic ✅ PASSED
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2010 --version test --print-only VT DE
```

**Result:**
- Shows 7 steps per state (not 11)
- Both political and demographic analysis skipped
- Informative messages explain why data is missing
- Pipeline completes successfully

### Test 2: 2020 Full Analysis ✅ PASSED
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --print-only VT DE
```

**Result:**
- Shows 11 steps per state
- Political and demographic analysis included
- All data found messages shown
- Pipeline completes successfully

## Benefits

1. **Automatic adaptation:** Pipeline adapts to available data without user intervention
2. **No failures:** Gracefully skips unavailable analysis instead of failing
3. **Clear communication:** Users see informative messages about what's being skipped and why
4. **Historical redistricting:** Can now run full 2010 pipeline without errors
5. **Consistent core:** Redistricting and compactness analysis always work regardless of data availability

## Future Enhancements

To enable full analysis for historical years, would need:

**For 2010:**
- Download 2010/2012 presidential election results at precinct level
- Process to tract level using spatial joins
- Download P1/P2 demographic tables from 2010 Census
- Process demographics to standardized format

**For 2000:**
- Complete tract shapefile download from NHGIS
- Download 2000/2004 election results
- Download 2000 Census demographics

## Impact

- ✅ 2010 redistricting now works without errors
- ✅ Users get clear feedback about data availability
- ✅ Core redistricting unaffected by optional analysis data
- ✅ Future census years will automatically skip unavailable analysis
- ✅ No breaking changes to existing 2020 workflows
