"""
Compute Geographic Sorting Indices for All 50 States

This script calculates the geographic sorting index (correlation between
Democratic vote share and population density) for each state, which measures
the degree to which Democratic voters are concentrated in high-density urban areas.

Based on: Chen & Rodden (2013), "Unintentional Gerrymandering"

Author: Claude
Date: 2026-02-07
"""

import json
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np
from scipy.stats import pearsonr
from typing import Dict, Tuple
import sys

# Add project root to path
project_root = Path(__file__).parents[3]
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / "src"))

from scripts.config_2020 import STATE_CONFIG_2020
from scripts.config.download_sources import STATE_NAMES

# Create STATES_2020 dict in expected format (lowercase_underscore_name -> districts)
STATES_2020 = {
    STATE_NAMES[abbrev]: config['districts']
    for abbrev, config in STATE_CONFIG_2020.items()
}

def load_prepared_tract_data(state_name: str) -> gpd.GeoDataFrame:
    """Load prepared tract data for a state."""
    data_dir = Path("research/gerry-recursive-bisection/data/geographic_sorting/tracts")
    filepath = data_dir / f"{state_name}_tracts_prepared.gpkg"

    if not filepath.exists():
        print(f"Warning: No prepared data found for {state_name}")
        return None

    gdf = gpd.read_file(filepath)
    return gdf


def compute_geographic_sorting_index(gdf: gpd.GeoDataFrame) -> Tuple[float, float, int]:
    """
    Compute geographic sorting index for a state.

    Returns:
        (correlation, p_value, n_tracts)
    """
    # Filter out tracts with no votes or zero density
    valid_data = gdf[
        (gdf['total_votes'] > 0) &
        (gdf['pop_density'] > 0) &
        (gdf['dem_vote_share'].notna()) &
        (gdf['pop_density'].notna())
    ].copy()

    if len(valid_data) < 10:
        print(f"Warning: Insufficient data ({len(valid_data)} tracts)")
        return None, None, len(valid_data)

    # Compute Pearson correlation
    correlation, p_value = pearsonr(
        valid_data['dem_vote_share'],
        valid_data['pop_density']
    )

    return correlation, p_value, len(valid_data)


def compute_state_urban_percentage(gdf: gpd.GeoDataFrame) -> float:
    """Compute percentage of state population living in urban areas."""
    # Define urban threshold: > 1000 people per square mile
    urban_threshold = 1000

    is_urban = gdf['pop_density'] > urban_threshold
    urban_population = gdf.loc[is_urban, 'total_population'].sum()
    total_population = gdf['total_population'].sum()

    return (urban_population / total_population) * 100 if total_population > 0 else 0


def analyze_all_states(year: int = 2020) -> pd.DataFrame:
    """Analyze geographic sorting for all 50 states."""
    results = []

    for state_name, n_districts in STATES_2020.items():
        print(f"Processing {state_name}...")

        # Skip single-district states (no redistricting)
        if n_districts == 1:
            print(f"  Skipping {state_name} (single district)")
            continue

        try:
            # Load prepared data
            gdf = load_prepared_tract_data(state_name)

            if gdf is None:
                print(f"  Skipping {state_name} (missing data)")
                continue

            # Compute geographic sorting index
            correlation, p_value, n_tracts = compute_geographic_sorting_index(gdf)

            if correlation is None:
                print(f"  Skipping {state_name} (computation failed)")
                continue

            # Compute urban percentage
            urban_pct = compute_state_urban_percentage(gdf)

            # Store results
            results.append({
                'state': state_name,
                'n_districts': n_districts,
                'geographic_sorting': correlation,
                'sorting_pvalue': p_value,
                'n_tracts': n_tracts,
                'urban_percent': urban_pct,
                'significant': p_value < 0.05
            })

            print(f"  [OK] {state_name}: sorting = {correlation:.3f} "
                  f"(p={p_value:.4f}), urban = {urban_pct:.1f}%")

        except Exception as e:
            print(f"  Error processing {state_name}: {e}")
            continue

    # Create DataFrame
    df = pd.DataFrame(results)

    # Sort by geographic sorting (descending)
    df = df.sort_values('geographic_sorting', ascending=False)

    return df


def main():
    """Main execution."""
    print("="*60)
    print("Computing Geographic Sorting Indices for All 50 States")
    print("="*60)
    print()

    # Analyze all states
    results_df = analyze_all_states(year=2020)

    # Save results
    output_dir = Path("research/gerry-recursive-bisection/data/geographic_sorting")
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / "geographic_sorting_by_state.csv"
    results_df.to_csv(output_file, index=False)

    print()
    print("="*60)
    print(f"Results saved to: {output_file}")
    print("="*60)
    print()

    # Print summary statistics
    print("Summary Statistics:")
    print(f"  States analyzed: {len(results_df)}")
    print(f"  Mean geographic sorting: {results_df['geographic_sorting'].mean():.3f}")
    print(f"  Std dev: {results_df['geographic_sorting'].std():.3f}")
    print(f"  Range: [{results_df['geographic_sorting'].min():.3f}, "
          f"{results_df['geographic_sorting'].max():.3f}]")
    print(f"  Significant correlations (p<0.05): "
          f"{results_df['significant'].sum()}/{len(results_df)}")
    print()

    # Print top 10 highest sorting states
    print("Top 10 States with Highest Geographic Sorting:")
    print(results_df[['state', 'geographic_sorting', 'urban_percent', 'n_districts']]
          .head(10).to_string(index=False))
    print()

    # Print bottom 10
    print("Bottom 10 States with Lowest Geographic Sorting:")
    print(results_df[['state', 'geographic_sorting', 'urban_percent', 'n_districts']]
          .tail(10).to_string(index=False))


if __name__ == "__main__":
    main()
