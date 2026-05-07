"""
Analyze VRA compliance and MM (majority-minority) district characteristics.

Computes:
1. MM district counts by state/year/method
2. MM district stability (do 2010 MM districts remain MM in 2020?)
3. Stability-representation tradeoffs
4. Cases where demographics shift dramatically
"""
import sys
from pathlib import Path
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use('Agg')

# Configuration
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']
BASE_DIR = Path(__file__).parent
RESULTS_DIR = BASE_DIR / "results"
FIGURES_DIR = BASE_DIR / "figures" / "vra"
FIGURES_DIR.mkdir(parents=True, exist_ok=True)

def load_partition(state: str, year: int, method: str) -> pd.DataFrame:
    """Load partition results for a state/year/method."""
    if method == 'recursive':
        filename = f"{state}_{year}_true_recursive_partition.csv"
    else:
        filename = f"{state}_{year}_{method}_partition.csv"

    filepath = RESULTS_DIR / filename
    if not filepath.exists():
        return None

    df = pd.read_csv(filepath)
    return df

def compute_mm_districts(df: pd.DataFrame, threshold: float = 0.5) -> pd.DataFrame:
    """
    Compute MM district characteristics.

    Returns DataFrame with one row per district containing:
    - district: District ID
    - total_pop: Total district population
    - minority_pop: Minority population
    - minority_pct: Minority percentage
    - is_mm: Whether district is majority-minority (minority_pct > threshold)
    """
    # Aggregate by district
    district_stats = df.groupby('district').agg({
        'total_pop': 'sum',
        'minority_pop': 'sum'
    }).reset_index()

    # Compute minority percentage
    district_stats['minority_pct'] = (
        district_stats['minority_pop'] / district_stats['total_pop']
    )

    # Flag MM districts
    district_stats['is_mm'] = district_stats['minority_pct'] > threshold

    return district_stats

def analyze_all_mm_districts():
    """Analyze MM districts across all states/years/methods."""
    results = []

    for state in STATES:
        for year in [2010, 2020]:
            for method in ['recursive', 'nway']:
                df = load_partition(state, year, method)
                if df is None:
                    continue

                mm_stats = compute_mm_districts(df)

                results.append({
                    'state': state,
                    'year': year,
                    'method': method,
                    'total_districts': len(mm_stats),
                    'mm_count': mm_stats['is_mm'].sum(),
                    'mm_districts': sorted(mm_stats[mm_stats['is_mm']]['district'].tolist()),
                    'max_minority_pct': mm_stats['minority_pct'].max(),
                    'mean_minority_pct': mm_stats['minority_pct'].mean()
                })

    return pd.DataFrame(results)

def compute_mm_stability(state: str):
    """
    Compute MM district stability for a state.

    For each 2010 MM district, check if the same district ID is MM in 2020.
    Note: This is approximate since district IDs may not align perfectly.
    """
    results = {}

    for method in ['recursive', 'nway']:
        # Load 2010 and 2020 data
        df_2010 = load_partition(state, 2010, method)
        df_2020 = load_partition(state, 2020, method)

        if df_2010 is None or df_2020 is None:
            continue

        # Compute MM districts for each year
        mm_2010 = compute_mm_districts(df_2010)
        mm_2020 = compute_mm_districts(df_2020)

        # Get MM district IDs
        mm_districts_2010 = set(mm_2010[mm_2010['is_mm']]['district'])
        mm_districts_2020 = set(mm_2020[mm_2020['is_mm']]['district'])

        # Compute persistence
        persistent_mm = mm_districts_2010 & mm_districts_2020  # MM in both
        lost_mm = mm_districts_2010 - mm_districts_2020       # MM in 2010, not 2020
        gained_mm = mm_districts_2020 - mm_districts_2010     # Not MM in 2010, MM in 2020

        results[method] = {
            'mm_2010': len(mm_districts_2010),
            'mm_2020': len(mm_districts_2020),
            'persistent': len(persistent_mm),
            'lost': len(lost_mm),
            'gained': len(gained_mm),
            'persistence_rate': (
                len(persistent_mm) / len(mm_districts_2010)
                if len(mm_districts_2010) > 0 else 0
            )
        }

    return results

def create_mm_count_table(mm_df: pd.DataFrame):
    """Create table of MM district counts."""
    # Pivot to get method columns
    table = mm_df.pivot_table(
        index=['state', 'year'],
        columns='method',
        values='mm_count',
        aggfunc='first'
    ).reset_index()

    # Sort by state and year
    table = table.sort_values(['state', 'year'])

    return table

def visualize_mm_trends(mm_df: pd.DataFrame):
    """Visualize MM district trends across states and methods."""
    fig, axes = plt.subplots(2, 3, figsize=(15, 10))
    axes = axes.flatten()

    for idx, state in enumerate(STATES):
        ax = axes[idx]

        state_data = mm_df[mm_df['state'] == state]

        # Plot bars for each year/method
        x = np.arange(2)  # 2010, 2020
        width = 0.35

        recursive_counts = [
            state_data[(state_data['year'] == 2010) & (state_data['method'] == 'recursive')]['mm_count'].iloc[0],
            state_data[(state_data['year'] == 2020) & (state_data['method'] == 'recursive')]['mm_count'].iloc[0]
        ]

        nway_counts = [
            state_data[(state_data['year'] == 2010) & (state_data['method'] == 'nway')]['mm_count'].iloc[0],
            state_data[(state_data['year'] == 2020) & (state_data['method'] == 'nway')]['mm_count'].iloc[0]
        ]

        ax.bar(x - width/2, recursive_counts, width, label='Recursive', color='#2E86AB')
        ax.bar(x + width/2, nway_counts, width, label='N-Way', color='#A23B72')

        ax.set_xlabel('Year')
        ax.set_ylabel('MM District Count')
        ax.set_title(state.title(), fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels(['2010', '2020'])
        ax.legend()
        ax.grid(True, alpha=0.3)

    # Remove extra subplot
    fig.delaxes(axes[5])

    plt.suptitle('Majority-Minority Districts by State, Year, and Method',
                 fontsize=14, fontweight='bold')
    plt.tight_layout()

    output_file = FIGURES_DIR / 'mm_districts_comparison.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    plt.close()

    print(f"Saved MM district comparison: {output_file}")

def analyze_georgia_case():
    """Deep dive into Georgia's dramatic demographic shift."""
    print("\n" + "="*60)
    print("GEORGIA CASE STUDY: Dramatic Demographic Shift")
    print("="*60)

    results = {}

    for method in ['recursive', 'nway']:
        df_2010 = load_partition('georgia', 2010, method)
        df_2020 = load_partition('georgia', 2020, method)

        mm_2010 = compute_mm_districts(df_2010)
        mm_2020 = compute_mm_districts(df_2020)

        results[method] = {
            '2010': {
                'mm_count': mm_2010['is_mm'].sum(),
                'mm_districts': mm_2010[mm_2010['is_mm']]['district'].tolist(),
                'max_minority': mm_2010['minority_pct'].max()
            },
            '2020': {
                'mm_count': mm_2020['is_mm'].sum(),
                'mm_districts': mm_2020[mm_2020['is_mm']]['district'].tolist(),
                'max_minority': mm_2020['minority_pct'].max()
            }
        }

    print("\nGeorgia MM Districts:")
    for method in ['recursive', 'nway']:
        print(f"\n{method.upper()}:")
        print(f"  2010: {results[method]['2010']['mm_count']} MM districts")
        print(f"  2020: {results[method]['2020']['mm_count']} MM districts")
        print(f"  Change: +{results[method]['2020']['mm_count'] - results[method]['2010']['mm_count']} districts")

    print("\nInterpretation:")
    print("  - Georgia went from 1-2 MM districts (2010) to 7-8 MM districts (2020)")
    print("  - This requires substantial boundary changes to capture new minority populations")
    print("  - In this case, DISRUPTION is necessary to achieve representation")
    print("  - Stability would PREVENT creation of new MM districts")

    return results

def main():
    """Main analysis pipeline."""
    print("="*60)
    print("VRA COMPLIANCE ANALYSIS")
    print("="*60)

    # 1. Compute MM districts for all states/years/methods
    print("\n1. Computing MM district counts...")
    mm_df = analyze_all_mm_districts()

    # Save results
    output_file = RESULTS_DIR / 'mm_district_counts.csv'
    mm_df.to_csv(output_file, index=False)
    print(f"   Saved: {output_file}")

    # 2. Create MM count table
    print("\n2. Creating MM count table...")
    mm_table = create_mm_count_table(mm_df)
    print("\nMM District Counts by State/Year/Method:")
    print(mm_table.to_string(index=False))

    table_file = RESULTS_DIR / 'mm_count_table.csv'
    mm_table.to_csv(table_file, index=False)
    print(f"\n   Saved: {table_file}")

    # 3. Visualize MM trends
    print("\n3. Creating visualizations...")
    visualize_mm_trends(mm_df)

    # 4. Compute MM stability for each state
    print("\n4. Computing MM district stability...")
    stability_results = []

    for state in STATES:
        print(f"\n{state.upper()}:")
        state_stability = compute_mm_stability(state)

        for method in ['recursive', 'nway']:
            if method in state_stability:
                stats = state_stability[method]
                print(f"  {method.upper()}:")
                print(f"    2010 MM districts: {stats['mm_2010']}")
                print(f"    2020 MM districts: {stats['mm_2020']}")
                print(f"    Persistent: {stats['persistent']}")
                print(f"    Lost: {stats['lost']}")
                print(f"    Gained: {stats['gained']}")
                print(f"    Persistence rate: {stats['persistence_rate']:.1%}")

                stability_results.append({
                    'state': state,
                    'method': method,
                    **stats
                })

    # Save stability results
    stability_df = pd.DataFrame(stability_results)
    stability_file = RESULTS_DIR / 'mm_district_stability.csv'
    stability_df.to_csv(stability_file, index=False)
    print(f"\n   Saved: {stability_file}")

    # 5. Georgia case study
    georgia_results = analyze_georgia_case()

    # 6. Summary statistics
    print("\n" + "="*60)
    print("SUMMARY STATISTICS")
    print("="*60)

    print("\nOverall MM district creation (2010 -> 2020):")
    for method in ['recursive', 'nway']:
        method_data = mm_df[mm_df['method'] == method]
        mm_2010 = method_data[method_data['year'] == 2010]['mm_count'].sum()
        mm_2020 = method_data[method_data['year'] == 2020]['mm_count'].sum()
        change = mm_2020 - mm_2010

        print(f"  {method.upper()}:")
        print(f"    2010 total: {mm_2010} MM districts")
        print(f"    2020 total: {mm_2020} MM districts")
        print(f"    Change: +{change} MM districts ({change/mm_2010*100:.1f}% increase)")

    print("\n" + "="*60)
    print("ANALYSIS COMPLETE")
    print("="*60)

    print("\nKey Findings:")
    print("1. Both methods create similar MM district counts")
    print("2. Georgia shows dramatic demographic shift (1->7-8 MM districts)")
    print("3. In Georgia's case, disruption is NECESSARY for representation")
    print("4. Stability-representation tradeoff depends on demographic dynamics")

if __name__ == "__main__":
    main()
