#!/usr/bin/env python3
"""
Save/backup a completed redistricting run to a safe location.

This script copies an entire output directory to a backup location,
protecting against corruption or accidental deletion during subsequent runs.
"""

import argparse
import shutil
import sys
from pathlib import Path
from datetime import datetime


def save_run(
    year: str = None,
    version: str = None,
    output_dir: str = None,
    save_base: str = None,
    savename: str = None,
    dry_run: bool = False
):
    """
    Save a run directory to a backup location.

    Args:
        year: Census year (2020, 2010, etc.)
        version: Version string (v1, v2, etc.)
        output_dir: Path relative to outputs/ directory (overrides year/version)
        save_base: Base directory for saves (default: outputs/saved)
        savename: Name for the saved directory (default: same as source dir name)
        dry_run: If True, show what would be done without copying
    """

    # Determine source directory
    if output_dir:
        # Assume output_dir is relative to outputs/ directory
        source_dir = Path('outputs') / output_dir
    elif year and version:
        source_dir = Path(f'outputs/us_{year}_{version}')
    else:
        print("ERROR: Must provide either --output-dir OR both --year and --version")
        sys.exit(1)

    # Check if source exists
    if not source_dir.exists():
        print(f"ERROR: Source directory does not exist: {source_dir}")
        sys.exit(1)

    if not source_dir.is_dir():
        print(f"ERROR: Source path is not a directory: {source_dir}")
        sys.exit(1)

    # Determine save base directory
    if save_base is None:
        save_base = Path('outputs/saved')
    else:
        save_base = Path(save_base)

    # Determine save name
    if savename is None:
        savename = source_dir.name

    dest_dir = save_base / savename

    # Check if destination already exists
    if dest_dir.exists():
        print(f"WARNING: Destination already exists: {dest_dir}")
        response = input("Overwrite? (y/N): ").strip().lower()
        if response != 'y':
            print("Aborted.")
            sys.exit(0)

        if not dry_run:
            print(f"Removing existing destination: {dest_dir}")
            shutil.rmtree(dest_dir)

    # Show summary
    print("\n" + "=" * 70)
    print("SAVE RUN SUMMARY")
    print("=" * 70)
    print(f"Source:      {source_dir.absolute()}")
    print(f"Destination: {dest_dir.absolute()}")
    print(f"Savename:    {savename}")

    # Calculate size
    def get_dir_size(path):
        total = 0
        for entry in path.rglob('*'):
            if entry.is_file():
                total += entry.stat().st_size
        return total

    source_size = get_dir_size(source_dir)
    print(f"Size:        {source_size / (1024**3):.2f} GB ({source_size:,} bytes)")
    print("=" * 70)

    if dry_run:
        print("\nDRY RUN - No files copied")
        return

    # Perform the copy
    print("\nCopying...")
    save_base.mkdir(parents=True, exist_ok=True)

    try:
        shutil.copytree(source_dir, dest_dir, dirs_exist_ok=False)
        print(f"\n✓ Successfully saved to: {dest_dir.absolute()}")

        # Create a metadata file
        metadata_file = dest_dir / '.save_metadata.txt'
        with open(metadata_file, 'w') as f:
            f.write(f"Saved from: {source_dir.absolute()}\n")
            f.write(f"Saved at:   {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"Size:       {source_size:,} bytes\n")

        print(f"✓ Metadata written to: {metadata_file}")

    except Exception as e:
        print(f"\n✗ ERROR during copy: {e}")
        sys.exit(1)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Save/backup a redistricting run to a safe location',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Save a run by year and version
  python scripts/save_run.py --year 2020 --version v5

  # Save with a custom name
  python scripts/save_run.py --year 2020 --version v5 --savename us_2020_v5_backup

  # Save an explicit directory (relative to outputs/)
  python scripts/save_run.py --output-dir us_2020_v5

  # Save to a custom location
  python scripts/save_run.py --year 2020 --version v5 --save-base /mnt/backup

  # Dry run to see what would be done
  python scripts/save_run.py --year 2020 --version v5 --dry-run
"""
    )

    parser.add_argument('--year', type=str, choices=['2020', '2010', '2000'],
                       help='Census year (required unless --output-dir is used)')
    parser.add_argument('--version', type=str,
                       help='Version string like v1, v2, etc (required unless --output-dir is used)')
    parser.add_argument('--output-dir', type=str,
                       help='Path to output directory relative to outputs/ (overrides --year and --version)')
    parser.add_argument('--save-base', type=str, default='outputs/saved',
                       help='Base directory for saves (default: outputs/saved)')
    parser.add_argument('--savename', type=str,
                       help='Name for the saved directory (default: same as source dir name)')
    parser.add_argument('--dry-run', action='store_true',
                       help='Show what would be done without actually copying')

    args = parser.parse_args()

    # Validate arguments
    if not args.output_dir and not (args.year and args.version):
        parser.error("Must provide either --output-dir OR both --year and --version")

    save_run(
        year=args.year,
        version=args.version,
        output_dir=args.output_dir,
        save_base=args.save_base,
        savename=args.savename,
        dry_run=args.dry_run
    )
