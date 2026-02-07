# 2010 Census Data Re-Download Instructions

## Problem Summary

Validation found that **only 12 out of 50 states** have complete 2010 census data. The rest are incomplete due to Census API rate limiting during the original download.

### Validation Results
- **Overall: 84.88% complete** (missing 46.6 million people)
- **38 states need re-downloading** (< 95% complete)
- **Most problematic**: MD (40%), WY (58%), IA (59%), MI (59%), WI (63%)

## Solution

Re-download all 38 incomplete states with improved rate limiting:
- 3.0s delay between API requests
- Exponential backoff retry logic
- Progress saving (can resume if interrupted)
- Data validation for each county

## Quick Start

### Option 1: Run All 38 States (Recommended)

Simply run the batch file:
```cmd
run_2010_redownload.bat
```

**Time estimate:** 36-48 hours

This will:
1. Download all 38 incomplete states sequentially
2. Automatically retry failed counties up to 5 times
3. Validate results when complete

### Option 2: Run in Batches

If you prefer to run in smaller batches (safer):

**Batch 1 - Worst states (7 states, ~8-10 hours):**
```cmd
python scripts\redownload_2010_states.py --states MD WY IA MI WI KS KY
```

**Batch 2 - Major issues part 1 (10 states, ~14-18 hours):**
```cmd
python scripts\redownload_2010_states.py --states MA MO MS MT NJ NY TX VA IN LA
```

**Batch 3 - Major issues part 2 (10 states, ~12-16 hours):**
```cmd
python scripts\redownload_2010_states.py --states ME MN NC ND NE NH NM OH OK OR
```

**Batch 4 - Major issues part 3 (10 states, ~10-14 hours):**
```cmd
python scripts\redownload_2010_states.py --states PA RI SC SD TN UT VA VT WA WV
```

**Batch 5 - Incomplete + Minor (2 states, ~2-3 hours):**
```cmd
python scripts\redownload_2010_states.py --states NV IL
```

After each batch, validate:
```cmd
python scripts\validate_2010_census_data.py
```

### Option 3: Single State

Download just one state (e.g., Michigan):
```cmd
python scripts\download_tracts_improved.py --state MI --year 2010
```

## Monitoring Progress

The script shows:
- Progress bar for each county within a state
- State completion (e.g., "3/10 counties")
- Estimated time remaining
- Retry attempts when hitting rate limits

## Resuming After Interruption

If you need to stop (Ctrl+C):
- Progress is saved automatically every 5 counties
- Re-run the same command to resume
- Already-downloaded counties will be skipped

To force re-download (ignore saved progress):
```cmd
python scripts\redownload_2010_states.py --no-resume
```

## Adjusting Settings

If still hitting rate limits, increase delays:
```cmd
python scripts\redownload_2010_states.py --delay 5.0 --delay-between-states 15.0
```

## Validation

Check results after downloading:
```cmd
python scripts\validate_2010_census_data.py
```

This will show:
- Which states are now complete
- Which states still have issues
- Overall US population coverage

## Troubleshooting

### "429 Client Error" (Rate Limiting)
- Increase `--delay` to 5.0 or higher
- Increase `--delay-between-states` to 15.0 or higher
- Run in smaller batches with longer breaks between

### Script Crashes / Connection Errors
- Check internet connection
- Re-run the same command (will resume automatically)
- Reduce number of states per batch

### Validation Still Shows Incomplete
- Check which counties failed (shown in output)
- Re-download just those states:
  ```cmd
  python scripts\redownload_2010_states.py --states MI NH
  ```

## 2020 Data Status

Your 2020 data is already validated and nearly perfect:
- **49 out of 50 states: 100% correct**
- **Ohio: 99.35% complete**
- **Overall: 99.98% complete**

No action needed for 2020 data.

## Questions?

After validation, you should see "VALIDATED: 50 of 50 states have correct population data"

If problems persist, check:
1. Census API is not down (test at https://api.census.gov/)
2. Rate limits may have changed (try longer delays)
3. Some states may have data quality issues in the Census API itself
