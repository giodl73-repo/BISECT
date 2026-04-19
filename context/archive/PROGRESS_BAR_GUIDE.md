# Progress Bar System Guide

## How Progress Bars Work

### The `tqdm` Library

The progress bars you see are powered by the `tqdm` library (means "progress" in Arabic). It provides:
- Real-time progress updates
- Time estimates (elapsed and remaining)
- Iteration speed
- Customizable display

**Example output**:
```
Counties:  76%|#######6  | 51/67 [01:04<00:20,  1.27s/it]
         ^^^^  ^^^^^^^^    ^^^^^  ^^^^^^^^^^^  ^^^^^^^^^
         %     bar         n/tot  time info    speed
```

- `76%` - Percentage complete
- `#######6` - Visual progress bar
- `51/67` - Current/Total items
- `[01:04<00:20, 1.27s/it]` - [Elapsed<Remaining, Speed]

## Current Usage in Scripts

### 1. download_tracts.py (Line 17, 167)
```python
from tqdm import tqdm

# Loop through counties with progress bar
for county_fips in tqdm(county_fips_list, desc="Counties"):
    # Download tract data for each county
    tract_data = conn.query(...)
```

**Display**: `Counties:  76%|#######6  | 51/67 [01:04<00:20,  1.27s/it]`

### 2. run_all_states.py (Line 12, 259-272)
```python
from tqdm import tqdm

# Process multi-district states with progress bar
with tqdm(multi_district_states, desc="Processing states", unit="state", ncols=100) as pbar:
    for state_code in pbar:
        config = STATE_CONFIG[state_code]
        state_name = config['name']

        # Update description dynamically
        pbar.set_description(f"{state_name}")

        # Process the state...
        success = process_state(state_code, us_dir, STATE_CONFIG, year=args.year,
                               skip_existing=True, print_only=args.print_only, pbar=pbar)

        # Update postfix with step info
        if pbar:
            pbar.set_postfix_str(f"[{step_num}/5] {step_labels[step_num-1]}")
```

**Display**:
```
California: 42%|████▏     | 18/43 [2:15:30<3:10:22, 457.29s/state] [3/5] Summary
```

### 3. Special Print Function for Progress Bars

When using progress bars, regular `print()` statements can disrupt the display. Use `tqdm.write()` or a wrapper:

```python
def pwrite(msg):
    if pbar:
        pbar.write(msg)  # Print without disrupting progress bar
    else:
        print(msg)       # Normal print if no progress bar

pwrite("State processing complete!")
```

## How to Add Progress Bars to Any Script

### Pattern 1: Simple Loop Progress

**Before**:
```python
for state in states:
    print(f"Processing {state}...")
    process_state(state)
```

**After**:
```python
from tqdm import tqdm

for state in tqdm(states, desc="Processing states"):
    process_state(state)
```

### Pattern 2: Progress with Dynamic Updates

**Before**:
```python
for i, state in enumerate(states):
    print(f"[{i+1}/{len(states)}] Processing {state}...")
    result = process_state(state)
    print(f"  Result: {result}")
```

**After**:
```python
from tqdm import tqdm

for state in tqdm(states, desc="Processing states", unit="state"):
    result = process_state(state)
    # No print needed - progress bar shows it!
```

### Pattern 3: Nested Progress Bars

**For nested operations** (like states → counties → tracts):

```python
from tqdm import tqdm

# Outer progress bar for states
for state in tqdm(states, desc="States", position=0):

    # Inner progress bar for counties
    counties = get_counties(state)
    for county in tqdm(counties, desc=f"{state} Counties", position=1, leave=False):
        process_county(state, county)
```

**Display**:
```
States:  40%|████      | 20/50 [00:10<00:15,  2.00it/s]
CA Counties:  76%|███████▋  | 51/67 [00:05<00:01,  8.33it/s]
```

### Pattern 4: Manual Progress Updates

**For complex operations where you control the progress**:

```python
from tqdm import tqdm
import time

total_steps = 100
with tqdm(total=total_steps, desc="Processing") as pbar:
    for i in range(total_steps):
        # Do some work
        process_data(i)

        # Update progress
        pbar.update(1)

        # Update description
        pbar.set_description(f"Processing batch {i//10}")

        # Update postfix (extra info)
        pbar.set_postfix({"batch": i//10, "status": "OK"})
```

**Display**:
```
Processing batch 5: 56%|█████▋    | 56/100 [00:30<00:24,  1.79it/s, batch=5, status=OK]
```

## Adding Progress to All Scripts

### Scripts That Should Have Progress Bars

1. **download_all_states_tracts.py** ✅ (could add)
2. **download_all_places.py** ✅ (could add)
3. **build_all_adjacency_graphs.py** ✅ (could add)
4. **run_all_states.py** ✅ (already has)
5. **run_state_redistricting.py** ✅ (could add for bisection rounds)
6. **create_individual_district_maps.py** ✅ (could add for districts)

### Example: Adding Progress to download_all_states_tracts.py

**Current code** (line 39-57):
```python
for i, state in enumerate(STATES):
    if state in skip_states:
        print(f"\n[{i+1}/{len(STATES)}] Skipping {state} (user-specified)")
        continue

    print(f"\n[{i+1}/{len(STATES)}] Downloading {state} ({args.year} Census)...")
    print("-" * 70)

    try:
        download_tracts(state=state, year=args.year)
        success_count += 1
        print(f"SUCCESS: {state} completed")
    except Exception as e:
        print(f"ERROR: {state} failed - {e}")
        failed_states.append(state)

    if i < len(STATES) - 1:
        time.sleep(2)
```

**With progress bar**:
```python
from tqdm import tqdm

states_to_download = [s for s in STATES if s not in skip_states]

with tqdm(states_to_download, desc="Downloading tracts", unit="state", ncols=100) as pbar:
    for state in pbar:
        pbar.set_description(f"Downloading {state}")

        try:
            download_tracts(state=state, year=args.year)
            success_count += 1
            pbar.set_postfix_str("✓ Complete")
        except Exception as e:
            failed_states.append(state)
            pbar.set_postfix_str(f"✗ Failed: {str(e)[:30]}")

        time.sleep(2)
```

**Display**:
```
Downloading TX: 23%|██▎       | 12/51 [00:25<01:20,  2.05s/state] ✓ Complete
```

### Example: Adding Progress to build_all_adjacency_graphs.py

**Current code** (line 95-103):
```python
for i, state in enumerate(to_build, 1):
    print(f"\n[{i}/{len(to_build)}] Processing {state}...")

    if build_adjacency_graph(state, args.year):
        successful.append(state)
        print(f"[OK] {state} completed")
    else:
        failed.append(state)
        print(f"[FAIL] {state} failed")
```

**With progress bar**:
```python
from tqdm import tqdm

with tqdm(to_build, desc="Building adjacency graphs", unit="state", ncols=100) as pbar:
    for state in pbar:
        pbar.set_description(f"Building {state}")

        if build_adjacency_graph(state, args.year):
            successful.append(state)
            pbar.set_postfix_str("✓ Built")
        else:
            failed.append(state)
            pbar.set_postfix_str("✗ Failed")
```

**Display**:
```
Building CA: 48%|████▊     | 24/50 [12:30<13:30, 31.15s/state] ✓ Built
```

## Advanced Progress Features

### 1. Progress with File Size
```python
import os
from tqdm import tqdm

file_size = os.path.getsize('large_file.csv')
with open('large_file.csv', 'rb') as f:
    with tqdm(total=file_size, unit='B', unit_scale=True, desc="Reading") as pbar:
        for chunk in iter(lambda: f.read(4096), b''):
            pbar.update(len(chunk))
```

**Display**: `Reading: 142MB/1.2GB [00:15<01:45, 10.2MB/s]`

### 2. Progress with Pandas
```python
from tqdm import tqdm
import pandas as pd

# Enable tqdm for pandas operations
tqdm.pandas(desc="Processing rows")

df['result'] = df['column'].progress_apply(lambda x: expensive_function(x))
```

**Display**: `Processing rows: 67%|██████▋   | 6789/10000 [00:15<00:07, 453.27it/s]`

### 3. Multiple Progress Bars (Parallel)
```python
from tqdm import tqdm
from concurrent.futures import ThreadPoolExecutor

def process_state(state, position):
    with tqdm(total=100, desc=f"{state}", position=position, leave=True) as pbar:
        for i in range(100):
            # Do work
            time.sleep(0.01)
            pbar.update(1)

with ThreadPoolExecutor(max_workers=4) as executor:
    futures = [executor.submit(process_state, state, i)
               for i, state in enumerate(['CA', 'TX', 'FL', 'NY'])]
    for future in futures:
        future.result()
```

**Display**:
```
CA:  78%|███████▊  | 78/100 [00:03<00:01, 25.33it/s]
TX:  54%|█████▍    | 54/100 [00:02<00:01, 23.45it/s]
FL:  91%|█████████▏| 91/100 [00:04<00:00, 22.67it/s]
NY:  32%|███▏      | 32/100 [00:01<00:02, 21.89it/s]
```

## Customization Options

### Progress Bar Appearance

```python
from tqdm import tqdm

# Customize the progress bar
for i in tqdm(range(100),
              desc="Processing",           # Left text
              unit="items",                 # Unit name
              ncols=100,                    # Width in characters
              ascii=False,                  # Use Unicode (True for ASCII only)
              bar_format="{l_bar}{bar}| {n_fmt}/{total_fmt} [{elapsed}<{remaining}]",
              colour='green'):              # Color (green, red, blue, etc.)
    process_item(i)
```

### Disable Progress Bar
```python
from tqdm import tqdm

# Useful for testing or when output needs to be clean
for i in tqdm(range(100), disable=True):  # No progress bar shown
    process_item(i)
```

### Progress Bar with Rich Integration
For even fancier output, combine with `rich` library:

```python
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn

with Progress(
    SpinnerColumn(),
    TextColumn("[progress.description]{task.description}"),
    BarColumn(),
    TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
) as progress:
    task = progress.add_task("[cyan]Processing...", total=100)

    for i in range(100):
        process_item(i)
        progress.update(task, advance=1)
```

## Best Practices

### 1. Use tqdm.write() Instead of print()
```python
from tqdm import tqdm

for i in tqdm(range(100), desc="Processing"):
    result = process_item(i)
    if result.is_error():
        tqdm.write(f"Error at item {i}: {result.error}")  # Won't disrupt progress bar
```

### 2. Provide Meaningful Descriptions
```python
# Bad
for i in tqdm(items):
    ...

# Good
for i in tqdm(items, desc="Processing census tracts", unit="tract"):
    ...
```

### 3. Use leave=False for Temporary Bars
```python
# Main progress stays, inner progress disappears when done
for state in tqdm(states, desc="States"):
    for county in tqdm(counties, desc=f"{state}", leave=False):  # Disappears when done
        process_county(county)
```

### 4. Handle Errors Gracefully
```python
from tqdm import tqdm

with tqdm(items, desc="Processing") as pbar:
    for item in pbar:
        try:
            process_item(item)
            pbar.set_postfix_str("✓ OK")
        except Exception as e:
            pbar.set_postfix_str(f"✗ {str(e)[:20]}")
            # Continue processing other items
```

## Complete Example: Enhanced download_all_states_tracts.py

Here's how to fully upgrade the download script:

```python
#!/usr/bin/env python3
"""Download tract data for all 50 US states."""

import argparse
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from download_tracts import download_tracts
import time
from tqdm import tqdm

# All 50 states + DC
STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY', 'DC'
]

def main():
    parser = argparse.ArgumentParser(description='Download tract data for all 50 US states')
    parser.add_argument('--year', type=int, default=2020, choices=[2020, 2010, 2000],
                        help='Census year (default: 2020)')
    parser.add_argument('--skip', type=str, nargs='*', default=[],
                        help='State codes to skip (e.g., CA TX)')
    parser.add_argument('--no-progress', action='store_true',
                        help='Disable progress bar')
    args = parser.parse_args()

    skip_states = [s.upper() for s in args.skip]
    states_to_download = [s for s in STATES if s not in skip_states]

    print(f"Downloading {args.year} tract data for {len(states_to_download)} states...")
    if skip_states:
        print(f"Skipping: {', '.join(skip_states)}")
    print("=" * 70)

    success_count = 0
    failed_states = []

    # Use progress bar
    with tqdm(states_to_download,
              desc="Downloading tracts",
              unit="state",
              ncols=100,
              disable=args.no_progress) as pbar:

        for state in pbar:
            # Update description to show current state
            pbar.set_description(f"Downloading {state}")

            try:
                download_tracts(state=state, year=args.year)
                success_count += 1
                pbar.set_postfix_str("✓ Complete")
            except Exception as e:
                tqdm.write(f"ERROR: {state} failed - {e}")
                failed_states.append(state)
                pbar.set_postfix_str(f"✗ Failed")

            # Small delay to avoid overwhelming Census servers
            time.sleep(2)

    print("\n" + "=" * 70)
    print(f"SUMMARY:")
    print(f"  Successful: {success_count}/{len(states_to_download)}")
    print(f"  Skipped: {len(skip_states)}")
    if failed_states:
        print(f"  Failed: {len(failed_states)} - {', '.join(failed_states)}")
    else:
        print(f"  Failed: 0")
    print("=" * 70)

if __name__ == '__main__':
    main()
```

## Summary

**Benefits of Progress Bars**:
- ✅ Real-time feedback on script progress
- ✅ Time estimates (how long remaining)
- ✅ Better user experience for long-running operations
- ✅ Easy to debug (see which item is causing slowdowns)
- ✅ Professional appearance

**When to Use**:
- Any loop over multiple items (states, files, records)
- Long-running operations (downloads, processing, calculations)
- Operations where user needs to wait
- Batch processing scripts

**When NOT to Use**:
- Very fast operations (< 1 second total)
- Operations with unpredictable counts
- When output needs to be machine-readable (use --no-progress flag)

**Installation**:
```bash
pip install tqdm
```

The library is already included in your environment since it's used in the existing scripts!
