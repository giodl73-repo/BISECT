"""
Compute compactness scores for all beta values from ablation study.

Uses cached tract geometries and district assignments from ablation results.
"""

import pickle
from pathlib import Path
import geopandas as gpd
import numpy as np
import pandas as pd
from tqdm import tqdm

def compute_polsby_popper(geometry):
    """Compute PP score."""
    area = geometry.area
    perimeter = geometry.length
    if perimeter == 0:
        return 0.0
    pp = (4 * np.pi * area) / (perimeter ** 2)
    return min(pp, 1.0)

# Load baseline tract geometries and districts
print("Loading tract geometries...")
with open('outputs/experimental/national_districts_2020.pkl', 'rb') as f:
    baseline = pickle.load(f)

geoid_to_index = baseline['geoid_to_index']

# Load tract geometries from compactness analysis
print("Loading cached geometries from previous analysis...")
tracts_gdf = gpd.read_file('outputs/experimental/compactness_comparison_2020.gpkg')
print(f"  Loaded {len(tracts_gdf)} district geometries")

# Get tract GeoDataFrame (need to reload from source)
print("Loading tract geometries...")
import sys
sys.path.insert(0, 'scripts')
from config.download_sources import STATE_FIPS, STATE_NAMES

all_tracts = []
for state_abbr, fips in tqdm(STATE_FIPS.items(), desc="Loading tracts"):
    state_name = STATE_NAMES[state_abbr]
    tract_file_shp = Path(f"data/2020/tiger/tracts/tl_2020_{fips}_tract/tl_2020_{fips}_tract.shp")

    if not tract_file_shp.exists():
        continue

    gdf = gpd.read_file(tract_file_shp)
    gdf = gdf[gdf['GEOID'].isin(geoid_to_index.keys())]
    all_tracts.append(gdf[['GEOID', 'geometry']])

tracts_gdf = gpd.GeoDataFrame(pd.concat(all_tracts, ignore_index=True), crs=all_tracts[0].crs)

print(f"  Loaded {len(tracts_gdf):,} tract geometries")

# Analyze each beta
results = []
betas = [0.0, 0.5, 1.0, 2.0, 5.0, 10.0]

for beta in tqdm(betas, desc="Analyzing beta values"):
    # Load districts for this beta
    with open(f'outputs/experimental/ablation_beta_{beta}_2020.pkl', 'rb') as f:
        data = pickle.load(f)

    geoid_to_district = data['geoid_to_district']

    # Add district assignments to tracts
    tracts_beta = tracts_gdf.copy()
    tracts_beta['district'] = tracts_beta['GEOID'].map(geoid_to_district)
    tracts_beta = tracts_beta[tracts_beta['district'].notna()]

    # Dissolve to districts
    districts_gdf = tracts_beta.dissolve(by='district', as_index=False)

    # Project for accurate measurements
    districts_proj = districts_gdf.to_crs('ESRI:102003')

    # Compute PP scores
    pp_scores = []
    for _, row in districts_proj.iterrows():
        pp = compute_polsby_popper(row.geometry)
        pp_scores.append(pp)

    pp_scores = np.array(pp_scores)

    results.append({
        'beta': beta,
        'mean_pp': pp_scores.mean(),
        'median_pp': np.median(pp_scores),
        'std_pp': pp_scores.std(),
        'min_pp': pp_scores.min(),
        'max_pp': pp_scores.max(),
        'n_cross_state': data['results']['n_cross_state_districts']
    })

# Save results
df = pd.DataFrame(results)
print("\\n" + "="*80)
print("Ablation Study: Compactness vs Boundary Crossing")
print("="*80)
print(df.to_string(index=False))
print("="*80)

# Save to file
with open('outputs/experimental/ablation_compactness_summary.pkl', 'wb') as f:
    pickle.dump({'results': df.to_dict('records')}, f)

print("\\nSaved to: outputs/experimental/ablation_compactness_summary.pkl")
