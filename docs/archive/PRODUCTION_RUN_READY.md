# Production Run Preparation - Complete

**Date**: January 9, 2026
**Status**: ✅ Ready for 50-State Production Run

## Summary

All infrastructure is now ready for a production 50-state congressional redistricting run with improved compactness and optimized directory structure.

## What We've Accomplished

### 1. ✅ Fixed niter Parameter Passing

**Problem**: The first "compactness test" wasn't actually testing anything - it used METIS default niter=10

**Solution**:
- Modified `metis_executable.py` to accept `niter` parameter (lines 63-64, 127)
- Modified `metis_wrapper.py` to pass `niter` to gpmetis.exe (line 71)
- Now properly passes `-niter={value}` to gpmetis executable

**Files Modified**:
- `src/apportionment/partition/metis_executable.py`
- `src/apportionment/partition/metis_wrapper.py`

### 2. ✅ Implemented niter Comparison Test

**Current Testing**: Running California with niter=20, 50, 100 to determine optimal value

**Test Script**: `scripts/test_niter_values.py`

**Output Directory**: `outputs/niter-comparison/`
- `niter20/california/` - Baseline (2x default)
- `niter50/california/` - Medium refinement
- `niter100/california/` - High refinement

**Metrics Being Measured**:
- Population balance (max deviation)
- Processing time per niter value
- Visual compactness inspection

### 3. ✅ Updated Directory Structure for Production

**New Structure** (as requested):
```
outputs/us_2020_redistricting/
  states/                          ← All individual states here
    california/
      california_52_districts.png
      district_summary.csv
      district_cities.csv
      ...
    texas/
    florida/
    ...
  us_all_districts.csv             ← US-level aggregates
  us_district_summary.csv
  US_National_Map_435_Districts.png
  US_National_Map_435_Districts_With_Cities.png
  US_2020_Redistricting_Report.md
```

**Updated Scripts**:
- ✅ `scripts/run_all_states.py` - Uses `states/` subdirectory
- ✅ `scripts/create_us_aggregate.py` - Looks in `states/`
- ✅ `scripts/create_us_national_map.py` - Looks in `states/`
- ✅ `scripts/create_rounds_hierarchy.py` - Looks in `states/`

### 4. ✅ Created Polsby-Popper Compactness Calculator

**New Script**: `scripts/calculate_compactness_metrics.py`

**Metrics Calculated**:
- **Polsby-Popper**: 4π × area / perimeter² (primary metric)
- **Reock**: area / minimum_bounding_circle
- **Convex Hull Ratio**: area / convex_hull_area

**Usage**: Run after redistricting to add compactness columns to district_summary.csv

### 5. ✅ Baseline Test Results (niter=10)

Completed test on CA, TX, FL with default niter=10:

| State | Districts | Max Deviation | Status |
|-------|-----------|---------------|--------|
| California | 52 | 0.32% | Excellent |
| Texas | 38 | 0.43% | Excellent |
| Florida | 28 | 0.22% | Excellent |

**Key Insight**: Florida used niter=20 mid-run (when fix kicked in) and achieved best results (0.22%)!

## Next Steps

### Step 1: Wait for niter Comparison Results (~1-2 hours)

The test is currently running to determine optimal niter value.

**Expected Results**:
- niter=20: ~0.3% deviation, reasonable speed
- niter=50: Better compactness, ~2.5x slower
- niter=100: Best compactness, ~5x slower

**Decision Criteria**:
- If niter=50 doesn't significantly improve over niter=20 → Use niter=20
- If niter=50 shows clear improvement → Consider niter=50
- niter=100 likely has diminishing returns

### Step 2: Choose Optimal niter

Based on comparison results, select the niter value that balances:
- Compactness quality
- Population balance
- Processing time (full 50-state run)

**Time Estimates for Full Run**:
- niter=20: ~8-12 hours
- niter=50: ~20-30 hours
- niter=100: ~40-60 hours

### Step 3: Run Production 50-State Redistricting

```bash
# Create timestamped output directory
python scripts/run_all_states.py

# This will create:
outputs/us_2020_redistricting/
  states/
    (all 44 multi-district states)
  (aggregate files)
```

**What Gets Created**:
- 44 multi-district states in `states/` subdirectory
- 6 single-district states (handled separately)
- US aggregate CSV files
- National maps (with and without cities)
- Comprehensive markdown report

### Step 4: Post-Processing (Optional)

After the full run:

```bash
# Calculate Polsby-Popper scores for all states
for state_dir in outputs/us_2020_redistricting/states/*/; do
    python scripts/calculate_compactness_metrics.py "$state_dir"
done

# This adds compactness columns to each state's district_summary.csv
```

## Technical Details

### County-Aware Water Adjacency

Already implemented in `build_tract_adjacency.py`:
- Island tracts prefer same-county connections
- Uses GEOID substring (chars 2-4) for county matching
- Tested on New York: 93 same-county, 0 cross-county

### Dynamic ufactor Strategy

Population balance tolerance adapts by tree depth:
- **Depth 1** (2 regions): ufactor=1.001 (0.1% tolerance) - Very tight
- **Depth 2** (4 regions): ufactor=1.002 (0.2% tolerance)
- **Depth 3** (8 regions): ufactor=1.003 (0.3% tolerance)
- **Depth 4+** (16+ regions): ufactor=1.005 (0.5% tolerance)

This ensures tight balance at early splits while allowing flexibility at later depths.

## Files Ready for Production

### Core Scripts
- ✅ `scripts/run_all_states.py` - Main 50-state orchestrator
- ✅ `scripts/run_state_redistricting.py` - Individual state processing
- ✅ `scripts/build_tract_adjacency.py` - County-aware adjacency
- ✅ `scripts/add_cities_to_districts.py` - City labeling
- ✅ `scripts/create_final_district_summary.py` - Summary statistics
- ✅ `scripts/create_individual_district_maps.py` - District visualizations
- ✅ `scripts/visualize_all_rounds.py` - Round-by-round viz

### Aggregate Scripts
- ✅ `scripts/create_us_aggregate.py` - Combine all states
- ✅ `scripts/create_us_national_map.py` - National visualizations
- ✅ `scripts/create_single_district_states.py` - Handle 1-district states
- ✅ `scripts/create_rounds_hierarchy.py` - Extract bisection tree data
- ✅ `scripts/fill_missing_cities.py` - Fill gaps in city data

### Analysis Scripts
- ✅ `scripts/calculate_compactness_metrics.py` - Polsby-Popper scores

### Core Algorithm
- ✅ `src/apportionment/partition/recursive_bisection.py` - Main algorithm
- ✅ `src/apportionment/partition/metis_wrapper.py` - METIS interface (with niter)
- ✅ `src/apportionment/partition/metis_executable.py` - gpmetis.exe wrapper (with niter)

## Performance Expectations

### With niter=20 (Recommended Baseline)

**Per-State Processing**:
- Small states (2-4 districts): 1-3 minutes
- Medium states (8-12 districts): 5-10 minutes
- Large states (26-52 districts): 15-30 minutes

**Full 50-State Run**:
- Redistricting: ~6-8 hours
- Post-processing (cities, maps, viz): ~2-4 hours
- **Total**: ~8-12 hours

**Output Size**:
- Per state: ~50-200 MB (depends on district count)
- Total: ~10-15 GB

## Quality Metrics

### Population Balance

**Current Results** (baseline niter=10):
- CA: 0.32% max deviation ✅
- TX: 0.43% max deviation ✅
- FL: 0.22% max deviation ✅ (used niter=20)

**Expected with niter=20+**:
- All states: < 0.5% max deviation
- Most states: < 0.3% max deviation
- Best case: < 0.2% max deviation

### Compactness

**Expected Polsby-Popper Scores**:
- Baseline (algorithmic): 0.20-0.35
- With niter optimization: 0.25-0.40
- Compare to gerrymandered districts: 0.05-0.15
- Compare to Iowa-style (very compact): 0.30-0.45

## Known Limitations

1. **Single-district states**: Handled separately (no redistricting needed)
2. **Island connectivity**: Requires water-based adjacency (already implemented)
3. **Processing time**: Increases linearly with niter value
4. **Memory usage**: Large states (CA, TX) require ~4-8 GB RAM during processing

## Recommendations

### For Immediate Use (Development/Testing)
- **niter=20**: Good balance of quality and speed
- **Full run time**: ~8-12 hours
- **Quality**: Expected < 0.3% deviation

### For Publication Quality (If Time Permits)
- **niter=50**: Better compactness, reasonable time
- **Full run time**: ~20-30 hours
- **Quality**: Expected < 0.25% deviation, better Polsby-Popper scores

### Not Recommended Unless Necessary
- **niter=100**: Diminishing returns, very slow
- **Full run time**: ~40-60 hours
- **Quality**: Marginal improvement over niter=50

## Questions?

Check these documents:
- `COMPACTNESS_IMPROVEMENTS.md` - Detailed improvement strategies
- `PHASE1_COMPACTNESS_IMPLEMENTATION.md` - What was implemented
- `README.md` - Project overview
- `claude_session_notes.md` - Development history

## Ready to Launch! 🚀

Once the niter comparison completes, you'll have data to make the final decision on niter value, then you're ready to kick off the full 50-state production run!
