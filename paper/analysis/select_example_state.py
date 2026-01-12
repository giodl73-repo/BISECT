#!/usr/bin/env python3
"""
Select the best 8-district state for paper visualizations.

Criteria:
- Exactly 8 districts (perfect for 3-round bisection: 1→2→4→8)
- Good compactness scores
- Interesting geography
- Clear round progression
"""

import pandas as pd
import numpy as np
from pathlib import Path
import shutil
import sys

# 2020 apportionment: states with exactly 8 districts
EIGHT_DISTRICT_STATES = {
    'arizona': 'Arizona',
    'colorado': 'Colorado',
    'minnesota': 'Minnesota',
    'missouri': 'Missouri',
    'tennessee': 'Tennessee',
    'wisconsin': 'Wisconsin'
}

def find_eight_district_states(output_dir='outputs/us_2020_v1'):
    """Find all states with exactly 8 districts."""

    output_dir = Path(output_dir)
    states_dir = output_dir / 'states'

    if not states_dir.exists():
        print(f"ERROR: {states_dir} not found")
        return []

    eight_district_states = []

    for state_dir in sorted(states_dir.iterdir()):
        if not state_dir.is_dir():
            continue

        state_name = state_dir.name.replace('_', ' ').title()
        summary_file = state_dir / 'district_summary.csv'

        if not summary_file.exists():
            continue

        df = pd.read_csv(summary_file)
        num_districts = len(df)

        if num_districts == 8:
            eight_district_states.append({
                'state_dir': state_dir,
                'state_name': state_name,
                'state_key': state_dir.name
            })

    return eight_district_states

def evaluate_state_quality(state_dir):
    """Evaluate state quality for paper example."""

    summary_file = state_dir / 'district_summary.csv'
    rounds_file = state_dir / 'rounds_hierarchy.csv'

    if not summary_file.exists() or not rounds_file.exists():
        return None

    # Load data
    df = pd.read_csv(summary_file)
    rounds = pd.read_csv(rounds_file)

    # Calculate metrics
    metrics = {
        'num_districts': len(df),
        'mean_polsby_popper': df['polsby_popper'].mean() if 'polsby_popper' in df.columns else 0,
        'median_polsby_popper': df['polsby_popper'].median() if 'polsby_popper' in df.columns else 0,
        'mean_reock': df['reock'].mean() if 'reock' in df.columns else 0,
        'num_rounds': len(rounds),
        'total_population': df['population'].sum(),
    }

    # Check for round visualizations
    maps_dir = state_dir / 'maps'
    if maps_dir.exists():
        round_maps = list(maps_dir.glob('round_*.png'))
        metrics['num_round_maps'] = len(round_maps)
    else:
        metrics['num_round_maps'] = 0

    return metrics

def select_best_state(states, output_dir='outputs/us_2020_v1'):
    """Select best state based on evaluation criteria."""

    print(f"\nEvaluating {len(states)} states with 8 districts...")
    print("="*70)

    evaluations = []

    for state_info in states:
        state_dir = state_info['state_dir']
        state_name = state_info['state_name']

        metrics = evaluate_state_quality(state_dir)
        if metrics is None:
            continue

        metrics['state_name'] = state_name
        metrics['state_key'] = state_info['state_key']
        metrics['state_dir'] = state_dir

        # Calculate quality score
        # Higher compactness = better
        # More round maps = better
        quality_score = (
            metrics['mean_polsby_popper'] * 0.4 +
            metrics['median_polsby_popper'] * 0.3 +
            metrics['mean_reock'] * 0.2 +
            (metrics['num_round_maps'] / 10.0) * 0.1  # Bonus for having visualizations
        )
        metrics['quality_score'] = quality_score

        evaluations.append(metrics)

    # Sort by quality score
    evaluations.sort(key=lambda x: x['quality_score'], reverse=True)

    # Print evaluation results
    print(f"\n{'State':<15} {'Districts':<10} {'PP Mean':<10} {'Reock Mean':<12} {'Rounds':<8} {'Quality':<10}")
    print("-"*70)

    for ev in evaluations:
        print(f"{ev['state_name']:<15} {ev['num_districts']:<10} "
              f"{ev['mean_polsby_popper']:<10.4f} {ev['mean_reock']:<12.4f} "
              f"{ev['num_round_maps']:<8} {ev['quality_score']:<10.4f}")

    return evaluations[0] if evaluations else None

def copy_example_state_files(state_info, output_dir='paper/figures'):
    """Copy round visualizations and maps to paper directory."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    state_dir = state_info['state_dir']
    state_key = state_info['state_key']
    state_name = state_info['state_name']

    print(f"\n" + "="*70)
    print(f"COPYING FILES FOR: {state_name}")
    print("="*70)

    maps_dir = state_dir / 'maps'
    if not maps_dir.exists():
        print("WARNING: No maps directory found")
        return

    # Copy round visualizations (round_0.png, round_1.png, round_2.png)
    for round_num in range(3):  # 0, 1, 2 for 8-district state
        round_file = maps_dir / f'round_{round_num}.png'
        if round_file.exists():
            dest_file = output_dir / f'example_state_round_{round_num}.png'
            shutil.copy2(round_file, dest_file)
            print(f"[OK] Copied: {dest_file}")
        else:
            print(f"[X] Missing: {round_file}")

    # Copy final district map
    district_map = maps_dir / f'{state_key}_8_districts.png'
    if district_map.exists():
        dest_file = output_dir / 'example_state_final_districts.png'
        shutil.copy2(district_map, dest_file)
        print(f"[OK] Copied: {dest_file}")

    # Copy district map with cities if available
    district_map_cities = maps_dir / f'{state_key}_8_districts_with_cities.png'
    if district_map_cities.exists():
        dest_file = output_dir / 'example_state_final_districts_cities.png'
        shutil.copy2(district_map_cities, dest_file)
        print(f"[OK] Copied: {dest_file}")

def save_selection(state_info, output_dir='paper/data'):
    """Save selected state info."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    selection = pd.DataFrame([{
        'state_name': state_info['state_name'],
        'state_key': state_info['state_key'],
        'num_districts': state_info['num_districts'],
        'mean_polsby_popper': state_info['mean_polsby_popper'],
        'median_polsby_popper': state_info['median_polsby_popper'],
        'mean_reock': state_info['mean_reock'],
        'quality_score': state_info['quality_score'],
    }])

    output_file = output_dir / 'example_state_selection.csv'
    selection.to_csv(output_file, index=False)
    print(f"\n[OK] Saved selection info: {output_file}")

def main():
    """Main function."""
    import argparse

    parser = argparse.ArgumentParser(description='Select example state for paper')
    parser.add_argument('--output-dir', type=str, default='outputs/us_2020_v1',
                       help='Output directory with redistricting results')
    args = parser.parse_args()

    print("="*70)
    print("EXAMPLE STATE SELECTION")
    print("="*70)

    # Find 8-district states
    states = find_eight_district_states(args.output_dir)

    if not states:
        print("ERROR: No states with exactly 8 districts found")
        return 1

    print(f"\nFound {len(states)} states with 8 districts:")
    for state in states:
        print(f"  - {state['state_name']}")

    # Select best state
    best_state = select_best_state(states, args.output_dir)

    if not best_state:
        print("ERROR: Could not evaluate states")
        return 1

    print("\n" + "="*70)
    print(f"SELECTED: {best_state['state_name']}")
    print("="*70)
    print(f"Districts: {best_state['num_districts']}")
    print(f"Mean Polsby-Popper: {best_state['mean_polsby_popper']:.4f}")
    print(f"Mean Reock: {best_state['mean_reock']:.4f}")
    print(f"Quality Score: {best_state['quality_score']:.4f}")

    # Copy files
    copy_example_state_files(best_state)

    # Save selection
    save_selection(best_state)

    print("\n" + "="*70)
    print("SELECTION COMPLETE")
    print("="*70)
    print("\nGenerated files:")
    print("  paper/data/example_state_selection.csv")
    print("  paper/figures/example_state_round_0.png")
    print("  paper/figures/example_state_round_1.png")
    print("  paper/figures/example_state_round_2.png")
    print("  paper/figures/example_state_final_districts.png")

    return 0

if __name__ == '__main__':
    sys.exit(main())
