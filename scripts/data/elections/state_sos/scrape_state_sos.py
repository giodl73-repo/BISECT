"""
Generic state SOS election-data scraper.

Reads cycles.json (per-state per-cycle URL config) and dispatches to a
format-specific parser to produce a normalized CSV: one row per
geography unit, with `dem_votes` and `rep_votes` columns where known.

The honest part: this is a framework. The actual URLs in cycles.json
must be verified by a contributor with browser access before the
first run. Each `verified: false` cycle prints a warning if attempted.

Why this exists:
After Callais (2026-04-29), Gingles 2/3 requires controlling for
partisan affiliation when proving racial-bloc voting. Primary-election
data — especially within-party — is now load-bearing for §2 evidence.
OpenElections covers most states already (see download_openelections.py),
but for active §2 litigation states (LA, AL, GA, the post-Callais
flashpoints) direct SOS scraping gives:
  - Freshness (SOS publishes within hours of certification)
  - Cross-validation against OpenElections
  - Coverage where OE is thin

Recommended approach:
  1. Use OpenElections as primary (`download_openelections.py`)
  2. Use this scraper for cross-validation or for cycles OE hasn't covered
  3. Diff against OE; submit corrections upstream

Usage:

    # List configured (state, cycle) pairs
    python scrape_state_sos.py --list

    # Scrape one cycle
    python scrape_state_sos.py --state LA --cycle 2020-presidential-primary

    # Override output
    python scrape_state_sos.py --state GA --cycle 2020-presidential-primary \\
        --output-dir data/raw/state-sos
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

CYCLES_PATH = Path(__file__).parent / "cycles.json"


def load_cycles() -> dict[str, Any]:
    if not CYCLES_PATH.exists():
        print(f"ERROR: cycles cache not found at {CYCLES_PATH}", file=sys.stderr)
        sys.exit(2)
    with CYCLES_PATH.open(encoding="utf-8") as f:
        return json.load(f)


def list_cycles(cache: dict) -> None:
    states = cache.get("states", {})
    print(f"{'STATE':<6} {'CYCLE':<32} {'FORMAT':<14} {'RES':<10} {'VERIFIED'}")
    print("-" * 78)
    for state, sdata in sorted(states.items()):
        cycles = sdata.get("cycles", {})
        for cycle_id, cdata in sorted(cycles.items()):
            verified = "yes" if cdata.get("verified") else "no"
            fmt = cdata.get("format", "?")
            res = cdata.get("resolution", "?")
            print(f"{state:<6} {cycle_id:<32} {fmt:<14} {res:<10} {verified}")


def lookup_cycle(cache: dict, state: str, cycle_id: str) -> dict | None:
    state_data = cache.get("states", {}).get(state.upper())
    if not state_data:
        return None
    return state_data.get("cycles", {}).get(cycle_id)


# ---------------------------------------------------------------------------
# Format-specific parsers (stubs — fill in as cycles get verified)
# ---------------------------------------------------------------------------

def parse_excel(local_path: Path) -> list[dict]:
    """Parse an Excel SOS file into normalized rows. Per-state heuristics needed."""
    raise NotImplementedError(
        "excel parser is a stub; add per-state column-mapping logic when first cycle is verified"
    )


def parse_clarity_xml(local_path: Path) -> list[dict]:
    """Parse a Clarity Elections XML download (Georgia and others use this)."""
    raise NotImplementedError(
        "clarity-xml parser is a stub; the Clarity schema is documented at "
        "results.enr.clarityelections.com — wire up xml.etree to extract precinct rows"
    )


def parse_pdf(local_path: Path) -> list[dict]:
    raise NotImplementedError(
        "PDF parsing is fragile and per-format. Strong recommendation: use OpenElections "
        "for any state whose primary results are PDF-only."
    )


PARSERS = {
    "excel": parse_excel,
    "clarity-xml": parse_clarity_xml,
    "pdf-or-excel": parse_pdf,  # treat as pdf — escalate to user
    "pdf": parse_pdf,
}


# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------

def download(url: str, output_path: Path) -> int:
    """Download a single URL with progress. Returns exit code."""
    try:
        import requests
        from tqdm import tqdm
    except ImportError as e:
        print(f"ERROR: requires `requests` and `tqdm` (pip install): {e}", file=sys.stderr)
        return 2

    if url.startswith("TODO"):
        print(f"ERROR: cycle URL is a TODO placeholder. Verify at the SOS site first.", file=sys.stderr)
        return 1

    print(f"[run] GET {url}")
    response = requests.get(url, stream=True)
    response.raise_for_status()
    total_size = int(response.headers.get("content-length", 0))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "wb") as f:
        with tqdm(total=total_size, unit="B", unit_scale=True, desc=output_path.name) as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    pbar.update(len(chunk))
    print(f"[OK]  {output_path}")
    return 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--list", action="store_true", help="List all configured (state, cycle) pairs")
    p.add_argument("--state", help="Two-letter state code")
    p.add_argument("--cycle", help="Cycle id (e.g., 2020-presidential-primary)")
    p.add_argument("--output-dir", default="data/raw/state-sos",
                   help="Where to write the downloaded files")
    p.add_argument("--allow-unverified", action="store_true",
                   help="Proceed even if the cycle is marked verified=false (default: refuse)")
    args = p.parse_args(argv)

    cache = load_cycles()

    if args.list:
        list_cycles(cache)
        return 0

    if not (args.state and args.cycle):
        print("ERROR: --state and --cycle are required (or pass --list)", file=sys.stderr)
        return 2

    entry = lookup_cycle(cache, args.state, args.cycle)
    if entry is None:
        print(
            f"No cycle '{args.cycle}' for state '{args.state.upper()}' in cycles.json.\n"
            f"Run with --list to see available entries.",
            file=sys.stderr,
        )
        return 1

    if not entry.get("verified") and not args.allow_unverified:
        print(
            f"Cycle '{args.cycle}' for {args.state.upper()} is marked verified=false. "
            f"The URL has not been confirmed live and the format has not been parse-tested.\n"
            f"  url:    {entry.get('url')}\n"
            f"  format: {entry.get('format')}\n"
            f"  notes:  {entry.get('warning') or entry.get('notes', '(none)')}\n"
            f"To proceed anyway: re-run with --allow-unverified.\n"
            f"To mark verified: edit cycles.json for this cycle and set verified=true after "
            f"manual confirmation.",
            file=sys.stderr,
        )
        return 1

    output_dir = Path(args.output_dir) / args.state.lower() / args.cycle
    filename = entry.get("url", "").split("/")[-1] or f"{args.cycle}.bin"
    output_path = output_dir / filename

    rc = download(entry["url"], output_path)
    if rc != 0:
        return rc

    fmt = entry.get("format", "?")
    parser_fn = PARSERS.get(fmt)
    if parser_fn is None:
        print(f"WARNING: no parser registered for format '{fmt}'. Downloaded raw to {output_path}.")
        print("Add a parse_<format> function to scrape_state_sos.py to enable normalization.")
        return 0

    try:
        rows = parser_fn(output_path)
    except NotImplementedError as e:
        print(f"WARNING: parser is a stub: {e}\nDownloaded raw to {output_path}.")
        return 0

    out_csv = output_dir / "normalized.csv"
    if rows:
        import csv
        with out_csv.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
            writer.writeheader()
            writer.writerows(rows)
        print(f"[OK]  Normalized -> {out_csv} ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
