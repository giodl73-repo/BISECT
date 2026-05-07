"""
Analyze cross-state districts from national redistricting.

Identifies which districts cross state lines and quantifies the degree
of state boundary crossing for fractional representation analysis.

Usage:
    python scripts/experimental/analyze_cross_state_districts.py --year 2020

Output:
    outputs/experimental/cross_state_analysis_2020.pkl
"""

import argparse
import pickle
from pathlib import Path
from typing import Dict, List, Set

import geopandas as gpd
import numpy as np
import pandas as pd
from collections import defaultdict
from tqdm import tqdm

# Add src and scripts to path
import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))
sys.path.insert(0, str(project_root / "scripts"))

from config.download_sources import STATE_FIPS, STATE_NAMES, FIPS_TO_STATE


def load_national_data(input_file: Path) -> Dict:
    """Load national district assignments and adjacency data."""
    print(f"\nLoading national districts from {input_file}...")

    with open(input_file, 'rb') as f:
        data = pickle.load(f)

    print(f"  Loaded {data['n_districts']} districts")
    print(f"  {data['n_cross_state_districts']} districts identified as cross-state")

    return data


def load_tract_data(
    year: int,
    data_dir: Path,
    geoid_to_index: Dict[str, int],
    state_fips_list: List[str]
) -> gpd.GeoDataFrame:
    """Load tract geometries with state information."""
    print(f"\nLoading tract data with state information...")

    all_tracts = []

    for state_abbr, fips in tqdm(STATE_FIPS.items(), desc="Loading tracts"):
        state_name = STATE_NAMES[state_abbr]

        # Try GPKG first, then shapefile
        tract_file_gpkg = data_dir / f"{year}/tiger/tracts/{state_name}/tracts_{state_name}_{year}.gpkg"
        tract_file_shp = data_dir / f"{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp"

        if tract_file_gpkg.exists():
            tract_file = tract_file_gpkg
        elif tract_file_shp.exists():
            tract_file = tract_file_shp
        else:
            continue

        gdf = gpd.read_file(tract_file)

        # Only keep tracts in our graph
        gdf = gdf[gdf['GEOID'].isin(geoid_to_index.keys())]

        # Add state information
        gdf['STATE_FIPS'] = fips
        gdf['STATE_ABBR'] = state_abbr
        gdf['STATE_NAME'] = STATE_NAMES[state_abbr]

        all_tracts.append(gdf[['GEOID', 'geometry', 'STATE_FIPS', 'STATE_ABBR', 'STATE_NAME']])

    # Concatenate all states
    national_gdf = gpd.GeoDataFrame(
        pd.concat(all_tracts, ignore_index=True),
        crs=all_tracts[0].crs
    )

    print(f"  Loaded {len(national_gdf):,} tracts with state information")

    return national_gdf


def analyze_district_state_composition(
    tracts_gdf: gpd.GeoDataFrame,
    geoid_to_district: Dict[str, int],
    populations: np.ndarray,
    geoid_to_index: Dict[str, int]
) -> pd.DataFrame:
    """
    Analyze which states each district contains tracts from.

    Returns:
        DataFrame with district-level state composition
    """
    print(f"\nAnalyzing district-state composition...")

    # Add district assignments
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['district'] = tracts_gdf['GEOID'].map(geoid_to_district)
    tracts_gdf['population'] = tracts_gdf['GEOID'].map(
        lambda g: populations[geoid_to_index[g]] if g in geoid_to_index else 0
    )

    # Remove unassigned tracts
    tracts_gdf = tracts_gdf[tracts_gdf['district'].notna()]

    # Group by district and state
    district_state = tracts_gdf.groupby(['district', 'STATE_ABBR']).agg({
        'GEOID': 'count',  # Number of tracts
        'population': 'sum'  # Total population
    }).rename(columns={'GEOID': 'n_tracts'}).reset_index()

    # Count states per district
    states_per_district = district_state.groupby('district')['STATE_ABBR'].count()
    states_per_district.name = 'n_states'

    # Identify cross-state districts
    cross_state_districts = states_per_district[states_per_district > 1].index.tolist()

    print(f"  {len(cross_state_districts)} districts span multiple states")
    print(f"  {len(states_per_district) - len(cross_state_districts)} districts within single states")

    # For cross-state districts, compute detailed breakdown
    cross_state_details = []

    for district in tqdm(cross_state_districts, desc="Analyzing cross-state districts"):
        district_data = district_state[district_state['district'] == district]

        # Sort by population (descending)
        district_data = district_data.sort_values('population', ascending=False)

        total_pop = district_data['population'].sum()
        primary_state = district_data.iloc[0]['STATE_ABBR']
        primary_pop = district_data.iloc[0]['population']
        primary_pct = (primary_pop / total_pop * 100) if total_pop > 0 else 0

        states_list = district_data['STATE_ABBR'].tolist()

        cross_state_details.append({
            'district': int(district),
            'n_states': len(states_list),
            'states': ', '.join(states_list),
            'primary_state': primary_state,
            'primary_state_pct': primary_pct,
            'total_population': int(total_pop)
        })

    cross_state_df = pd.DataFrame(cross_state_details)

    return cross_state_df, district_state


def analyze_state_pair_borders(
    cross_state_df: pd.DataFrame,
    district_state: pd.DataFrame
) -> pd.DataFrame:
    """
    Analyze which state pairs share districts most frequently.

    Returns:
        DataFrame with state-pair statistics
    """
    print(f"\nAnalyzing state-pair borders...")

    # For each cross-state district, identify all state pairs
    state_pairs = defaultdict(lambda: {'count': 0, 'districts': []})

    for _, row in cross_state_df.iterrows():
        district = row['district']
        states = row['states'].split(', ')

        # Generate all pairs
        for i in range(len(states)):
            for j in range(i + 1, len(states)):
                pair = tuple(sorted([states[i], states[j]]))
                state_pairs[pair]['count'] += 1
                state_pairs[pair]['districts'].append(district)

    # Convert to DataFrame
    pairs_data = []
    for (state1, state2), data in state_pairs.items():
        pairs_data.append({
            'state1': state1,
            'state2': state2,
            'pair': f"{state1}-{state2}",
            'n_shared_districts': data['count'],
            'districts': data['districts']
        })

    pairs_df = pd.DataFrame(pairs_data)
    pairs_df = pairs_df.sort_values('n_shared_districts', ascending=False)

    print(f"  {len(pairs_df)} unique state-pair combinations")
    print(f"\n  Top 10 state pairs by shared districts:")
    for _, row in pairs_df.head(10).iterrows():
        print(f"    {row['pair']}: {row['n_shared_districts']} districts")

    return pairs_df


def compute_fractional_representation(
    cross_state_df: pd.DataFrame,
    district_state: pd.DataFrame
) -> Dict:
    """
    Compute fractional representation statistics.

    For cross-state districts, calculate what fraction of representation
    each state receives.
    """
    print(f"\nComputing fractional representation...")

    # For each cross-state district, compute state fractions
    fractional_data = []

    for district in cross_state_df['district']:
        district_data = district_state[district_state['district'] == district]

        total_pop = district_data['population'].sum()

        for _, row in district_data.iterrows():
            fraction = row['population'] / total_pop if total_pop > 0 else 0
            fractional_data.append({
                'district': int(district),
                'state': row['STATE_ABBR'],
                'population': int(row['population']),
                'fraction': fraction
            })

    fractional_df = pd.DataFrame(fractional_data)

    # Statistics
    stats = {
        'mean_primary_fraction': cross_state_df['primary_state_pct'].mean() / 100,
        'median_primary_fraction': cross_state_df['primary_state_pct'].median() / 100,
        'min_primary_fraction': cross_state_df['primary_state_pct'].min() / 100,
        'max_primary_fraction': cross_state_df['primary_state_pct'].max() / 100,
    }

    print(f"\n  Primary state representation:")
    print(f"    Mean: {stats['mean_primary_fraction']:.1%}")
    print(f"    Median: {stats['median_primary_fraction']:.1%}")
    print(f"    Range: {stats['min_primary_fraction']:.1%} - {stats['max_primary_fraction']:.1%}")

    return fractional_df, stats


def save_results(
    output_file: Path,
    cross_state_df: pd.DataFrame,
    pairs_df: pd.DataFrame,
    fractional_df: pd.DataFrame,
    fractional_stats: Dict,
    district_state: pd.DataFrame
) -> None:
    """Save cross-state analysis results."""
    print(f"\nSaving results to {output_file}...")

    data = {
        'cross_state_districts': cross_state_df.to_dict('records'),
        'state_pairs': pairs_df.to_dict('records'),
        'fractional_representation': fractional_df.to_dict('records'),
        'fractional_stats': fractional_stats,
        'district_state_composition': district_state.to_dict('records'),
        'n_cross_state_districts': len(cross_state_df),
        'n_state_pairs': len(pairs_df)
    }

    with open(output_file, 'wb') as f:
        pickle.dump(data, f)

    print(f"  [OK] Saved cross-state analysis")


def main():
    parser = argparse.ArgumentParser(description="Analyze cross-state districts")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/experimental'), help='Output directory')
    args = parser.parse_args()

    print("="*70)
    print("National Redistricting Cross-State Analysis (Paper #13)")
    print("="*70)

    # Load national districts
    districts_file = args.input_dir / f"national_districts_{args.year}.pkl"
    district_data = load_national_data(districts_file)

    # Load tract data with state information
    tracts_gdf = load_tract_data(
        args.year,
        args.data_dir,
        district_data['geoid_to_index'],
        []  # Not needed - we load all states anyway
    )

    # Analyze district-state composition
    cross_state_df, district_state = analyze_district_state_composition(
        tracts_gdf,
        district_data['geoid_to_district'],
        district_data['populations'],
        district_data['geoid_to_index']
    )

    # Analyze state-pair borders
    pairs_df = analyze_state_pair_borders(cross_state_df, district_state)

    # Compute fractional representation
    fractional_df, fractional_stats = compute_fractional_representation(
        cross_state_df,
        district_state
    )

    # Save results
    output_file = args.output_dir / f"cross_state_analysis_{args.year}.pkl"
    save_results(
        output_file,
        cross_state_df,
        pairs_df,
        fractional_df,
        fractional_stats,
        district_state
    )

    print("\n" + "="*70)
    print("Cross-state analysis complete!")
    print("="*70)
    print(f"Output: {output_file}")
    print(f"\nKey findings:")
    print(f"  - {len(cross_state_df)} districts span multiple states ({len(cross_state_df)/435*100:.1f}%)")
    print(f"  - {len(pairs_df)} unique state-pair combinations")
    print(f"  - Mean primary state: {fractional_stats['mean_primary_fraction']:.1%} of district population")


if __name__ == '__main__':
    main()
