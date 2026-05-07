"""
50-State VRA Threshold Analysis - Comprehensive Expansion

Expands from N=5 to N=50 states to validate the 42% threshold pattern.
Tests edge-weighted optimization across all states with congressional districts.

This addresses P1.4 (sample size) from panel reviews by providing
statistically robust evidence for threshold generalizability.
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from datetime import datetime

# Add paths
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

import geopandas as gpd
from scipy.sparse import lil_matrix
from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable
from apportionment.partition.vra_utils import load_tract_demographics, create_vra_vertex_weights
from scripts.config_2020 import STATE_CONFIG_2020

# Configuration
YEAR = 2020
DATA_DIR = project_root / 'data'
OUTPUT_DIR = Path('results')  # Relative to script location

# Test parameters (focused on most informative configs)
WEIGHT_FACTORS = [1, 5, 10, 50, 100]  # Reduced from 7 to 5 for faster computation
MINORITY_THRESHOLDS = [0.40, 0.45, 0.50]  # Reduced from 4 to 3 for speed

# States to exclude (single-district states: no redistricting needed)
SINGLE_DISTRICT_STATES = ['alaska', 'delaware', 'north_dakota', 'south_dakota',
                          'vermont', 'wyoming', 'montana']

# State FIPS codes (needed for TIGER files)
STATE_FIPS = {
    'alabama': '01', 'alaska': '02', 'arizona': '04', 'arkansas': '05',
    'california': '06', 'colorado': '08', 'connecticut': '09', 'delaware': '10',
    'florida': '12', 'georgia': '13', 'hawaii': '15', 'idaho': '16',
    'illinois': '17', 'indiana': '18', 'iowa': '19', 'kansas': '20',
    'kentucky': '21', 'louisiana': '22', 'maine': '23', 'maryland': '24',
    'massachusetts': '25', 'michigan': '26', 'minnesota': '27', 'mississippi': '28',
    'missouri': '29', 'montana': '30', 'nebraska': '31', 'nevada': '32',
    'new_hampshire': '33', 'new_jersey': '34', 'new_mexico': '35', 'new_york': '36',
    'north_carolina': '37', 'north_dakota': '38', 'ohio': '39', 'oklahoma': '40',
    'oregon': '41', 'pennsylvania': '42', 'rhode_island': '44', 'south_carolina': '45',
    'south_dakota': '46', 'tennessee': '47', 'texas': '48', 'utah': '49',
    'vermont': '50', 'virginia': '51', 'washington': '53', 'west_virginia': '54',
    'wisconsin': '55', 'wyoming': '56'
}


def load_tiger_tracts(state_name, year):
    """Load tract geometries from TIGER shapefiles."""
    fips = STATE_FIPS.get(state_name)
    if not fips:
        raise ValueError(f"Unknown state: {state_name}")

    tiger_path = project_root / 'data' / str(year) / 'tiger' / 'tracts' / f'tl_{year}_{fips}_tract'
    shapefile = tiger_path / f'tl_{year}_{fips}_tract.shp'

    if not shapefile.exists():
        raise FileNotFoundError(f"Tract shapefile not found: {shapefile}")

    tracts_gdf = gpd.read_file(shapefile)

    # Create consistent GEOID (11-character: state+county+tract)
    tracts_gdf['GEOID'] = tracts_gdf['STATEFP'] + tracts_gdf['COUNTYFP'] + tracts_gdf['TRACTCE']

    return tracts_gdf


def compute_state_demographics(state_name):
    """
    Compute state-level minority percentage and district count.

    Returns:
        dict with state_minority_pct, num_districts, state info
    """
    try:
        # Load demographics
        demographics = load_tract_demographics(state_name, year=YEAR, data_dir=str(DATA_DIR))

        # Compute state-wide minority percentage
        total_pop = demographics['total_pop'].sum()
        white_pop = demographics['white_non_hispanic'].sum()
        minority_pop = total_pop - white_pop
        state_minority_pct = minority_pop / total_pop if total_pop > 0 else 0

        # Get district count
        state_upper = state_name.replace('_', ' ').title().replace(' ', '')
        # Convert to abbreviation if needed
        state_abbrev = {
            'california': 'CA', 'texas': 'TX', 'florida': 'FL', 'newyork': 'NY',
            'pennsylvania': 'PA', 'illinois': 'IL', 'ohio': 'OH', 'georgia': 'GA',
            'northcarolina': 'NC', 'michigan': 'MI', 'newjersey': 'NJ', 'virginia': 'VA',
            'washington': 'WA', 'arizona': 'AZ', 'massachusetts': 'MA', 'tennessee': 'TN',
            'indiana': 'IN', 'maryland': 'MD', 'missouri': 'MO', 'wisconsin': 'WI',
            'colorado': 'CO', 'minnesota': 'MN', 'southcarolina': 'SC', 'alabama': 'AL',
            'louisiana': 'LA', 'kentucky': 'KY', 'oregon': 'OR', 'oklahoma': 'OK',
            'connecticut': 'CT', 'utah': 'UT', 'iowa': 'IA', 'nevada': 'NV',
            'arkansas': 'AR', 'mississippi': 'MS', 'kansas': 'KS', 'newmexico': 'NM',
            'nebraska': 'NE', 'idaho': 'ID', 'westvirginia': 'WV', 'hawaii': 'HI',
            'newhampshire': 'NH', 'maine': 'ME', 'rhodeisland': 'RI', 'montana': 'MT',
            'delaware': 'DE', 'southdakota': 'SD', 'northdakota': 'ND', 'alaska': 'AK',
            'vermont': 'VT', 'wyoming': 'WY', 'districtofcolumbia': 'DC'
        }.get(state_upper.lower(), state_name.upper()[:2])

        num_districts = STATE_CONFIG_2020.get(state_abbrev, {}).get('districts', 1)

        # Compute proportional MM target (for analysis purposes)
        target_mm = round(state_minority_pct * num_districts)
        target_mm = max(1, target_mm)  # At least 1 if any significant minority

        return {
            'state': state_name,
            'state_minority_pct': state_minority_pct,
            'num_districts': num_districts,
            'target_mm': target_mm,
            'mm_proportion': target_mm / num_districts,
            'total_pop': total_pop,
            'minority_pop': minority_pop,
        }
    except Exception as e:
        print(f"  [ERROR] {state_name}: {e}")
        return None


def run_edge_weighted_optimization(state_name, state_info, weight_factor, minority_threshold):
    """
    Run edge-weighted optimization for one configuration.

    Returns:
        dict with results or None if failed
    """
    try:
        k = state_info['num_districts']
        target_mm = state_info['target_mm']

        # Load tract data
        tracts_gdf = load_tiger_tracts(state_name, YEAR)
        demographics = load_tract_demographics(state_name, YEAR, data_dir=str(DATA_DIR))

        # Create vertex weights (this also merges demographics with tracts)
        vertex_weights, tracts_with_demo = create_vra_vertex_weights(tracts_gdf, demographics)

        # Add 'population' column for adjacency builder (it expects this name)
        tracts_with_demo['population'] = tracts_with_demo['total_pop']

        # Build adjacency using the merged data (returns adjacency list directly)
        adjacency_list, adj_vertex_weights, index_to_geoid, geoid_to_index, adj_edge_weights = build_adjacency_graph(tracts_with_demo)

        # Identify minority tracts
        is_minority = tracts_with_demo['pct_minority'] >= minority_threshold

        # Create edge weights (iterate over adjacency list)
        edge_weights = {}
        for i, neighbors in enumerate(adjacency_list):
            for j in neighbors:
                if i < j:  # Only process each edge once
                    if is_minority.iloc[i] and is_minority.iloc[j]:
                        edge_weights[(i, j)] = weight_factor  # High weight for minority-minority edges
                    else:
                        edge_weights[(i, j)] = 1.0  # Normal weight

        # Run METIS with edge weights
        partition = partition_graph_with_executable(
            adjacency=adjacency_list,
            vertex_weights=vertex_weights[:, 0],  # Use population only (keep as numpy array)
            nparts=k,
            edge_weights=edge_weights
        )

        # Analyze MM districts
        mm_count = 0
        max_minority_pct = 0
        district_pcts = []

        for district_id in range(k):
            district_mask = [p == district_id for p in partition]
            district_tracts = tracts_with_demo[district_mask]

            if len(district_tracts) > 0:
                district_minority_pct = district_tracts['pct_minority'].mean()
                district_pcts.append(district_minority_pct)
                max_minority_pct = max(max_minority_pct, district_minority_pct)

                if district_minority_pct >= 0.50:
                    mm_count += 1

        # Success criteria: achieved target MM count
        success = mm_count >= target_mm

        return {
            'state': state_name,
            'weight_factor': weight_factor,
            'minority_threshold': minority_threshold,
            'mm_count': mm_count,
            'target_mm': target_mm,
            'success': success,
            'max_minority_pct': max_minority_pct,
            'district_pcts': district_pcts,
        }

    except Exception as e:
        import traceback
        print(f"  [ERROR] {state_name} (w={weight_factor}, t={minority_threshold}): {e}")
        # Always print traceback for debugging
        traceback.print_exc()
        return None


def analyze_state(state_name):
    """
    Run complete analysis for one state: compute demographics and test all configs.

    Returns:
        list of result dicts for all configurations
    """
    print(f"\nAnalyzing {state_name.replace('_', ' ').title()}...")

    # Skip single-district states
    if state_name in SINGLE_DISTRICT_STATES:
        print(f"  [SKIP] Single-district state")
        return []

    # Compute demographics
    state_info = compute_state_demographics(state_name)
    if state_info is None:
        return []

    print(f"  Minority: {state_info['state_minority_pct']:.1%}, Districts: {state_info['num_districts']}, Target MM: {state_info['target_mm']}")

    # Run all configurations
    results = []
    total_configs = len(WEIGHT_FACTORS) * len(MINORITY_THRESHOLDS)

    for i, weight_factor in enumerate(WEIGHT_FACTORS):
        for j, minority_threshold in enumerate(MINORITY_THRESHOLDS):
            config_num = i * len(MINORITY_THRESHOLDS) + j + 1
            print(f"  [{config_num}/{total_configs}] Testing w={weight_factor}, t={minority_threshold:.0%}...", end=' ')

            result = run_edge_weighted_optimization(
                state_name, state_info, weight_factor, minority_threshold
            )

            if result:
                # Add state demographics
                result.update({
                    'state_minority_pct': state_info['state_minority_pct'],
                    'num_districts': state_info['num_districts'],
                    'mm_proportion': state_info['mm_proportion'],
                })
                results.append(result)
                print(f"MM={result['mm_count']}, {'SUCCESS' if result['success'] else 'FAIL'}")
            else:
                print("FAILED")

    # Compute success rate for this state
    if results:
        success_count = sum(1 for r in results if r['success'])
        success_rate = success_count / len(results)
        print(f"  Success rate: {success_count}/{len(results)} ({success_rate:.1%})")

        # Best result
        best = max(results, key=lambda r: r['mm_count'])
        achieves_target = best['mm_count'] >= state_info['target_mm']
        print(f"  Best: {best['mm_count']} MM districts (target: {state_info['target_mm']}) - {'ACHIEVES' if achieves_target else 'FAILS'}")

    return results


def main():
    """Run 50-state threshold analysis."""
    print("=" * 80)
    print("50-STATE VRA THRESHOLD ANALYSIS")
    print("=" * 80)
    print(f"\nConfiguration:")
    print(f"  Weight factors: {WEIGHT_FACTORS}")
    print(f"  Minority thresholds: {[f'{t:.0%}' for t in MINORITY_THRESHOLDS]}")
    print(f"  Total configs per state: {len(WEIGHT_FACTORS) * len(MINORITY_THRESHOLDS)}")
    print(f"  Year: {YEAR}")
    print(f"\nStarting analysis at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # Get all states (convert abbreviations to lowercase full names)
    abbrev_to_name = {
        'CA': 'california', 'TX': 'texas', 'FL': 'florida', 'NY': 'new_york',
        'PA': 'pennsylvania', 'IL': 'illinois', 'OH': 'ohio', 'GA': 'georgia',
        'NC': 'north_carolina', 'MI': 'michigan', 'NJ': 'new_jersey', 'VA': 'virginia',
        'WA': 'washington', 'AZ': 'arizona', 'MA': 'massachusetts', 'TN': 'tennessee',
        'IN': 'indiana', 'MD': 'maryland', 'MO': 'missouri', 'WI': 'wisconsin',
        'CO': 'colorado', 'MN': 'minnesota', 'SC': 'south_carolina', 'AL': 'alabama',
        'LA': 'louisiana', 'KY': 'kentucky', 'OR': 'oregon', 'OK': 'oklahoma',
        'CT': 'connecticut', 'UT': 'utah', 'IA': 'iowa', 'NV': 'nevada',
        'AR': 'arkansas', 'MS': 'mississippi', 'KS': 'kansas', 'NM': 'new_mexico',
        'NE': 'nebraska', 'ID': 'idaho', 'WV': 'west_virginia', 'HI': 'hawaii',
        'NH': 'new_hampshire', 'ME': 'maine', 'RI': 'rhode_island', 'MT': 'montana',
        'DE': 'delaware', 'SD': 'south_dakota', 'ND': 'north_dakota', 'AK': 'alaska',
        'VT': 'vermont', 'WY': 'wyoming'
    }
    all_states = [abbrev_to_name[abbr] for abbr in STATE_CONFIG_2020.keys() if abbr in abbrev_to_name]
    multi_district_states = [s for s in all_states if s not in SINGLE_DISTRICT_STATES]

    print(f"\nStates to analyze: {len(multi_district_states)} (excluding {len(SINGLE_DISTRICT_STATES)} single-district states)")

    # Run analysis for all states
    all_results = []

    for i, state in enumerate(multi_district_states, 1):
        print(f"\n{'='*80}")
        print(f"STATE {i}/{len(multi_district_states)}: {state.replace('_', ' ').title()}")
        print(f"{'='*80}")

        state_results = analyze_state(state)
        all_results.extend(state_results)

    # Save results
    if all_results:
        OUTPUT_DIR.mkdir(exist_ok=True, parents=True)

        df = pd.DataFrame(all_results)
        output_file = OUTPUT_DIR / '50_states_threshold_analysis.csv'
        df.to_csv(output_file, index=False, float_format='%.4f')

        print(f"\n{'='*80}")
        print("ANALYSIS COMPLETE")
        print(f"{'='*80}")
        print(f"\nSaved results to: {output_file}")
        print(f"Total configurations: {len(df)}")
        print(f"States analyzed: {df['state'].nunique()}")
        print(f"Overall success rate: {df['success'].mean():.1%}")

        # Summary by state
        print("\n=== STATE SUMMARY ===")
        summary = df.groupby('state').agg({
            'state_minority_pct': 'first',
            'num_districts': 'first',
            'target_mm': 'first',
            'success': 'mean',
            'mm_count': 'max'
        }).round(4)

        summary['achieves_target'] = summary['mm_count'] >= summary['target_mm']
        summary = summary.sort_values('state_minority_pct', ascending=False)

        print(f"\n{'State':<20} {'Minority%':<12} {'Districts':<10} {'Target MM':<10} {'Success Rate':<15} {'Achieves'}")
        print("-" * 90)
        for state, row in summary.iterrows():
            achieves = "YES" if row['achieves_target'] else "NO"
            print(f"{state:<20} {row['state_minority_pct']:<12.1%} {int(row['num_districts']):<10} "
                  f"{int(row['target_mm']):<10} {row['success']:<15.1%} {achieves}")

        print(f"\nCompleted at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    else:
        print("\n[ERROR] No results generated.")


if __name__ == '__main__':
    main()
