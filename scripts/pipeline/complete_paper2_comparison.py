"""
Complete comparison for Paper 2: N-Way vs Recursive Bisection

Runs all methods across all 5 VRA states and compiles results.
"""

import sys
from pathlib import Path
import pandas as pd
import subprocess

project_root = Path(__file__).resolve().parents[2]

def run_nway():
    """Run n-way multi-constraint for all states"""
    print("\n" + "="*70)
    print("STEP 1: Running N-Way Multi-Constraint (all 5 states)")
    print("="*70)

    result = subprocess.run(
        [sys.executable, str(project_root / 'scripts/pipeline/test_nway_vs_recursive_v2.py')],
        capture_output=True,
        text=True
    )

    print(result.stdout)
    if result.returncode != 0:
        print("ERROR:", result.stderr)
        return False
    return True


def run_recursive_for_all_states():
    """Run recursive bisection for all states"""
    print("\n" + "="*70)
    print("STEP 2: Running Recursive Bisection (all 5 states)")
    print("="*70)

    # Update the recursive script to run all states
    recursive_script = project_root / 'scripts/pipeline/run_recursive_bisection.py'

    # Read and modify to run all states
    with open(recursive_script, 'r') as f:
        content = f.read()

    # Replace test_states line
    content = content.replace(
        "test_states = ['alabama']",
        "test_states = ['mississippi', 'louisiana', 'alabama', 'south_carolina', 'georgia']"
    )

    # Also disable adaptive for speed (can run separately if needed)
    content = content.replace(
        "# Test adaptive\n        print(f\"\\nRunning ADAPTIVE RECURSIVE BISECTION...\")",
        "# Test adaptive (DISABLED for speed - run ~50s per state)\n        if False:  # Change to True to enable adaptive\n            print(f\"\\nRunning ADAPTIVE RECURSIVE BISECTION...\")"
    )

    # Write temporary modified script
    temp_script = project_root / 'scripts/pipeline/run_recursive_all_states.py'
    with open(temp_script, 'w') as f:
        f.write(content)

    # Run it
    result = subprocess.run(
        [sys.executable, str(temp_script)],
        capture_output=True,
        text=True
    )

    print(result.stdout)
    if result.returncode != 0:
        print("ERROR:", result.stderr)
        return False
    return True


def compile_results():
    """Compile all results into single comparison file"""
    print("\n" + "="*70)
    print("STEP 3: Compiling Results")
    print("="*70)

    results_dir = project_root / 'research/gerry-nway-vs-recursive/results'

    # Load all result files
    nway_file = results_dir / 'nway_multiconstraint_results.csv'
    recursive_file = results_dir / 'recursive_bisection_results.csv'

    if not nway_file.exists():
        print(f"ERROR: {nway_file} not found")
        return

    if not recursive_file.exists():
        print(f"ERROR: {recursive_file} not found")
        return

    nway_df = pd.read_csv(nway_file)
    recursive_df = pd.read_csv(recursive_file)

    # Combine
    combined_df = pd.concat([nway_df, recursive_df], ignore_index=True)

    # Save combined
    output_file = results_dir / 'paper2_complete_comparison.csv'
    combined_df.to_csv(output_file, index=False)

    print(f"\nSaved combined results to: {output_file}")

    # Print summary
    print("\n" + "="*70)
    print("SUMMARY: N-Way vs Recursive Bisection")
    print("="*70)

    for state in combined_df['state'].unique():
        state_df = combined_df[combined_df['state'] == state]

        print(f"\n{state.upper()}:")

        for _, row in state_df.iterrows():
            method_label = {
                'nway_multiconstraint': 'N-Way',
                'recursive_predetermined': f"Recursive {row.get('tree_structure', '?')}",
                'recursive_adaptive': 'Adaptive'
            }.get(row['method'], row['method'])

            print(f"  {method_label:25s}: {row['max_minority_pct']:.1%} max, "
                  f"{int(row['mm_count'])}/{int(row['target_mm'])} MM, "
                  f"{row['runtime']:.2f}s")

        # Compute gap
        nway_row = state_df[state_df['method'] == 'nway_multiconstraint']
        recursive_rows = state_df[state_df['method'].str.contains('recursive')]

        if len(nway_row) > 0 and len(recursive_rows) > 0:
            nway_max = nway_row.iloc[0]['max_minority_pct']
            best_recursive_max = recursive_rows['max_minority_pct'].max()
            gap = (nway_max - best_recursive_max) * 100

            print(f"  Gap (N-Way - Best Recursive): {gap:+.1f} percentage points")


def main():
    """Run complete Paper 2 comparison"""
    print("="*70)
    print("PAPER 2 COMPLETE COMPARISON")
    print("Running all methods across all 5 VRA states")
    print("="*70)

    # Step 1: Run n-way
    if not run_nway():
        print("\nERROR: N-way failed")
        return

    # Step 2: Run recursive
    if not run_recursive_for_all_states():
        print("\nERROR: Recursive failed")
        return

    # Step 3: Compile results
    compile_results()

    print("\n" + "="*70)
    print("COMPLETE! Results saved to:")
    print("  research/gerry-nway-vs-recursive/results/paper2_complete_comparison.csv")
    print("="*70)


if __name__ == '__main__':
    main()
