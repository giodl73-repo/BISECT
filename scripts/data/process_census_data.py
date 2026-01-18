#!/usr/bin/env python3
"""
Process census data for a given year.

This script processes raw census data and generates all required files
for the redistricting pipeline by calling existing processing scripts
with the correct new paths (outputs/data/).

Stages:
  1. Census tract parquet files (from PL 94-171 or NHGIS)
  2. Adjacency graphs for all states
  3. Places parquet files (for map labels)
  4. Election data (tract-level parquet)
  5. Demographics data (tract-level parquet)

Input:  data/Census {year}/ (raw data)
Output: outputs/data/ (processed data)

Called by run_complete_redistricting.py before state processing.
Creates .downloads_complete marker when finished.

Usage:
  python scripts/data/process_census_data.py --year 2020
  python scripts/data/process_census_data.py --year 2020 --stages tracts adjacency
  python scripts/data/process_census_data.py --year 2020 --states CA TX NY
  python scripts/data/process_census_data.py --year 2020 --dry-run
"""

import argparse
import sys
from pathlib import Path
from typing import List, Optional
import time
import subprocess
import os

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.utils.paths import (
    get_tract_file,
    get_adjacency_file,
    get_places_file,
    get_election_data_file,
    get_demographic_data_file,
)


# ============================================================================
# STATUS Protocol (for hierarchical progress display)
# ============================================================================

def report_progress(message: str, is_standalone: bool = None):
    """
    Report progress using STATUS protocol for hierarchical display.

    Args:
        message: Progress message to display
        is_standalone: Whether running standalone (None = auto-detect)
    """
    # Auto-detect standalone mode if not specified
    if is_standalone is None:
        is_standalone = int(os.environ.get('TQDM_POSITION', '-1')) < 0

    # Use STATUS protocol if running as worker process
    if not is_standalone:
        position = int(os.environ.get('TQDM_POSITION', '-1'))
        if position >= 0:
            print(f"STATUS:{position}:{message}", flush=True)
    else:
        # Standalone mode: print normally
        print(message, flush=True)


def log_error(error_log_file: Path, stage: str, error_msg: str):
    """
    Log error to error.log file.

    Args:
        error_log_file: Path to error.log
        stage: Stage name where error occurred
        error_msg: Error message
    """
    from datetime import datetime
    timestamp = datetime.now().isoformat()
    with open(error_log_file, 'a', encoding='utf-8') as f:
        f.write(f"[{timestamp}] {stage}: {error_msg}\n")


def get_stage_marker(output_dir: Path, stage: str, resolution: str = 'tract') -> Path:
    """
    Get path to per-stage marker file with resolution.

    Args:
        output_dir: Output directory (e.g., outputs/v1/2020/)
        stage: Stage name (tracts, adjacency, elections, demographics)
        resolution: Resolution level (tract, block)

    Returns:
        Path to marker file (e.g., .tract_tracts_complete, .block_adjacency_complete)
    """
    return output_dir / f'.{resolution}_{stage}_complete'


def check_stage_complete(output_dir: Path, stage: str, resolution: str = 'tract', force: bool = False) -> bool:
    """
    Check if a stage has already been completed.

    Args:
        output_dir: Output directory
        stage: Stage name
        resolution: Resolution level
        force: If True, ignore existing markers

    Returns:
        True if stage is complete and should be skipped
    """
    if force:
        return False

    marker = get_stage_marker(output_dir, stage, resolution)
    return marker.exists()


def create_stage_marker(output_dir: Path, stage: str, resolution: str = 'tract', dry_run: bool = False):
    """
    Create marker file for completed stage.

    Args:
        output_dir: Output directory
        stage: Stage name
        resolution: Resolution level
        dry_run: If True, don't actually create file
    """
    if dry_run:
        return

    from datetime import datetime
    marker = get_stage_marker(output_dir, stage, resolution)
    timestamp = datetime.now().isoformat()
    marker.write_text(f"{resolution} {stage} processing completed: {timestamp}\n")


def check_all_stages_complete(output_dir: Path, stages: List[str], resolution: str = 'tract', force: bool = False) -> bool:
    """
    Check if all stages for a resolution are complete.

    Args:
        output_dir: Output directory
        stages: List of stage names to check
        resolution: Resolution level
        force: If True, ignore existing markers

    Returns:
        True if ALL stages are complete
    """
    if force:
        return False

    return all(check_stage_complete(output_dir, stage, resolution, force) for stage in stages)


# All 50 states
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]


def run_command(cmd: List[str], description: str, error_log_file: Path,
                dry_run: bool = False, is_standalone: bool = None):
    """
    Run a subprocess command with STATUS protocol and error logging.

    Args:
        cmd: Command to run
        description: Description for progress/error messages
        error_log_file: Path to error.log
        dry_run: If True, don't actually run
        is_standalone: Whether running standalone (None = auto-detect)

    Returns:
        True if successful, False otherwise
    """
    report_progress(f"Running: {description}", is_standalone)

    if dry_run:
        report_progress(f"[DRY RUN] Would execute: {description}", is_standalone)
        return True

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        # Don't print stdout in worker mode (interferes with STATUS protocol)
        if result.stdout and is_standalone:
            print(result.stdout)
        return True
    except subprocess.CalledProcessError as e:
        error_msg = f"{description} failed: {e}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, description, error_msg)

        if e.stdout and is_standalone:
            print(f"  STDOUT: {e.stdout}")
        if e.stderr:
            report_progress(f"  STDERR: {e.stderr}", is_standalone)
            log_error(error_log_file, description, f"STDERR: {e.stderr}")

        return False
    except Exception as e:
        error_msg = f"{description} failed with exception: {e}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, description, error_msg)
        return False


def create_directory_structure(year: int, is_standalone: bool = None):
    """Create outputs/data directory structure for the year."""
    report_progress(f"[1/5] Creating directory structure", is_standalone)

    dirs = [
        f'outputs/data/tracts/{year}',
        f'outputs/data/adjacency/{year}',
        f'outputs/data/places/{year}',
        f'outputs/data/elections',
        f'outputs/data/demographics',
    ]

    for dir_path in dirs:
        Path(dir_path).mkdir(parents=True, exist_ok=True)

    report_progress(f"[OK] Directory structure created", is_standalone)


def process_census_tracts(year: int, error_log_file: Path, states: Optional[List[str]] = None,
                          force: bool = False, dry_run: bool = False, is_standalone: bool = None):
    """
    Process raw census data into tract parquet files.

    For 2000: Parse NHGIS national shapefile, split by state
    For 2010: Parse PL 94-171 files, download TIGER/Line, merge
    For 2020: Download via cenpy API
    """
    report_progress(f"[2/5] Processing census tracts ({year})", is_standalone)

    if states is None:
        states = ALL_STATES

    # Check which processing script exists based on year
    if year == 2020:
        script = 'scripts/data/census/download_all_states_tracts.py'
        strategy = "Download via cenpy API"
    elif year == 2010:
        script = 'scripts/data/census/parse_pl94171_tracts_2010.py'
        strategy = "Parse PL 94-171 + download TIGER/Line"
    elif year == 2000:
        script = 'scripts/data/census/parse_pl94171_tracts_2000.py'
        strategy = "Parse NHGIS national shapefile"
    else:
        error_msg = f"Unsupported year: {year}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, "process_census_tracts", error_msg)
        return False

    if not Path(script).exists():
        error_msg = f"Processing script not found: {script}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, "process_census_tracts", error_msg)
        return False

    # For 2020, use the download script with new output path
    if year == 2020:
        cmd = [
            sys.executable, script,
            '--year', str(year),
            '--output-dir', f'outputs/data/tracts/{year}'
        ]

        if states != ALL_STATES:
            # For now, the script doesn't support state filtering
            report_progress(f"[WARN] State filtering not supported for {script}", is_standalone)

        success = run_command(cmd, f"Process census tracts ({year})", error_log_file, dry_run, is_standalone)
        return success
    else:
        error_msg = f"Script {script} needs path updates"
        report_progress(f"[TODO] {error_msg}", is_standalone)
        log_error(error_log_file, "process_census_tracts", error_msg)
        return False


def build_adjacency_graphs(year: int, error_log_file: Path, states: Optional[List[str]] = None,
                           compute_boundary_lengths: bool = True, water_distance: float = 1.0,
                           minimum_boundary_length: float = 10.0, force: bool = False,
                           dry_run: bool = False, is_standalone: bool = None):
    """
    Build adjacency graphs for all states.

    Args:
        year: Census year
        error_log_file: Path to error log
        states: List of state codes (None = all states)
        compute_boundary_lengths: Whether to compute boundary lengths for edge weighting
        water_distance: Water distance threshold in km
        minimum_boundary_length: Minimum shared boundary length (meters) to filter tiny adjacencies
        force: Force rebuild (delete existing graphs)
        dry_run: Show what would be done without executing
        is_standalone: Whether running standalone (None = auto-detect)
    """
    report_progress(f"[3/5] Building adjacency graphs ({year})", is_standalone)

    if states is None:
        states = ALL_STATES

    script = 'scripts/data/geography/build_all_adjacency_graphs.py'

    if not Path(script).exists():
        error_msg = f"Script not found: {script}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, "build_adjacency_graphs", error_msg)
        return False

    # Call build_all_adjacency_graphs.py with correct paths
    cmd = [
        sys.executable, script,
        '--year', str(year),
        '--input-dir', f'outputs/data/tracts/{year}',
        '--output-dir', f'outputs/data/adjacency/{year}',
        '--water-distance', str(water_distance),
        '--minimum-boundary-length', str(minimum_boundary_length)
    ]

    if compute_boundary_lengths:
        cmd.append('--compute-boundary-lengths')

    if force:
        cmd.append('--reset')

    success = run_command(cmd, f"Build adjacency graphs ({year})", error_log_file, dry_run, is_standalone)
    return success


def process_election_data(year: int, error_log_file: Path, election_year: Optional[int] = None,
                          force: bool = False, dry_run: bool = False, is_standalone: bool = None):
    """Process election data to tract-level parquet."""
    report_progress(f"[4/5] Processing election data", is_standalone)

    if election_year is None:
        election_year = year

    # Check if raw election data exists
    if year == 2020:
        raw_csv = Path(f'data/Census {year}/elections/2020_president/Contiguous USA - Main Method/Census Tracts/tracts-2020-RLCR.csv')
    else:
        raw_csv = Path(f'data/Census {year}/elections/tracts-{year}.csv')

    if not raw_csv.exists():
        report_progress(f"[SKIP] No election data found", is_standalone)
        return True

    script = 'scripts/data/elections/process_election_data.py'

    if not Path(script).exists():
        error_msg = f"Script not found: {script}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, "process_election_data", error_msg)
        return False

    output_dir = Path(f'outputs/data/elections')
    output_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        sys.executable, script,
        '--year', str(election_year),
        '--input-file', str(raw_csv),
        '--output-dir', str(output_dir)
    ]

    success = run_command(cmd, f"Process election data ({election_year})", error_log_file, dry_run, is_standalone)
    return success


def process_demographic_data(year: int, error_log_file: Path, force: bool = False,
                              dry_run: bool = False, is_standalone: bool = None):
    """Process demographic data to tract-level parquet."""
    report_progress(f"[5/5] Processing demographic data", is_standalone)

    # Check if raw demographic data exists
    raw_data_dir = Path(f'data/Census {year}/demographics')
    if not raw_data_dir.exists() or not list(raw_data_dir.glob('*.csv')):
        report_progress(f"[SKIP] No demographic data found", is_standalone)
        return True

    script = 'scripts/data/demographics/process_demographic_data.py'

    if not Path(script).exists():
        error_msg = f"Script not found: {script}"
        report_progress(f"[ERROR] {error_msg}", is_standalone)
        log_error(error_log_file, "process_demographic_data", error_msg)
        return False

    output_dir = Path(f'outputs/data/demographics')
    output_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        sys.executable, script,
        '--year', str(year),
        '--input-dir', str(raw_data_dir),
        '--output-dir', str(output_dir)
    ]

    success = run_command(cmd, f"Process demographic data ({year})", error_log_file, dry_run, is_standalone)
    return success


def validate_outputs(year: int, states: Optional[List[str]] = None):
    """Validate all required outputs exist."""
    print(f"\n[7/7] Validating outputs...")

    if states is None:
        states = ALL_STATES

    required_files = []
    missing_files = []

    # Check tract files
    for state in states:
        tract_file = get_tract_file(state, str(year))
        required_files.append(tract_file)
        if not tract_file.exists():
            missing_files.append(tract_file)

    # Check adjacency files
    for state in states:
        adj_file = get_adjacency_file(state, str(year))
        required_files.append(adj_file)
        if not adj_file.exists():
            missing_files.append(adj_file)

    print(f"  Total required files: {len(required_files)}")
    print(f"  Existing files: {len(required_files) - len(missing_files)}")
    print(f"  Missing files: {len(missing_files)}")

    if missing_files and len(missing_files) <= 10:
        print(f"\n  Missing files:")
        for f in missing_files[:10]:
            print(f"    - {f}")

    if missing_files:
        print(f"\n  [WARN] {len(missing_files)} files missing")
        return False
    else:
        print(f"\n  [OK] All required files exist!")
        return True


def main():
    parser = argparse.ArgumentParser(
        description='Build all processed data for a census year',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Process all data for 2020
  python scripts/data/build_all_processed_data.py --year 2020

  # Process only tracts and adjacency
  python scripts/data/build_all_processed_data.py --year 2020 --stages tracts adjacency

  # Process only specific states
  python scripts/data/build_all_processed_data.py --year 2020 --states CA TX NY

  # Force reprocess (skip existing files)
  python scripts/data/build_all_processed_data.py --year 2020 --force

  # Dry run (show what would be done)
  python scripts/data/build_all_processed_data.py --year 2020 --dry-run
        """
    )

    parser.add_argument('--year', type=int, required=True, choices=[2000, 2010, 2020],
                       help='Census year')
    parser.add_argument('--resolution', type=str, nargs='+', default=['tract'],
                       choices=['tract', 'block'],
                       help='Resolution levels to build (default: tract)')
    parser.add_argument('--output-dir', type=str,
                       help='Output directory for results and error log (default: outputs/v1/{year} or outputs/dev/)')
    parser.add_argument('--states', type=str, nargs='*',
                       help='Process only specific states (e.g., CA TX NY)')
    parser.add_argument('--stages', type=str, nargs='*',
                       choices=['tracts', 'adjacency', 'elections', 'demographics'],
                       help='Process only specific stages (default: all except elections/demographics if not available)')
    parser.add_argument('--election-year', type=int,
                       help='Election year (if different from census year)')
    parser.add_argument('--compute-boundary-lengths', action='store_true', default=True,
                       help='Compute boundary lengths for edge-weighted partitioning (default: True)')
    parser.add_argument('--water-distance', type=float, default=1.0,
                       help='Water distance threshold in km (default: 1.0)')
    parser.add_argument('--minimum-boundary-length', type=float, default=10.0,
                       help='Minimum shared boundary length (meters) to filter tiny adjacencies (default: 10)')
    parser.add_argument('--force', action='store_true',
                       help='Force reprocessing (skip existing files)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Show what would be done without executing')

    args = parser.parse_args()

    # Validate states
    states = None
    if args.states:
        states = [s.upper() for s in args.states]
        invalid_states = [s for s in states if s not in ALL_STATES]
        if invalid_states:
            print(f"ERROR: Invalid states: {', '.join(invalid_states)}")
            return 1

    # Set up output directory and error log
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        # Default to outputs/dev/ for testing
        output_dir = Path(f'outputs/dev/census_data_{args.year}')

    output_dir.mkdir(parents=True, exist_ok=True)
    error_log_file = output_dir / 'error.log'

    # Auto-detect standalone mode
    is_standalone = int(os.environ.get('TQDM_POSITION', '-1')) < 0

    # Determine stages to run
    all_stages = ['tracts', 'adjacency', 'elections', 'demographics']
    requested_stages = args.stages if args.stages else ['tracts', 'adjacency']  # Default: core stages only

    # Resolution-aware processing
    resolution = args.resolution[0] if args.resolution else 'tract'

    # Check which stages are already complete (per-stage markers: .tract_tracts_complete, etc.)
    stages_to_run = []
    stages_skipped = []
    for stage in requested_stages:
        if check_stage_complete(output_dir, stage, resolution, args.force):
            stages_skipped.append(stage)
        else:
            stages_to_run.append(stage)

    # If all requested stages complete, exit early
    if not stages_to_run:
        if is_standalone:
            print("=" * 70)
            print(f"All {resolution}-level stages already complete:")
            for stage in requested_stages:
                marker = get_stage_marker(output_dir, stage, resolution)
                print(f"  [OK] {stage}: {marker.name}")
            print("=" * 70)
            print("Use --force to reprocess")
        return 0

    # Update stages to only process what's needed
    stages = stages_to_run

    if is_standalone and stages_skipped:
        print("=" * 70)
        print(f"Skipping {len(stages_skipped)} completed stages:")
        for stage in stages_skipped:
            marker = get_stage_marker(output_dir, stage, resolution)
            print(f"  [SKIP] {stage}: {marker.name}")
        print("=" * 70)

    # Standalone mode: print header
    if is_standalone:
        print("=" * 70)
        print("PROCESS CENSUS DATA")
        print("=" * 70)
        print(f"Census year: {args.year}")
        print(f"Resolution: {', '.join(args.resolution)}")
        print(f"States: {len(states) if states else len(ALL_STATES)} ({', '.join(states[:5]) if states else 'all'}{'...' if states and len(states) > 5 else ''})")
        print(f"Stages: {', '.join(stages)}")
        print(f"Election year: {args.election_year or args.year}")
        print(f"Output dir: {output_dir}")
        print(f"Force reprocess: {args.force}")
        print(f"Dry run: {args.dry_run}")
        print("=" * 70)

        if args.dry_run:
            print("\n[DRY RUN MODE - No changes will be made]\n")

    start_time = time.time()

    # Phase 1: Create directory structure
    create_directory_structure(args.year, is_standalone)

    # Phase 2-5: Run processing stages for each resolution level
    results = {}

    for resolution in args.resolution:
        report_progress(f"Processing {resolution}-level data", is_standalone)

        # TODO: Update processing functions to accept resolution parameter
        # For now, tract is the only implemented resolution

    if 'tracts' in stages:
        results['tracts'] = process_census_tracts(args.year, error_log_file, states, args.force, args.dry_run, is_standalone)
        if results['tracts']:
            create_stage_marker(output_dir, 'tracts', resolution, args.dry_run)

    if 'adjacency' in stages:
        results['adjacency'] = build_adjacency_graphs(
            args.year, error_log_file, states,
            args.compute_boundary_lengths, args.water_distance, args.minimum_boundary_length,
            args.force, args.dry_run, is_standalone
        )
        if results['adjacency']:
            create_stage_marker(output_dir, 'adjacency', resolution, args.dry_run)

    if 'elections' in stages:
        results['elections'] = process_election_data(args.year, error_log_file, args.election_year, args.force, args.dry_run, is_standalone)
        if results['elections']:
            create_stage_marker(output_dir, 'elections', resolution, args.dry_run)

    if 'demographics' in stages:
        results['demographics'] = process_demographic_data(args.year, error_log_file, args.force, args.dry_run, is_standalone)
        if results['demographics']:
            create_stage_marker(output_dir, 'demographics', resolution, args.dry_run)

    elapsed = time.time() - start_time
    success = all(results.values())

    # Report marker creation in standalone mode
    if success and not args.dry_run and is_standalone:
        print("\n" + "=" * 70)
        print("MARKER FILES CREATED:")
        for stage in stages:
            marker = get_stage_marker(output_dir, stage, resolution)
            print(f"  [OK] {marker.name}")
        print("=" * 70)

    # Standalone mode: print summary
    if is_standalone:
        print("\n" + "=" * 70)
        print("STAGE RESULTS:")
        for stage, stage_success in results.items():
            status = "[OK]" if stage_success else "[FAIL]"
            print(f"  {status} {stage}")
        print("=" * 70)

        if args.dry_run:
            print("DRY RUN COMPLETE")
        elif success:
            print("PROCESSING COMPLETE")
        else:
            print("PROCESSING INCOMPLETE - Some stages failed")
            print(f"Check error log: {error_log_file}")
        print("=" * 70)
        print(f"Elapsed time: {elapsed:.1f}s")
        print("=" * 70)
    else:
        # Worker mode: final status
        if success:
            report_progress(f"[OK] All stages complete ({elapsed:.1f}s)", is_standalone)
        else:
            report_progress(f"[FAIL] Processing incomplete ({elapsed:.1f}s)", is_standalone)

    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())
