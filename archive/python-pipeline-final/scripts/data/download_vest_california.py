"""
Download VEST California precinct-level election results.

VEST (Voting and Election Science Team) provides precinct-level election
results as shapefiles. This script downloads California 2020 and 2016 data.

Data source: https://dataverse.harvard.edu/dataverse/electionscience
"""

import requests
from pathlib import Path
import logging
from zipfile import ZipFile
from io import BytesIO

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# VEST data files
VEST_FILES = {
    '2020': {
        'url': 'https://dataverse.harvard.edu/api/access/datafile/4498586',  # CA 2020
        'filename': 'ca_2020_vest.zip',
        'description': 'California 2020 Precinct-Level Election Results'
    },
    '2016': {
        'url': 'https://dataverse.harvard.edu/api/access/datafile/4299682',  # CA 2016
        'filename': 'ca_2016_vest.zip',
        'description': 'California 2016 Precinct-Level Election Results'
    }
}

OUTPUT_DIR = Path(__file__).parent.parent.parent / "data" / "vest_california"


def download_vest_file(year: str, output_dir: Path) -> bool:
    """
    Download VEST precinct data for a given year.

    Args:
        year: Election year ('2020' or '2016')
        output_dir: Directory to save files

    Returns:
        True if successful, False otherwise
    """
    if year not in VEST_FILES:
        logger.error(f"Year {year} not available")
        return False

    file_info = VEST_FILES[year]
    url = file_info['url']
    output_path = output_dir / year
    zip_path = output_path / file_info['filename']

    logger.info(f"Downloading {file_info['description']}")
    logger.info(f"URL: {url}")

    try:
        # Create output directory
        output_path.mkdir(parents=True, exist_ok=True)

        # Download file
        response = requests.get(url, timeout=300, stream=True)
        response.raise_for_status()

        # Get file size
        total_size = int(response.headers.get('content-length', 0))
        logger.info(f"File size: {total_size / 1024 / 1024:.1f} MB")

        # Save ZIP file
        with open(zip_path, 'wb') as f:
            downloaded = 0
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)
                downloaded += len(chunk)
                if total_size > 0:
                    percent = (downloaded / total_size) * 100
                    if downloaded % (1024 * 1024) == 0:  # Log every MB
                        logger.info(f"Downloaded: {percent:.1f}%")

        logger.info(f"Saved to: {zip_path}")

        # Extract ZIP
        logger.info("Extracting ZIP file...")
        with ZipFile(zip_path, 'r') as zip_file:
            zip_file.extractall(output_path)

        logger.info(f"Extracted to: {output_path}")

        # List extracted files
        shp_files = list(output_path.glob("*.shp"))
        if shp_files:
            logger.info(f"Found shapefile: {shp_files[0].name}")

        return True

    except Exception as e:
        logger.error(f"Failed to download {year} data: {e}")
        return False


def main():
    """Main execution."""
    logger.info("="*80)
    logger.info("VEST California Precinct Data Download")
    logger.info("="*80)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    # Download 2020 data
    logger.info("\n--- Downloading 2020 Data ---")
    success_2020 = download_vest_file('2020', OUTPUT_DIR)

    # Download 2016 data
    logger.info("\n--- Downloading 2016 Data ---")
    success_2016 = download_vest_file('2016', OUTPUT_DIR)

    logger.info("\n" + "="*80)
    logger.info("Download Summary")
    logger.info("="*80)
    logger.info(f"2020 Data: {'SUCCESS' if success_2020 else 'FAILED'}")
    logger.info(f"2016 Data: {'SUCCESS' if success_2016 else 'FAILED'}")
    logger.info(f"\nFiles saved to: {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
