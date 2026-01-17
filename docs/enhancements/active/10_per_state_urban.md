# Enhancement 10: Per-State Urban Area Processing

**Status**: 📋 PLANNED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026

### Current State
- Urban area maps are generated in post-processing batch stage
- All urban areas processed sequentially after all states complete
- Each urban area has a known "primary state" (the state containing the largest portion)
- Urban processing happens as a single bottleneck after parallel state processing

### Goal
- Move urban area map generation into per-state pipeline (parallel execution)
- Generate urban area maps for metro areas whose primary state matches current state
- Post-processing only generates national urban overview (aggregation)
- Follows established scope-based analysis pattern

### Implementation Plan

#### Files to Modify

1. **`scripts/urban/visualize_urban_areas.py`**
   - Add `--scope state|national` parameter (following scope-based pattern)
   - Add `--state` parameter for state scope
   - **State scope**: Load only urban areas where primary_state matches current state
   - **State scope**: Generate individual urban area maps for matching metros
   - **National scope**: Aggregate all per-state results into national overview map
   - Follow pattern established in political/demographic analysis

2. **`scripts/pipeline/process_single_state.py`**
   - Add urban area visualization step (step 9 of 10)
   - Call: `visualize_urban_areas.py --scope state --state {state_code} --state-dir {state_dir}`
   - Runs in parallel with all other per-state processing

3. **`scripts/pipeline/run_complete_redistricting.py`**
   - Update post-processing to call urban visualization with `--scope national`
   - Remove old batch urban processing stage
   - Ensure conditional on `not args.skip-analysis`

4. **`scripts/urban/config_urban.py`** (if exists)
   - Ensure each metro area has `primary_state` field defined
   - Example: `'new_york_newark': {'primary_state': 'NY', ...}`

#### Technical Details

**State Scope Processing**:
```python
# Load metro config
from scripts.urban.config_urban import METRO_AREAS

# Filter to current state's metros
state_metros = {
    metro_id: config
    for metro_id, config in METRO_AREAS.items()
    if config.get('primary_state') == state_code
}

# Generate map for each metro in this state
for metro_id, config in state_metros.items():
    # Load district assignments for this state
    # Generate urban area map
    # Save to state_dir/maps/urban/{metro_id}.png
```

**National Scope Processing**:
```python
# Aggregate all per-state urban results
# Create national overview map showing all metros
# Save to output_dir/us_urban_overview.png
```

#### Expected Changes

**Before**:
```
Post-processing:
  - [Sequential] Process 53 urban areas (10-30 minutes)
  - [Sequential] Create national urban map
```

**After**:
```
Per-state (parallel):
  - CA: Process 5 urban areas (LA, SF, SD, SAC, SJ)
  - NY: Process 1 urban area (NYC metro)
  - TX: Process 4 urban areas (Houston, Dallas, San Antonio, Austin)
  - ... (all states in parallel)

Post-processing:
  - [Fast] Create national urban overview (aggregation only)
```

### Benefits

1. **Performance**: Urban area processing happens in parallel with state processing
2. **Consistency**: Follows established scope-based pattern (like political/demographic)
3. **Incremental**: Urban maps available immediately after each state completes
4. **Maintainable**: Single script with two scopes, not separate scripts
5. **Scalability**: No sequential bottleneck for urban processing

### Implementation Complexity

**Medium** (3-5 hours)
- Requires refactoring existing urban visualization script
- Need to define primary_state for all metro areas
- Must follow established scope-based pattern
- Testing required to ensure correct metro-to-state assignment

### Success Criteria

- [ ] Each metro area has defined primary_state
- [ ] Urban maps generated during per-state processing for matching metros
- [ ] National urban overview map successfully aggregates all results
- [ ] No sequential bottleneck for urban processing
- [ ] Output quality matches current batch-mode results
- [ ] Code follows scope-based pattern from Enhancement 9

---

**Date**: January 12, 2026
**Status**: Enhancements 1-6, 9 complete; 7-8, 10 planned
**Order**: ✅ 1 → ✅ 2 → ✅ 3 → ✅ 4 → ✅ 5 → ✅ 6 → 📋 7 → 📋 8 → ✅ 9 → 📋 10
