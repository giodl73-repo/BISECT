#!/usr/bin/env python3
"""
Stage-aware download checking for redistricting pipeline integration.

This module checks what raw data exists and what needs to be downloaded
to support the census data processing pipeline stages.

Pipeline stages (from process_census_data.py):
  1. redistricting - Parse PL 94-171 + merge with TIGER geometries (tract or block level)
  2. adjacency - Build adjacency graphs
  3. elections - Process election data
  4. demographics - Process demographic data

This module determines what raw downloads are needed for each stage.

Created: 2026-01-18 (Enhancement #48)
"""

from pathlib import Path
from typing import List, Dict, Tuple
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.config.download_sources import ALL_STATES


# =============================================================================
# STAGE DEFINITIONS
# =============================================================================

# What raw data each stage needs
STAGE_REQUIREMENTS = {
    'redistricting': {
        'description': 'Census redistricting data (tract or block level geometries and population)',
        'raw_data': [
            ('data/{year}/redistricting/', 'PL 94-171 files or NHGIS shapefiles'),
        ],
        'required_states': 50,  # All 50 states + DC (51 total, but DC optional)
        'check_files': {
            2000: [
                'data/2000/redistricting/',  # PL 94-171 or NHGIS files
            ],
            2010: [
                'data/2010/redistricting/',  # PL 94-171 files
            ],
            2020: [
                'data/2020/redistricting/',  # PL 94-171 files
            ],
        },
        'state_file_patterns': {
            2000: None,  # National file or varies by source
            2010: 'data/2010/redistricting/{state_lower}2010.pl',
            2020: 'data/2020/redistricting/{state_lower}2020.pl',
        }
    },
    'adjacency': {
        'description': 'Adjacency graphs (depends on tracts)',
        'raw_data': [],  # Generated from tracts, no download needed
        'required_states': 0,
        'check_files': {}
    },
    'elections': {
        'description': 'Tract-level election results',
        'raw_data': [
            ('data/{year}/elections/{year}_president/', 'Harvard Dataverse election data'),
        ],
        'required_states': 48,  # Only 48 states (missing AK, HI)
        'excluded_states': ['AK', 'HI'],
        'check_files': {
            2016: ['data/2016/elections/2016_president/'],
            2020: ['data/2020/elections/2020_president/'],
        }
    },
    'demographics': {
        'description': 'Tract-level demographics (age, race, sex)',
        'raw_data': [
            ('data/{year}/demographics/', 'Census API demographic CSVs'),
        ],
        'required_states': 50,  # All 50 states
        'check_files': {
            2000: ['data/2000/demographics/'],
            2010: ['data/2010/demographics/'],
            2020: ['data/2020/demographics/'],
        },
        'state_file_patterns': {
            2000: 'data/2000/demographics/{state_name}_demographics_2000.csv',
            2010: 'data/2010/demographics/{state_name}_demographics_2010.csv',
            2020: 'data/2020/demographics/{state_name}_demographics_2020.csv',
        }
    },
    'places': {
        'description': 'Place/city boundaries (for city labels on maps)',
        'raw_data': [
            ('data/{year}/places/', 'TIGER/Line place shapefiles'),
        ],
        'required_states': 50,  # All 50 states (or national file for 2000)
        'check_files': {
            2000: ['data/2000/places/US_place_2000.shp'],  # National file
            2010: ['data/2010/places/'],
            2020: ['data/2020/places/'],
        },
        'state_file_patterns': {
            2000: None,  # National file (US_place_2000.shp)
            2010: 'data/2010/places/{state_lower}_places_2010.shp',
            2020: 'data/2020/places/{state_lower}_places_2020.shp',
        }
    },
    'metros': {
        'description': 'Metro/CBSA boundaries (for urban area analysis)',
        'raw_data': [
            ('data/{year}/metros/', 'Census CBSA/Metro Area shapefiles'),
        ],
        'required_states': 0,  # National file, not per-state
        'check_files': {
            2000: ['data/2000/metros/'],
            2010: ['data/2010/metros/'],
            2020: ['data/2020/metros/'],
        },
        'state_file_patterns': {}  # National files only
    },
    'enacted_districts': {
        'description': 'Enacted congressional districts (baseline for comparison)',
        'raw_data': [
            ('data/{year}/baseline/', 'Census TIGER congressional district shapefiles'),
        ],
        'required_states': 0,  # National file, not per-state
        'check_files': {
            2000: ['data/2000/baseline/'],  # 108th Congress (2003-2005)
            2010: ['data/2010/baseline/'],  # 113th Congress (2013-2015)
            2020: ['data/2020/baseline/'],  # 118th Congress (2023-2025)
        },
        'state_file_patterns': {}  # National files only
    }
}


# =============================================================================
# MARKER FILE SUPPORT
# =============================================================================

def get_download_marker_path(year: int, stage: str) -> Path:
    """
    Get path to download completion marker file.

    Args:
        year: Census year
        stage: Stage name

    Returns:
        Path to marker file (e.g., data/2020/.download_redistricting_complete)
    """
    return Path(f'data/{year}/.download_{stage}_complete')


def check_download_marker(year: int, stage: str) -> bool:
    """
    Check if download completion marker exists for a stage.

    Fast path - O(1) file existence check instead of O(n) state scanning.

    Args:
        year: Census year
        stage: Stage name

    Returns:
        True if marker exists
    """
    marker = get_download_marker_path(year, stage)
    return marker.exists()


def create_download_marker(year: int, stage: str):
    """
    Create download completion marker file.

    Should only be called after validating all required states are present.

    Args:
        year: Census year
        stage: Stage name
    """
    marker = get_download_marker_path(year, stage)
    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.touch()
    print(f"[MARKER] Created: {marker}")


def remove_download_marker(year: int, stage: str):
    """
    Remove download completion marker (e.g., if data is incomplete).

    Args:
        year: Census year
        stage: Stage name
    """
    marker = get_download_marker_path(year, stage)
    if marker.exists():
        marker.unlink()
        print(f"[MARKER] Removed: {marker}")


# =============================================================================
# STATE-LEVEL VALIDATION
# =============================================================================

def get_required_states_for_stage(stage: str) -> List[str]:
    """
    Get list of required states for a stage.

    Args:
        stage: Stage name

    Returns:
        List of state codes (e.g., ['AL', 'AK', ...])
    """
    if stage not in STAGE_REQUIREMENTS:
        return []

    requirements = STAGE_REQUIREMENTS[stage]
    excluded = requirements.get('excluded_states', [])

    # Return all states except excluded ones
    return [state for state in ALL_STATES if state not in excluded]


def check_state_file_exists(year: int, stage: str, state_code: str) -> bool:
    """
    Check if data exists for a specific state.

    Args:
        year: Census year
        stage: Stage name
        state_code: 2-letter state code (e.g., 'CA')

    Returns:
        True if state data exists
    """
    if stage not in STAGE_REQUIREMENTS:
        return False

    requirements = STAGE_REQUIREMENTS[stage]
    patterns = requirements.get('state_file_patterns', {})

    if year not in patterns or patterns[year] is None:
        # National file or no pattern defined
        return True

    pattern = patterns[year]
    state_lower = state_code.lower()

    # Get state name for demographics (uses full names like 'alabama', not 'al')
    from scripts.config.download_sources import STATE_NAMES
    state_name = STATE_NAMES.get(state_code, state_lower)

    # Try to format pattern with available variables
    try:
        file_path = Path(pattern.format(state_lower=state_lower, state_name=state_name))
    except KeyError:
        # Pattern uses variable we don't have
        file_path = Path(pattern.format(state_lower=state_lower))

    return file_path.exists()


def check_all_states_present(year: int, stage: str) -> Tuple[bool, List[str]]:
    """
    Check if all required states have data for a stage.

    Args:
        year: Census year
        stage: Stage name

    Returns:
        Tuple of (all_present: bool, missing_states: List[str])
    """
    required_states = get_required_states_for_stage(stage)

    if not required_states:
        # Stage doesn't require per-state files (e.g., adjacency, or national files)
        return (True, [])

    missing_states = []

    for state in required_states:
        if not check_state_file_exists(year, stage, state):
            missing_states.append(state)

    return (len(missing_states) == 0, missing_states)


# =============================================================================
# STAGE CHECKING (Updated with marker support)
# =============================================================================

def check_raw_data_exists(year: int, stage: str, check_marker: bool = True) -> Tuple[bool, List[str]]:
    """
    Check if raw data exists for a stage.

    Performs two levels of checking:
    1. Fast path: Check download marker file (optional)
    2. Thorough path: Check directory existence + state-level completeness

    Args:
        year: Census year (2000, 2010, 2020)
        stage: Stage name (redistricting, adjacency, elections, demographics, places)
        check_marker: If True, check marker file first for fast validation (default: True)

    Returns:
        Tuple of (all_exist: bool, missing_items: List[str])
            missing_items can be directory paths or state codes
    """
    if stage not in STAGE_REQUIREMENTS:
        return (False, [f"Unknown stage: {stage}"])

    # Fast path: check marker file if requested
    if check_marker and check_download_marker(year, stage):
        return (True, [])

    requirements = STAGE_REQUIREMENTS[stage]
    check_files = requirements.get('check_files', {})

    if year not in check_files:
        # Stage not applicable for this year (e.g., 2000 elections)
        return (True, [])

    # Check if base directories/files exist
    files_to_check = check_files[year]
    missing = []

    for file_or_dir in files_to_check:
        path = Path(file_or_dir)

        # For national files (specific file extensions), check file directly
        if path.suffix in ['.shp', '.csv', '.geojson']:
            if not path.exists():
                missing.append(str(path))
        # For directories, check directory exists
        else:
            if not path.exists() or not path.is_dir():
                missing.append(str(path))

    # If directories/files don't exist, return early
    if missing:
        return (False, missing)

    # Thorough validation: check state-level completeness
    all_present, missing_states = check_all_states_present(year, stage)

    if not all_present:
        # Return state codes as missing items
        return (False, missing_states)

    return (True, [])


def check_stage_complete(output_dir: Path, stage: str, resolution: str = 'tract') -> bool:
    """
    Check if a processing stage has been completed.

    Args:
        output_dir: Output directory (e.g., outputs/v1/2020/)
        stage: Stage name
        resolution: Resolution level (tract, block)

    Returns:
        True if stage is complete
    """
    marker = output_dir / f'.{resolution}_{stage}_complete'
    return marker.exists()


def check_all_stages_complete(output_dir: Path, stages: List[str], resolution: str = 'tract') -> bool:
    """
    Check if all stages are complete.

    Args:
        output_dir: Output directory
        stages: List of stage names
        resolution: Resolution level

    Returns:
        True if ALL stages are complete
    """
    return all(check_stage_complete(output_dir, stage, resolution) for stage in stages)


def get_missing_downloads(year: int, stages: List[str]) -> Dict[str, List[str]]:
    """
    Determine what downloads are missing for requested stages.

    Args:
        year: Census year
        stages: List of stages to check (e.g., ['redistricting', 'demographics'])

    Returns:
        Dict mapping download type to list of missing items
        Example: {'redistricting': ['data/Census 2020/tracts/'], 'demographics': [...]}
    """
    missing_downloads = {}

    for stage in stages:
        if stage == 'adjacency':
            # Adjacency is generated, not downloaded
            continue

        exists, missing_files = check_raw_data_exists(year, stage)

        if not exists:
            missing_downloads[stage] = missing_files

    return missing_downloads


def get_download_plan(year: int, stages: List[str], output_dir: Path, force: bool = False) -> Dict[str, any]:
    """
    Create a download plan for requested stages.

    Checks:
    1. What stages are already complete (skip those unless force=True)
    2. What raw data exists with state-level validation (skip downloads unless force=True)
    3. What downloads are needed

    Args:
        year: Census year
        stages: List of stages to process
        output_dir: Output directory for checking completion markers
        force: If True, bypass all cache checks and force redownload

    Returns:
        Dict with download plan:
        {
            'complete_stages': [...],  # Already done (empty if force=True)
            'available_data': [...],   # Data exists, no download needed (empty if force=True)
            'needed_downloads': {...}  # Need to download
        }
    """
    plan = {
        'complete_stages': [],
        'available_data': [],
        'needed_downloads': {}
    }

    for stage in stages:
        # Skip completion check if force mode
        if not force and check_stage_complete(output_dir, stage):
            plan['complete_stages'].append(stage)
            continue

        # Check if raw data exists (with state-level validation)
        # Pass check_marker=not force to bypass marker in force mode
        exists, missing_items = check_raw_data_exists(year, stage, check_marker=not force)

        if not force and exists:
            plan['available_data'].append(stage)
        else:
            plan['needed_downloads'][stage] = missing_items if missing_items else ['all states']

    return plan


def print_download_status(year: int, stages: List[str], output_dir: Path):
    """
    Print download status for debugging.

    Args:
        year: Census year
        stages: List of stages to check
        output_dir: Output directory
    """
    print(f"\n[{year}] Download Status for Stages: {', '.join(stages)}")
    print("=" * 70)

    plan = get_download_plan(year, stages, output_dir)

    if plan['complete_stages']:
        print(f"\n[OK] Already Complete:")
        for stage in plan['complete_stages']:
            print(f"  - {stage}")

    if plan['available_data']:
        print(f"\n[OK] Raw Data Available (no download needed):")
        for stage in plan['available_data']:
            print(f"  - {stage}")

    if plan['needed_downloads']:
        print(f"\n[DOWNLOAD] Needed:")
        for stage, missing_files in plan['needed_downloads'].items():
            print(f"  - {stage}:")
            for file in missing_files:
                print(f"      {file}")
    else:
        print(f"\n[OK] All data available!")

    print("=" * 70)


# =============================================================================
# STATE-LEVEL CHECKING
# =============================================================================

def check_state_data_exists(state_code: str, year: int, data_type: str) -> bool:
    """
    Check if data exists for a specific state.

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year
        data_type: Data type ('demographics', 'tracts', 'places')

    Returns:
        True if data exists for this state
    """
    state_lower = state_code.lower()

    if data_type == 'demographics':
        csv_path = Path(f'data/Census {year}/demographics/{state_lower}_demographics_{year}.csv')
        return csv_path.exists()

    elif data_type == 'tracts':
        # For tracts, check if national file exists (2000) or state PL file (2010/2020)
        if year == 2000:
            return Path(f'data/Census 2000/tracts/US_tract_2000.shp').exists()
        elif year == 2010:
            return Path(f'data/Census 2010/geos/{state_lower}geo2010.pl').exists()
        else:  # 2020
            return Path(f'data/Census 2020/{state_lower}2020.pl').exists()

    elif data_type == 'places':
        # Similar to tracts
        if year == 2000:
            return Path(f'data/Census 2000/places/US_place_2000.shp').exists()
        else:
            return Path(f'data/Census {year}/places/{state_lower}_places_{year}.shp').exists()

    return False


def get_states_needing_download(states: List[str], year: int, data_type: str) -> List[str]:
    """
    Filter list of states to only those needing download.

    Args:
        states: List of state codes
        year: Census year
        data_type: Data type to check

    Returns:
        List of state codes that need download
    """
    return [state for state in states if not check_state_data_exists(state, year, data_type)]
