"""
Download California ballot proposition data for multi-party simulation.

This script downloads precinct-level ballot proposition results from the
California Statewide Database and converts them to census tract level using
the precinct-to-block conversion tables.

Process:
1. Download SR Precinct results for selected propositions (2020, 2016, 2012)
2. Download precinct-to-block conversion tables
3. Aggregate blocks to census tracts
4. Create synthetic party vote shares based on proposition positions

Data Source: https://statewidedatabase.org/
"""

import requests
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np
from typing import Dict, List, Tuple
import logging
from zipfile import ZipFile
from io import BytesIO

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Base URL for Statewide Database
SWDB_BASE = "https://statewidedatabase.org/pub/data"

# Propositions to download (divisive issues that split parties)
PROPOSITIONS = {
    2020: {
        'Prop15': {  # Property tax on commercial property (Economic)
            'name': 'Property Tax Increase',
            'description': 'Tax commercial property to fund schools',
            'file': 'G20/statewide/sov_g20_state_prop.csv'
        },
        'Prop22': {  # Gig worker classification (Labor)
            'name': 'Gig Worker Classification',
            'description': 'Exempt app-based drivers from AB5',
            'file': 'G20/statewide/sov_g20_state_prop.csv'
        },
    },
    2016: {
        'Prop64': {  # Marijuana legalization (Social)
            'name': 'Marijuana Legalization',
            'description': 'Legalize recreational marijuana',
            'file': 'G16/statewide/sov_g16_state_prop.csv'
        },
        'Prop63': {  # Gun control (Social)
            'name': 'Ammunition Background Checks',
            'description': 'Require background checks for ammo',
            'file': 'G16/statewide/sov_g16_state_prop.csv'
        },
    }
}

# Output directory
OUTPUT_DIR = Path(__file__).parent.parent.parent / "data" / "california_propositions"


def download_file(url: str, output_path: Path) -> bool:
    """
    Download file from URL to output path.

    Args:
        url: URL to download from
        output_path: Path to save file

    Returns:
        True if successful, False otherwise
    """
    try:
        logger.info(f"Downloading {url}")
        response = requests.get(url, timeout=60)
        response.raise_for_status()

        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Handle ZIP files
        if url.endswith('.zip'):
            with ZipFile(BytesIO(response.content)) as zip_file:
                zip_file.extractall(output_path.parent)
                logger.info(f"Extracted ZIP to {output_path.parent}")
        else:
            with open(output_path, 'wb') as f:
                f.write(response.content)
            logger.info(f"Saved to {output_path}")

        return True

    except Exception as e:
        logger.error(f"Failed to download {url}: {e}")
        return False


def download_proposition_results(year: int, output_dir: Path) -> Dict[str, Path]:
    """
    Download proposition results for a given year.

    Args:
        year: Election year (2020, 2016, etc.)
        output_dir: Directory to save files

    Returns:
        Dictionary mapping proposition names to file paths
    """
    year_dir = output_dir / str(year)
    year_dir.mkdir(parents=True, exist_ok=True)

    results = {}

    if year not in PROPOSITIONS:
        logger.warning(f"No propositions defined for {year}")
        return results

    # Download each proposition's results
    for prop_id, prop_info in PROPOSITIONS[year].items():
        url = f"{SWDB_BASE}/{prop_info['file']}"
        output_path = year_dir / f"{prop_id.lower()}_results.csv"

        if output_path.exists():
            logger.info(f"{prop_id} already downloaded: {output_path}")
            results[prop_id] = output_path
        else:
            if download_file(url, output_path):
                results[prop_id] = output_path

    return results


def download_conversion_tables(year: int, output_dir: Path) -> Dict[str, Path]:
    """
    Download precinct-to-block conversion tables.

    Args:
        year: Election year
        output_dir: Directory to save files

    Returns:
        Dictionary mapping conversion types to file paths
    """
    conv_dir = output_dir / str(year) / "conversions"
    conv_dir.mkdir(parents=True, exist_ok=True)

    results = {}

    # Conversion files for 2020
    if year == 2020:
        conversions = {
            'precinct_to_block': f"{SWDB_BASE}/G20/g20_srprec_to_2010blk.zip",
            'block_to_precinct': f"{SWDB_BASE}/G20/g20_2010blk_to_mprec.zip",
        }
    elif year == 2016:
        conversions = {
            'precinct_to_block': f"{SWDB_BASE}/G16/g16_srprec_to_2010blk.zip",
            'block_to_precinct': f"{SWDB_BASE}/G16/g16_2010blk_to_mprec.zip",
        }
    else:
        logger.warning(f"No conversion tables defined for {year}")
        return results

    for conv_name, url in conversions.items():
        output_path = conv_dir / f"{conv_name}.csv"

        if output_path.exists():
            logger.info(f"{conv_name} already downloaded: {output_path}")
            results[conv_name] = output_path
        else:
            if download_file(url, conv_dir / f"{conv_name}.zip"):
                # Find the extracted CSV
                csv_files = list(conv_dir.glob("*.csv"))
                if csv_files:
                    results[conv_name] = csv_files[0]

    return results


def aggregate_precincts_to_tracts(
    precinct_results: pd.DataFrame,
    precinct_to_block: pd.DataFrame,
    block_to_tract_mapping: Dict[str, str]
) -> pd.DataFrame:
    """
    Aggregate precinct-level results to census tracts.

    Args:
        precinct_results: Precinct-level proposition results
        precinct_to_block: Precinct-to-block conversion table with shares
        block_to_tract_mapping: Dictionary mapping block GEOIDs to tract GEOIDs

    Returns:
        DataFrame with tract-level results
    """
    logger.info("Aggregating precincts to tracts")

    # Merge precinct results with conversion table
    merged = precinct_results.merge(
        precinct_to_block,
        on='precinct_id',
        how='inner'
    )

    # Apply share weights
    for col in ['yes_votes', 'no_votes', 'total_votes']:
        if col in merged.columns:
            merged[f'{col}_weighted'] = merged[col] * merged['share']

    # Map blocks to tracts
    merged['tract_id'] = merged['block_id'].map(block_to_tract_mapping)

    # Aggregate to tract level
    tract_results = merged.groupby('tract_id').agg({
        'yes_votes_weighted': 'sum',
        'no_votes_weighted': 'sum',
        'total_votes_weighted': 'sum'
    }).reset_index()

    tract_results.rename(columns={
        'yes_votes_weighted': 'yes_votes',
        'no_votes_weighted': 'no_votes',
        'total_votes_weighted': 'total_votes'
    }, inplace=True)

    # Calculate vote shares
    tract_results['yes_share'] = (
        tract_results['yes_votes'] / tract_results['total_votes']
    )
    tract_results['no_share'] = (
        tract_results['no_votes'] / tract_results['total_votes']
    )

    logger.info(f"Aggregated to {len(tract_results)} tracts")
    return tract_results


def create_synthetic_parties(
    tract_results: Dict[str, pd.DataFrame],
    num_parties: int = 4
) -> pd.DataFrame:
    """
    Create synthetic party vote shares based on proposition positions.

    Args:
        tract_results: Dictionary mapping proposition IDs to tract-level results
        num_parties: Number of synthetic parties (4 or 8)

    Returns:
        DataFrame with synthetic party vote shares per tract
    """
    logger.info(f"Creating {num_parties}-party system")

    # For 4-party system: Use 2 propositions (2^2 = 4 combinations)
    # For 8-party system: Use 3 propositions (2^3 = 8 combinations)

    if num_parties == 4:
        # Economic (Prop 15) × Social (Prop 64)
        prop_ids = ['Prop15', 'Prop64']
    elif num_parties == 8:
        # Economic × Social × Labor
        prop_ids = ['Prop15', 'Prop64', 'Prop22']
    else:
        raise ValueError(f"Unsupported number of parties: {num_parties}")

    # Get tract-level data for selected propositions
    # TODO: Implement combination logic

    pass


def main():
    """Main execution."""
    logger.info("="*80)
    logger.info("California Ballot Proposition Download")
    logger.info("="*80)

    # Create output directory
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Download 2020 propositions
    logger.info("\n--- Downloading 2020 Propositions ---")
    prop_2020 = download_proposition_results(2020, OUTPUT_DIR)
    conv_2020 = download_conversion_tables(2020, OUTPUT_DIR)

    # Download 2016 propositions
    logger.info("\n--- Downloading 2016 Propositions ---")
    prop_2016 = download_proposition_results(2016, OUTPUT_DIR)
    conv_2016 = download_conversion_tables(2016, OUTPUT_DIR)

    logger.info("\n" + "="*80)
    logger.info("Download Summary")
    logger.info("="*80)
    logger.info(f"2020 Propositions: {len(prop_2020)}")
    logger.info(f"2020 Conversions: {len(conv_2020)}")
    logger.info(f"2016 Propositions: {len(prop_2016)}")
    logger.info(f"2016 Conversions: {len(conv_2016)}")

    logger.info(f"\nFiles saved to: {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
