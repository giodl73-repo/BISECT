"""
Compute geographic clustering metrics for VRA states.

Metrics:
1. Moran's I - Global spatial autocorrelation (-1 to +1)
   +1: Highly clustered (easier to create MM districts)
   0: Random distribution
   -1: Dispersed (harder to create MM districts)

2. Local Clustering Index - % of minority population in MM tracts
   Higher values indicate minority population is concentrated in few areas

3. Feasibility precursor metrics
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np
from esda.moran import Moran
from libpysal.weights import Queen

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

VRA_STATES = {
    'alabama': {'code': 'AL', 'fips': '01'},
    'georgia': {'code': 'GA', 'fips': '13'},
    'louisiana': {'code': 'LA', 'fips': '22'},
    'mississippi': {'code': 'MS', 'fips': '28'},
    'south_carolina': {'code': 'SC', 'fips': '45'},
}


def load_tract_demographics(state_name, state_fips, year=2020):
    """Load census tract data with demographics and geometries."""
    # Path to tract shapefiles
    tiger_path = Path(f'../../data/{year}/tiger/tracts/tl_{year}_{state_fips}_tract')
    shapefile = tiger_path / f'tl_{year}_{state_fips}_tract.shp'

    # Check if shapefile exists
    if not shapefile.exists():
        print(f"  [SKIP] No tract geometry found: {shapefile}")
        return None

    # Load tract geometries
    tracts_gdf = gpd.read_file(shapefile)

    # Load demographics
    demographics_file = Path(f'../../data/{year}/demographics/{state_name}_demographics_{year}.csv')

    if not demographics_file.exists():
        print(f"  [SKIP] No demographics found: {demographics_file}")
        return None

    demographics_df = pd.read_csv(demographics_file)

    # Ensure GEOID is 11-character format with leading zeros
    demographics_df['GEOID'] = demographics_df['GEOID'].astype(str).str.zfill(11)

    # Merge demographics with geometries
    # The shapefile GEOID is STATEFP + COUNTYFP + TRACTCE
    tracts_gdf['GEOID'] = tracts_gdf['STATEFP'] + tracts_gdf['COUNTYFP'] + tracts_gdf['TRACTCE']

    tracts_gdf = tracts_gdf.merge(
        demographics_df,
        on='GEOID',
        how='inner'
    )

    if len(tracts_gdf) == 0:
        print(f"  [SKIP] No matching tracts after merge")
        return None

    # Calculate minority percentages (following VRA utils convention)
    tracts_gdf['pct_white'] = tracts_gdf['white_non_hispanic'] / tracts_gdf['total_pop']
    tracts_gdf['pct_minority'] = 1.0 - tracts_gdf['pct_white']

    # Handle NaN values (zero-population tracts)
    tracts_gdf['pct_minority'] = tracts_gdf['pct_minority'].fillna(0)

    # Calculate absolute minority population
    tracts_gdf['minority_pop'] = tracts_gdf['total_pop'] * tracts_gdf['pct_minority']

    return tracts_gdf


def compute_morans_i(tracts_gdf):
    """Compute Moran's I for spatial autocorrelation of minority %."""
    try:
        # Create spatial weights matrix (Queen contiguity)
        w = Queen.from_dataframe(tracts_gdf, use_index=False)

        # Compute Moran's I on minority percentage (already calculated in load function)
        moran = Moran(tracts_gdf['pct_minority'], w)

        return moran.I, moran.p_sim, tracts_gdf

    except Exception as e:
        print(f"  [ERROR] Failed to compute Moran's I: {e}")
        return None, None, None


def compute_clustering_index(tracts_gdf, threshold=0.50):
    """
    Compute local clustering index.

    Returns the percentage of total minority population that lives in
    tracts with >=threshold minority percentage.
    """
    try:
        # Identify high-minority tracts
        high_minority_tracts = tracts_gdf[tracts_gdf['pct_minority'] >= threshold]

        # Compute clustering index
        total_minority = tracts_gdf['minority_pop'].sum()
        minority_in_high_tracts = high_minority_tracts['minority_pop'].sum()

        if total_minority == 0:
            return 0

        clustering_index = minority_in_high_tracts / total_minority

        return clustering_index

    except Exception as e:
        print(f"  [ERROR] Failed to compute clustering index: {e}")
        return None


def compute_dispersion_metrics(tracts_gdf):
    """Compute additional dispersion metrics."""
    try:
        # Standard deviation of minority percentages
        std_minority = tracts_gdf['pct_minority'].std()

        # Coefficient of variation
        mean_minority = tracts_gdf['pct_minority'].mean()
        cv_minority = std_minority / mean_minority if mean_minority > 0 else 0

        # Percentage of tracts that are majority-minority
        mm_tract_pct = (tracts_gdf['pct_minority'] >= 0.50).sum() / len(tracts_gdf)

        return {
            'std_minority': std_minority,
            'cv_minority': cv_minority,
            'mm_tract_pct': mm_tract_pct,
        }

    except Exception as e:
        print(f"  [ERROR] Failed to compute dispersion metrics: {e}")
        return None


def analyze_state(state_name, state_info):
    """Analyze geographic clustering for one state."""
    state_code = state_info['code']
    state_fips = state_info['fips']

    print(f"\nAnalyzing {state_name.replace('_', ' ').title()} ({state_code})...")

    # Load data
    tracts_gdf = load_tract_demographics(state_name, state_fips)
    if tracts_gdf is None:
        return None

    print(f"  Loaded {len(tracts_gdf)} tracts")

    # Compute Moran's I
    morans_i, p_value, tracts_gdf = compute_morans_i(tracts_gdf)
    if morans_i is None:
        return None

    print(f"  Moran's I: {morans_i:.4f} (p={p_value:.4f})")

    # Compute clustering indices at different thresholds
    clustering_40 = compute_clustering_index(tracts_gdf, threshold=0.40)
    clustering_45 = compute_clustering_index(tracts_gdf, threshold=0.45)
    clustering_50 = compute_clustering_index(tracts_gdf, threshold=0.50)

    print(f"  Clustering indices:")
    print(f"    40%+ tracts: {clustering_50:.1%} of minority pop")
    print(f"    45%+ tracts: {clustering_45:.1%} of minority pop")
    print(f"    50%+ tracts: {clustering_50:.1%} of minority pop")

    # Compute dispersion metrics
    dispersion = compute_dispersion_metrics(tracts_gdf)
    if dispersion:
        print(f"  Dispersion metrics:")
        print(f"    Std dev: {dispersion['std_minority']:.4f}")
        print(f"    CV: {dispersion['cv_minority']:.4f}")
        print(f"    MM tracts: {dispersion['mm_tract_pct']:.1%}")

    return {
        'state_code': state_code,
        'state_name': state_name.replace('_', ' ').title(),
        'num_tracts': len(tracts_gdf),
        'morans_i': morans_i,
        'morans_i_p_value': p_value,
        'clustering_index_40': clustering_40,
        'clustering_index_45': clustering_45,
        'clustering_index_50': clustering_50,
        'std_minority_pct': dispersion['std_minority'] if dispersion else None,
        'cv_minority_pct': dispersion['cv_minority'] if dispersion else None,
        'mm_tract_pct': dispersion['mm_tract_pct'] if dispersion else None,
    }


def main():
    """Compute clustering metrics for all VRA states."""
    print("Computing geographic clustering metrics...")

    results = []
    for state_name, state_info in VRA_STATES.items():
        result = analyze_state(state_name, state_info)
        if result:
            results.append(result)

    if len(results) == 0:
        print("\n[ERROR] No results computed. Check data availability.")
        return

    # Create DataFrame
    df = pd.DataFrame(results)

    # Sort by Moran's I descending
    df = df.sort_values('morans_i', ascending=False)

    # Save results
    output_path = Path('results')
    output_path.mkdir(exist_ok=True)

    output_file = output_path / 'geographic_clustering_metrics.csv'
    df.to_csv(output_file, index=False, float_format='%.4f')

    print(f"\nSaved clustering metrics to: {output_file}")
    print(f"Total states analyzed: {len(df)}")

    # Print summary
    print("\n=== CLUSTERING SUMMARY ===")
    print(f"{'State':<15} {'Moran I':<10} {'50%+ Clustering':<18} {'MM Tracts':<10}")
    print("-" * 55)
    for _, row in df.iterrows():
        print(f"{row['state_code']:<15} {row['morans_i']:<10.4f} "
              f"{row['clustering_index_50']:<18.1%} {row['mm_tract_pct']:<10.1%}")


if __name__ == '__main__':
    main()
