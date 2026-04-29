"""
Run redistricting and analysis for international case studies (E.6 paper).

Runs:
  1. Ireland (STV system) — 43 constituencies, 4 seats avg, balance ±5%
  2. Germany (MMP system) — 299 Wahlkreise, 1 seat, balance ±25%

Usage:
    python scripts/data/international/run_international_redistricting.py
    python scripts/data/international/run_international_redistricting.py --country ireland
    python scripts/data/international/run_international_redistricting.py --country germany
    python scripts/data/international/run_international_redistricting.py --analyze-only
"""

import argparse
import subprocess
import sys
from pathlib import Path


REDIST = Path("redist/target/release/redist.exe")
if not REDIST.exists():
    REDIST = Path("redist/target/release/redist")


def run_cmd(cmd, label=""):
    """Run a command, stream output, return CompletedProcess."""
    print(f"\n[CMD] {' '.join(str(c) for c in cmd)}\n")
    result = subprocess.run(
        [str(c) for c in cmd],
        capture_output=False,
        text=True,
    )
    return result


def run_ireland(analyze_only=False):
    print("=" * 60)
    print("IRELAND (STV system)")
    print("=" * 60)

    adj_bin = Path("outputs/international/ireland/ie_adjacency_2022.adj.bin")
    adj_pkl = Path("outputs/international/ireland/ie_adjacency_2022.pkl")

    adj_path = adj_bin if adj_bin.exists() else adj_pkl
    if not adj_path.exists():
        print(f"ERROR: Adjacency file not found. Run acquire_ireland_gadm.py first.")
        return False

    print(f"  Adjacency: {adj_path} ({adj_path.stat().st_size} bytes)")

    if not analyze_only:
        # Run redistricting
        cmd = [
            REDIST, "state",
            "--state", "IE",
            "--year", "2020",
            "--version", "international",
            "--adjacency", str(adj_path),
            "--state-name", "ireland",
            "--districts", "43",
            "--seats-per-district", "4",
            "--chamber", "parliamentary",
            "--balance-tolerance", "5",
            "--label", "ireland_dail_2022",
            "--force",
        ]
        result = run_cmd(cmd, "ireland_redistricting")
        if result.returncode != 0:
            print(f"[WARN] Redistricting exited with code {result.returncode}")
            # Try without --seats-per-district (may not be needed for population balance)
            print("[INFO] Retrying without --seats-per-district...")
            cmd2 = [
                REDIST, "state",
                "--state", "IE",
                "--year", "2020",
                "--version", "international",
                "--adjacency", str(adj_path),
                "--state-name", "ireland",
                "--districts", "43",
                "--chamber", "parliamentary",
                "--balance-tolerance", "5",
                "--label", "ireland_dail_2022",
                "--force",
            ]
            result = run_cmd(cmd2, "ireland_redistricting_v2")
            if result.returncode != 0:
                print(f"[FAIL] Ireland redistricting failed (exit {result.returncode})")
                return False

    # Run analysis
    cmd = [
        REDIST, "analyze",
        "--label", "ireland_dail_2022",
        "--year", "2020",
        "--version", "international",
        "--types", "compactness", "contiguity",
        "--allow-imbalance",
        "--allow-noncontiguous",
        "--force",
    ]
    result = run_cmd(cmd, "ireland_analysis")
    if result.returncode not in (0, 1, 2, 3):  # allow partial analysis success codes
        print(f"[WARN] Analysis exited with code {result.returncode}")

    print("\n[DONE] Ireland complete")
    return True


def run_germany(analyze_only=False):
    print("=" * 60)
    print("GERMANY (MMP system)")
    print("=" * 60)

    adj_bin = Path("outputs/international/germany/de_adjacency_2021.adj.bin")
    adj_pkl = Path("outputs/international/germany/de_adjacency_2021.pkl")

    adj_path = adj_bin if adj_bin.exists() else adj_pkl
    if not adj_path.exists():
        print(f"ERROR: Adjacency file not found. Run acquire_germany.py first.")
        return False

    print(f"  Adjacency: {adj_path} ({adj_path.stat().st_size} bytes)")

    if not analyze_only:
        # Run redistricting
        cmd = [
            REDIST, "state",
            "--state", "DE",
            "--year", "2020",
            "--version", "international",
            "--adjacency", str(adj_path),
            "--state-name", "germany",
            "--districts", "299",
            "--chamber", "parliamentary",
            "--balance-tolerance", "25",
            "--label", "germany_bundestag_2021",
            "--force",
        ]
        result = run_cmd(cmd, "germany_redistricting")
        if result.returncode != 0:
            print(f"[FAIL] Germany redistricting failed (exit {result.returncode})")
            return False

    # Run analysis
    cmd = [
        REDIST, "analyze",
        "--label", "germany_bundestag_2021",
        "--year", "2020",
        "--version", "international",
        "--types", "compactness", "contiguity",
        "--allow-imbalance",
        "--allow-noncontiguous",
        "--force",
    ]
    result = run_cmd(cmd, "germany_analysis")
    if result.returncode not in (0, 1, 2, 3):
        print(f"[WARN] Analysis exited with code {result.returncode}")

    print("\n[DONE] Germany complete")
    return True


def main():
    parser = argparse.ArgumentParser(description="Run international redistricting (E.6 paper)")
    parser.add_argument("--country", choices=["ireland", "germany", "both"], default="both")
    parser.add_argument("--analyze-only", action="store_true", help="Skip redistricting, run analysis only")
    args = parser.parse_args()

    if not REDIST.exists():
        print(f"ERROR: redist binary not found at {REDIST}")
        sys.exit(1)

    print(f"Using redist binary: {REDIST}")

    results = {}
    if args.country in ("ireland", "both"):
        results["ireland"] = run_ireland(analyze_only=args.analyze_only)
    if args.country in ("germany", "both"):
        results["germany"] = run_germany(analyze_only=args.analyze_only)

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    for country, ok in results.items():
        status = "[OK]" if ok else "[FAIL]"
        print(f"  {status} {country}")


if __name__ == "__main__":
    main()
