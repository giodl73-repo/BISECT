#!/usr/bin/env python3
"""
Test compactness improvements on select states.

This script runs redistricting with Phase 1 compactness improvements:
- Increased METIS refinement iterations (niter=20, up from default 10)
- County-aware water adjacency (already implemented)
- Tests on CA, TX, FL first to validate improvements

Results saved to outputs/compactness-testing/ for comparison with baseline.
"""

import sys
import subprocess
from pathlib import Path
from datetime import datetime

# Test states: Start with 3 largest states
TEST_STATES = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
}

# Full state configuration for later expansion
STATE_CONFIG = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
    'NY': {'name': 'New York', 'districts': 26},
    'PA': {'name': 'Pennsylvania', 'districts': 17},
    'IL': {'name': 'Illinois', 'districts': 17},
    'OH': {'name': 'Ohio', 'districts': 15},
    'GA': {'name': 'Georgia', 'districts': 14},
    'NC': {'name': 'North Carolina', 'districts': 14},
    'MI': {'name': 'Michigan', 'districts': 13},
    'NJ': {'name': 'New Jersey', 'districts': 12},
    'VA': {'name': 'Virginia', 'districts': 11},
    'WA': {'name': 'Washington', 'districts': 10},
    'AZ': {'name': 'Arizona', 'districts': 9},
    'MA': {'name': 'Massachusetts', 'districts': 9},
    'TN': {'name': 'Tennessee', 'districts': 9},
    'IN': {'name': 'Indiana', 'districts': 9},
    'MD': {'name': 'Maryland', 'districts': 8},
    'MO': {'name': 'Missouri', 'districts': 8},
    'WI': {'name': 'Wisconsin', 'districts': 8},
    'CO': {'name': 'Colorado', 'districts': 8},
    'MN': {'name': 'Minnesota', 'districts': 8},
    'SC': {'name': 'South Carolina', 'districts': 7},
    'AL': {'name': 'Alabama', 'districts': 7},
    'LA': {'name': 'Louisiana', 'districts': 6},
    'KY': {'name': 'Kentucky', 'districts': 6},
    'OR': {'name': 'Oregon', 'districts': 6},
    'OK': {'name': 'Oklahoma', 'districts': 5},
    'CT': {'name': 'Connecticut', 'districts': 5},
    'UT': {'name': 'Utah', 'districts': 4},
    'IA': {'name': 'Iowa', 'districts': 4},
    'NV': {'name': 'Nevada', 'districts': 4},
    'AR': {'name': 'Arkansas', 'districts': 4},
    'MS': {'name': 'Mississippi', 'districts': 4},
    'KS': {'name': 'Kansas', 'districts': 4},
    'NM': {'name': 'New Mexico', 'districts': 3},
    'NE': {'name': 'Nebraska', 'districts': 3},
    'ID': {'name': 'Idaho', 'districts': 2},
    'WV': {'name': 'West Virginia', 'districts': 2},
    'HI': {'name': 'Hawaii', 'districts': 2},
    'NH': {'name': 'New Hampshire', 'districts': 2},
    'ME': {'name': 'Maine', 'districts': 2},
    'RI': {'name': 'Rhode Island', 'districts': 2},
    'MT': {'name': 'Montana', 'districts': 2},
}


def check_prerequisites(state_code):
    """Check if state has necessary data files."""
    state_code_lower = state_code.lower()

    tracts_file = Path(f'data/raw/{state_code_lower}_tracts_2020.parquet')
    places_file = Path(f'data/raw/{state_code_lower}_places_2020.parquet')
    graph_file = Path(f'data/processed/{state_code_lower}_tracts_graph.pkl')

    missing = []
    if not tracts_file.exists():
        missing.append('tracts')
    if not places_file.exists():
        missing.append('places')
    if not graph_file.exists():
        missing.append('adjacency graph')

    return missing


def run_command(cmd, description, timeout=600):
    """Run a command and return success status."""
    print(f"\n{'='*70}")
    print(f"{description}")
    print(f"{'='*70}")

    try:
        result = subprocess.run(
            cmd,
            shell=True,
            check=True,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        print(result.stdout)
        if result.stderr:
            print("STDERR:", result.stderr)
        return True
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Command failed with exit code {e.returncode}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False
    except subprocess.TimeoutExpired:
        print(f"ERROR: Command timed out after {timeout} seconds")
        return False


def process_state(state_code, test_dir):
    """Process a single state through the full pipeline."""

    config = STATE_CONFIG[state_code]
    state_name = config['name']
    num_districts = config['districts']

    print(f"\n{'#'*70}")
    print(f"# Testing Compactness: {state_name} ({state_code}) - {num_districts} districts")
    print(f"{'#'*70}")

    # Check prerequisites
    missing = check_prerequisites(state_code)
    if missing:
        print(f"ERROR: Missing data for {state_name}: {', '.join(missing)}")
        return False

    # Use the final directory path for all operations
    state_dir = test_dir / state_name.lower().replace(' ', '_')
    state_dir.mkdir(parents=True, exist_ok=True)

    # Step 1: Run redistricting (will use niter=20 from metis_wrapper.py)
    print(f"\nStep 1/5: Running redistricting with compactness improvements...")
    print(f"  - METIS niter=20 (up from default 10)")
    print(f"  - County-aware water adjacency")
    if not run_command(
        f'python scripts/run_state_redistricting.py --state {state_code} --output-dir {state_dir}',
        f'Redistricting {state_name}',
        timeout=1800  # 30 minutes
    ):
        return False

    # Step 2: Add cities
    print(f"\nStep 2/5: Adding cities...")
    if not run_command(
        f'python scripts/add_cities_to_districts.py {state_dir} --state {state_code}',
        f'Adding cities to {state_name}',
        timeout=600
    ):
        return False

    # Step 3: Create district summary
    print(f"\nStep 3/5: Creating district summary...")
    if not run_command(
        f'python scripts/create_final_district_summary.py {state_dir} --state {state_code}',
        f'Creating summary for {state_name}',
        timeout=300
    ):
        return False

    # Step 4: Create individual district maps
    print(f"\nStep 4/5: Creating individual district maps...")
    if not run_command(
        f'python scripts/create_individual_district_maps.py {state_dir} --state {state_code}',
        f'Creating {num_districts} district maps for {state_name}',
        timeout=1800
    ):
        return False

    # Step 5: Visualize intermediate rounds
    print(f"\nStep 5/5: Visualizing intermediate rounds...")
    if not run_command(
        f'python scripts/visualize_all_rounds.py {state_dir} --state {state_code}',
        f'Creating round visualizations for {state_name}',
        timeout=600
    ):
        return False

    print(f"\n{'#'*70}")
    print(f"# {state_name} COMPLETE!")
    print(f"# Output: {state_dir}")
    print(f"{'#'*70}")

    return True


def main():
    """Process test states with compactness improvements."""

    test_dir = Path('outputs/compactness-testing')
    test_dir.mkdir(parents=True, exist_ok=True)

    # Create README for the test directory
    readme_content = f"""# Compactness Testing Results

**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Phase 1 Improvements Tested

### 1. Increased METIS Refinement Iterations
- **Change**: `niter=20` (up from default 10)
- **Expected Impact**: +5-10% compactness improvement, ~2x slower
- **Implementation**: Modified `metis_wrapper.py`

### 2. County-Aware Water Adjacency
- **Change**: Prefer same-county connections for island tracts
- **Expected Impact**: Better respect for county boundaries
- **Implementation**: Modified `build_tract_adjacency.py`

## Test States

Starting with 3 largest states to validate improvements:
- **California (CA)**: 52 districts
- **Texas (TX)**: 38 districts
- **Florida (FL)**: 28 districts

## Comparison

To compare with baseline results, see:
- Baseline: `outputs/us_2020_redistricting/`
- Test results: `outputs/compactness-testing/`

Compare district summaries:
```bash
diff outputs/us_2020_redistricting/california/district_summary.csv \\
     outputs/compactness-testing/california/district_summary.csv
```

## Expected Results

- **Compactness**: +10-15% improvement (visual inspection + future metrics)
- **Speed**: ~2x slower (still reasonable for full US run)
- **Population balance**: Same or better
"""

    readme_path = test_dir / 'README.md'
    with open(readme_path, 'w') as f:
        f.write(readme_content)

    print(f"\n{'='*70}")
    print(f"Compactness Testing - Phase 1 Improvements")
    print(f"{'='*70}")
    print(f"Output directory: {test_dir}")
    print(f"\nTesting on {len(TEST_STATES)} states:")
    for code, config in TEST_STATES.items():
        print(f"  - {config['name']} ({code}): {config['districts']} districts")
    print(f"{'='*70}\n")

    # Track results
    successful = []
    failed = []

    for state_code in TEST_STATES.keys():
        success = process_state(state_code, test_dir)
        if success:
            successful.append(state_code)
        else:
            failed.append(state_code)

    # Summary
    print(f"\n\n{'='*70}")
    print(f"COMPACTNESS TEST SUMMARY")
    print(f"{'='*70}")
    print(f"Successful: {len(successful)}/{len(TEST_STATES)}")
    for s in successful:
        print(f"  [OK] {STATE_CONFIG[s]['name']} ({s}) - {STATE_CONFIG[s]['districts']} districts")

    if failed:
        print(f"\nFailed: {len(failed)}")
        for s in failed:
            print(f"  [FAIL] {STATE_CONFIG[s]['name']} ({s})")

    print(f"\n{'='*70}")
    print(f"Results saved to: {test_dir}")
    print(f"\nNext Steps:")
    print(f"  1. Compare visual quality of district maps")
    print(f"  2. Check district_summary.csv for population balance")
    print(f"  3. If satisfied, expand to all 50 states")
    print(f"{'='*70}")

    return 0 if not failed else 1


if __name__ == '__main__':
    sys.exit(main())
