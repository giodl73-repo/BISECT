#!/usr/bin/env python3
"""
Visualize district compactness scores.

Creates maps showing Polsby-Popper and Reock compactness scores
for all districts in a state.

Usage:
    python scripts/compactness/visualize_compactness.py <state_dir> --census-year 2020 --dpi 150
"""

import sys
import argparse
from pathlib import Path
import pandas as pd
import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.cm as cm
from matplotlib.colors import Normalize
import pickle
import numpy as np


def create_compactness_map(tracts_gdf, metric_name, metric_col, output_file, dpi=150):
    """Create a compactness visualization map"""
    
    fig, ax = plt.subplots(1, 1, figsize=(12, 10))
    
    # Color by compactness score (red = low, green = high)
    cmap = cm.get_cmap('RdYlGn')
    norm = Normalize(vmin=0, vmax=1)
    
    tracts_gdf.plot(
        ax=ax,
        column=metric_col,
        cmap=cmap,
        norm=norm,
        edgecolor='white',
        linewidth=0.1,
        alpha=0.9,
        missing_kwds={'color': 'lightgray'}
    )
    
    # Add district boundaries
    districts_gdf = tracts_gdf.dissolve(by='district')
    districts_gdf.boundary.plot(ax=ax, linewidth=1.0, edgecolor='black', alpha=0.7)
    
    ax.set_axis_off()
    
    # Add title
    state_name = tracts_gdf['state'].iloc[0].replace('_', ' ').title()
    num_districts = len(tracts_gdf['district'].unique())
    avg_score = tracts_gdf.drop_duplicates('district')[metric_col].mean()
    
    plt.title(f'{state_name} - {metric_name} Compactness\n{num_districts} Districts | Average: {avg_score:.3f}',
              fontsize=16, fontweight='bold', pad=20)
    
    # Add colorbar
    sm = cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = plt.colorbar(sm, ax=ax, orientation='horizontal', pad=0.05, aspect=50, shrink=0.8)
    cbar.set_label(f'{metric_name} Score (0 = irregular, 1 = compact)', fontsize=12)
    
    plt.tight_layout()
    output_file.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()


def main():
    parser = argparse.ArgumentParser(description='Visualize district compactness')
    parser.add_argument('state_dir', type=str, help='State directory path')
    parser.add_argument('--census-year', type=str, required=True,
                       choices=['2010', '2020'],
                       help='Census year')
    parser.add_argument('--dpi', type=int, default=150,
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps')
    
    args = parser.parse_args()
    
    state_dir = Path(args.state_dir)
    state_name = state_dir.name
    
    # Load district_summary.csv for compactness data
    summary_file = state_dir / 'district_summary.csv'
    if not summary_file.exists():
        print(f"ERROR: {summary_file} not found")
        return 1
    
    summary_df = pd.read_csv(summary_file)
    
    # Check if compactness columns exist
    if 'polsby_popper' not in summary_df.columns:
        print(f"ERROR: polsby_popper column not found in {summary_file}")
        return 1
    
    # Load tract geometries
    state_abbrev = state_name[:2].upper() if len(state_name) > 2 else state_name.upper()
    
    # Try common state abbreviations
    state_abbrev_map = {
        'california': 'ca', 'texas': 'tx', 'florida': 'fl', 'new_york': 'ny',
        'pennsylvania': 'pa', 'illinois': 'il', 'ohio': 'oh', 'georgia': 'ga',
        'north_carolina': 'nc', 'michigan': 'mi', 'new_jersey': 'nj', 'virginia': 'va',
        'washington': 'wa', 'arizona': 'az', 'massachusetts': 'ma', 'tennessee': 'tn',
        'indiana': 'in', 'maryland': 'md', 'missouri': 'mo', 'wisconsin': 'wi',
        'colorado': 'co', 'minnesota': 'mn', 'south_carolina': 'sc', 'alabama': 'al',
        'louisiana': 'la', 'kentucky': 'ky', 'oregon': 'or', 'oklahoma': 'ok',
        'connecticut': 'ct', 'utah': 'ut', 'iowa': 'ia', 'nevada': 'nv',
        'arkansas': 'ar', 'mississippi': 'ms', 'kansas': 'ks', 'new_mexico': 'nm',
        'nebraska': 'ne', 'idaho': 'id', 'west_virginia': 'wv', 'hawaii': 'hi',
        'new_hampshire': 'nh', 'maine': 'me', 'rhode_island': 'ri', 'montana': 'mt',
        'delaware': 'de', 'south_dakota': 'sd', 'north_dakota': 'nd', 'alaska': 'ak',
        'vermont': 'vt', 'wyoming': 'wy'
    }
    
    state_code = state_abbrev_map.get(state_name, state_name[:2]).lower()
    tracts_file = Path(f'data/raw/{state_code}_tracts_{args.census_year}.parquet')
    
    if not tracts_file.exists():
        print(f"ERROR: {tracts_file} not found")
        return 1
    
    tracts = gpd.read_parquet(tracts_file)
    
    # Load assignments
    assignments_file = state_dir / 'final_assignments.pkl'
    if not assignments_file.exists():
        print(f"ERROR: {assignments_file} not found")
        return 1
    
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)
    
    tracts['district'] = tracts.index.map(assignments)
    tracts['state'] = state_name
    
    # Merge compactness data
    tracts = tracts.merge(
        summary_df[['district', 'polsby_popper', 'reock']],
        on='district',
        how='left'
    )
    
    # Create output directory
    output_dir = state_dir / 'compactness_analysis' / 'maps'
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Creating compactness visualizations for {state_name.title()}...")
    
    # Create Polsby-Popper map
    pp_output = output_dir / f'polsby_popper_districts_{args.census_year}.png'
    create_compactness_map(tracts, 'Polsby-Popper', 'polsby_popper', pp_output, args.dpi)
    print(f"  Saved: {pp_output}")
    
    # Create Reock map
    reock_output = output_dir / f'reock_districts_{args.census_year}.png'
    create_compactness_map(tracts, 'Reock', 'reock', reock_output, args.dpi)
    print(f"  Saved: {reock_output}")
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
