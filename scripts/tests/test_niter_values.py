#!/usr/bin/env python3
"""
Test multiple niter values (20, 50, 100) on California.

Compares the impact of different refinement iteration counts on:
- Population balance (deviation)
- Processing time
- Compactness (Polsby-Popper scores)

Results saved to outputs/niter-comparison/
"""

import sys
import subprocess
import time
from pathlib import Path
from datetime import datetime

# Test California only
STATE_CODE = 'CA'
STATE_NAME = 'California'
NUM_DISTRICTS = 52

# Test values
NITER_VALUES = [20, 50, 100]


def check_prerequisites():
    """Check if state has necessary data files."""
    tracts_file = Path(f'data/raw/ca_tracts_2020.parquet')
    places_file = Path(f'data/raw/ca_places_2020.parquet')
    graph_file = Path(f'data/processed/ca_tracts_graph.pkl')

    missing = []
    if not tracts_file.exists():
        missing.append('tracts')
    if not places_file.exists():
        missing.append('places')
    if not graph_file.exists():
        missing.append('adjacency graph')

    return missing


def run_command(cmd, description, timeout=3600):
    """Run a command and return success status and elapsed time."""
    print(f"\n{'='*70}")
    print(f"{description}")
    print(f"{'='*70}")

    start_time = time.time()

    try:
        result = subprocess.run(
            cmd,
            shell=True,
            check=True,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        elapsed = time.time() - start_time
        print(result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return True, elapsed
    except subprocess.CalledProcessError as e:
        elapsed = time.time() - start_time
        print(f"ERROR: Command failed with exit code {e.returncode}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False, elapsed
    except subprocess.TimeoutExpired:
        elapsed = time.time() - start_time
        print(f"ERROR: Command timed out after {timeout} seconds")
        return False, elapsed


def update_niter_default(niter_value):
    """Update the niter default in metis files."""
    from pathlib import Path

    files_to_update = [
        Path('src/apportionment/partition/metis_wrapper.py'),
        Path('src/apportionment/partition/metis_executable.py')
    ]

    old_contents = {}

    for file_path in files_to_update:
        if not file_path.exists():
            continue

        with open(file_path, 'r') as f:
            content = f.read()
            old_contents[str(file_path)] = content

        # Replace all niter default values
        # Handle both "niter: int = X" and "niter=X" patterns
        import re
        new_content = re.sub(r'niter: int = \d+', f'niter: int = {niter_value}', content)
        new_content = re.sub(r'niter=\d+', f'niter={niter_value}', new_content)

        with open(file_path, 'w') as f:
            f.write(new_content)

    return old_contents


def restore_files(old_contents):
    """Restore original file contents."""
    for file_path, content in old_contents.items():
        with open(file_path, 'w') as f:
            f.write(content)


def test_niter_value(niter, test_dir):
    """Test a specific niter value on California."""

    print(f"\n{'#'*70}")
    print(f"# Testing niter={niter}: {STATE_NAME} - {NUM_DISTRICTS} districts")
    print(f"{'#'*70}")

    # Create niter-specific subdirectory with proper state naming
    niter_dir = test_dir / f'niter{niter}'
    niter_dir.mkdir(parents=True, exist_ok=True)

    state_dir = niter_dir / 'california'
    state_dir.mkdir(parents=True, exist_ok=True)

    # Update niter default
    print(f"\nSetting niter={niter}...")
    old_contents = update_niter_default(niter)

    try:
        # Step 1: Run redistricting
        print(f"\nStep 1/5: Running redistricting with niter={niter}...")
        success, elapsed_redistrict = run_command(
            f'python scripts/run_state_redistricting.py --state {STATE_CODE} --output-dir {state_dir}',
            f'Redistricting {STATE_NAME}',
            timeout=7200  # 2 hours max
        )

        if not success:
            return False, elapsed_redistrict

        # Step 2: Add cities
        print(f"\nStep 2/5: Adding cities...")
        success, elapsed_cities = run_command(
            f'python scripts/add_cities_to_districts.py {state_dir} --state {state_code}',
            f'Adding cities to {STATE_NAME}',
            timeout=600
        )

        if not success:
            return False, elapsed_redistrict + elapsed_cities

        # Step 3: Create district summary
        print(f"\nStep 3/5: Creating district summary...")
        success, elapsed_summary = run_command(
            f'python scripts/create_final_district_summary.py {state_dir} --state {state_code}',
            f'Creating summary for {STATE_NAME}',
            timeout=300
        )

        if not success:
            return False, elapsed_redistrict + elapsed_cities + elapsed_summary

        # Step 4: Skip compactness metrics for speed (can calculate later)
        print(f"\nStep 4/5: Skipping compactness metrics (will calculate after all tests complete)")
        elapsed_metrics = 0

        # Step 5: Create visualizations (optional, can skip for speed)
        print(f"\nStep 5/5: Creating final map...")
        success, elapsed_viz = run_command(
            f'python scripts/visualize_all_rounds.py {state_dir} --state {state_code}',
            f'Creating round visualizations',
            timeout=600
        )

        total_time = elapsed_redistrict + elapsed_cities + elapsed_summary + elapsed_viz

        print(f"\n{'#'*70}")
        print(f"# niter={niter} COMPLETE!")
        print(f"# Output: {state_dir}")
        print(f"# Redistricting time: {elapsed_redistrict:.1f} seconds ({elapsed_redistrict/60:.1f} minutes)")
        print(f"# Total time: {total_time:.1f} seconds ({total_time/60:.1f} minutes)")
        print(f"{'#'*70}")

        return success, total_time

    finally:
        # Always restore original values
        print(f"\nRestoring original niter defaults...")
        restore_files(old_contents)


def main():
    """Test multiple niter values on California."""

    # Check prerequisites
    missing = check_prerequisites()
    if missing:
        print(f"ERROR: Missing data: {', '.join(missing)}")
        return 1

    test_dir = Path('outputs/niter-comparison')
    test_dir.mkdir(parents=True, exist_ok=True)

    # Create README
    readme_content = f"""# niter Comparison Results

**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
**State**: California (52 districts)

## Test Configuration

Testing impact of different METIS refinement iteration counts:
- **niter = 20**: 2x default (baseline)
- **niter = 50**: 5x default
- **niter = 100**: 10x default

## Metrics Measured

1. **Population Balance**: Max deviation from ideal population
2. **Processing Time**: Time for redistricting step
3. **Compactness**: Polsby-Popper scores for each district

## Expected Trade-offs

- Higher niter -> Better compactness
- Higher niter -> Longer processing time
- Diminishing returns expected at higher values

## Results

See individual directories:
- `niter20/california/` - Baseline (2x default)
- `niter50/california/` - Medium refinement (5x default)
- `niter100/california/` - High refinement (10x default)

Compare district summaries:
```bash
diff niter20/california/district_summary.csv niter50/california/district_summary.csv
diff niter50/california/district_summary.csv niter100/california/district_summary.csv
```

## Compactness Comparison

Check mean Polsby-Popper scores in each district_summary.csv file.
"""

    readme_path = test_dir / 'README.md'
    with open(readme_path, 'w', encoding='utf-8') as f:
        f.write(readme_content)

    print(f"\n{'='*70}")
    print(f"niter Comparison Test")
    print(f"{'='*70}")
    print(f"Output directory: {test_dir}")
    print(f"\nTesting {len(NITER_VALUES)} configurations: {NITER_VALUES}")
    print(f"State: {STATE_NAME} ({NUM_DISTRICTS} districts)")
    print(f"{'='*70}\n")

    # Track results
    results = []

    for niter in NITER_VALUES:
        success, elapsed_time = test_niter_value(niter, test_dir)

        results.append({
            'niter': niter,
            'success': success,
            'time_seconds': elapsed_time,
            'time_minutes': elapsed_time / 60
        })

        if not success:
            print(f"\nERROR: niter={niter} failed, stopping tests")
            break

    # Summary
    print(f"\n\n{'='*70}")
    print(f"NITER COMPARISON SUMMARY")
    print(f"{'='*70}")

    for result in results:
        niter = result['niter']
        if result['success']:
            print(f"[OK] niter={niter:3d}: {result['time_minutes']:6.1f} minutes")
        else:
            print(f"[FAIL] niter={niter:3d}: Failed")

    if all(r['success'] for r in results):
        print(f"\n{'='*70}")
        print(f"Next Steps:")
        print(f"  1. Compare district_summary.csv files")
        print(f"  2. Review Polsby-Popper scores (compactness)")
        print(f"  3. Check max deviation (population balance)")
        print(f"  4. Evaluate time vs. quality trade-off")
        print(f"  5. Choose optimal niter for production run")
        print(f"{'='*70}")

    return 0 if all(r['success'] for r in results) else 1


if __name__ == '__main__':
    sys.exit(main())
