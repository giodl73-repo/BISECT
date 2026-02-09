#!/usr/bin/env python3
"""
Validate existing download data and optionally create completion markers.

This script scans your existing data directories and validates:
1. Directory existence
2. State-level completeness (all required states present)
3. Optionally creates .download_{stage}_complete marker files

Usage:
    # Check what data exists (no markers created)
    python scripts/data/validate_downloads.py --year 2020 --check-only

    # Validate and create markers for complete data
    python scripts/data/validate_downloads.py --year 2020 --create-markers

    # Validate all years
    python scripts/data/validate_downloads.py --year all --check-only

    # Validate specific stages
    python scripts/data/validate_downloads.py --year 2020 --stages redistricting demographics

Created: 2026-01-18 (Enhancement #48)
"""

import sys
import argparse
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.data.download_stages import (
    STAGE_REQUIREMENTS,
    check_raw_data_exists,
    check_all_states_present,
    get_required_states_for_stage,
    create_download_marker,
    check_download_marker,
    remove_download_marker
)


def validate_stage(year: int, stage: str, create_markers: bool = False) -> dict:
    """
    Validate a stage's data completeness.

    Args:
        year: Census year
        stage: Stage name
        create_markers: If True, create marker files for complete data

    Returns:
        Dict with validation results
    """
    print(f"\n  [{stage.upper()}] {STAGE_REQUIREMENTS[stage]['description']}")

    # Check if marker already exists
    has_marker = check_download_marker(year, stage)
    if has_marker:
        print(f"    [MARKER] Completion marker exists: data/{year}/.download_{stage}_complete")

    # Check raw data (skip marker check to force validation)
    exists, missing_items = check_raw_data_exists(year, stage, check_marker=False)

    if exists:
        # Check state-level completeness
        all_present, missing_states = check_all_states_present(year, stage)
        required_states = get_required_states_for_stage(stage)

        if not required_states:
            # No per-state files needed (national files or generated data)
            print(f"    [OK] Data complete (no per-state validation needed)")
            status = 'complete'
        elif all_present:
            print(f"    [OK] All {len(required_states)} required states present")
            status = 'complete'
        else:
            print(f"    [INCOMPLETE] {len(missing_states)}/{len(required_states)} states missing:")
            for state in missing_states[:5]:
                print(f"      - {state}")
            if len(missing_states) > 5:
                print(f"      - ... and {len(missing_states) - 5} more")
            status = 'incomplete'

        # Create marker if requested and data is complete
        if create_markers and status == 'complete' and not has_marker:
            create_download_marker(year, stage)
            print(f"    [CREATED] Marker file created")
        elif has_marker and status == 'incomplete':
            # Remove stale marker
            remove_download_marker(year, stage)
            print(f"    [REMOVED] Stale marker removed (data incomplete)")

    else:
        print(f"    [MISSING] Base directories or files not found:")
        for item in missing_items[:3]:
            print(f"      - {item}")
        if len(missing_items) > 3:
            print(f"      - ... and {len(missing_items) - 3} more")
        status = 'missing'

        # Remove marker if data is missing
        if has_marker:
            remove_download_marker(year, stage)
            print(f"    [REMOVED] Stale marker removed (data missing)")

    return {
        'stage': stage,
        'year': year,
        'status': status,
        'has_marker': check_download_marker(year, stage),
        'missing_items': missing_items if not exists else []
    }


def main():
    parser = argparse.ArgumentParser(
        description='Validate existing download data and optionally create completion markers'
    )
    parser.add_argument('--year', type=str, default='2020',
                       choices=['2020', '2010', '2000', 'all'],
                       help='Census year to validate (default: 2020)')
    parser.add_argument('--stages', type=str, nargs='*',
                       choices=['redistricting', 'demographics', 'elections', 'places', 'metros', 'enacted_districts', 'adjacency'],
                       help='Specific stages to validate (default: all applicable stages)')
    parser.add_argument('--create-markers', action='store_true',
                       help='Create .download_{stage}_complete marker files for complete data')
    parser.add_argument('--check-only', action='store_true',
                       help='Only check data, do not create or remove markers')
    args = parser.parse_args()

    # Build year list
    if args.year == 'all':
        years = [2020, 2010, 2000]
    else:
        years = [int(args.year)]

    # Build stage list
    if args.stages:
        stages = args.stages
    else:
        stages = ['redistricting', 'demographics', 'elections', 'places', 'metros', 'enacted_districts', 'adjacency']

    print("="*70)
    print("DOWNLOAD DATA VALIDATION")
    print("="*70)
    print(f"Years: {', '.join(str(y) for y in years)}")
    print(f"Stages: {', '.join(stages)}")
    if args.create_markers:
        print("Mode: Validate and create markers for complete data")
    elif args.check_only:
        print("Mode: Check only (no marker changes)")
    else:
        print("Mode: Validate (will remove stale markers but not create new ones)")
    print("="*70)

    results = []

    for year in years:
        print(f"\n[{year}] Validating census data...")

        for stage in stages:
            # Skip if stage not applicable for this year
            if year not in STAGE_REQUIREMENTS[stage].get('check_files', {}):
                print(f"\n  [{stage.upper()}] Not applicable for {year} census")
                continue

            result = validate_stage(
                year,
                stage,
                create_markers=args.create_markers and not args.check_only
            )
            results.append(result)

    # Summary
    print("\n" + "="*70)
    print("VALIDATION SUMMARY")
    print("="*70)

    complete_count = sum(1 for r in results if r['status'] == 'complete')
    incomplete_count = sum(1 for r in results if r['status'] == 'incomplete')
    missing_count = sum(1 for r in results if r['status'] == 'missing')
    marker_count = sum(1 for r in results if r['has_marker'])

    print(f"Complete: {complete_count}/{len(results)} stages")
    print(f"Incomplete: {incomplete_count}/{len(results)} stages")
    print(f"Missing: {missing_count}/{len(results)} stages")
    print(f"Markers: {marker_count}/{len(results)} stages have completion markers")

    if complete_count > 0:
        print(f"\n[OK] {complete_count} stage(s) have complete data")
    if incomplete_count > 0:
        print(f"[WARN] {incomplete_count} stage(s) have partial data")
    if missing_count > 0:
        print(f"[MISSING] {missing_count} stage(s) have no data")

    if args.check_only:
        print("\n[CHECK-ONLY MODE] No markers were created or removed")
    elif args.create_markers:
        print(f"\n[MARKERS] Created markers for {complete_count} complete stages")
    else:
        print("\n[INFO] To create markers, run with --create-markers")

    print("="*70)

    return 0


if __name__ == '__main__':
    sys.exit(main())
