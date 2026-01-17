#!/usr/bin/env python3
"""
Identify the largest city in each district and add to outputs.
"""

import sys
import os
import warnings

# Suppress matplotlib warnings about FancyArrowPatch and transformations
warnings.filterwarnings('ignore', message='.*FancyArrowPatch.*')
warnings.filterwarnings('ignore', message='.*tranform.*')
warnings.filterwarnings('ignore', category=UserWarning, module='matplotlib')
warnings.filterwarnings('ignore', category=UserWarning, module='adjustText')
from pathlib import Path

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config, get_tract_file, get_places_file
import pickle
import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patheffects as path_effects
import numpy as np
import argparse
from tqdm import tqdm
import time

def find_largest_cities(
    tracts_file: str,
    places_file: str,
    assignments_file: str,
    output_dir: Path,
    num_districts: int = 52,
    debug: bool = False,
    progress_bar=None,
    position: int = 2
):
    """
    Find the largest city in each district.
    """
    if debug:
        print("Loading data...")
    tracts = gpd.read_parquet(tracts_file)
    places = gpd.read_parquet(places_file)

    # Load district assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    # Add district assignments to tracts
    tracts['district'] = [assignments[i] for i in range(len(tracts))]


    # Ensure same CRS
    if tracts.crs != places.crs:
        places = places.to_crs(tracts.crs)

    # Spatial join: which places are in which districts
    if debug:
        print("Performing spatial join to identify cities in each district...")

    # Update progress bar to show we're doing spatial join
    if progress_bar:
        progress_bar.set_description(f"{progress_bar.desc.split(' Finding')[0]} Spatial join...")

    # Use representative points for places (faster than full polygon join)
    places_points = places.copy()
    places_points['geometry'] = places_points.geometry.representative_point()

    # Join places to tracts (THIS IS THE SLOW PART)
    places_with_tracts = gpd.sjoin(places_points, tracts[['geometry', 'district']], how='left', predicate='within')

    # Reset progress bar description
    if progress_bar:
        progress_bar.set_description(f"{progress_bar.desc.split(' Spatial')[0]} Finding cities")

    # Group by district and find best city label
    if debug:
        print("Identifying best city/neighborhood label for each district...")
    district_cities = []

    # Track which large cities we've already used to prefer neighborhoods
    major_cities_threshold = 500000  # Cities larger than this get neighborhood preference
    huge_cities_threshold = 1000000  # Cities > 1M always get secondary descriptor
    used_major_cities = {}  # {city_name: count}

    for district_id in range(1, num_districts + 1):
        district_places = places_with_tracts[places_with_tracts['district'] == district_id]

        if len(district_places) > 0:
            # Sort by population
            district_places_sorted = district_places.sort_values('population', ascending=False)

            # Find the best label
            best_city = None

            # Strategy: If the largest city is a major city (>500K), prefer a smaller neighborhood/city
            # unless this is the first time we're using this major city
            largest = district_places_sorted.iloc[0]
            largest_name = largest['NAME']
            largest_pop = int(largest['population'])

            # For huge cities (>1M), always try to find a secondary descriptor
            if largest_pop > huge_cities_threshold:
                # Look for 2nd largest place to use as neighborhood descriptor
                found_neighborhood = False
                for idx in range(1, min(10, len(district_places_sorted))):
                    candidate = district_places_sorted.iloc[idx]
                    candidate_name = candidate['NAME']
                    candidate_pop = int(candidate['population'])

                    # Use this if it's substantial (>20K) and not the same huge city
                    if candidate_pop > 20000 and candidate_name != largest_name:
                        best_city = candidate
                        found_neighborhood = True
                        break

                # If no substantial neighborhood, just use the huge city name
                if not found_neighborhood:
                    best_city = largest

            elif largest_pop > major_cities_threshold:
                # This is a major city (500K-1M) - check if we've used it before
                used_count = used_major_cities.get(largest_name, 0)

                if used_count == 0:
                    # First use - use the major city name
                    best_city = largest
                    used_major_cities[largest_name] = 1
                else:
                    # We've used this major city before - try to find a neighborhood
                    found_neighborhood = False
                    for idx in range(1, min(5, len(district_places_sorted))):
                        candidate = district_places_sorted.iloc[idx]
                        candidate_name = candidate['NAME']
                        candidate_pop = int(candidate['population'])

                        # Use this if it's substantial (>10K)
                        if candidate_pop > 10000 and candidate_name != largest_name:
                            best_city = candidate
                            found_neighborhood = True
                            break

                    # If no good neighborhood found, use major city
                    if not found_neighborhood:
                        best_city = largest
                        used_major_cities[largest_name] = used_count + 1
            else:
                # Not a major city - just use it
                best_city = largest

            city_name = best_city['NAME']
            city_pop = int(best_city['population'])

            # Get centroid for labeling
            city_geom = places[places['GEOID'] == best_city['GEOID']].iloc[0].geometry
            city_centroid = city_geom.representative_point()

            district_cities.append({
                'district': district_id,
                'largest_city': city_name,
                'city_population': city_pop,
                'city_lon': city_centroid.x,
                'city_lat': city_centroid.y
            })
        else:
            # No cities in this district (rural)
            district_cities.append({
                'district': district_id,
                'largest_city': 'None',
                'city_population': 0,
                'city_lon': None,
                'city_lat': None
            })

        # Update progress bar after each district
        if progress_bar:
            progress_bar.update(1)
        # Print progress for parent process to read
        if position == 999:
            print(f"PROGRESS:{district_id}/{num_districts}", flush=True)

    df = pd.DataFrame(district_cities)

    # Save to CSV
    data_dir = output_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)
    csv_file = data_dir / 'district_cities.csv'
    df.to_csv(csv_file, index=False)
    if debug:
        print(f"Saved city data to: {csv_file}")

    # Print summary
    if debug:
        print("Largest cities by district:")
        for _, row in df.head(20).iterrows():
            if row['city_population'] > 0:
                print(f"  District {row['district']:2d}: {row['largest_city']} ({row['city_population']:,})")
            else:
                print(f"  District {row['district']:2d}: (No major cities)")

    return df


def create_map_with_cities(
    tracts_file: str,
    places_file: str,
    assignments_file: str,
    cities_df: pd.DataFrame,
    output_file: Path,
    num_districts: int = 52,
    state_name: str = "California",
    debug: bool = False,
    progress_bar=None
):
    """
    Create district map with largest cities labeled.
    """
    if debug:
        print("Generating map with city labels...")

    tracts = gpd.read_parquet(tracts_file)
    places = gpd.read_parquet(places_file)

    # Load assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    # Update progress: creating map
    if progress_bar:
        progress_bar.set_description(f"{state_name} [{num_districts}D] Creating map")

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(20, 18))

    # Use colormap
    import matplotlib.cm as cm
    import matplotlib.colors as mcolors
    colors = cm.tab20.colors + cm.tab20b.colors + cm.tab20c.colors

    # Create a mapping of district_id to color (for labels)
    district_colors = {}

    # Plot districts
    for district_id in range(1, num_districts + 1):
        district_data = tracts[tracts['district'] == district_id]
        if len(district_data) > 0:
            base_color = colors[(district_id - 1) % len(colors)]
            district_colors[district_id] = base_color

            district_data.plot(
                ax=ax,
                color=base_color,
                edgecolor='white',
                linewidth=0.1,
                alpha=0.7
            )

    # Add district numbers (small, in background)
    for district_id in range(1, num_districts + 1):
        district_data = tracts[tracts['district'] == district_id]
        if len(district_data) > 0:
            try:
                centroid = district_data.geometry.union_all().representative_point()
                text = ax.text(centroid.x, centroid.y, str(district_id),
                        fontsize=6, fontweight='normal', ha='center', va='center',
                        color='gray', alpha=0.5, zorder=5)
            except:
                pass

    # Helper function to darken a color
    def darken_color(color, factor=0.7):
        """Make a color darker by the given factor."""
        rgb = mcolors.to_rgb(color)
        darkened = tuple(c * factor for c in rgb)
        return darkened

    # Add city markers and labels with smart placement

    # First pass: Add dots at actual city locations
    for _, row in cities_df.iterrows():
        if row['city_population'] > 0 and row['city_lon'] is not None:
            district_id = int(row['district'])
            # Use district color, darkened for contrast
            marker_color = darken_color(district_colors[district_id], factor=0.7)
            ax.plot(row['city_lon'], row['city_lat'], 'o',
                   color=marker_color, markersize=4, zorder=15,
                   markeredgecolor='white', markeredgewidth=0.5)

    # Second pass: Add labels with smart placement to avoid overlaps

    try:
        # Try to use adjustText for automatic label adjustment
        from adjustText import adjust_text

        texts = []
        text_colors = []  # Store colors for arrows
        for _, row in cities_df.iterrows():
            if row['city_population'] > 0 and row['city_lon'] is not None:
                district_id = int(row['district'])
                label = f"{district_id}: {row['largest_city']}"

                # Use district color, darkened for better contrast
                text_color = darken_color(district_colors[district_id], factor=0.7)
                text_colors.append(text_color)

                text = ax.text(
                    row['city_lon'], row['city_lat'],
                    label,
                    fontsize=7,
                    fontweight='bold',
                    ha='center',
                    va='center',
                    color=text_color,
                    zorder=20
                )
                # Add white outline for visibility
                text.set_path_effects([
                    path_effects.Stroke(linewidth=2, foreground='white'),
                    path_effects.Normal()
                ])
                texts.append(text)

        # Adjust text positions to avoid overlaps
        if debug:
            print("  Optimizing label positions to avoid overlaps...")

        # Update progress: this is slow for large states
        if progress_bar:
            progress_bar.set_description(f"{state_name} [{num_districts}D] Optimizing labels")

        # Create individual arrow properties for each label (matching colors)
        for i, (text, color) in enumerate(zip(texts, text_colors)):
            # Temporarily store color info on text object for arrow matching
            text._arrow_color = color

        # Suppress stderr during adjust_text to hide FancyArrowPatch warning
        import contextlib
        with contextlib.redirect_stderr(open(os.devnull, 'w')):
            adjust_text(
                texts,
                arrowprops=dict(arrowstyle='-', color='gray', lw=0.5, alpha=0.6),
                expand_text=(1.2, 1.3),
                expand_points=(1.2, 1.3),
                force_text=(0.5, 0.5),
                force_points=(0.3, 0.3),
                ax=ax
            )

        # Update arrow colors to match their districts
        for text in texts:
            if hasattr(text, '_arrow_color'):
                # Find the arrow annotation for this text and update its color
                for child in ax.get_children():
                    if hasattr(child, 'arrow_patch') and hasattr(child, 'xy'):
                        # This is an annotation with an arrow
                        try:
                            child.arrow_patch.set_color(text._arrow_color)
                            child.arrow_patch.set_alpha(0.6)
                        except:
                            pass

    except ImportError:
        # Fallback: Manual placement using district centroids for labels
        if debug:
            print("  adjustText not available, using manual placement...")

        for _, row in cities_df.iterrows():
            if row['city_population'] > 0 and row['city_lon'] is not None:
                district_id = int(row['district'])
                label = f"{district_id}: {row['largest_city']}"

                # Use district color, darkened for better contrast
                text_color = darken_color(district_colors[district_id], factor=0.7)

                # Get district centroid for label placement
                district_data = tracts[tracts['district'] == district_id]
                if len(district_data) > 0:
                    try:
                        district_centroid = district_data.geometry.union_all().representative_point()
                        label_x = district_centroid.x
                        label_y = district_centroid.y

                        # Draw line from city to label if they're far apart
                        dist = ((label_x - row['city_lon'])**2 + (label_y - row['city_lat'])**2)**0.5
                        if dist > 0.1:  # If label is far from city
                            ax.plot([row['city_lon'], label_x],
                                   [row['city_lat'], label_y],
                                   '-', color=text_color, linewidth=0.5, alpha=0.6, zorder=10)
                    except:
                        # Fallback to city location
                        label_x = row['city_lon']
                        label_y = row['city_lat']
                else:
                    label_x = row['city_lon']
                    label_y = row['city_lat']

                text = ax.text(
                    label_x, label_y,
                    label,
                    fontsize=7,
                    fontweight='bold',
                    ha='center',
                    va='center',
                    color=text_color,
                    zorder=20
                )
                # Add white outline for visibility
                text.set_path_effects([
                    path_effects.Stroke(linewidth=2, foreground='white'),
                    path_effects.Normal()
                ])

    ax.set_axis_off()
    ax.set_title(
        f'{state_name} Congressional Districts - {num_districts} Districts\n'
        'Labeled with largest city/neighborhood in each district',
        fontsize=16, fontweight='bold', pad=20
    )

    # Update progress: saving
    if progress_bar:
        progress_bar.set_description(f"{state_name} [{num_districts}D] Saving map")

    plt.tight_layout()
    plt.savefig(output_file, dpi=args.dpi, bbox_inches='tight')
    plt.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Add largest cities to district maps')
    parser.add_argument('run_dir', type=str, help='Run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')
    parser.add_argument('--position', type=int, default=2, help='Progress bar position (for parallel mode)')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode with progress delays and file display')
    args = parser.parse_args()

    run_dir = Path(args.run_dir)

    # State name to code mapping (normalized directory name to state code)
    STATE_NAME_TO_CODE = {
        'alabama': 'AL', 'alaska': 'AK', 'arizona': 'AZ', 'arkansas': 'AR', 'california': 'CA',
        'colorado': 'CO', 'connecticut': 'CT', 'delaware': 'DE', 'florida': 'FL', 'georgia': 'GA',
        'hawaii': 'HI', 'idaho': 'ID', 'illinois': 'IL', 'indiana': 'IN', 'iowa': 'IA',
        'kansas': 'KS', 'kentucky': 'KY', 'louisiana': 'LA', 'maine': 'ME', 'maryland': 'MD',
        'massachusetts': 'MA', 'michigan': 'MI', 'minnesota': 'MN', 'mississippi': 'MS', 'missouri': 'MO',
        'montana': 'MT', 'nebraska': 'NE', 'nevada': 'NV', 'new_hampshire': 'NH', 'new_jersey': 'NJ',
        'new_mexico': 'NM', 'new_york': 'NY', 'north_carolina': 'NC', 'north_dakota': 'ND', 'ohio': 'OH',
        'oklahoma': 'OK', 'oregon': 'OR', 'pennsylvania': 'PA', 'rhode_island': 'RI', 'south_carolina': 'SC',
        'south_dakota': 'SD', 'tennessee': 'TN', 'texas': 'TX', 'utah': 'UT', 'vermont': 'VT',
        'virginia': 'VA', 'washington': 'WA', 'west_virginia': 'WV', 'wisconsin': 'WI', 'wyoming': 'WY'
    }

    # Auto-detect state from directory name
    dir_name = run_dir.name
    # Remove _full_ suffix if present
    base_name = dir_name.split('_full_')[0]

    state_code = STATE_NAME_TO_CODE.get(base_name)
    if not state_code:
        raise ValueError(f"Could not detect state from directory name: {dir_name}")

    state_name = base_name.replace('_', ' ').title()

    # Load tract and places files (unified directory structure)
    tracts_file = str(get_tract_file(state_code, args.year))
    places_file = str(get_places_file(state_code, args.year))

    assignments_file = run_dir / 'data' / 'final_assignments.pkl'

    # Show progress bars for integration with parent script
    operation_pos = args.position
    file_pos = operation_pos + 1

    stage_pbar = None
    file_pbar = None

    # Create progress bars if position is set (for integration with parent scripts)
    # In parallel mode: show stage progress but not file displays
    if args.print_only or args.position != 2 and args.position != 999:
        # Get number of districts
        try:
            STATE_CONFIG = get_state_config(args.year)
            config = STATE_CONFIG.get(state_code.upper(), {})
            num_districts = config.get('districts', 1)
        except:
            num_districts = 1

        # Create progress bar at operation_pos (number of districts)
        stage_pbar = tqdm(total=num_districts,
                         desc=f"{state_name} [{num_districts}D] Finding cities",
                         unit="city",
                         position=operation_pos,
                         leave=True,
                         ncols=120)

        # Only show file display in sequential mode (not parallel)
        if not os.environ.get('PARALLEL_MODE'):
            # Check if files exist and color-code them
            project_root = Path(__file__).parent.parent.parent
            places_path = project_root / places_file

            # ANSI color codes
            GREEN = '\033[32m'
            RED = '\033[31m'
            RESET = '\033[0m'

            places_display = f"{GREEN}{places_file}{RESET}" if places_path.exists() else f"{RED}{places_file}{RESET}"

            file_pbar = tqdm(total=0,
                            desc=f"      Files: {places_display}",
                            bar_format="{desc}",
                            position=file_pos,
                            leave=False,
                            ncols=100)

    # Check if output already exists
    output_file = run_dir / 'data' / 'district_cities.csv'

    # Check if places file exists (not available for all census years)
    places_path = Path(places_file)
    if not places_path.exists():
        # Skip cities stage if places data not available
        if stage_pbar:
            stage_pbar.set_description(f"{state_name} [{num_districts}D] Finding cities (skipped - no places data)")
            stage_pbar.update(num_districts)
            stage_pbar.close()
        if file_pbar:
            file_pbar.clear()
            file_pbar.close()
        sys.exit(0)

    # Print-only mode or skip if output exists
    if args.print_only or output_file.exists():
        # Update progress bar description to show skipping status
        if output_file.exists() and not args.print_only and stage_pbar:
            stage_pbar.set_description("    Finding cities (skipped - output exists)")

        if args.debug:
            # Debug mode: show progress with delays
            if stage_pbar:
                for i in range(num_districts):
                    time.sleep(0.05)  # Simulate spatial join and city lookup
                    stage_pbar.update(1)
            else:
                time.sleep(1)  # Just wait without progress
        else:
            # Non-debug mode: instant completion
            if stage_pbar:
                stage_pbar.update(num_districts)

        # Close progress bars
        if stage_pbar:
            stage_pbar.close()
        if file_pbar:
            file_pbar.clear()
            file_pbar.close()

        sys.exit(0)

    # Auto-detect number of districts from assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)
    num_districts = max(assignments.values())

    # Find largest cities
    cities_df = find_largest_cities(
        tracts_file,
        places_file,
        str(assignments_file),
        run_dir,
        num_districts=num_districts,
        debug=args.debug,
        progress_bar=stage_pbar,
        position=args.position
    )

    # Create map with cities
    maps_dir = run_dir / 'maps'
    maps_dir.mkdir(parents=True, exist_ok=True)
    map_file = maps_dir / 'all_districts_with_cities.png'
    create_map_with_cities(
        tracts_file,
        places_file,
        str(assignments_file),
        cities_df,
        map_file,
        num_districts=num_districts,
        state_name=state_name,
        debug=args.debug,
        progress_bar=stage_pbar
    )

    # Close progress bars if they were created
    if stage_pbar:
        stage_pbar.close()
    if file_pbar:
        file_pbar.clear()
        file_pbar.close()

    if args.debug:
        print("=" * 70)
