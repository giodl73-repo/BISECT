---
uuid: b93cd050-51c5-e296-41ca-0de0fc72c492
slug: compactness-integration
name: Compactness Integration
wave_uuid: 79fae8
created: '2026-01-10'
status: COMPLETED
---

# E1: Integrate Compactness into Main Pipeline

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

### Current State
- Compactness calculation exists as standalone script: `scripts/pipeline/calculate_compactness_metrics.py`
- Computes Polsby-Popper and Reock scores
- Must be run manually after redistricting

### Goal
- Automatically calculate compactness metrics as part of the main pipeline
- Add compactness scores to `district_summary.csv` during the Summary stage
- No separate manual step required

### Implementation Plan

#### Files to Modify
1. **`scripts/pipeline/run_complete_redistricting.py`**
   - No changes needed (orchestrator calls run_all_states)

2. **`scripts/pipeline/create_final_district_summary.py`**
   - Import compactness calculation functions from `calculate_compactness_metrics.py`
   - Add compactness score calculation after district geometry creation
   - Add `polsby_popper` and `reock` columns to district_summary.csv

#### Changes Required

**create_final_district_summary.py:**
```python
# Add imports
from calculate_compactness_metrics import polsby_popper_score, reock_score

# In create_summary function, after creating district geometries:
# Calculate compactness metrics
districts_gdf['polsby_popper'] = districts_gdf.geometry.apply(polsby_popper_score)
districts_gdf['reock'] = districts_gdf.geometry.apply(reock_score)

# Include in output CSV
summary_df['polsby_popper'] = districts_gdf['polsby_popper']
summary_df['reock'] = districts_gdf['reock']
```

#### Output Changes
- `district_summary.csv` gains two new columns:
  - `polsby_popper` (float 0-1)
  - `reock` (float 0-1)

#### Benefits
- Compactness automatically calculated for all 50 states
- No manual post-processing required
- Data immediately available for dashboard and analysis

**Completion Date:** January 10, 2026
**Implementation:** Compactness metrics integrated into `create_final_district_summary.py`. All district_summary.csv files now include polsby_popper and reock columns. Compactness visualization pipeline added as post-processing steps.
