#!/usr/bin/env python3
"""
Download election data from Harvard Dataverse.

Dataset: "Reallocating u.s. election results from precincts to census geographies"
Author: Amir Fekrazad
DOI: 10.7910/DVN/Z8TSH3
License: CC0 1.0 Universal

This script downloads census tract-level 2020 presidential election results.
"""

import requests
import argparse
from pathlib import Path
import json
from tqdm import tqdm


def download_file(url, output_path, description=None):
    """Download a file with progress bar."""
    response = requests.get(url, stream=True)
    response.raise_for_status()

    total_size = int(response.headers.get('content-length', 0))

    with open(output_path, 'wb') as f:
        with tqdm(total=total_size, unit='B', unit_scale=True,
                 desc=description or output_path.name) as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    pbar.update(len(chunk))

    print(f"Downloaded: {output_path}")


def get_dataverse_files(doi):
    """Get file list from Harvard Dataverse dataset using API."""
    base_url = "https://dataverse.harvard.edu/api/datasets/:persistentId"
    params = {'persistentId': doi}

    print(f"Fetching file list from Dataverse DOI: {doi}")
    response = requests.get(base_url, params=params)
    response.raise_for_status()

    data = response.json()

    if 'data' not in data or 'latestVersion' not in data['data']:
        raise ValueError("Unexpected response structure from Dataverse API")

    files = data['data']['latestVersion']['files']
    return files


def main():
    parser = argparse.ArgumentParser(description='Download election data from Harvard Dataverse')
    parser.add_argument('--year', type=str, default='2020', choices=['2016', '2020'],
                       help='Election year (default: 2020)')
    parser.add_argument('--scope', type=str, default='usa',
                       choices=['usa', 'state'],
                       help='Download all USA data or individual states (default: usa)')
    parser.add_argument('--state', type=str,
                       help='State code (e.g., CA) when scope=state')
    parser.add_argument('--method', type=str, default='main',
                       choices=['main', 'alternative'],
                       help='Main method (RLCR - recommended) or alternative methods (default: main)')
    parser.add_argument('--output-dir', type=str,
                       default='data/raw/elections',
                       help='Output directory')
    parser.add_argument('--doi', type=str,
                       default='10.7910/DVN/Z8TSH3',
                       help='Harvard Dataverse DOI to fetch from (default: Fekrazad tract reallocation). '
                            'Pass without the "doi:" prefix.')
    parser.add_argument('--output-subdir', type=str,
                       default=None,
                       help='Override output subdirectory name (default: {year}_president)')
    args = parser.parse_args()

    # Validate state argument
    if args.scope == 'state' and not args.state:
        print("ERROR: --state is required when --scope=state")
        return 1

    # Harvard Dataverse DOI for the dataset (parameterized; default is Fekrazad)
    doi = f"doi:{args.doi}" if not args.doi.startswith("doi:") else args.doi

    subdir_name = args.output_subdir or f'{args.year}_president'
    output_dir = Path(args.output_dir) / subdir_name
    output_dir.mkdir(parents=True, exist_ok=True)

    print("="*70)
    print("DOWNLOADING ELECTION DATA FROM HARVARD DATAVERSE")
    print("="*70)
    print(f"Year: {args.year}")
    print(f"Scope: {args.scope}")
    if args.scope == 'state':
        print(f"State: {args.state}")
    print(f"Method: {args.method}")
    print(f"Output: {output_dir}")
    print("="*70)
    print()

    # State FIPS code mapping
    state_fips = {
        'AL': '010', 'AZ': '040', 'AR': '050', 'CA': '060', 'CO': '080',
        'CT': '090', 'DE': '100', 'DC': '110', 'FL': '120', 'GA': '130',
        'ID': '160', 'IL': '170', 'IN': '180', 'IA': '190', 'KS': '200',
        'KY': '210', 'LA': '220', 'ME': '230', 'MD': '240', 'MA': '250',
        'MI': '260', 'MN': '270', 'MS': '280', 'MO': '290', 'MT': '300',
        'NE': '310', 'NV': '320', 'NH': '330', 'NJ': '340', 'NM': '350',
        'NY': '360', 'NC': '370', 'ND': '380', 'OH': '390', 'OK': '400',
        'OR': '410', 'PA': '420', 'RI': '440', 'SC': '450', 'SD': '460',
        'TN': '470', 'TX': '480', 'UT': '490', 'VT': '500', 'VA': '510',
        'WA': '530', 'WV': '540', 'WI': '550', 'WY': '560'
    }

    try:
        # Get file list from Dataverse API
        files = get_dataverse_files(doi)

        # Determine target filename based on scope
        if args.scope == 'usa':
            if args.method == 'main':
                target_filename = '000 Contiguous USA - Main Method.zip'
            else:
                target_filename = '001 Contiguous USA - Alternative Methods.zip'
        else:  # state scope
            state_code = args.state.upper()
            if state_code not in state_fips:
                print(f"ERROR: Unknown state code: {state_code}")
                return 1
            fips = state_fips[state_code]
            target_filename = f'{fips} {state_code}.zip'

        # Find the file
        target_file = None
        for file_info in files:
            filename = file_info['dataFile']['filename']
            if filename == target_filename:
                target_file = file_info
                break

        if not target_file:
            print(f"ERROR: Could not find file: {target_filename}")
            print("\nAvailable files:")
            for file_info in files[:20]:  # Show first 20
                print(f"  - {file_info['dataFile']['filename']}")
            return 1

        # Download the file
        file_id = target_file['dataFile']['id']
        download_url = f"https://dataverse.harvard.edu/api/access/datafile/{file_id}"
        output_path = output_dir / target_filename

        download_file(download_url, output_path, description=target_filename)

        # Extract ZIP file
        print(f"\nExtracting: {output_path}")
        import zipfile
        with zipfile.ZipFile(output_path, 'r') as zip_ref:
            zip_ref.extractall(output_dir)
        print(f"Extracted to: {output_dir}")

        # List extracted files
        extracted_files = list(output_dir.glob('*.csv'))
        print(f"\nExtracted {len(extracted_files)} CSV files")
        for csv_file in extracted_files[:10]:  # Show first 10
            print(f"  - {csv_file.name}")
        if len(extracted_files) > 10:
            print(f"  ... and {len(extracted_files) - 10} more")

        # Save metadata
        metadata = {
            'source': 'Harvard Dataverse',
            'doi': doi,
            'year': args.year,
            'scope': args.scope,
            'method': args.method,
            'filename': target_filename,
            'file_id': file_id,
            'download_url': download_url,
            'license': 'CC0 1.0 Universal',
            'citation': 'Fekrazad, A. (2025). A dataset of US precinct votes allocated to Census geographies with precision. Scientific Data, 12, 794.',
            'extracted_files': [f.name for f in extracted_files]
        }

        metadata_path = output_dir / 'metadata.json'
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)

        print(f"\nMetadata saved: {metadata_path}")
        print("\n" + "="*70)
        print("DOWNLOAD COMPLETE!")
        print("="*70)

        return 0

    except requests.exceptions.RequestException as e:
        print(f"ERROR: Failed to download data: {e}")
        return 1
    except Exception as e:
        print(f"ERROR: {e}")
        return 1


if __name__ == '__main__':
    exit(main())
