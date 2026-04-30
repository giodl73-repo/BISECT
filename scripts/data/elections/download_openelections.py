"""
Download per-state OpenElections precinct-level results.

OpenElections (https://github.com/openelections) is the most-comprehensive
opensource archive of US precinct-level election data. Each state lives in
its own GitHub repo: github.com/openelections/openelections-data-{STATE}.

This fetcher does a shallow git clone of the requested state(s) into the
output directory. After clone, the per-state CSV layout varies — see each
repo's README.

Why git clone instead of a tarball download:
  - GitHub serves snapshot tarballs but they require API rate-limiting
    knowledge for batches.
  - Git clone --depth=1 is fast (one HTTPS round-trip) and resumable.
  - Users can `git pull` later for updates.

Schema variation: each state's CSVs use different column names (e.g.,
G20PREDBID for 2020 Biden, BIDEN_2020, etc.). Per-state schema mapping
is the consumer's responsibility — not done here. Use this script to
fetch raw data; transform downstream.

Example invocations:

    # Clone California's OpenElections repo
    python scripts/data/elections/download_openelections.py --states CA

    # Clone multiple states
    python scripts/data/elections/download_openelections.py --states CA TX NY

    # Override output dir
    python scripts/data/elections/download_openelections.py --states VT \\
        --output-dir outputs/data/raw/openelections

Plan 03 / Callais context:
The new Gingles 2/3 standard requires controlling for partisan affiliation
when proving racial-bloc voting. Primary-election precinct data is the most
direct evidence; OpenElections is the canonical opensource source. Use this
to fetch raw state archives, then build per-state aggregators that produce
the partisan-shares.tsv format consumed by `redist state --partisan-shares`
or the election CSV consumed by `redist analyze --types political`.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable

GITHUB_ORG = "openelections"
REPO_PREFIX = "openelections-data-"

# State codes recognized by OpenElections (most US states, all 50 are covered).
# Generated 2026-04-29 from github.com/openelections — verify via gh api if updating.
KNOWN_STATES = {
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "DC", "FL", "GA", "HI",
    "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN",
    "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND", "OH",
    "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA",
    "WV", "WI", "WY",
}


def repo_url(state_code: str) -> str:
    return f"https://github.com/{GITHUB_ORG}/{REPO_PREFIX}{state_code.lower()}.git"


def clone_state(state_code: str, output_dir: Path, depth: int = 1, force: bool = False) -> int:
    """Clone one state's OpenElections repo. Returns 0 on success, non-zero on failure."""
    state_code = state_code.upper()
    if state_code not in KNOWN_STATES:
        print(f"WARNING: '{state_code}' not in the known-state list; trying anyway.")

    repo_dir = output_dir / f"{REPO_PREFIX}{state_code.lower()}"
    if repo_dir.exists():
        if not force:
            print(f"[skip] {state_code}: {repo_dir} exists (pass --force to re-clone)")
            return 0
        shutil.rmtree(repo_dir)

    output_dir.mkdir(parents=True, exist_ok=True)
    cmd = ["git", "clone", "--depth", str(depth), repo_url(state_code), str(repo_dir)]
    print(f"[run] {' '.join(cmd)}")
    result = subprocess.run(cmd)
    if result.returncode != 0:
        print(f"[FAIL] {state_code}: git clone exited {result.returncode}")
        return result.returncode
    print(f"[OK]  {state_code} -> {repo_dir}")
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--states", nargs="+", required=True,
                   help="State codes to clone (e.g., CA TX NY). Use 'ALL' for every known state.")
    p.add_argument("--output-dir", default="data/raw/openelections",
                   help="Where to clone the repos (default: data/raw/openelections)")
    p.add_argument("--depth", type=int, default=1,
                   help="git clone --depth (default 1; use 0 to disable)")
    p.add_argument("--force", action="store_true",
                   help="Re-clone if the target directory already exists")
    args = p.parse_args(argv)

    if shutil.which("git") is None:
        print("ERROR: git not found on PATH", file=sys.stderr)
        return 2

    states: Iterable[str] = (
        sorted(KNOWN_STATES) if [s.upper() for s in args.states] == ["ALL"]
        else args.states
    )

    output_dir = Path(args.output_dir)
    failures = []
    for s in states:
        depth = args.depth if args.depth > 0 else None
        rc = clone_state(s, output_dir, depth=depth or 1, force=args.force)
        if rc != 0:
            failures.append(s)

    if failures:
        print(f"\n[SUMMARY] {len(failures)} failed: {', '.join(failures)}", file=sys.stderr)
        return 1
    print(f"\n[SUMMARY] cloned {len(list(states))} state(s) under {output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
