#!/usr/bin/env python3
"""
Master script to generate all statistics and figures for the paper.

Runs all analysis scripts in order:
1. Population deviation statistics
2. Compactness statistics
3. Political statistics
4. Select example state and copy visualizations
"""

import subprocess
import sys
from pathlib import Path

def run_script(script_name, description, output_dir='outputs/us_2020_v1'):
    """Run an analysis script."""

    print("\n" + "="*80)
    print(f"  {description}")
    print("="*80)

    script_path = Path(__file__).parent / script_name
    cmd = [sys.executable, str(script_path), '--output-dir', output_dir]

    result = subprocess.run(cmd, capture_output=False)

    if result.returncode != 0:
        print(f"\n✗ ERROR: {script_name} failed with exit code {result.returncode}")
        return False

    print(f"\n[OK] {description} - COMPLETE")
    return True

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Generate all paper statistics')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    print("="*80)
    print(" "*20 + "PAPER STATISTICS GENERATION")
    print("="*80)
    print(f"\nSource data: {args.output_dir}")
    print(f"Output data: paper/data/")
    print(f"Output figures: paper/figures/")

    # Track success
    all_success = True

    # 1. Population statistics
    if not run_script('compute_population_stats.py',
                     '1/4: Computing Population Deviation Statistics',
                     args.output_dir):
        all_success = False

    # 2. Compactness statistics
    if not run_script('compute_compactness_stats.py',
                     '2/4: Computing Compactness Statistics',
                     args.output_dir):
        all_success = False

    # 3. Political statistics
    if not run_script('compute_political_stats.py',
                     '3/4: Computing Political Statistics',
                     args.output_dir):
        all_success = False

    # 4. Select example state
    if not run_script('select_example_state.py',
                     '4/4: Selecting Example State and Copying Visualizations',
                     args.output_dir):
        all_success = False

    # Final summary
    print("\n" + "="*80)
    if all_success:
        print(" "*25 + "[OK] ALL ANALYSES COMPLETE")
    else:
        print(" "*20 + "✗ SOME ANALYSES FAILED")
    print("="*80)

    # List all generated files
    print("\nGenerated Data Files:")
    data_dir = Path('paper/data')
    if data_dir.exists():
        for f in sorted(data_dir.glob('*.csv')):
            print(f"  [OK] {f}")
    else:
        print("  ✗ No data files generated")

    print("\nGenerated Figure Files:")
    figures_dir = Path('paper/figures')
    if figures_dir.exists():
        for f in sorted(figures_dir.glob('*.png')):
            print(f"  [OK] {f}")
    else:
        print("  ✗ No figure files generated")

    print("\n" + "="*80)
    print("Next step: Run 'python paper/build_paper.py' to compile the paper")
    print("="*80)

    return 0 if all_success else 1

if __name__ == '__main__':
    sys.exit(main())
