"""
Post-process all Rust redistricting outputs: analytics → maps → aggregate → dashboard.

Runs in parallel across states, then sequentially for national steps.

Usage:
    python scripts/pipeline/run_rust_postprocess.py
    python scripts/pipeline/run_rust_postprocess.py --versions RustV3 RustV4
    python scripts/pipeline/run_rust_postprocess.py --years 2020 --workers 8
    python scripts/pipeline/run_rust_postprocess.py --skip-maps   # analytics only
    python scripts/pipeline/run_rust_postprocess.py --skip-dash   # no dashboard

Steps per version/year:
  1. redist analyze --types all   (per state, parallel)
  2. redist map --types all       (per state, parallel)
  3. redist aggregate             (national rollup)
  4. redist map --scope national  (national maps)
  5. deploy_docs.py               (dashboard HTML)
"""

import argparse
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


# ── Config ────────────────────────────────────────────────────────────────────

DEFAULT_VERSIONS = [
    ("RustV3", ["2020", "2010", "2000"]),
    ("RustV4", ["2020"]),
]
DEFAULT_WORKERS = 8

DASHBOARD_MAP = {
    ("RustV3", "2020"): "docs/dashboard_rust_2020.html",
    ("RustV3", "2010"): "docs/dashboard_rust_2010.html",
    ("RustV3", "2000"): "docs/dashboard_rust_2000.html",
    ("RustV4", "2020"): "docs/dashboard_rust_vra.html",
}


# ── Helpers ───────────────────────────────────────────────────────────────────

def find_binary() -> str:
    for p in ["redist/target/release/redist.exe",
              "redist/target/release/redist"]:
        if Path(p).exists():
            return str(Path(p).resolve())
    sys.exit("ERROR: redist binary not found. Build: cargo build --release --manifest-path redist/Cargo.toml")


def discover_states(version: str, year: str) -> list[str]:
    """Find all states with final_assignments.json for this version/year."""
    states_dir = Path("outputs") / version / year / "states"
    if not states_dir.exists():
        return []
    return sorted(
        d.name for d in states_dir.iterdir()
        if d.is_dir() and (d / "data" / "final_assignments.json").exists()
    )


def state_name_to_code(name: str) -> str:
    CODES = {
        "alabama": "AL", "alaska": "AK", "arizona": "AZ", "arkansas": "AR",
        "california": "CA", "colorado": "CO", "connecticut": "CT", "delaware": "DE",
        "florida": "FL", "georgia": "GA", "hawaii": "HI", "idaho": "ID",
        "illinois": "IL", "indiana": "IN", "iowa": "IA", "kansas": "KS",
        "kentucky": "KY", "louisiana": "LA", "maine": "ME", "maryland": "MD",
        "massachusetts": "MA", "michigan": "MI", "minnesota": "MN",
        "mississippi": "MS", "missouri": "MO", "montana": "MT", "nebraska": "NE",
        "nevada": "NV", "new_hampshire": "NH", "new_jersey": "NJ",
        "new_mexico": "NM", "new_york": "NY", "north_carolina": "NC",
        "north_dakota": "ND", "ohio": "OH", "oklahoma": "OK", "oregon": "OR",
        "pennsylvania": "PA", "rhode_island": "RI", "south_carolina": "SC",
        "south_dakota": "SD", "tennessee": "TN", "texas": "TX", "utah": "UT",
        "vermont": "VT", "virginia": "VA", "washington": "WA",
        "west_virginia": "WV", "wisconsin": "WI", "wyoming": "WY",
        "district_of_columbia": "DC",
    }
    return CODES.get(name, name.upper()[:2])


def run_cmd(cmd: list[str], label: str) -> tuple[bool, str]:
    """Run a command, return (success, stderr)."""
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        return False, r.stderr.strip()
    return True, ""


def analyze_state(binary: str, state_code: str, year: str, version: str) -> tuple[str, bool, str]:
    ok, err = run_cmd(
        [binary, "analyze", "--state", state_code, "--year", year,
         "--version", version, "--types", "all", "--force"],
        f"analyze {state_code}"
    )
    return state_code, ok, err


def map_state(binary: str, state_code: str, year: str, version: str) -> tuple[str, bool, str]:
    ok, err = run_cmd(
        [binary, "map", "--state", state_code, "--year", year,
         "--version", version, "--types", "districts", "political",
         "demographic", "compactness", "--force"],
        f"map {state_code}"
    )
    return state_code, ok, err


# ── Main ──────────────────────────────────────────────────────────────────────

def process_version_year(binary: str, version: str, year: str,
                          workers: int, skip_maps: bool, skip_dash: bool,
                          force: bool) -> dict:
    states = discover_states(version, year)
    if not states:
        print(f"  [{version}/{year}] No states found — skipping")
        return {}

    print(f"\n{'='*60}")
    print(f"  {version} / {year} — {len(states)} states")
    print(f"{'='*60}")
    results = {"version": version, "year": year, "states": len(states),
               "analyze_ok": 0, "analyze_fail": 0,
               "map_ok": 0, "map_fail": 0}

    state_codes = [state_name_to_code(s) for s in states]

    # ── Step 1: Analyze (parallel) ────────────────────────────────────────
    print(f"\n  [1/4] analyze --types all ({workers} workers)...")
    t0 = time.perf_counter()
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {pool.submit(analyze_state, binary, code, year, version): code
                   for code in state_codes}
        for fut in as_completed(futures):
            code, ok, err = fut.result()
            if ok:
                results["analyze_ok"] += 1
            else:
                results["analyze_fail"] += 1
                print(f"    [FAIL] analyze {code}: {err[:80]}")
    t1 = time.perf_counter()
    print(f"  analyze done: {results['analyze_ok']}/{len(states)} OK  ({t1-t0:.1f}s)")

    # ── Step 2: Maps (parallel) ───────────────────────────────────────────
    if not skip_maps:
        print(f"\n  [2/4] map --types districts political demographic compactness ({workers} workers)...")
        t0 = time.perf_counter()
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = {pool.submit(map_state, binary, code, year, version): code
                       for code in state_codes}
            for fut in as_completed(futures):
                code, ok, err = fut.result()
                if ok:
                    results["map_ok"] += 1
                else:
                    results["map_fail"] += 1
                    print(f"    [FAIL] map {code}: {err[:80]}")
        t1 = time.perf_counter()
        print(f"  maps done: {results['map_ok']}/{len(states)} OK  ({t1-t0:.1f}s)")
    else:
        print("\n  [2/4] maps — skipped")

    # ── Step 3: Aggregate (sequential) ───────────────────────────────────
    print(f"\n  [3/4] aggregate --types all --csv...")
    ok, err = run_cmd(
        [binary, "aggregate", "--year", year, "--version", version,
         "--types", "all", "--csv", "--force"],
        "aggregate"
    )
    if ok:
        print(f"  aggregate done")
    else:
        print(f"  [FAIL] aggregate: {err[:120]}")

    # ── Step 4: National maps ─────────────────────────────────────────────
    if not skip_maps:
        print(f"\n  [4a/4] map --scope national --types all...")
        ok, err = run_cmd(
            [binary, "map", "--scope", "national", "--version", version,
             "--year", year, "--types", "districts", "political",
             "demographic", "compactness", "--force"],
            "national map"
        )
        if ok:
            print(f"  national maps done")
        else:
            print(f"  [WARN] national maps: {err[:120]}")
    else:
        print("\n  [4a/4] national maps — skipped")

    # ── Step 4b: Dashboard ────────────────────────────────────────────────
    if not skip_dash:
        out_html = DASHBOARD_MAP.get((version, year))
        if out_html:
            print(f"\n  [4b/4] dashboard → {out_html}...")
            py = sys.executable
            ok, err = run_cmd(
                [py, "scripts/web/deploy_docs.py",
                 "--version", version, "--year", year, "--out", out_html],
                "dashboard"
            )
            if ok:
                print(f"  dashboard done → {out_html}")
            else:
                print(f"  [WARN] dashboard: {err[:120]}")
        else:
            print(f"\n  [4b/4] dashboard — no output path configured for ({version},{year})")
    else:
        print("\n  [4b/4] dashboard — skipped")

    return results


def main():
    parser = argparse.ArgumentParser(description="Post-process all Rust redistricting outputs")
    parser.add_argument("--versions", nargs="+", default=None,
                        help="Versions to process (default: RustV3 RustV4)")
    parser.add_argument("--years", nargs="+", default=None,
                        choices=["2020", "2010", "2000"],
                        help="Years to process (default: all configured years per version)")
    parser.add_argument("--workers", type=int, default=DEFAULT_WORKERS,
                        help=f"Parallel workers for analyze/map (default: {DEFAULT_WORKERS})")
    parser.add_argument("--skip-maps", action="store_true",
                        help="Skip map rendering (analytics + aggregate only)")
    parser.add_argument("--skip-dash", action="store_true",
                        help="Skip dashboard generation")
    parser.add_argument("--force", action="store_true", default=True,
                        help="Force re-run even if outputs exist (default: True)")
    args = parser.parse_args()

    binary = find_binary()

    # Resolve version/year combinations to process
    version_years = []
    for version, years in DEFAULT_VERSIONS:
        if args.versions and version not in args.versions:
            continue
        for year in years:
            if args.years and year not in args.years:
                continue
            version_years.append((version, year))

    if not version_years:
        print("No version/year combinations to process. Check --versions and --years.")
        sys.exit(1)

    print(f"redist post-process pipeline")
    print(f"Binary: {binary}")
    print(f"Versions/years: {version_years}")
    print(f"Workers: {args.workers}")
    print(f"Skip maps: {args.skip_maps}  Skip dashboard: {args.skip_dash}")

    t_total = time.perf_counter()
    all_results = []
    for version, year in version_years:
        r = process_version_year(binary, version, year, args.workers,
                                  args.skip_maps, args.skip_dash, args.force)
        if r:
            all_results.append(r)

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY")
    print(f"{'='*60}")
    for r in all_results:
        print(f"  {r['version']}/{r['year']}: {r['states']} states | "
              f"analyze {r['analyze_ok']}/{r['states']} | "
              f"map {r.get('map_ok', '-')}/{r['states']}")
    print(f"\nTotal time: {time.perf_counter() - t_total:.1f}s")


if __name__ == "__main__":
    main()
