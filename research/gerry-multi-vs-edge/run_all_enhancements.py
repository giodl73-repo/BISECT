"""
Master script to run all optional enhancements for Paper 4
"""

import subprocess
import sys
from pathlib import Path

def run_script(script_name, description):
    """Run a Python script and report status"""
    print(f"\n{'='*70}")
    print(f"{description}")
    print('='*70)

    try:
        result = subprocess.run(
            [sys.executable, script_name],
            cwd=Path(__file__).parent,
            capture_output=False,
            text=True,
            check=True
        )
        print(f"\n[OK] {description} completed successfully!")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n[FAIL] {description} failed with error code {e.returncode}")
        return False
    except FileNotFoundError:
        print(f"\n[FAIL] Script not found: {script_name}")
        return False

def main():
    print("\n" + "="*70)
    print("RUNNING ALL OPTIONAL ENHANCEMENTS FOR PAPER 4")
    print("="*70)

    enhancements = [
        ("create_alabama_maps.py", "Enhancement 1: Alabama District Maps"),
        ("calculate_compactness_metrics.py", "Enhancement 2: Compactness Metrics (Polsby-Popper & Reock)"),
        ("run_multi_year_validation.py", "Enhancement 3: Multi-Year Validation (2000/2010/2020)"),
    ]

    results = {}
    for script, description in enhancements:
        success = run_script(script, description)
        results[description] = success

    # Summary
    print("\n" + "="*70)
    print("ENHANCEMENT SUMMARY")
    print("="*70)

    for description, success in results.items():
        status = "[OK]" if success else "[FAIL]"
        print(f"{status} {description}")

    success_count = sum(results.values())
    total_count = len(results)

    print(f"\n{success_count}/{total_count} enhancements completed successfully")

    # List generated files
    print("\n" + "="*70)
    print("GENERATED FILES")
    print("="*70)

    files_to_check = [
        "results/figure5_alabama_comparison.png",
        "results/figure5_alabama_comparison.pdf",
        "results/compactness_metrics.csv",
        "results/multi_year_validation.csv",
    ]

    for file_path in files_to_check:
        full_path = Path(__file__).parent / file_path
        if full_path.exists():
            size = full_path.stat().st_size
            print(f"[OK] {file_path} ({size:,} bytes)")
        else:
            print(f"[MISSING] {file_path}")

    print("\n" + "="*70)
    print("ALL ENHANCEMENTS COMPLETE!")
    print("="*70)

    print("\nNext steps:")
    print("1. Review generated figures and data")
    print("2. Update paper sections with new results")
    print("3. Recompile LaTeX")

if __name__ == '__main__':
    main()
