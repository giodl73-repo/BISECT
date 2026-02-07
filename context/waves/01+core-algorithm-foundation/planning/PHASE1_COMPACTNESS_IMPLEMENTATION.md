# Phase 1 Compactness Improvements - Implementation Summary

**Date**: January 9, 2026
**Status**: Ready for testing

## Changes Made

### 1. Modified `metis_wrapper.py`

**File**: `src/apportionment/partition/metis_wrapper.py`

**Changes**:
- Added `niter` parameter to `partition_graph()` function (default: 20, up from METIS default of 10)
- Added `ufactor` parameter support for load imbalance tolerance
- Updated `_partition_with_pymetis()` to pass options to pymetis via `pymetis.Options()`
- Set `options.niter = 20` for increased refinement iterations
- Implemented ufactor conversion to METIS ubvec format

**Code snippet**:
```python
def partition_graph(
    adjacency: List[List[int]],
    vertex_weights: np.ndarray,
    nparts: int = 2,
    target_weights: Optional[List[float]] = None,
    recursive: bool = True,
    ufactor: Optional[float] = None,
    niter: int = 20  # NEW: increased from default 10
) -> np.ndarray:
```

**Expected Impact**:
- +5-10% compactness improvement
- ~2x slower processing (still reasonable for full US run)
- Better optimization at each split level

### 2. Created `run_compactness_test.py`

**File**: `scripts/run_compactness_test.py`

**Purpose**: Test Phase 1 improvements on subset of states before full 50-state run

**Features**:
- Outputs to `outputs/compactness-testing/` directory
- Tests on CA (52), TX (38), FL (28) first
- Creates comprehensive README.md in test directory
- Provides comparison guidance for baseline vs. test results

**Usage**:
```bash
python scripts/run_compactness_test.py
```

### 3. County-Aware Water Adjacency (Already Implemented)

**File**: `scripts/build_tract_adjacency.py`

**Feature**: Water adjacency connections prefer same-county tracts using GEOID county codes

**Status**: Already implemented in previous session

## Testing Plan

### Phase 1a: Initial Testing (3 states)
1. California (52 districts) - largest, most complex
2. Texas (38 districts) - large, diverse geography
3. Florida (28 districts) - complex coastline, islands

### Phase 1b: Validation
Compare outputs in these files:
```bash
# District summaries
outputs/us_2020_redistricting/california/district_summary.csv
outputs/compactness-testing/california/district_summary.csv

# Visual comparison
outputs/us_2020_redistricting/california/*.png
outputs/compactness-testing/california/*.png
```

**Metrics to check**:
- Population deviation (should be similar or better)
- Visual compactness (should look more compact)
- County splits (should be fewer due to county-aware adjacency)
- Processing time (expect ~2x slower)

### Phase 2: Full 50-State Run
If Phase 1a results look good:
1. Expand test to all multi-district states (44 states)
2. Create full comparison report
3. Replace baseline with new improved results

## Technical Details

### METIS niter Parameter
- **Default**: 10 iterations
- **New value**: 20 iterations
- **What it does**: Number of Kernighan-Lin refinement passes after initial partition
- **Trade-off**: Better compactness at cost of ~2x processing time

### pymetis.Options()
The pymetis library provides an Options object for METIS configuration:
```python
options = pymetis.Options()
options.niter = 20  # Refinement iterations
options.ufactor = ubvec_value  # Load imbalance tolerance
```

### ufactor Conversion
METIS expects imbalance tolerance as an integer:
- ufactor 1.001 (0.1% tolerance) → ubvec = 1
- ufactor 1.005 (0.5% tolerance) → ubvec = 5
- Conversion: `ubvec = max(1, int((ufactor - 1.0) * 1000))`

## Next Steps

1. **Run initial test**: `python scripts/run_compactness_test.py`
2. **Visual inspection**: Compare district maps side-by-side
3. **Quantitative analysis**: Compare deviation statistics in CSV files
4. **Decision**: If improvements are significant, proceed to all 50 states
5. **Future work**: Implement Phase 2 improvements (k-way partitioning, geographic distance weighting)

## Expected Outcomes

### Baseline (us_2020_redistricting)
- METIS niter = 10 (default)
- Water adjacency: nearest neighbor (no county preference)
- Results: Generally good but room for improvement

### Phase 1 Test (compactness-testing)
- METIS niter = 20 (+100% refinement)
- Water adjacency: county-aware
- Expected: +10-15% compactness, better county respect, ~2x slower

## Notes

- All changes are backward compatible
- Default parameters in metis_wrapper.py ensure new behavior is applied automatically
- No changes needed to existing scripts - they will use niter=20 by default
- Original baseline results preserved in `outputs/us_2020_redistricting/`

## References

- See `COMPACTNESS_IMPROVEMENTS.md` for full strategy document
- METIS manual: http://glaros.dtc.umn.edu/gkhome/metis/metis/overview
- Polsby-Popper compactness: 4π × area / perimeter²
