#!/usr/bin/env python3
"""
Download data for the REDIST Congressional Redistricting project.

Three categories of data:

  1. Curated inputs  (GitHub Release: data-inputs-v1)
     Pre-processed elections, demographics, baseline enacted districts, metro areas.
     ~400MB download. Needed to run analysis and comparison steps.

  2. MAUP adjacency graphs  (GitHub Release: data-inputs-v1)
     Pre-computed block and block-group adjacency graphs for all 50 states.
     ~200MB download. Needed only for multi-resolution (MAUP) sensitivity analysis.

  3. Raw Census data  (downloaded directly from US Census Bureau)
     TIGER/Line shapefiles and PL 94-171 redistricting files.
     ~55GB total. Needed to run the full pipeline from scratch.

  4. Pipeline outputs  (GitHub Release: outputs-v3 / outputs-v4)
     Pre-computed district assignments, compactness scores, and analysis CSVs.
     Download these to explore results without running the pipeline.

Usage:
  python setup_data.py --inputs          # curated inputs + adjacency
  python setup_data.py --outputs v3      # V3 edge-weighted results
  python setup_data.py --outputs v4      # V4 VRA results
  python setup_data.py --census 2020     # raw Census data (large, slow)
  python setup_data.py --all             # everything except raw Census
"""

import argparse
import subprocess
import sys
import os
from pathlib import Path

REPO = "giodl73-repo/REDIST"

RELEASES = {
    "inputs": {
        "tag": "data-inputs-v1",
        "files": ["data-inputs.tar.gz", "adjacency-maup.tar.gz"],
        "desc": "Curated input data (~600MB)",
    },
    "v3": {
        "tag": "outputs-v3",
        "files": ["outputs-v3.tar.gz"],
        "desc": "V3 edge-weighted pipeline outputs",
    },
    "v4": {
        "tag": "outputs-v4",
        "files": ["outputs-v4.tar.gz"],
        "desc": "V4 VRA pipeline outputs",
    },
}


def check_gh():
    try:
        subprocess.run(["gh", "--version"], capture_output=True, check=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def download_release(tag, files, desc):
    print(f"\nDownloading {desc}...")
    for fname in files:
        print(f"  Fetching {fname} from release {tag}...")
        result = subprocess.run(
            ["gh", "release", "download", tag, "--repo", REPO,
             "--pattern", fname, "--skip-existing"],
            capture_output=True, text=True
        )
        if result.returncode != 0:
            print(f"  ERROR: {result.stderr.strip()}")
            return False

        tarball = Path(fname)
        if tarball.exists():
            print(f"  Extracting {fname}...")
            subprocess.run(["tar", "-xzf", str(tarball)], check=True)
            tarball.unlink()
            print(f"  Done.")

    return True


def download_census(year):
    print(f"\nDownloading raw Census data for {year} (~25GB, this will take a while)...")
    print("  This downloads directly from the US Census Bureau.")
    result = subprocess.run(
        [sys.executable, "scripts/data/download_orchestrator.py",
         "--stages", "redistricting", "demographics",
         "--year", str(year), "--workers", "4"],
    )
    return result.returncode == 0


def main():
    parser = argparse.ArgumentParser(description="Set up REDIST project data")
    parser.add_argument("--inputs", action="store_true",
                        help="Download curated inputs + adjacency graphs")
    parser.add_argument("--outputs", choices=["v3", "v4", "both"],
                        help="Download pre-computed pipeline outputs")
    parser.add_argument("--census", type=str, choices=["2020", "2010", "2000"],
                        help="Download raw Census data for a year (large)")
    parser.add_argument("--all", action="store_true",
                        help="Download inputs + v3 + v4 outputs (no raw Census)")
    args = parser.parse_args()

    if not any([args.inputs, args.outputs, args.census, args.all]):
        parser.print_help()
        return

    if not check_gh():
        print("ERROR: GitHub CLI (gh) is required. Install from https://cli.github.com/")
        print("  Then run: gh auth login")
        sys.exit(1)

    if args.all:
        args.inputs = True
        args.outputs = "both"

    if args.inputs:
        download_release(**RELEASES["inputs"])

    if args.outputs in ("v3", "both"):
        download_release(**RELEASES["v3"])

    if args.outputs in ("v4", "both"):
        download_release(**RELEASES["v4"])

    if args.census:
        download_census(args.census)

    print("\nDone! Run the pipeline with:")
    print("  run -y 2020 -v v1")


if __name__ == "__main__":
    main()
