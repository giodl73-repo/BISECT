"""
Master script to run all temporal stability experiments.

Orchestrates the full experimental pipeline:
1. Run 2010 redistricting (both methods, 5 states)
2. Compute stability metrics
3. Analyze community disruption
4. Generate visualizations
"""

import sys
import subprocess
from pathlib import Path
from datetime import datetime


def run_script(script_name: str, description: str):
    """Run a Python script and report status."""
    print()
    print("="*70)
    print(f"STEP: {description}")
    print("="*70)
    print(f"Running: {script_name}")
    print(f"Started: {datetime.now().strftime('%H:%M:%S')}")
    print()

    script_path = Path(__file__).parent / 'scripts' / script_name

    try:
        result = subprocess.run(
            [sys.executable, str(script_path)],
            capture_output=False,
            text=True,
            check=True
        )
        print()
        print(f"[OK] {description} completed")
        return True

    except subprocess.CalledProcessError as e:
        print()
        print(f"[FAIL] {description} failed with exit code {e.returncode}")
        return False

    except FileNotFoundError:
        print()
        print(f"[FAIL] Script not found: {script_path}")
        return False


def check_prerequisites():
    """Check if required data exists."""
    print()
    print("="*70)
    print("CHECKING PREREQUISITES")
    print("="*70)

    issues = []

    # Get project root (two levels up from this script)
    project_root = Path(__file__).parent.parent.parent

    # Check for 2010 data
    states = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']

    # FIPS mapping
    state_to_fips = {
        'alabama': '01',
        'georgia': '13',
        'louisiana': '22',
        'mississippi': '28',
        'south_carolina': '45'
    }

    print("\n2010 Census Data:")
    for state in states:
        fips = state_to_fips[state]
        tracts_dir = project_root / f'data/2010/tiger/tracts/tl_2010_{fips}_tract10'
        demographics_file = project_root / f'data/2010/demographics/{state}_demographics_2010.csv'
        adjacency_file = project_root / f'outputs/data/2010/adjacency/{state}_adjacency.npz'

        status = []
        if not tracts_dir.exists():
            status.append("tracts MISSING")
        if not demographics_file.exists():
            status.append("demographics MISSING")
        if not adjacency_file.exists():
            status.append("adjacency MISSING")

        if status:
            print(f"  {state:20s}: {', '.join(status)}")
            issues.append(f"{state}: {', '.join(status)}")
        else:
            print(f"  {state:20s}: OK")

    # Check for 2020 data (should exist from Papers 1-2)
    # Note: We don't need 2020 adjacency - we'll load existing partition results
    print("\n2020 Census Data:")
    print("  (Using existing 2020 partition results from Papers 1-2)")
    print("  Adjacency not required for stability analysis")

    # Check for tract relationship files
    print("\nTract Relationship Files:")
    relationships_dir = project_root / 'data/tract_relationships'
    if not relationships_dir.exists():
        print(f"  Directory MISSING: {relationships_dir}")
        issues.append("Tract relationship directory missing")
    else:
        for state in states:
            rel_file = relationships_dir / f'{state}_2010_2020.csv'
            if not rel_file.exists():
                print(f"  {state:20s}: MISSING")
                issues.append(f"{state}: relationship file missing")
            else:
                print(f"  {state:20s}: OK")

    print()
    if issues:
        print("[WARNING] Prerequisites incomplete:")
        for issue in issues:
            print(f"  - {issue}")
        print()
        print("To download missing data:")
        print("  python scripts/data/download_orchestrator.py --year 2010 --states alabama georgia louisiana mississippi south_carolina")
        print("  python scripts/data/build_adjacency.py --year 2010 --states alabama georgia louisiana mississippi south_carolina")
        print()
        print("To download tract relationship files:")
        print("  https://www.census.gov/geographies/reference-files/time-series/geo/relationship-files.html")
        print()
        return False
    else:
        print("[OK] All prerequisites satisfied")
        return True


def main():
    """Run complete experimental pipeline."""
    print()
    print("="*70)
    print("TEMPORAL STABILITY EXPERIMENTS - FULL PIPELINE")
    print("="*70)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()

    # Check prerequisites
    if not check_prerequisites():
        print()
        print("="*70)
        print("ABORTING: Prerequisites not met")
        print("="*70)
        return

    # Pipeline steps
    steps = [
        ('run_2010_redistricting.py', 'Run 2010 Redistricting (N-Way + Recursive)'),
        ('compute_stability_metrics.py', 'Compute Temporal Stability Metrics'),
        ('analyze_community_disruption.py', 'Analyze County-Level Disruption'),
        ('visualize_stability.py', 'Generate Visualizations'),
    ]

    results = {}

    for script, description in steps:
        success = run_script(script, description)
        results[description] = 'SUCCESS' if success else 'FAILED'

        if not success:
            print()
            print(f"[WARNING] {description} failed, but continuing...")

    # Summary
    print()
    print("="*70)
    print("PIPELINE COMPLETE")
    print("="*70)
    print(f"Finished: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()
    print("Results Summary:")
    for step, status in results.items():
        symbol = "[OK]" if status == 'SUCCESS' else "[FAIL]"
        print(f"  {symbol} {step}")

    print()
    print("Output files:")
    print("  - Results: research/gerry-temporal-stability/results/")
    print("  - Figures: research/gerry-temporal-stability/figures/")
    print()
    print("Next steps:")
    print("  1. Review results and figures")
    print("  2. Fill results tables in sections/04_results.tex")
    print("  3. Compile paper: pdflatex main.tex")


if __name__ == '__main__':
    main()
