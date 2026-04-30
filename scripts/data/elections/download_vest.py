"""
Download per-state VEST (Voting and Election Science Team) precinct shapefiles
+ results from Harvard Dataverse.

VEST publishes one Dataverse dataset per state per cycle; there's no central
index, so this script reads a local cache (vest_dois.json) of known
state/year → DOI mappings and dispatches to download_election_data.py.

Adding a new state/year:
  1. Browse https://dataverse.harvard.edu/dataverse/electionscience and find
     the dataset for the state + year you want.
  2. Find its DOI (e.g., 10.7910/DVN/ABCDEF on the dataset page).
  3. Run discovery to find the right filename:
        python scripts/data/elections/download_election_data.py \\
            --list-files --doi 10.7910/DVN/ABCDEF
  4. Add an entry to vest_dois.json under by_state_and_year.<STATE>.<YEAR>.

Most VEST datasets DO require a DATAVERSE_API_KEY (they're guestbook-protected
the same way MIT EDSL is). Get a token from dataverse.harvard.edu →
Account → API Token, then `export DATAVERSE_API_KEY=...`.

Plan 03 / Callais context: VEST is the gold-standard precinct shapefile +
result join used by Princeton Gerrymandering Project and other §2 expert
witnesses. The new Gingles 2/3 standard makes precinct-level election data
load-bearing — VEST is the cleanest single source per state.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

DOI_CACHE_PATH = Path(__file__).parent / "vest_dois.json"


def load_doi_cache() -> dict:
    if not DOI_CACHE_PATH.exists():
        print(f"ERROR: DOI cache not found at {DOI_CACHE_PATH}", file=sys.stderr)
        sys.exit(2)
    with DOI_CACHE_PATH.open(encoding="utf-8") as f:
        return json.load(f)


def lookup(cache: dict, state: str, year: str) -> dict | None:
    by_state = cache.get("by_state_and_year", {})
    state_entries = by_state.get(state.upper())
    if not state_entries:
        return None
    return state_entries.get(year)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--state", required=True, help="State code (e.g., WI)")
    p.add_argument("--year", required=True, help="Election year (e.g., 2020)")
    p.add_argument("--output-dir", default="data/raw/vest",
                   help="Where to put the downloaded data")
    p.add_argument("--list-known", action="store_true",
                   help="List all (state, year) pairs in the local DOI cache and exit")
    args = p.parse_args(argv)

    cache = load_doi_cache()

    if args.list_known:
        print("State    Year   DOI                        Filename")
        print("-" * 78)
        by = cache.get("by_state_and_year", {})
        for state, years in sorted(by.items()):
            if state.startswith("_"):
                continue  # skip _template
            for year, entry in sorted(years.items()):
                print(f"{state:<8} {year:<6} {entry.get('doi', '?'):<26} {entry.get('filename', '?')}")
        return 0

    entry = lookup(cache, args.state, args.year)
    if entry is None:
        print(
            f"No VEST DOI cached for {args.state.upper()} / {args.year}.\n"
            f"  1. Browse https://dataverse.harvard.edu/dataverse/electionscience\n"
            f"  2. Find the matching dataset's DOI\n"
            f"  3. python scripts/data/elections/download_election_data.py "
            f"--list-files --doi <DOI>\n"
            f"  4. Add the (state, year, doi, filename) tuple to "
            f"{DOI_CACHE_PATH}",
            file=sys.stderr,
        )
        return 1

    doi = entry["doi"]
    filename = entry["filename"]
    output_subdir = f"vest_{args.state.lower()}_{args.year}"

    cmd = [
        sys.executable,
        str(Path(__file__).parent / "download_election_data.py"),
        "--doi", doi,
        "--filename", filename,
        "--output-dir", args.output_dir,
        "--output-subdir", output_subdir,
    ]
    print(f"[run] {' '.join(cmd)}")
    return subprocess.call(cmd)


if __name__ == "__main__":
    sys.exit(main())
