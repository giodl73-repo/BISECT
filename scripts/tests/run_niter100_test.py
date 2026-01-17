#!/usr/bin/env python3
"""
Test niter=100 to see impact on compactness.

This script modifies the default niter value to 100 (up from 20)
to test whether additional refinement iterations improve compactness.

Test on California only for quick comparison.
Results saved to outputs/niter100-test/
"""

import sys
import subprocess
from pathlib import Path
from datetime import datetime

# Test state: California only
TEST_STATE = {'CA': {'name': 'California', 'districts': 52}}


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


def update_niter_default(niter_value):
    """
    Temporarily update the niter default in metis files.
    Returns the old values for restoration.
    """
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
            old_contents[str(file_path)] = f.read()

        # Replace niter default value
        new_content = old_contents[str(file_path)].replace(
            'niter: int = 20',
            f'niter: int = {niter_value}'
        )

        with open(file_path, 'w') as f:
            f.write(new_content)

        print(f"Updated {file_path.name}: niter default = {niter_value}")

    return old_contents


def restore_niter_default(old_contents):
    """Restore original niter values."""
    for file_path, content in old_contents.items():
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"Restored {Path(file_path).name}")


def process_california_niter100(test_dir):
    """Process California with niter=100."""

    state_code = 'CA'
    state_name = 'California'
    num_districts = 52

    print(f"\n{'#'*70}")
    print(f"# Testing niter=100: {state_name} - {num_districts} districts")
    print(f"{'#'*70}")

    # Check prerequisites
    missing = check_prerequisites(state_code)
    if missing:
        print(f"ERROR: Missing data for {state_name}: {', '.join(missing)}")
        return False

    state_dir = test_dir / 'california'
    state_dir.mkdir(parents=True, exist_ok=True)

    # Step 1: Run redistricting (will use niter=100)
    print(f"\nStep 1/5: Running redistricting with niter=100...")
    print(f"  - METIS niter=100 (testing high refinement)")
    print(f"  - County-aware water adjacency")
    if not run_command(
        f'python scripts/run_state_redistricting.py --state {state_code} --output-dir {state_dir}',
        f'Redistricting {state_name}',
        timeout=3600  # 60 minutes for higher refinement
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
    """Test niter=100 on California."""

    test_dir = Path('outputs/niter100-test')
    test_dir.mkdir(parents=True, exist_ok=True)

    # Create README for the test directory
    readme_content = f"""# niter=100 Test Results

**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Test Configuration

Testing the impact of higher refinement iterations on compactness:
- **niter = 100** (up from 20, and 10x METIS default of 10)
- **State**: California (52 districts)
- **Purpose**: Determine if additional refinement improves compactness

## Hypothesis

Increasing niter from 20 to 100 may:
1. Improve district compactness (Polsby-Popper scores)
2. Better population balance (lower deviation)
3. Take significantly longer (~5x processing time)

## Comparison Baselines

### Baseline (us_2020_redistricting)
- niter: default (likely 10)
- California max deviation: Unknown (need to check)

### Compactness-testing (niter=20)
- niter: 20
- California max deviation: 0.32%

### This Test (niter=100)
- niter: 100
- California max deviation: To be determined

## Results

Compare:
```bash
# Population balance
diff outputs/compactness-testing/california/district_summary.csv \\
     outputs/niter100-test/california/district_summary.csv

# Visual comparison
compare outputs/compactness-testing/california/california_52_districts.png \\
        outputs/niter100-test/california/california_52_districts.png
```

## Expected Outcome

If niter=100 doesn't significantly improve results over niter=20, then:
- niter=20 is the sweet spot (good quality, reasonable speed)
- Further refinement shows diminishing returns
"""

    readme_path = test_dir / 'README.md'
    with open(readme_path, 'w') as f:
        f.write(readme_content)

    print(f"\n{'='*70}")
    print(f"niter=100 Test - High Refinement Experiment")
    print(f"{'='*70}")
    print(f"Output directory: {test_dir}")
    print(f"\nTesting on California (52 districts)")
    print(f"This will take longer due to 5x more refinement iterations")
    print(f"{'='*70}\n")

    # Update niter default to 100
    print("Updating niter default to 100...")
    old_contents = update_niter_default(100)

    try:
        # Process California
        success = process_california_niter100(test_dir)

        # Summary
        print(f"\n\n{'='*70}")
        print(f"niter=100 TEST SUMMARY")
        print(f"{'='*70}")
        if success:
            print(f"[OK] California completed with niter=100")
            print(f"\nResults saved to: {test_dir}")
            print(f"\nNext Steps:")
            print(f"  1. Compare district_summary.csv with compactness-testing")
            print(f"  2. Calculate Polsby-Popper scores for both")
            print(f"  3. Visual comparison of district maps")
            print(f"  4. Decide if niter=100 is worth the extra time")
        else:
            print(f"[FAIL] California processing failed")
        print(f"{'='*70}")

        return 0 if success else 1

    finally:
        # Always restore original values
        print("\nRestoring original niter defaults...")
        restore_niter_default(old_contents)
        print("Original values restored")


if __name__ == '__main__':
    sys.exit(main())
