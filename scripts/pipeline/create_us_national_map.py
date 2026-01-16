#!/usr/bin/env python3
"""
Create a national US map showing all 435 congressional districts.
"""

import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import pickle
from pathlib import Path
import argparse
import sys
import os
from tqdm import tqdm

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Import configuration files
try:
    from scripts.config_2020 import STATE_CONFIG_2020
except ImportError:
    STATE_CONFIG_2020 = None

try:
    from scripts.config_2010 import STATE_CONFIG_2010
except ImportError:
    STATE_CONFIG_2010 = None

try:
    from scripts.config_2000 import STATE_CONFIG_2000
except ImportError:
    STATE_CONFIG_2000 = None

# Fallback hardcoded config (2020)
STATE_CONFIG_FALLBACK = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
    'NY': {'name': 'New York', 'districts': 26},
    'PA': {'name': 'Pennsylvania', 'districts': 17},
    'IL': {'name': 'Illinois', 'districts': 17},
    'OH': {'name': 'Ohio', 'districts': 15},
    'GA': {'name': 'Georgia', 'districts': 14},
    'NC': {'name': 'North Carolina', 'districts': 14},
    'MI': {'name': 'Michigan', 'districts': 13},
    'NJ': {'name': 'New Jersey', 'districts': 12},
    'VA': {'name': 'Virginia', 'districts': 11},
    'WA': {'name': 'Washington', 'districts': 10},
    'AZ': {'name': 'Arizona', 'districts': 9},
    'MA': {'name': 'Massachusetts', 'districts': 9},
    'TN': {'name': 'Tennessee', 'districts': 9},
    'IN': {'name': 'Indiana', 'districts': 9},
    'MD': {'name': 'Maryland', 'districts': 8},
    'MO': {'name': 'Missouri', 'districts': 8},
    'WI': {'name': 'Wisconsin', 'districts': 8},
    'CO': {'name': 'Colorado', 'districts': 8},
    'MN': {'name': 'Minnesota', 'districts': 8},
    'SC': {'name': 'South Carolina', 'districts': 7},
    'AL': {'name': 'Alabama', 'districts': 7},
    'LA': {'name': 'Louisiana', 'districts': 6},
    'KY': {'name': 'Kentucky', 'districts': 6},
    'OR': {'name': 'Oregon', 'districts': 6},
    'OK': {'name': 'Oklahoma', 'districts': 5},
    'CT': {'name': 'Connecticut', 'districts': 5},
    'UT': {'name': 'Utah', 'districts': 4},
    'IA': {'name': 'Iowa', 'districts': 4},
    'NV': {'name': 'Nevada', 'districts': 4},
    'AR': {'name': 'Arkansas', 'districts': 4},
    'MS': {'name': 'Mississippi', 'districts': 4},
    'KS': {'name': 'Kansas', 'districts': 4},
    'NM': {'name': 'New Mexico', 'districts': 3},
    'NE': {'name': 'Nebraska', 'districts': 3},
    'ID': {'name': 'Idaho', 'districts': 2},
    'WV': {'name': 'West Virginia', 'districts': 2},
    'HI': {'name': 'Hawaii', 'districts': 2},
    'NH': {'name': 'New Hampshire', 'districts': 2},
    'ME': {'name': 'Maine', 'districts': 2},
    'RI': {'name': 'Rhode Island', 'districts': 2},
    'MT': {'name': 'Montana', 'districts': 2},
    'DE': {'name': 'Delaware', 'districts': 1},
    'SD': {'name': 'South Dakota', 'districts': 1},
    'ND': {'name': 'North Dakota', 'districts': 1},
    'AK': {'name': 'Alaska', 'districts': 1},
    'VT': {'name': 'Vermont', 'districts': 1},
    'WY': {'name': 'Wyoming', 'districts': 1},
}


def load_all_states_with_districts(us_dir=None, year='2020', state_config=None, show_progress=True):
    """Load all states with their district assignments."""

    if us_dir is None:
        us_dir = Path(f'outputs/us_{year}_v2')
    else:
        us_dir = Path(us_dir)

    if state_config is None:
        state_config = STATE_CONFIG_FALLBACK

    all_tracts = []

    # Get position from parent (or 0 if standalone)
    position = int(os.environ.get('TQDM_POSITION', '0'))

    # Iterate through states with progress bar if requested
    state_items = sorted(state_config.items(), key=lambda x: x[1]['districts'], reverse=True)
    if show_progress:
        state_items = tqdm(state_items,
                          desc="  Loading states" if position > 0 else "Loading states",
                          unit="state",
                          position=position,
                          ncols=100,
                          leave=False)

    for state_code, config in state_items:
        state_name = config['name']
        num_districts = config['districts']

        state_dir = us_dir / 'states' / state_name.lower().replace(' ', '_')

        # Load tracts (unified directory structure)
        state_code_lower = state_code.lower()
        tracts_file = f'data/tracts/{year}/{state_code_lower}_tracts_{year}.parquet'

        assignments_file = state_dir / 'data' / 'final_assignments.pkl'

        if not Path(tracts_file).exists():
            continue

        tracts = gpd.read_parquet(tracts_file)

        if assignments_file.exists():
            with open(assignments_file, 'rb') as f:
                assignments = pickle.load(f)
            tracts['district'] = [assignments[i] for i in range(len(tracts))]
        else:
            # Single-district state
            tracts['district'] = 1

        tracts['state_code'] = state_code
        tracts['state_name'] = state_name

        all_tracts.append(tracts)

    us_tracts = pd.concat(all_tracts, ignore_index=True)

    # Create unique district IDs across all states
    us_tracts['unique_district_id'] = (us_tracts.groupby(['state_code', 'district']).ngroup() + 1)

    return us_tracts


def create_national_map(us_tracts):
    """Create national map showing all 435 districts with separate panels for AK and HI."""

    import matplotlib.patheffects as path_effects
    import numpy as np

    # Separate continental US, Alaska, and Hawaii
    continental = us_tracts[~us_tracts['state_code'].isin(['AK', 'HI'])].copy()
    alaska = us_tracts[us_tracts['state_code'] == 'AK'].copy()
    hawaii = us_tracts[us_tracts['state_code'] == 'HI'].copy()

    # Create figure with 3 panels
    fig = plt.figure(figsize=(28, 18))

    # Main continental US (takes most of the space)
    ax_main = plt.subplot2grid((4, 4), (0, 0), colspan=4, rowspan=3)

    # Alaska inset (bottom left)
    ax_alaska = plt.subplot2grid((4, 4), (3, 0), colspan=1, rowspan=1)

    # Hawaii inset (bottom left-center)
    ax_hawaii = plt.subplot2grid((4, 4), (3, 1), colspan=1, rowspan=1)

    # Use extended color palette
    colors = list(cm.tab20.colors) + list(cm.tab20b.colors) + list(cm.tab20c.colors)

    # Calculate district areas for label sizing
    # Reproject to US Albers Equal Area (EPSG:5070) for accurate area calculation
    us_tracts_projected = us_tracts.to_crs('EPSG:5070')
    us_tracts['area'] = us_tracts_projected.geometry.area
    district_areas = us_tracts.groupby('unique_district_id')['area'].sum()

    def plot_region(tracts_data, ax, region_name, base_fontsize=6):
        """Plot a region with districts and labels."""
        if len(tracts_data) == 0:
            return

        # Get unique districts in this region
        districts_in_region = tracts_data['unique_district_id'].unique()

        # Plot each district with thin white tract boundaries
        for district_id in districts_in_region:
            district_data = tracts_data[tracts_data['unique_district_id'] == district_id]
            if len(district_data) > 0:
                district_data.plot(
                    ax=ax,
                    color=colors[(district_id - 1) % len(colors)],
                    edgecolor='white',
                    linewidth=0.1,
                    alpha=0.8
                )

        # Add thick district boundaries on top
        districts_dissolved = tracts_data.dissolve(by='unique_district_id', as_index=False)
        districts_dissolved.boundary.plot(
            ax=ax,
            edgecolor='black',
            linewidth=1.0,
            zorder=10
        )

        # Add district labels with size proportional to district area

        # Get area range for this region
        region_areas = district_areas[districts_in_region]
        min_area = region_areas.min()
        max_area = region_areas.max()
        area_range = max_area - min_area if max_area > min_area else 1

        for district_id in districts_in_region:
            district_data = tracts_data[tracts_data['unique_district_id'] == district_id]
            if len(district_data) > 0:
                try:
                    centroid = district_data.geometry.union_all().representative_point()

                    # Scale font size based on district area (relative to region)
                    area = district_areas[district_id]
                    area_ratio = (area - min_area) / area_range if area_range > 0 else 0.5
                    fontsize = base_fontsize * (0.5 + area_ratio)  # 0.5x to 1.5x base size

                    text = ax.text(centroid.x, centroid.y, str(district_id),
                            fontsize=fontsize, fontweight='bold', ha='center', va='center',
                            color='white', zorder=10)
                    text.set_path_effects([
                        path_effects.Stroke(linewidth=fontsize/4, foreground='black'),
                        path_effects.Normal()
                    ])
                except:
                    pass

        # Add state boundaries
        region_states = tracts_data.dissolve(by='state_code')
        region_states.boundary.plot(
            ax=ax,
            linewidth=0.8,
            edgecolor='black',
            alpha=0.6
        )

        ax.set_axis_off()

    # Plot continental US
    plot_region(continental, ax_main, "Continental US", base_fontsize=4)

    # Plot Alaska
    plot_region(alaska, ax_alaska, "Alaska", base_fontsize=12)
    ax_alaska.set_title('Alaska', fontsize=10, fontweight='bold')

    # Plot Hawaii
    plot_region(hawaii, ax_hawaii, "Hawaii", base_fontsize=12)
    ax_hawaii.set_title('Hawaii', fontsize=10, fontweight='bold')

    # Main title
    fig.suptitle('United States Congressional Districts - 435 Districts
2020 Census Algorithmic Redistricting',
                 fontsize=22, fontweight='bold', y=0.98)

    # Stats text box removed for cleaner visualization


    # Information available in CSV data files

    plt.tight_layout(rect=[0, 0, 1, 0.97])

    return fig


def create_national_map_with_cities(us_tracts, us_dir=None, year='2020'):
    """Create national map with city markers and labels."""

    import matplotlib.patheffects as path_effects
    import numpy as np

    # Load city data
    if us_dir is None:
        us_dir = Path(f'outputs/us_{year}_v2')
    else:
        us_dir = Path(us_dir)

    # Check if cities file exists (created by create_us_aggregate.py)
    cities_file = us_dir / 'data' / 'us_all_districts.csv'
    if not cities_file.exists():
        print(f"Warning: {cities_file} not found. Run create_us_aggregate.py first.")
        print("Creating map without city labels...")
        return create_national_map(us_tracts)

    cities_df = pd.read_csv(cities_file)

    # Separate continental US, Alaska, and Hawaii
    continental = us_tracts[~us_tracts['state_code'].isin(['AK', 'HI'])].copy()
    alaska = us_tracts[us_tracts['state_code'] == 'AK'].copy()
    hawaii = us_tracts[us_tracts['state_code'] == 'HI'].copy()

    continental_cities = cities_df[~cities_df['state_code'].isin(['AK', 'HI'])].copy()
    alaska_cities = cities_df[cities_df['state_code'] == 'AK'].copy()
    hawaii_cities = cities_df[cities_df['state_code'] == 'HI'].copy()

    # Create figure with 3 panels
    fig = plt.figure(figsize=(28, 18))

    # Main continental US
    ax_main = plt.subplot2grid((4, 4), (0, 0), colspan=4, rowspan=3)
    ax_alaska = plt.subplot2grid((4, 4), (3, 0), colspan=1, rowspan=1)
    ax_hawaii = plt.subplot2grid((4, 4), (3, 1), colspan=1, rowspan=1)

    # Use extended color palette
    colors = list(cm.tab20.colors) + list(cm.tab20b.colors) + list(cm.tab20c.colors)

    # Calculate district areas for label sizing
    # Reproject to US Albers Equal Area (EPSG:5070) for accurate area calculation
    us_tracts_projected = us_tracts.to_crs('EPSG:5070')
    us_tracts['area'] = us_tracts_projected.geometry.area
    district_areas = us_tracts.groupby('unique_district_id')['area'].sum()

    def plot_region_with_cities(tracts_data, cities_data, ax, region_name, base_fontsize=6, marker_size=8):
        """Plot a region with districts, cities, and labels."""
        if len(tracts_data) == 0:
            return

        # Get unique districts in this region
        districts_in_region = tracts_data['unique_district_id'].unique()

        # Plot each district with thin white tract boundaries
        for district_id in districts_in_region:
            district_data = tracts_data[tracts_data['unique_district_id'] == district_id]
            if len(district_data) > 0:
                district_data.plot(
                    ax=ax,
                    color=colors[(district_id - 1) % len(colors)],
                    edgecolor='white',
                    linewidth=0.1,
                    alpha=0.8
                )

        # Add thick district boundaries on top
        districts_dissolved = tracts_data.dissolve(by='unique_district_id', as_index=False)
        districts_dissolved.boundary.plot(
            ax=ax,
            edgecolor='black',
            linewidth=1.0,
            zorder=10
        )

        # Add city markers and labels

        # Get area range for this region
        region_areas = district_areas[districts_in_region]
        min_area = region_areas.min()
        max_area = region_areas.max()
        area_range = max_area - min_area if max_area > min_area else 1

        for district_id in districts_in_region:
            district_data = tracts_data[tracts_data['unique_district_id'] == district_id]
            if len(district_data) > 0:
                # Get city info for this district
                city_row = cities_data[cities_data['unique_district_id'] == district_id]

                if len(city_row) > 0 and not pd.isna(city_row.iloc[0]['city_lon']):
                    city_info = city_row.iloc[0]
                    city_name = city_info['largest_city']
                    city_lon = city_info['city_lon']
                    city_lat = city_info['city_lat']

                    # Plot city marker (small red dot)
                    ax.plot(city_lon, city_lat, 'o', color='red', markersize=marker_size*0.5,
                           zorder=15, markeredgecolor='white', markeredgewidth=0.5)

                    # Scale font size based on district area
                    area = district_areas[district_id]
                    area_ratio = (area - min_area) / area_range if area_range > 0 else 0.5
                    fontsize = base_fontsize * (0.5 + area_ratio)

                    # Create label: "district_id: city_name"
                    label = f"{district_id}: {city_name}"

                    # Add label without background
                    text = ax.text(city_lon, city_lat, label,
                            fontsize=fontsize, fontweight='bold', ha='left', va='center',
                            color='black', zorder=16)
                    text.set_path_effects([
                        path_effects.Stroke(linewidth=fontsize/3, foreground='white'),
                        path_effects.Normal()
                    ])
                else:
                    # No city - just show district number at centroid
                    try:
                        centroid = district_data.geometry.union_all().representative_point()
                        area = district_areas[district_id]
                        area_ratio = (area - min_area) / area_range if area_range > 0 else 0.5
                        fontsize = base_fontsize * (0.5 + area_ratio)

                        text = ax.text(centroid.x, centroid.y, str(district_id),
                                fontsize=fontsize, fontweight='bold', ha='center', va='center',
                                color='white', zorder=10)
                        text.set_path_effects([
                            path_effects.Stroke(linewidth=fontsize/4, foreground='black'),
                            path_effects.Normal()
                        ])
                    except:
                        pass

        # Add state boundaries
        region_states = tracts_data.dissolve(by='state_code')
        region_states.boundary.plot(
            ax=ax,
            linewidth=0.8,
            edgecolor='black',
            alpha=0.6
        )

        ax.set_axis_off()

    # Match districts with unique IDs
    cities_df['unique_district_id'] = cities_df.apply(
        lambda row: us_tracts[
            (us_tracts['state_code'] == row['state_code']) &
            (us_tracts['district'] == row['district'])
        ]['unique_district_id'].iloc[0] if len(us_tracts[
            (us_tracts['state_code'] == row['state_code']) &
            (us_tracts['district'] == row['district'])
        ]) > 0 else None,
        axis=1
    )

    # Update the regional city dataframes
    continental_cities = cities_df[~cities_df['state_code'].isin(['AK', 'HI'])].copy()
    alaska_cities = cities_df[cities_df['state_code'] == 'AK'].copy()
    hawaii_cities = cities_df[cities_df['state_code'] == 'HI'].copy()

    # Plot all regions
    plot_region_with_cities(continental, continental_cities, ax_main, "Continental US",
                           base_fontsize=3.5, marker_size=6)
    plot_region_with_cities(alaska, alaska_cities, ax_alaska, "Alaska",
                           base_fontsize=10, marker_size=12)
    ax_alaska.set_title('Alaska', fontsize=10, fontweight='bold')

    plot_region_with_cities(hawaii, hawaii_cities, ax_hawaii, "Hawaii",
                           base_fontsize=10, marker_size=12)
    ax_hawaii.set_title('Hawaii', fontsize=10, fontweight='bold')

    # Main title
    fig.suptitle('United States Congressional Districts - 435 Districts with Cities
2020 Census Algorithmic Redistricting',
                 fontsize=22, fontweight='bold', y=0.98)

    # Stats text box removed for cleaner visualization


    # Information available in CSV data files

    plt.tight_layout(rect=[0, 0, 1, 0.97])

    return fig


def main(output_dir=None, year='2020', print_only=False, debug=False, force=False, dpi=150):
    """Create national US maps."""

    # Only print header if running standalone (not called from parent)
    is_standalone = not os.environ.get('TQDM_POSITION')

    # Select the correct STATE_CONFIG based on year
    if year == '2020':
        STATE_CONFIG = STATE_CONFIG_2020 if STATE_CONFIG_2020 else STATE_CONFIG_FALLBACK
    elif year == '2010':
        STATE_CONFIG = STATE_CONFIG_2010 if STATE_CONFIG_2010 else STATE_CONFIG_FALLBACK
    elif year == '2000':
        STATE_CONFIG = STATE_CONFIG_2000 if STATE_CONFIG_2000 else STATE_CONFIG_FALLBACK
    else:
        if is_standalone:
            print(f"ERROR: Unsupported year {year}")
        sys.exit(1)

    if output_dir is None:
        us_dir = Path(f'outputs/us_{year}_v2')
    else:
        us_dir = Path(output_dir)

    if is_standalone:
        print("
" + "="*70)
        print(f"Creating US National Congressional Districts Maps - {year} Census")
        print("="*70)
        print(f"Output directory: {us_dir}")
        print("="*70)

    # Create maps directory if it doesn't exist
    maps_dir = us_dir / 'maps'
    maps_dir.mkdir(parents=True, exist_ok=True)

    # Check if outputs already exist
    output_file = maps_dir / 'us_all_districts.png'
    output_file_cities = maps_dir / 'us_all_districts_with_cities.png'

    # In print-only mode, skip actual work but show progress bars
    if print_only:
        import time
        position = int(os.environ.get('TQDM_POSITION', '0'))

        pbar = tqdm(total=50,
                   desc="  Loading states" if position > 0 else "Loading states",
                   unit="state",
                   position=position,
                   ncols=100,
                   leave=False)

        if debug:
            for i in range(50):
                time.sleep(0.05)  # Slower since loading is expensive
                pbar.update(1)
        else:
            pbar.update(50)

        pbar.close()
        return

    # If outputs already exist, skip (but show progress bar completion)
    if not force and output_file.exists() and output_file_cities.exists():
        if is_standalone:
            print("
US national maps already exist - skipping")
            print(f"  {output_file.name}")
            print(f"  {output_file_cities.name}")
            print("
Use --force to regenerate")
        position = int(os.environ.get('TQDM_POSITION', '0'))
        with tqdm(total=4,
                  desc="  Creating maps (skipped - exists)" if position > 0 else "Creating maps (skipped - exists)",
                  unit="step",
                  position=position,
                  ncols=100,
                  leave=False) as map_pbar:
            map_pbar.update(4)  # Instant completion
        return

    # Get position for map creation progress
    position = int(os.environ.get('TQDM_POSITION', '0'))

    # Check if we should send status messages to parent
    send_status = position > 0

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Load all states (with progress bar)
    report_progress("Create US national maps - Loading 50 states...")
    us_tracts = load_all_states_with_districts(us_dir, year=year, state_config=STATE_CONFIG, show_progress=True)

    # Create map without cities (matplotlib plotting + rendering ~2-3 min)
    report_progress("Create US national maps - Plotting 435 districts (1/4)")
    fig = create_national_map(us_tracts)

    # Save basic map (writing high-res PNG ~1-2 min)
    report_progress("Create US national maps - Saving basic map (2/4)")
    fig.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    # Create map with cities (matplotlib plotting + cities ~3-4 min)
    report_progress("Create US national maps - Plotting with cities (3/4)")
    fig_cities = create_national_map_with_cities(us_tracts, us_dir, year=year)

    # Save cities map (writing high-res PNG ~1-2 min)
    report_progress("Create US national maps - Saving cities map (4/4)")
    fig_cities.savefig(output_file_cities, dpi=dpi, bbox_inches='tight')
    plt.close()

    # Print summary only at the end and only if standalone
    if is_standalone:
        print("
" + "="*70)
        print("SUCCESS! Both maps created")
        print("="*70)
        print(f"  1. {output_file}")
        print(f"  2. {output_file_cities}")
        print("="*70)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Create US national maps')
    parser.add_argument('--output-dir', type=str, help='Output directory (default: outputs/us_YEAR_v2)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                        help='Census year (default: 2020)')
    parser.add_argument('--print-only', action='store_true',
                        help='Print what would be done without executing')
    parser.add_argument('--debug', action='store_true',
                        help='Enable debug mode with progress delays and file display')
    parser.add_argument('--force', action='store_true',
                        help='Force regeneration even if outputs exist')
    parser.add_argument('--dpi', type=int, default=150, choices=[72, 100, 150, 200, 300],
                        help='DPI for output maps (default: 150)')
    args = parser.parse_args()

    main(args.output_dir, args.year, args.print_only, args.debug, args.force, args.dpi)
