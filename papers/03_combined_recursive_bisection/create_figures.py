#!/usr/bin/env python3
"""
Create all figures for Paper 3: Combined Recursive Bisection with Edge-Weighted Cuts.

Generates:
1. Round progression maps for Minnesota (3 rounds)
2. Round progression maps for Alabama (3 rounds)
3. Analysis figures (comparison charts, scatter plots)
"""

import shutil
import subprocess
import sys
from pathlib import Path

# Create figures directory in outputs
figures_dir = Path('../../outputs/papers/03_combined_recursive_bisection/figures')
figures_dir.mkdir(parents=True, exist_ok=True)

print("=" * 70)
print("Generating Figures for Paper 3: Combined Recursive Bisection")
print("=" * 70)
print()

# =============================================================================
# Part 1: Copy Round Progression Maps from Pipeline Outputs
# =============================================================================
print("[1/2] Copying round progression maps from pipeline outputs...")
print("-" * 70)

# Check if pipeline outputs exist
pipeline_output_dir = Path('../../outputs/us_2020_v1')
if not pipeline_output_dir.exists():
    print(f"[WARNING] Pipeline outputs not found at: {pipeline_output_dir}")
    print("          Run the full pipeline first to generate round maps.")
    print("          Skipping round progression figures...")
else:
    # Minnesota round maps (3 rounds -> 8 districts)
    minnesota_rounds = [
        ('outputs/us_2020_v1/states/minnesota/maps/rounds/round_01.png',
         'minnesota_round_1_2_regions.png'),
        ('outputs/us_2020_v1/states/minnesota/maps/rounds/round_02.png',
         'minnesota_round_2_4_regions.png'),
        ('outputs/us_2020_v1/states/minnesota/maps/rounds/round_03.png',
         'minnesota_round_3_8_regions.png'),
    ]

    # Alabama round maps (3 rounds -> 7 districts)
    alabama_rounds = [
        ('outputs/us_2020_v1/states/alabama/maps/rounds/round_01.png',
         'alabama_round_1_2_regions.png'),
        ('outputs/us_2020_v1/states/alabama/maps/rounds/round_02.png',
         'alabama_round_2_4_regions.png'),
        ('outputs/us_2020_v1/states/alabama/maps/rounds/round_03.png',
         'alabama_round_3_7_regions.png'),
    ]

    # Copy all round maps
    for source, dest in minnesota_rounds + alabama_rounds:
        source_path = Path(source)
        dest_path = figures_dir / dest

        if source_path.exists():
            shutil.copy2(source_path, dest_path)
            print(f"  ✓ Copied: {dest}")
        else:
            print(f"  ✗ Missing: {source}")

print()

# =============================================================================
# Part 2: Generate Analysis Figures
# =============================================================================
print("[2/2] Generating analysis figures (comparison charts)...")
print("-" * 70)

# Check if comparison data exists
comparison_csv = Path('../../outputs/baseline_comparison_edge/three_way_comparison.csv')
if not comparison_csv.exists():
    print(f"[WARNING] Comparison data not found at: {comparison_csv}")
    print("          Run the baseline comparison analysis first.")
    print("          Skipping analysis figures...")
else:
    # Call the visualization script
    script_path = Path('../../scripts/baseline/visualize_three_way_comparison.py')

    result = subprocess.run(
        [sys.executable, str(script_path),
         '--comparison-csv', str(comparison_csv),
         '--output-dir', str(figures_dir)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        print("  ✓ Analysis figures generated successfully")
        # Print the script's output
        for line in result.stdout.strip().split('\n'):
            if line.strip():
                print(f"    {line}")
    else:
        print(f"  ✗ Error generating analysis figures:")
        print(result.stderr)

print()
print("=" * 70)
print("Figure Generation Complete!")
print("=" * 70)
print(f"Location: {figures_dir.resolve()}")
print()
