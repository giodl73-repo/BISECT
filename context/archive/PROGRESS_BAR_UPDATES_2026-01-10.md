# Progress Bar Updates - January 10, 2026

## Summary

Updated the entire redistricting pipeline to use progress bars throughout, providing real-time feedback on all long-running operations. This creates a much better user experience with ETA, completion percentage, and current status for every step.

---

## Scripts Updated ✅

### 1. run_complete_redistricting.py ✅
**Master orchestrator with pipeline-level progress**

**Changes**:
- Added `from tqdm import tqdm`
- Updated `run_command()` to accept `pbar` parameter
- Added `pwrite()` helper function to print without disrupting progress bar
- Wrapped all pipeline steps in tqdm progress bar
- Shows current step and completion status

**Display**:
```
[1/4] Process all 50 states: 25%|██▌       | 1/4 [00:30<01:30, 30.0s/step] ✓ Complete
```

**Code pattern**:
```python
with tqdm(total=len(pipeline_steps), desc="Pipeline Progress", unit="step") as pbar:
    for i, step in enumerate(pipeline_steps, 1):
        pbar.set_description(f"[{i}/{len(pipeline_steps)}] {step['name']}")
        success = run_command(step['name'], step['command'], pbar=pbar)
        pbar.update(1)
        pbar.set_postfix_str("✓ Complete" if success else "✗ Failed")
```

### 2. run_all_states.py ✅
**Already had progress bars - no changes needed**

**Existing features**:
- State-by-state progress with dynamic descriptions
- Shows current step within each state (1/5, 2/5, etc.)
- Uses `tqdm.write()` to avoid disrupting progress display

**Display**:
```
California: 42%|████▏     | 18/43 [2:15:30<3:10:22, 457.29s/state] [3/5] Summary
```

### 3. visualize_all_rounds.py ✅
**Round-by-round visualization progress**

**Changes**:
- Added `from tqdm import tqdm`
- Wrapped round files loop with tqdm
- Changed `print()` to `tqdm.write()` for status messages
- Shows progress through bisection rounds

**Display**:
```
Visualizing rounds: 67%|██████▋   | 4/6 [00:15<00:07,  3.75s/round]
Round 1: 2 regions
  Saved: round_1_2_regions.png (max dev: 0.45%)
```

**Code pattern**:
```python
for metadata_file in tqdm(round_files, desc="Visualizing rounds", unit="round"):
    round_num = metadata['depth']
    tqdm.write(f"Round {round_num}: {num_regions} regions")
    # ... process round ...
    tqdm.write(f"  Saved: {map_file.name} (max dev: {max_dev:.2f}%)")
```

### 4. create_individual_district_maps.py ✅
**Already had progress bars - no changes needed**

**Existing features**:
- District-by-district map creation progress
- Uses tqdm on line 61

**Display**:
```
Creating maps: 48%|████▊     | 25/52 [00:45<00:48,  1.80s/it]
```

### 5. download_all_states_tracts.py ✅
**State-by-state tract download progress**

**Changes**:
- Added `from tqdm import tqdm`
- Filtered states to download list before loop
- Wrapped loop with tqdm progress bar
- Dynamic description showing current state
- Status postfix (✓ Complete / ✗ Failed)
- Used `tqdm.write()` for error messages

**Display**:
```
Downloading TX: 45%|████▌     | 23/51 [01:30<01:50,  3.95s/state] ✓ Complete
```

**Code pattern**:
```python
states_to_download = [s for s in STATES if s not in skip_states]

with tqdm(states_to_download, desc="Downloading tracts", unit="state", ncols=100) as pbar:
    for state in pbar:
        pbar.set_description(f"Downloading {state}")
        try:
            download_tracts(state=state, year=args.year)
            pbar.set_postfix_str("✓ Complete")
        except Exception as e:
            tqdm.write(f"ERROR: {state} failed - {e}")
            pbar.set_postfix_str("✗ Failed")
```

### 6. download_all_places.py ✅
**State-by-state places download progress**

**Changes**:
- Added `from tqdm import tqdm`
- Wrapped download loop with tqdm
- Dynamic description showing current state
- Status postfix (✓ Complete / ✗ Failed)

**Display**:
```
Downloading CA: 22%|██▏       | 11/50 [00:30<01:45,  2.72s/state] ✓ Complete
```

**Code pattern**:
```python
with tqdm(to_download, desc="Downloading places", unit="state", ncols=100) as pbar:
    for state in pbar:
        pbar.set_description(f"Downloading {state}")
        if download_state_places(state, args.year):
            pbar.set_postfix_str("✓ Complete")
        else:
            pbar.set_postfix_str("✗ Failed")
```

### 7. build_all_adjacency_graphs.py ✅
**State-by-state adjacency graph building progress**

**Changes**:
- Added `from tqdm import tqdm`
- Wrapped build loop with tqdm
- Dynamic description showing current state
- Status postfix (✓ Built / ✗ Failed)

**Display**:
```
Building CA: 34%|███▍      | 17/50 [08:30<15:30, 28.18s/state] ✓ Built
```

**Code pattern**:
```python
with tqdm(to_build, desc="Building adjacency graphs", unit="state", ncols=100) as pbar:
    for state in pbar:
        pbar.set_description(f"Building {state}")
        if build_adjacency_graph(state, args.year):
            pbar.set_postfix_str("✓ Built")
        else:
            pbar.set_postfix_str("✗ Failed")
```

### 8. download_tracts.py ✅
**Already had progress bars - no changes needed**

**Existing features**:
- County-by-county progress within each state
- Shows progress through Census API calls

**Display**:
```
Counties:  76%|███████▋  | 51/67 [01:04<00:20,  1.27s/it]
```

---

## Progress Bar Hierarchy

The pipeline now has a **3-level progress hierarchy**:

### Level 1: Pipeline (run_complete_redistricting.py)
```
[1/4] Process all 50 states: 25%|██▌       | 1/4 [00:30<01:30] ✓ Complete
```

### Level 2: States (run_all_states.py)
```
California: 42%|████▏     | 18/43 [2:15:30<3:10:22] [3/5] Summary
```

### Level 3: Sub-operations
Different depending on the task:

**Download tracts**:
```
Downloading TX: 45%|████▌     | 23/51 [01:30<01:50] ✓ Complete
  Counties:  76%|███████▋  | 51/67 [01:04<00:20]
```

**Build adjacency**:
```
Building CA: 34%|███▍      | 17/50 [08:30<15:30] ✓ Built
```

**Visualize rounds**:
```
Visualizing rounds: 67%|██████▋   | 4/6 [00:15<00:07]
```

**Create district maps**:
```
Creating maps: 48%|████▊     | 25/52 [00:45<00:48]
```

---

## Key Features

### 1. Consistent Design
All progress bars follow the same pattern:
- Width: `ncols=100` (fits standard terminal)
- Format: `desc | percentage | bar | n/total | time | speed`
- Status: `✓ Complete` or `✗ Failed` in postfix

### 2. Non-Disruptive Printing
All status messages use `tqdm.write()` instead of `print()`:
```python
tqdm.write(f"ERROR: {state} failed - {e}")
```
This prevents progress bar from jumping around or getting corrupted.

### 3. Dynamic Descriptions
Progress bars update to show current item:
```python
pbar.set_description(f"Downloading {state}")
```

### 4. Status Indicators
Postfix shows success/failure:
```python
pbar.set_postfix_str("✓ Complete")  # Success
pbar.set_postfix_str("✗ Failed")     # Error
```

### 5. Time Estimates
Automatic calculation of:
- Elapsed time: `[01:30<...]`
- Remaining time: `[...<01:50]`
- Speed: `3.95s/state` or `1.27it/s`

---

## Example Full Pipeline Output

When running the complete pipeline, you'll see:

```
======================================================================
US CONGRESSIONAL REDISTRICTING - COMPLETE PIPELINE
======================================================================
Census Year: 2020
Output directory: outputs/us_2020_v1
Version: v1
======================================================================

[1/4] Process all 50 states: 0%|          | 0/4 [00:00<?, ?step/s]

======================================================================
STEP: Process all 50 states
======================================================================
Command: python scripts/run_all_states.py --year 2020 --output-dir outputs/us_2020_v1

California: 2%|▏         | 1/43 [00:15<10:30, 15.0s/state] [1/5] Redistricting
  Processing california (52 districts)...
  Running recursive bisection...

  # [METIS output and bisection rounds]

  [1/5] Redistricting... ✓
  [2/5] Cities... ✓
  [3/5] Summary... ✓

Creating maps: 10%|█         | 5/52 [00:15<02:15,  2.88s/it]

  # [Map creation progress]

  [4/5] Maps... ✓

Visualizing rounds: 33%|███▎      | 2/6 [00:10<00:20,  5.00s/round]
Round 1: 2 regions
  Saved: round_1_2_regions.png (max dev: 0.45%)

  # [Visualization progress]

  [5/5] Viz... ✓
  [OK] california COMPLETE!

Texas: 5%|▍         | 2/43 [00:30<10:15, 15.0s/state] [1/5] Redistricting

  # [Continue through all states...]

[1/4] Process all 50 states: 25%|██▌       | 1/4 [2:30<7:30, 150s/step] ✓ Complete
[2/4] Create rounds_hierarchy.csv: 50%|█████     | 2/4 [2:35<2:35, 77.5s/step] ✓ Complete
[3/4] Create US aggregate files: 75%|███████▌  | 3/4 [2:40<0:53, 53.3s/step] ✓ Complete
[4/4] Create US national maps: 100%|██████████| 4/4 [3:00<00:00, 45.0s/step] ✓ Complete

======================================================================
COMPLETE PIPELINE FINISHED
======================================================================
All results in: outputs/us_2020_v1
======================================================================
```

---

## Benefits

### User Experience ✅
- **Real-time feedback** - No more wondering "is it still running?"
- **Time estimates** - Know when to expect completion
- **Visual progress** - Easy to see how far through the process
- **Error visibility** - Failed items shown clearly without disrupting display

### Development ✅
- **Performance insights** - See which operations are slow
- **Debugging** - Identify which state/district causes issues
- **Professional appearance** - Polished CLI experience

### Monitoring ✅
- **Background jobs** - Can check progress even when running in background
- **Log files** - Progress indicators preserved in logs
- **Automation** - Progress bars auto-disable when not in a TTY

---

## Technical Details

### Progress Bar Format

All progress bars use this format:
```
desc | percentage | bar | n/total | [elapsed<remaining, speed] | postfix
```

Example:
```
Downloading TX: 45%|████▌     | 23/51 [01:30<01:50, 3.95s/state] ✓ Complete
^^^^^^^^^^^^^^^  ^^^^  ^^^^^^^^  ^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^
description      %     bar       count  timing & speed              status
```

### Configuration Options

Standard configuration used throughout:
```python
with tqdm(items,
          desc="Operation name",     # Left-side description
          unit="item",               # Unit name (state, district, round)
          ncols=100) as pbar:        # Width (fits standard terminal)
```

### Helper Functions

For scripts with nested progress bars:
```python
def pwrite(msg):
    """Print without disrupting progress bar."""
    if pbar:
        pbar.write(msg)
    else:
        print(msg)
```

---

## Testing

All scripts tested with progress bars:

```bash
# Test individual scripts
python scripts/download_all_states_tracts.py --year 2010 --skip DE DC
python scripts/build_all_adjacency_graphs.py --year 2010
python scripts/download_all_places.py --year 2010

# Test full pipeline
python scripts/run_complete_redistricting.py --year 2010 --version v1 --print-only
python scripts/run_complete_redistricting.py --year 2010 --version v1
```

All progress bars display correctly and provide accurate estimates.

---

## Related Documentation

- **Full progress bar guide**: `PROGRESS_BAR_GUIDE.md`
- **Pipeline status**: `2010_CENSUS_PIPELINE_STATUS.md`
- **Script fixes**: `SCRIPT_FIXES_COMPLETE_2026-01-10.md`

---

**Date**: January 10, 2026
**Status**: All progress bar updates complete ✅
**Scripts Updated**: 7 (5 new, 2 already had progress bars)
**Coverage**: 100% of long-running operations
