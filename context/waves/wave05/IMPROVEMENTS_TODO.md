# Improvements TODO

## User Experience

### Progress Display in run_all_states.py
**Date**: 2026-01-09
**Priority**: Medium
**Requested by**: User

Add real-time progress display showing:
- Current state being processed (e.g., "Processing Massachusetts (14/44)")
- States completed so far with checkmarks
- Estimated time remaining based on completed states
- Live update in place (not scrolling output)

**Implementation idea**:
```python
from tqdm import tqdm
# or use rich.progress for fancier display

# Show progress bar
with tqdm(total=len(states_to_process), desc="States") as pbar:
    for state in states_to_process:
        pbar.set_description(f"Processing {state_name}")
        process_state(...)
        pbar.update(1)
```

**Benefits**:
- User can see progress at a glance
- Better estimate of completion time
- More professional/polished output
- Easier to spot if something is stuck

## Other Potential Improvements

### Code Cleanup

#### Remove pymetis, Use gpmetis Only
**Date**: 2026-01-09
**Priority**: Medium

Currently the code tries pymetis first, then falls back to gpmetis.exe. Since we always use gpmetis.exe:
- Remove pymetis import attempts
- Remove fallback logic in `metis_wrapper.py`
- Simplify to only use `metis_executable.py`
- Cleaner code, fewer dependencies

**Files to modify**:
- `src/apportionment/partition/metis_wrapper.py` - Remove pymetis try/except
- Remove pymetis from any requirements

#### Reduce Console Printing
**Date**: 2026-01-09
**Priority**: Low

Focus output on progress bar and key information only:
- Remove verbose "pymetis not available" messages
- Remove "Trying fallback methods..." messages
- Keep only essential state start/complete messages
- Let progress bar be the primary status indicator
- Still log everything to file for debugging

**Benefits**:
- Cleaner output
- Easier to see progress
- Less noise in terminal
- Professional appearance

### Visualization Improvements

#### Fix \n in Map Titles
**Date**: 2026-01-09
**Priority**: High

**Problem**: Intermediate round maps show literal `\n` in titles instead of actual newlines

**Example Current**:
```
Title: "California\nRound 1: 2 regions"
```

**Should be**:
```
California
Round 1: 2 regions
```

**Fix**: Update `visualize_all_rounds.py` to properly render newlines in matplotlib titles

#### Show District Counts in Round Labels
**Date**: 2026-01-09
**Priority**: Medium

**Problem**: Round maps show region numbers but not how many districts each region will become

**Current**: "1" and "2" when dividing California 52 districts into 26 and 26

**Desired**: "1 (26)" and "2 (26)" to show each region becomes 26 districts

**Implementation**:
```python
# In visualize_all_rounds.py, when labeling regions:
# Calculate districts per region based on target split
for region_id in unique_regions:
    num_districts = calculate_districts_for_region(region_id, round_num, total_districts)
    label = f"{region_id} ({num_districts})"
```

**Benefits**:
- See at a glance how many districts each region will become
- Better understand the bisection hierarchy
- More informative visualizations

**Example for California (52 districts)**:
- Round 1: "1 (26)" and "2 (26)"
- Round 2: "1 (13)", "2 (13)", "3 (13)", "4 (13)"
- Round 3: Eight regions with 6-7 districts each
- etc.

### Compactness Metrics
- Run Polsby-Popper calculation automatically after each state completes
- Add to district_summary.csv by default
- Show compactness scores in final summary

### Parallel Processing
- Process multiple small states in parallel
- Could significantly reduce total runtime
- Need to be careful with memory usage

### Resume Capability
- Save progress after each state completes
- Allow resuming if interrupted
- Skip already-completed states automatically (✅ Already implemented with skip_existing=True)

### Error Recovery
- If a state fails, continue with remaining states
- Save failed states list for retry
- Don't abort entire run on single failure
