#!/usr/bin/env python3
"""
Fix missing outputs in 2010 census run based on validation report.

This script systematically re-runs the scripts that failed to generate
outputs for the 2010 census run, as identified by the validation framework.

Usage:
    python scripts/pipeline/fix_2010_missing_outputs.py --year 2010 --version v1
    python scripts/pipeline/fix_2010_missing_outputs.py --year 2010 --version v1 --dry-run
"""

import sys
import subprocess
import argparse
from pathlib import Path
from tqdm import tqdm
import multiprocessing
from concurrent.futures import ProcessPoolExecutor, as_completed

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config


def run_script_for_state(state_code, state_name, script_path, output_dir, year, args_extra=''):
    """Run a script for a single state."""
    state_dir = output_dir / 'states' / state_name.lower().replace(' ', '_')

    if not state_dir.exists():
        return (state_code, 'SKIP', f"Directory not found: {state_dir}")

    cmd = [
        sys.executable,
        str(script_path),
        str(state_dir),
        '--year', year
    ]

    if args_extra:
        cmd.extend(args_extra.split())

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        if result.returncode == 0:
            return (state_code, 'SUCCESS', '')
        else:
            return (state_code, 'FAILED', result.stderr[:200])
    except subprocess.TimeoutExpired:
        return (state_code, 'TIMEOUT', 'Script exceeded 5 minute timeout')
    except Exception as e:
        return (state_code, 'ERROR', str(e))


def fix_missing_cities_csv(state_config, output_dir, year, dry_run=False):
    """Fix missing district_cities.csv files for all states."""
    print("\n" + "="*70)
    print("  Phase 1: Adding Cities to Districts")
    print("="*70)

    script_path = Path('scripts/pipeline/add_cities_to_districts.py')
    if not script_path.exists():
        print(f"ERROR: Script not found: {script_path}")
        return False

    states_to_fix = list(state_config.keys())
    print(f"Processing {len(states_to_fix)} states...")

    if dry_run:
        print("[DRY RUN] Would run add_cities_to_districts.py for all 50 states")
        return True

    successful = 0
    failed = []

    # Use parallel processing
    max_workers = min(multiprocessing.cpu_count(), 8)

    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        futures = {}
        for state_code in states_to_fix:
            config = state_config[state_code]
            future = executor.submit(
                run_script_for_state,
                state_code,
                config['name'],
                script_path,
                output_dir,
                year
            )
            futures[future] = state_code

        # Progress bar
        with tqdm(total=len(states_to_fix), desc="Adding cities", unit="state") as pbar:
            for future in as_completed(futures):
                state_code, status, message = future.result()

                if status == 'SUCCESS':
                    successful += 1
                elif status == 'SKIP':
                    pass  # Skip doesn't count as failure
                else:
                    failed.append((state_code, message))

                pbar.update(1)
                pbar.set_postfix_str(f"OK {successful}/{len(states_to_fix)}")

    print(f"\nPhase 1 Complete: {successful}/{len(states_to_fix)} successful")
    if failed:
        print(f"Failed states: {', '.join([f[0] for f in failed])}")

    return len(failed) == 0


def fix_missing_district_maps(state_config, output_dir, year, dry_run=False):
    """Fix missing individual district maps for all states."""
    print("\n" + "="*70)
    print("  Phase 2: Creating Individual District Maps")
    print("="*70)

    script_path = Path('scripts/pipeline/visualize_individual_districts.py')
    if not script_path.exists():
        print(f"ERROR: Script not found: {script_path}")
        return False

    states_to_fix = list(state_config.keys())
    print(f"Processing {len(states_to_fix)} states...")

    if dry_run:
        print("[DRY RUN] Would run visualize_individual_districts.py for all 50 states")
        return True

    successful = 0
    failed = []

    # Use parallel processing
    max_workers = min(multiprocessing.cpu_count(), 8)

    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        futures = {}
        for state_code in states_to_fix:
            config = state_config[state_code]
            future = executor.submit(
                run_script_for_state,
                state_code,
                config['name'],
                script_path,
                output_dir,
                year,
                '--dpi 150'
            )
            futures[future] = state_code

        # Progress bar
        with tqdm(total=len(states_to_fix), desc="Creating maps", unit="state") as pbar:
            for future in as_completed(futures):
                state_code, status, message = future.result()

                if status == 'SUCCESS':
                    successful += 1
                elif status == 'SKIP':
                    pass
                else:
                    failed.append((state_code, message))

                pbar.update(1)
                pbar.set_postfix_str(f"OK {successful}/{len(states_to_fix)}")

    print(f"\nPhase 2 Complete: {successful}/{len(states_to_fix)} successful")
    if failed:
        print(f"Failed states: {', '.join([f[0] for f in failed])}")

    return len(failed) == 0


def fix_national_outputs(output_dir, year, version, dry_run=False):
    """Fix missing national aggregation outputs."""
    print("\n" + "="*70)
    print("  Phase 3: National Post-Processing")
    print("="*70)

    scripts = [
        ('scripts/pipeline/create_rounds_hierarchy.py', 'rounds_hierarchy.csv'),
        ('scripts/pipeline/visualize_national_districts.py', 'us_all_districts.png'),
        ('scripts/pipeline/visualize_national_rounds.py', 'us_rounds/')
    ]

    if dry_run:
        print("[DRY RUN] Would run national post-processing scripts:")
        for script, _ in scripts:
            print(f"  - {script}")
        return True

    successful = 0
    failed = []

    for script_path, output_file in scripts:
        script = Path(script_path)
        if not script.exists():
            print(f"  SKIP: {script.name} (script not found)")
            continue

        print(f"  Running {script.name}...")

        cmd = [
            sys.executable,
            str(script),
            '--year', year,
            '--version', version,
            '--output-dir', str(output_dir)
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
            if result.returncode == 0:
                print(f"    [OK] SUCCESS: {output_file}")
                successful += 1
            else:
                print(f"    [X] FAILED: {script.name}")
                failed.append((script.name, result.stderr[:200]))
        except subprocess.TimeoutExpired:
            print(f"    [X] TIMEOUT: {script.name}")
            failed.append((script.name, 'Timeout'))
        except Exception as e:
            print(f"    [X] ERROR: {script.name} - {e}")
            failed.append((script.name, str(e)))

    print(f"\nPhase 3 Complete: {successful}/{len(scripts)} successful")
    return len(failed) == 0


def run_validation(year, version, output_dir):
    """Run validation to verify all outputs are present."""
    print("\n" + "="*70)
    print("  Final Validation")
    print("="*70)

    validation_script = Path('scripts/validation/validate_pipeline_outputs.py')
    if not validation_script.exists():
        print("WARNING: Validation script not found")
        return

    cmd = [
        sys.executable,
        str(validation_script),
        '--year', year,
        '--version', version,
        '--output-dir', str(output_dir),
        '--force'
    ]

    subprocess.run(cmd)


def main():
    parser = argparse.ArgumentParser(
        description='Fix missing outputs in 2020/2010/2000 census runs'
    )
    parser.add_argument('--year', type=str, required=True, choices=['2020', '2010', '2000'],
                       help='Census year to fix')
    parser.add_argument('--version', type=str, default='v1',
                       help='Run version (default: v1)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--skip-phase', type=str, choices=['1', '2', '3'],
                       help='Skip a specific phase (1=cities, 2=maps, 3=national)')

    args = parser.parse_args()

    # Determine output directory
    output_dir = Path(f'outputs/us_{args.year}_{args.version}')
    if not output_dir.exists():
        print(f"ERROR: Output directory not found: {output_dir}")
        return 1

    print("="*70)
    print(f"  Fixing Missing Outputs: {args.year} Census")
    print("="*70)
    print(f"Output directory: {output_dir}")
    if args.dry_run:
        print("Mode: DRY RUN (no changes will be made)")
    print()

    # Load state configuration
    try:
        state_config = get_state_config(args.year)
    except Exception as e:
        print(f"ERROR: Failed to load state config: {e}")
        return 1

    success = True

    # Phase 1: Fix missing district_cities.csv
    if args.skip_phase != '1':
        if not fix_missing_cities_csv(state_config, output_dir, args.year, args.dry_run):
            success = False

    # Phase 2: Fix missing district maps
    if args.skip_phase != '2':
        if not fix_missing_district_maps(state_config, output_dir, args.year, args.dry_run):
            success = False

    # Phase 3: Fix national outputs
    if args.skip_phase != '3':
        if not fix_national_outputs(output_dir, args.year, args.version, args.dry_run):
            success = False

    # Final validation
    if not args.dry_run:
        run_validation(args.year, args.version, output_dir)

    print("\n" + "="*70)
    if success:
        print("  [OK] All fixes completed successfully!")
    else:
        print("  [WARN] Some fixes failed - check output above")
    print("="*70)

    return 0 if success else 1


if __name__ == '__main__':
    # Required for multiprocessing on Windows
    multiprocessing.freeze_support()
    sys.exit(main())
