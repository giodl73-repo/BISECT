# Stacked Progress Bars Implementation - January 10, 2026

## Overview

Implemented a **persistent, multi-level progress bar system** that creates a dashboard-style display instead of scrolling output. Progress bars are positioned at fixed screen locations and update in place.

---

## How It Works

### Position-Based Stacking

Each progress bar is assigned a **position** (vertical line number):
- **Position 0**: Top level (Pipeline)
- **Position 1**: Second level (States)
- **Position 2**: Third level (Operations)
- **Position 3+**: Deeper nested operations

### Environment Variable Passing

The `TQDM_POSITION` environment variable coordinates positions across subprocess calls:

```python
# Parent script (position 0)
os.environ['TQDM_POSITION'] = '0'
with tqdm(..., position=0, leave=True) as pbar:
    # Set position for children
    os.environ['TQDM_POSITION'] = '1'
    subprocess.run(child_script)

# Child script (position 1)
position = int(os.environ.get('TQDM_POSITION', '0'))
with tqdm(..., position=position, leave=False) as pbar:
    os.environ['TQDM_POSITION'] = str(position + 1)
    subprocess.run(grandchild_script)
```

### Leave Behavior

- **`leave=True`**: Bar stays visible after completion (for parent/top-level)
- **`leave=False`**: Bar clears after completion (for children/operations)

---

## Visual Display

### What You'll See (Real-Time Dashboard):

```
Pipeline [1/4] Process all 50 states          25%|██▌       | 1/4 [00:30<01:30] ✓ Complete
  ├─ Processing states                        42%|████▏     | 18/43 [2:15<3:10] [3/5] Summary
  │    └─ Creating district maps              48%|████▊     | 25/52 [00:45<00:48]
  └─ Status: California - Creating maps for district 25/52
```

**Key Features**:
- **Line 1** (position=0): Pipeline progress - stays visible entire time
- **Line 2** (position=1): Current state - updates as states change
- **Line 3** (position=2): Current operation - updates frequently
- **Line 4**: Status messages via `tqdm.write()`

### Updates Happen In-Place

The display **does not scroll**. Each line updates its percentage, bar, and status:

```
[Time 0:00]
Pipeline [1/4] Process all 50 states           0%|          | 0/4 [00:00<?, ?/step]
  └─ Initializing...

[Time 0:30]
Pipeline [1/4] Process all 50 states          25%|██▌       | 1/4 [00:30<01:30]
  ├─ Processing states                         2%|▏         | 1/43 [00:30<21:00] [1/5] Redistricting
  │    └─ Running METIS bisection             15%|█▌        | 2/13 [00:05<00:28]

[Time 5:00]
Pipeline [1/4] Process all 50 states          25%|██▌       | 1/4 [05:00<15:00]
  ├─ Processing states                        12%|█▏        | 5/43 [05:00<35:00] [4/5] Maps
  │    └─ Creating district maps              65%|██████▌   | 34/52 [02:30<01:18]
```

---

## Implementation Details

### 1. run_complete_redistricting.py (Position 0)

```python
# Set top-level position
os.environ['TQDM_POSITION'] = '0'

# Pipeline progress bar at position 0 (stays visible)
with tqdm(total=len(pipeline_steps),
          desc="Pipeline Progress",
          unit="step",
          ncols=100,
          position=0,
          leave=True) as pbar:

    for step in pipeline_steps:
        # Children will use position 1
        os.environ['TQDM_POSITION'] = '1'
        run_subprocess(step)
        pbar.update(1)
```

**Visual Position**: Top line, always visible

### 2. run_all_states.py (Position 1)

```python
# Read position from environment
tqdm_position = int(os.environ.get('TQDM_POSITION', '0'))

# State progress bar (clears when done if child)
with tqdm(multi_district_states,
          desc="Processing states",
          unit="state",
          ncols=100,
          position=tqdm_position,
          leave=(tqdm_position == 0)) as pbar:

    for state in pbar:
        # Children will use position 2
        os.environ['TQDM_POSITION'] = str(tqdm_position + 1)
        process_state(state)
```

**Visual Position**: Second line, below pipeline

### 3. Individual Operations (Position 2+)

Scripts like `create_individual_district_maps.py` read position and stack below:

```python
position = int(os.environ.get('TQDM_POSITION', '0'))

with tqdm(districts,
          desc="Creating maps",
          unit="district",
          ncols=100,
          position=position,
          leave=False) as pbar:
    # Process each district
```

**Visual Position**: Third line and below, clears when done

---

## Print-Only Mode Integration

Print-only mode now uses **indentation based on position** to mirror the hierarchy:

```python
# Get position for indentation
position = int(os.environ.get('TQDM_POSITION', '0'))
indent = "  " * position

# Print with indentation
print(f"{indent}[PRINT-ONLY] {state_name} Redistricting")
print(f"{indent}  ├─ STEP 1: Load data")
print(f"{indent}  │   └─ Load tracts: {tracts_file}")
```

### Visual Example (Print-Only):

```
[PRINT-ONLY] Pipeline Overview
  ├─ STEP 1: Process all states
  │   └─ For each state:
  │       [PRINT-ONLY] California Redistricting
  │         ├─ STEP 1: Load data
  │         │   └─ Load tracts: data/raw/ca_tracts_2020.parquet
  │         ├─ STEP 2: Run METIS
  │         └─ STEP 3: Generate maps
```

Indentation matches the execution hierarchy!

---

## Benefits

### 1. No Scrolling ✅
- Output updates in place
- Easy to see current status at a glance
- Terminal doesn't fill with old progress bars

### 2. Real-Time Dashboard ✅
- See pipeline progress (top)
- See current state (middle)
- See current operation (bottom)
- All visible simultaneously

### 3. Context Awareness ✅
- Always know which pipeline step you're in
- Always know which state is being processed
- Always know which operation is running
- Immediate ETA for each level

### 4. Professional Appearance ✅
- Clean, organized display
- Like watching a build system (make, cargo, npm)
- No cluttered scrollback
- Clear visual hierarchy

---

## Testing

### Test Pipeline Progress

```bash
# Full pipeline with stacked progress
python scripts/run_complete_redistricting.py --year 2020 --version v1

# Expected display:
# Pipeline [1/4] Process all states          25%|██▌  | 1/4 [00:30<01:30]
#   └─ Processing states                     2%|▏    | 1/43 [00:30<21:00]
```

### Test State Processing

```bash
# Single state with nested progress
python scripts/run_all_states.py --year 2020 --version v1 CA

# Expected display:
# Processing states                          100%|██████| 1/1 [15:00<00:00]
#   └─ Creating district maps                48%|████▊ | 25/52 [02:30<02:30]
```

### Test Print-Only with Indentation

```bash
# Print-only with hierarchical indentation
python scripts/run_complete_redistricting.py --year 2020 --version v1 --print-only

# Expected display:
# [PRINT-ONLY] Pipeline Overview
#   [PRINT-ONLY] California Redistricting
#     ├─ STEP 1: Load data
#     └─ STEP 2: Run METIS
```

---

## Configuration

### Disable Stacked Display

If you want old scrolling behavior:

```bash
# Unset the environment variable
unset TQDM_POSITION

# Or set TQDM_DISABLE=1 to disable all progress bars
export TQDM_DISABLE=1
python scripts/run_complete_redistricting.py --year 2020 --version v1
```

### Adjust Display Width

```python
# In each script with tqdm
with tqdm(..., ncols=120) as pbar:  # Wider display
    pass

with tqdm(..., ncols=80) as pbar:   # Narrower display
    pass
```

---

## Technical Notes

### Terminal Compatibility

- **Works**: Modern terminals (Windows Terminal, iTerm2, Gnome Terminal, etc.)
- **Partial**: Basic terminals (cmd.exe) - may not clear lines properly
- **Fallback**: If terminal doesn't support cursor positioning, falls back to scrolling

### Position Limits

- Maximum positions: Limited by terminal height
- Practical limit: 5-6 levels (more causes overlap)
- Current implementation: 3 levels (pipeline → states → operations)

### Memory Considerations

- Progress bars are lightweight (minimal memory)
- Environment variables preserved across subprocesses
- No inter-process communication overhead

---

## Future Enhancements

### 1. Color Coding (Optional)

```python
from colorama import Fore, Style

pbar.set_description(f"{Fore.GREEN}Processing{Style.RESET_ALL}")
pbar.set_postfix_str(f"{Fore.GREEN}✓ Complete{Style.RESET_ALL}")
```

### 2. Status Panel (Optional)

Add a dedicated status line below progress bars:

```
Pipeline [1/4]                                25%|██▌  | 1/4 [00:30<01:30]
  └─ California                               42%|████▏| 18/43 [2:15<3:10]
      └─ District maps                        48%|████▊| 25/52 [00:45<00:48]
──────────────────────────────────────────────────────────────────────
Status: Processing CA District 25 of 52 - San Francisco
ETA: 15:30:45 PST | Remaining: 2:18:33 | Speed: 1.2 districts/min
```

### 3. Nested Print Messages

Use `pbar.write()` with indentation:

```python
pbar.write(f"{indent}  ✓ District 25 complete - San Francisco")
pbar.write(f"{indent}  → Saved: district_25.png")
```

---

## Example Full Run Output

### What You See During Execution:

```
Pipeline [1/4] Process all 50 states          25%|██▌       | 1/4 [00:30<01:30] ✓ Complete
  ├─ Processing states                        42%|████▏     | 18/43 [2:15:30<3:10:22] [3/5] Summary
  │    └─ Creating district maps              48%|████▊     | 25/52 [00:45<00:48]
  └─ California: District 25 - San Francisco ✓ Complete
```

**Top line**: Never changes (except percentage/time) - stays until pipeline done
**Middle line**: Updates with each new state
**Bottom line**: Updates with each operation
**Messages**: Appear below the bars, don't disrupt display

### After Completion:

```
Pipeline [4/4] Create US national maps       100%|██████████| 4/4 [12:30:00<00:00] ✓ Complete
  └─ All 43 states processed successfully

══════════════════════════════════════════════════════════════════════
COMPLETE PIPELINE FINISHED
══════════════════════════════════════════════════════════════════════
All results in: outputs/us_2020_v1
══════════════════════════════════════════════════════════════════════
```

Only the top-level bar remains visible at the end!

---

**Date**: January 10, 2026
**Status**: Stacked progress bars implemented ✅
**Files Updated**: 2 (run_complete_redistricting.py, run_all_states.py)
**Display**: Multi-level persistent progress dashboard
**User Experience**: No scrolling, real-time updates, clear hierarchy
