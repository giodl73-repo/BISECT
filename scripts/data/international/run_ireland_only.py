"""
Run Ireland redistricting with appropriate parameters for 106-unit GADM dataset.

The GADM level 2 dataset has 106 municipal districts with uniform population.
With 106 units and 43 districts, each district gets ~2.5 units — too coarse
for ±5% balance. We use ±25% to match German Bundeswahlgesetz tolerance, which
is appropriate for a coarse prototype.

Note: For the E.6 paper, the key result is compactness/contiguity, not
exact population balance — that requires CSO small area data (~18K units).

Usage:
    python scripts/data/international/run_ireland_only.py
    python scripts/data/international/run_ireland_only.py --tolerance 25
    python scripts/data/international/run_ireland_only.py --districts 26
"""

import argparse
import subprocess
from pathlib import Path
import sys

REDIST = Path("redist/target/release/redist.exe")
if not REDIST.exists():
    REDIST = Path("redist/target/release/redist")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tolerance", type=float, default=25.0,
                        help="Balance tolerance %% (default: 25)")
    parser.add_argument("--districts", type=int, default=43,
                        help="Number of districts (default: 43)")
    parser.add_argument("--label", default="ireland_dail_2022",
                        help="Plan label")
    parser.add_argument("--analyze-only", action="store_true")
    args = parser.parse_args()

    if not REDIST.exists():
        print(f"ERROR: redist binary not found at {REDIST}")
        sys.exit(1)

    adj = Path("outputs/international/ireland/ie_adjacency_2022.adj.bin")
    if not adj.exists():
        adj = Path("outputs/international/ireland/ie_adjacency_2022.pkl")
    if not adj.exists():
        print("ERROR: No adjacency file found")
        sys.exit(1)

    print(f"=== Ireland Redistricting ===")
    print(f"  Adjacency: {adj} ({adj.stat().st_size} bytes)")
    print(f"  Districts: {args.districts}, Tolerance: {args.tolerance}%")

    if not args.analyze_only:
        cmd = [
            str(REDIST), "state",
            "--state", "IE",
            "--year", "2020",
            "--version", "international",
            "--adjacency", str(adj),
            "--state-name", "ireland",
            "--districts", str(args.districts),
            "--seats-per-district", "4",
            "--chamber", "parliamentary",
            "--balance-tolerance", str(args.tolerance),
            "--label", args.label,
            "--force",
        ]
        print(f"\n[CMD] {' '.join(cmd)}\n")
        result = subprocess.run(cmd, text=True)
        print(f"\nExit code: {result.returncode}")

        if result.returncode != 0:
            # Try without seats-per-district
            print("\n[RETRY] Without --seats-per-district...")
            cmd2 = [
                str(REDIST), "state",
                "--state", "IE",
                "--year", "2020",
                "--version", "international",
                "--adjacency", str(adj),
                "--state-name", "ireland",
                "--districts", str(args.districts),
                "--chamber", "parliamentary",
                "--balance-tolerance", str(args.tolerance),
                "--label", args.label,
                "--force",
            ]
            print(f"\n[CMD] {' '.join(cmd2)}\n")
            result = subprocess.run(cmd2, text=True)
            print(f"\nExit code: {result.returncode}")
            if result.returncode != 0:
                print(f"[FAIL] Redistricting failed")
                return

    print(f"\n=== Analysis ===")
    cmd = [
        str(REDIST), "analyze",
        "--label", args.label,
        "--year", "2020",
        "--version", "international",
        "--types", "compactness", "contiguity",
        "--allow-imbalance",
        "--allow-noncontiguous",
        "--force",
    ]
    print(f"[CMD] {' '.join(cmd)}\n")
    result = subprocess.run(cmd, text=True)
    print(f"\nAnalysis exit code: {result.returncode}")

    # Check for output files
    plan_dirs = list(Path("outputs/international").glob(f"**/plans/{args.label}"))
    if plan_dirs:
        for d in plan_dirs:
            print(f"\n[OK] Plan directory: {d}")
            for f in sorted(d.iterdir()):
                print(f"     {f.name} ({f.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
