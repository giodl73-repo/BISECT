"""
Master Script for Phase 1: Geographic Sorting Data Preparation

Runs all Phase 1 steps in sequence:
1. Prepare tract-level data (election + population + geometry)
2. Compute geographic sorting indices
3. Compute partisan bias
4. Merge results

Author: Claude
Date: 2026-02-07
"""

import subprocess
import sys
from pathlib import Path

def run_script(script_path: str, description: str) -> bool:
    """Run a Python script and report status."""
    print()
    print("="*70)
    print(f"STEP: {description}")
    print("="*70)
    print()

    try:
        result = subprocess.run(
            [sys.executable, script_path],
            check=True,
            capture_output=False,
            text=True
        )
        print()
        print(f"[OK] {description} - COMPLETE")
        return True

    except subprocess.CalledProcessError as e:
        print()
        print(f"[FAIL] {description}")
        print(f"Error: {e}")
        return False


def main():
    """Run complete Phase 1 analysis."""
    scripts_dir = Path("scripts/pipeline/p2_geographic_sorting")

    print()
    print("="*70)
    print("PHASE 1: GEOGRAPHIC SORTING DATA PREPARATION")
    print("="*70)
    print()
    print("This will run 3 analysis scripts in sequence:")
    print("  1. prepare_tract_data.py (~15-20 min)")
    print("  2. compute_geographic_sorting.py (~5 min)")
    print("  3. compute_partisan_bias.py (~5 min)")
    print()
    print("Total estimated time: 25-30 minutes")
    print()
    input("Press Enter to continue or Ctrl+C to cancel...")

    # Step 1: Prepare tract data
    success1 = run_script(
        scripts_dir / "prepare_tract_data.py",
        "Step 1/3: Prepare tract-level data"
    )

    if not success1:
        print()
        print("ERROR: Data preparation failed. Stopping.")
        return

    # Step 2: Compute geographic sorting
    success2 = run_script(
        scripts_dir / "compute_geographic_sorting.py",
        "Step 2/3: Compute geographic sorting indices"
    )

    if not success2:
        print()
        print("WARNING: Geographic sorting computation failed.")
        print("Continuing to next step...")

    # Step 3: Compute partisan bias
    success3 = run_script(
        scripts_dir / "compute_partisan_bias.py",
        "Step 3/3: Compute partisan bias"
    )

    if not success3:
        print()
        print("WARNING: Partisan bias computation failed.")

    # Summary
    print()
    print("="*70)
    print("PHASE 1 ANALYSIS COMPLETE")
    print("="*70)
    print()
    print("Results:")
    print(f"  Step 1 (Data Preparation): {'[OK] SUCCESS' if success1 else '[FAIL] FAILED'}")
    print(f"  Step 2 (Geographic Sorting): {'[OK] SUCCESS' if success2 else '[FAIL] FAILED'}")
    print(f"  Step 3 (Partisan Bias): {'[OK] SUCCESS' if success3 else '[FAIL] FAILED'}")
    print()

    if success1 and success2 and success3:
        print("[OK] All Phase 1 steps completed successfully!")
        print()
        print("Output files:")
        print("  - research/gerry-recursive-bisection/data/geographic_sorting/tracts/*.gpkg")
        print("  - research/gerry-recursive-bisection/data/geographic_sorting/geographic_sorting_by_state.csv")
        print("  - research/gerry-recursive-bisection/data/geographic_sorting/partisan_bias_by_state.csv")
        print()
        print("Next step: Phase 2 - Empirical Analysis")
    else:
        print("[WARN] Some steps failed. Review error messages above.")


if __name__ == "__main__":
    main()
