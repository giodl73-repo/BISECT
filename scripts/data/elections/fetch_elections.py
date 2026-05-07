"""
Election-data registry: list available sources, filter, and dispatch to the
right fetcher.

The registry lives in `scripts/data/elections/sources.json`. Each source records
its name, license, resolution, year coverage, schema, and (optionally) a
fetcher script. This tool is the single entry point users call when they want
election data — it's source-agnostic.

Usage:

    # List all known sources
    python scripts/data/elections/fetch_elections.py list

    # Filter sources by criteria
    python scripts/data/elections/fetch_elections.py list --resolution tract
    python scripts/data/elections/fetch_elections.py list --year 2020 --license CC0-1.0

    # Show a specific source's details
    python scripts/data/elections/fetch_elections.py show harvard-fekrazad-2020

    # Fetch from a specific source (dispatches to the source's fetcher)
    python scripts/data/elections/fetch_elections.py fetch --source harvard-fekrazad-2020

    # Fetch from the default (first source with a fetcher matching --year)
    python scripts/data/elections/fetch_elections.py fetch --year 2020

When you add a new source: edit sources.json. If its fetcher is null, this tool
will print the URL + manual-fetch instructions instead of dispatching.

Pipeline integration: `redist analyze --types political` consumes whatever
this tool produced (or any tract-level CSV with `geoid, dem_votes, rep_votes`).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

REGISTRY_PATH = Path(__file__).parent / "sources.json"


# ---------------------------------------------------------------------------
# Registry loading
# ---------------------------------------------------------------------------

def load_registry() -> dict[str, Any]:
    if not REGISTRY_PATH.exists():
        print(f"ERROR: registry not found at {REGISTRY_PATH}", file=sys.stderr)
        sys.exit(2)
    with REGISTRY_PATH.open(encoding="utf-8") as f:
        return json.load(f)


def get_source(registry: dict, source_id: str) -> dict | None:
    for s in registry.get("sources", []):
        if s.get("id") == source_id:
            return s
    return None


# ---------------------------------------------------------------------------
# Filtering
# ---------------------------------------------------------------------------

def filter_sources(
    sources: list[dict],
    resolution: str | None = None,
    year: int | None = None,
    license_: str | None = None,
    automated_only: bool = False,
) -> list[dict]:
    out = []
    for s in sources:
        if resolution and resolution.lower() not in s.get("resolution", "").lower():
            continue
        if year is not None:
            years = s.get("years", [])
            if year not in years:
                continue
        if license_ and license_.lower() not in s.get("license", "").lower():
            continue
        if automated_only and not s.get("fetcher"):
            continue
        out.append(s)
    return out


# ---------------------------------------------------------------------------
# Display
# ---------------------------------------------------------------------------

def print_table(sources: list[dict]) -> None:
    if not sources:
        print("(no sources match)")
        return
    print(f"{'ID':<28} {'RES':<10} {'YEARS':<22} {'AUTO':<5} {'LICENSE'}")
    print("-" * 90)
    for s in sources:
        sid = s.get("id", "")[:27]
        res = (s.get("resolution") or "")[:10]
        years = s.get("years", [])
        if isinstance(years, list) and len(years) > 4:
            years_s = f"{years[0]}..{years[-1]} ({len(years)})"
        else:
            years_s = ", ".join(str(y) for y in years) if isinstance(years, list) else str(years)
        years_s = years_s[:22]
        auto = "yes" if s.get("fetcher") else "no"
        lic = (s.get("license") or "")[:30]
        print(f"{sid:<28} {res:<10} {years_s:<22} {auto:<5} {lic}")


def print_detail(s: dict) -> None:
    print(f"id:           {s.get('id')}")
    print(f"name:         {s.get('name')}")
    if s.get("doi"):
        print(f"doi:          {s.get('doi')}")
    print(f"url:          {s.get('url', '(none)')}")
    print(f"license:      {s.get('license')}")
    print(f"resolution:   {s.get('resolution')}")
    print(f"level:        {s.get('level')}")
    years = s.get("years", [])
    if isinstance(years, list):
        if len(years) > 6:
            print(f"years:        {years[0]}..{years[-1]} ({len(years)} cycles)")
        else:
            print(f"years:        {', '.join(str(y) for y in years)}")
    print(f"states:       {s.get('states')}")
    print(f"format:       {s.get('format')}")
    if s.get("fetcher"):
        print(f"fetcher:      {s.get('fetcher')}")
        if s.get("fetcher_args"):
            print(f"fetcher_args: {' '.join(s.get('fetcher_args'))}")
    else:
        print("fetcher:      (none — manual fetch via URL above)")
    if s.get("schema"):
        sch = s.get("schema")
        if isinstance(sch, dict):
            print("schema:")
            for k, v in sch.items():
                print(f"              {k}: {v}")
        else:
            print(f"schema:       {sch}")
    if s.get("notes"):
        print(f"notes:        {s.get('notes')}")


# ---------------------------------------------------------------------------
# Fetch dispatch
# ---------------------------------------------------------------------------

def fetch(s: dict, extra_args: list[str]) -> int:
    fetcher = s.get("fetcher")
    if not fetcher:
        print(f"Source '{s['id']}' has no automated fetcher.")
        print(f"Manual fetch: {s.get('url', '(no URL)')}")
        if s.get("notes"):
            print(f"Notes: {s.get('notes')}")
        return 1
    fetcher_path = Path(fetcher)
    if not fetcher_path.exists():
        print(f"ERROR: fetcher script not found at {fetcher}", file=sys.stderr)
        return 2
    cmd = [sys.executable, str(fetcher_path)] + list(s.get("fetcher_args") or []) + extra_args
    print(f"Running: {' '.join(cmd)}")
    return subprocess.call(cmd)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="command", required=True)

    p_list = sub.add_parser("list", help="List available sources (table)")
    p_list.add_argument("--resolution")
    p_list.add_argument("--year", type=int)
    p_list.add_argument("--license", dest="license_")
    p_list.add_argument("--automated-only", action="store_true",
                        help="Only show sources with an auto-fetcher")

    p_show = sub.add_parser("show", help="Show details of one source")
    p_show.add_argument("source_id")

    p_fetch = sub.add_parser("fetch", help="Run the fetcher for a source")
    p_fetch.add_argument("--source", help="Source id (default: first auto-fetchable for --year)")
    p_fetch.add_argument("--year", type=int)
    p_fetch.add_argument("extra", nargs=argparse.REMAINDER,
                         help="Extra args passed through to the underlying fetcher")

    args = p.parse_args(argv)
    registry = load_registry()
    sources = registry.get("sources", [])

    if args.command == "list":
        filtered = filter_sources(
            sources,
            resolution=args.resolution,
            year=args.year,
            license_=args.license_,
            automated_only=args.automated_only,
        )
        print_table(filtered)
        return 0

    if args.command == "show":
        s = get_source(registry, args.source_id)
        if not s:
            print(f"ERROR: no source with id '{args.source_id}'", file=sys.stderr)
            print("Available ids:", file=sys.stderr)
            for src in sources:
                print(f"  {src.get('id')}", file=sys.stderr)
            return 2
        print_detail(s)
        return 0

    if args.command == "fetch":
        s = None
        if args.source:
            s = get_source(registry, args.source)
            if not s:
                print(f"ERROR: no source with id '{args.source}'", file=sys.stderr)
                return 2
        else:
            # Default: first auto-fetchable source matching --year (if given)
            candidates = filter_sources(sources, year=args.year, automated_only=True)
            if not candidates:
                print("ERROR: no auto-fetchable source matches the given filters.", file=sys.stderr)
                print("Run `list --automated-only` to see options.", file=sys.stderr)
                return 2
            s = candidates[0]
            print(f"(default source: {s['id']})")
        # Strip the leading "--" if argparse REMAINDER captured it
        extra = list(args.extra or [])
        if extra and extra[0] == "--":
            extra = extra[1:]
        return fetch(s, extra)

    return 1


if __name__ == "__main__":
    sys.exit(main())
