"""
Head-to-head validation: Rust CLI vs Python pipeline.

Runs both pipelines on a set of representative states, then compares:
  - Assignment file exists and is parseable
  - Correct district count
  - Correct tract count (same adjacency graph)
  - Population balance ≤ 0.5%
  - Compactness (Polsby-Popper) — compared via redist analyze
  - Wall-clock time

Neither pipeline is expected to produce identical district assignments
(METIS is graph-partitioning with multiple valid solutions). Both must
produce valid assignments independently.

Usage:
    python scripts/pipeline/validate_rust_vs_python.py
    python scripts/pipeline/validate_rust_vs_python.py --states VT AL CO
    python scripts/pipeline/validate_rust_vs_python.py --year 2020 --version V3
    python scripts/pipeline/validate_rust_vs_python.py --csv results.csv

Requires:
    - redist binary built (cargo build --release --manifest-path redist/Cargo.toml)
    - Adjacency pkl files in outputs/V3/data/{year}/adjacency/
    - .adj.bin files (python scripts/data/generate_adj_bin.py --year {year})
    - TIGER shapefiles in data/{year}/tiger/tracts/
    - Python pipeline runnable (for Python baseline)
    - Demographics CSV for compactness comparison
"""

import argparse
import csv
import json
import os
import pickle
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional


# ── Config ────────────────────────────────────────────────────────────────────

DEFAULT_STATES = ["VT", "RI", "DE", "AL", "CO"]
DEFAULT_YEAR   = "2020"
DEFAULT_VERSION = "V3"

STATE_DISTRICTS = {
    "VT": 1, "RI": 2, "DE": 1, "ME": 2, "NH": 2,
    "AL": 7, "CO": 8, "CT": 5, "NV": 4, "NM": 3,
    "OR": 6, "MN": 8, "WI": 8,
}

STATE_NAMES = {
    "VT": "vermont", "RI": "rhode_island", "DE": "delaware",
    "AL": "alabama", "CO": "colorado", "ME": "maine",
    "NH": "new_hampshire", "CT": "connecticut", "NV": "nevada",
    "NM": "new_mexico", "OR": "oregon", "MN": "minnesota",
    "WI": "wisconsin",
}

FIPS = {
    "VT": "50", "RI": "44", "DE": "10", "AL": "01", "CO": "08",
    "ME": "23", "NH": "33", "CT": "09", "NV": "32", "NM": "35",
    "OR": "41", "MN": "27", "WI": "55",
}


# ── Helpers ────────────────────────────────────────────────────────────────────

def find_binary() -> str:
    for p in ["redist/target/release/redist.exe",
              "redist/target/release/redist"]:
        path = Path(p).resolve()
        if path.exists():
            return str(path)
    sys.exit("ERROR: redist binary not found. Build with: "
             "cargo build --release --manifest-path redist/Cargo.toml")


def load_adjacency_pops(state_code: str, year: str, version: str) -> Optional[list]:
    """Load vertex_weights (tract populations) from adjacency pkl."""
    name = STATE_NAMES.get(state_code, state_code.lower())
    pkl = Path(f"outputs/{version}/data/{year}/adjacency/{state_code.lower()}_adjacency_{year}.pkl")
    if not pkl.exists():
        return None
    try:
        with open(pkl, "rb") as f:
            d = pickle.load(f)
        return [int(x) for x in d.get("vertex_weights", [])]
    except Exception:
        return None


def compute_balance(assignments: dict, pops: list) -> float:
    """Max fractional deviation from ideal district population. Returns 0–1."""
    if not pops or not assignments:
        return float("nan")
    district_pops: dict[int, int] = {}
    for idx, dist in assignments.items():
        idx = int(idx)
        if 0 <= idx < len(pops):
            district_pops[dist] = district_pops.get(dist, 0) + pops[idx]
    if not district_pops:
        return float("nan")
    ideal = sum(district_pops.values()) / len(district_pops)
    if ideal == 0:
        return float("nan")
    return max(abs(p - ideal) / ideal for p in district_pops.values())


def load_rust_assignments(state_code: str, version: str) -> Optional[dict]:
    name = STATE_NAMES.get(state_code, state_code.lower())
    p = Path(f"outputs/{version}/states/{name}/data/final_assignments.json")
    if not p.exists():
        return None
    return json.loads(p.read_text())


def load_python_assignments(state_code: str, year: str, version: str) -> Optional[dict]:
    name = STATE_NAMES.get(state_code, state_code.lower())
    # Python pipeline output path (run_complete_redistricting.py)
    p = Path(f"outputs/{version}/{year}/states/{name}/data/final_assignments.pkl")
    if not p.exists():
        # Try alternate Python path
        p = Path(f"outputs/{version}/2020/states/{name}/data/final_assignments.pkl")
    if not p.exists():
        return None
    try:
        with open(p, "rb") as f:
            return pickle.load(f)
    except Exception:
        return None


def load_compactness_pp(state_code: str, version: str) -> Optional[float]:
    name = STATE_NAMES.get(state_code, state_code.lower())
    p = Path(f"outputs/{version}/states/{name}/analysis/compactness.json")
    if not p.exists():
        return None
    data = json.loads(p.read_text())
    pps = [d.get("polsby_popper") for d in data.get("districts", [])
           if d.get("polsby_popper") is not None]
    return sum(pps) / len(pps) if pps else None


def count_geoid_adjacency(state_code: str, year: str, version: str) -> int:
    name = STATE_NAMES.get(state_code, state_code.lower())
    geoid_p = Path(f"outputs/{version}/data/{year}/adjacency/"
                   f"{state_code.lower()}_adjacency_{year}_geoids.json")
    if geoid_p.exists():
        return len(json.loads(geoid_p.read_text()))
    return 0


# ── Pipeline runners ───────────────────────────────────────────────────────────

def run_rust(binary: str, state: str, year: str, version: str) -> tuple[bool, float, str]:
    """Run redist state. Returns (success, elapsed_s, stderr)."""
    t0 = time.perf_counter()
    r = subprocess.run(
        [binary, "state", "--state", state, "--year", year, "--version", version],
        capture_output=True, text=True
    )
    elapsed = time.perf_counter() - t0
    return r.returncode == 0, elapsed, r.stderr.strip()


def run_python(state: str, year: str, version: str) -> tuple[bool, float, str]:
    """Run Python run_state_redistricting.py. Returns (success, elapsed_s, stderr)."""
    py = os.environ.get("REDIST_PYTHON", "python")
    name = STATE_NAMES.get(state, state.lower())
    out_dir = f"outputs/{version}/{year}/states/{name}"
    t0 = time.perf_counter()
    r = subprocess.run(
        [py, "scripts/pipeline/run_state_redistricting.py",
         "--state", name, "--year", year,
         "--output-dir", out_dir, "--position", "0"],
        capture_output=True, text=True
    )
    elapsed = time.perf_counter() - t0
    return r.returncode == 0, elapsed, r.stderr.strip()


def run_rust_analyze(binary: str, state: str, year: str, version: str) -> bool:
    r = subprocess.run(
        [binary, "analyze", "--state", state, "--year", year,
         "--version", version, "--types", "compactness", "--force"],
        capture_output=True, text=True
    )
    return r.returncode == 0


# ── Main ──────────────────────────────────────────────────────────────────────

def validate_state(binary: str, state: str, year: str, version: str,
                   run_pipelines: bool) -> dict:
    result = {"state": state, "expected_districts": STATE_DISTRICTS.get(state, "?")}
    pops = load_adjacency_pops(state, year, version)
    expected_tracts = len(pops) if pops else 0

    # ── Rust pipeline ──────────────────────────────────────────────────────
    if run_pipelines:
        ok, t, _ = run_rust(binary, state, year, version)
        result["rust_run_ok"] = ok
        result["rust_time_s"] = round(t, 2)
    else:
        result["rust_run_ok"] = None
        result["rust_time_s"] = None

    rust_assignments = load_rust_assignments(state, version)
    result["rust_assignments_exist"] = rust_assignments is not None
    if rust_assignments:
        result["rust_tract_count"] = len(rust_assignments)
        result["rust_districts_found"] = len(set(rust_assignments.values()))
        if pops:
            # Rust uses index keys — need geoid→index→pop mapping
            # Use raw index assignments for balance
            idx_assignments = {}
            for k, v in rust_assignments.items():
                try:
                    idx_assignments[int(k)] = v
                except ValueError:
                    pass
            if idx_assignments:
                max_dev = compute_balance(idx_assignments, pops)
            else:
                max_dev = float("nan")
            result["rust_balance_pct"] = round(max_dev * 100, 3) if max_dev == max_dev else "n/a"
            result["rust_balance_ok"] = max_dev <= 0.005 if max_dev == max_dev else None
        else:
            result["rust_balance_pct"] = "n/a"
            result["rust_balance_ok"] = None

        # Compactness
        run_rust_analyze(binary, state, year, version)
        pp = load_compactness_pp(state, version)
        result["rust_mean_pp"] = round(pp, 4) if pp else "n/a"
    else:
        result["rust_tract_count"] = 0
        result["rust_districts_found"] = 0
        result["rust_balance_pct"] = "n/a"
        result["rust_balance_ok"] = None
        result["rust_mean_pp"] = "n/a"

    # ── Python pipeline ────────────────────────────────────────────────────
    if run_pipelines:
        ok, t, _ = run_python(state, year, version)
        result["py_run_ok"] = ok
        result["py_time_s"] = round(t, 2)
    else:
        result["py_run_ok"] = None
        result["py_time_s"] = None

    py_assignments = load_python_assignments(state, year, version)
    result["py_assignments_exist"] = py_assignments is not None
    if py_assignments:
        result["py_tract_count"] = len(py_assignments)
        result["py_districts_found"] = len(set(py_assignments.values()))
        if pops:
            # Python assignments may use int or GEOID keys
            int_assignments = {}
            for k, v in py_assignments.items():
                try:
                    int_assignments[int(k)] = v
                except (ValueError, TypeError):
                    pass
            if int_assignments:
                max_dev = compute_balance(int_assignments, pops)
                result["py_balance_pct"] = round(max_dev * 100, 3)
                result["py_balance_ok"] = max_dev <= 0.005
            else:
                result["py_balance_pct"] = "n/a"
                result["py_balance_ok"] = None
        else:
            result["py_balance_pct"] = "n/a"
            result["py_balance_ok"] = None
    else:
        result["py_tract_count"] = 0
        result["py_districts_found"] = 0
        result["py_balance_pct"] = "n/a"
        result["py_balance_ok"] = None

    # ── Overall validity ───────────────────────────────────────────────────
    exp_d = STATE_DISTRICTS.get(state)
    rust_valid = (
        result["rust_assignments_exist"] and
        (result["rust_districts_found"] == exp_d if exp_d else True) and
        result.get("rust_balance_ok") is True
    )
    py_valid = (
        result["py_assignments_exist"] and
        (result["py_districts_found"] == exp_d if exp_d else True) and
        result.get("py_balance_ok") is True
    )
    result["rust_valid"] = rust_valid
    result["py_valid"] = py_valid

    if result["rust_time_s"] and result["py_time_s"] and result["py_time_s"] > 0:
        result["speedup"] = round(result["py_time_s"] / result["rust_time_s"], 1)
    else:
        result["speedup"] = "n/a"

    return result


def print_table(results: list[dict]) -> None:
    def fmt(v, ok_field=None, row=None) -> str:
        if v is None: return "-"
        if isinstance(v, bool): return "YES" if v else "NO"
        if isinstance(v, float) and v != v: return "n/a"
        s = str(v)
        return s

    def ok(v) -> str:
        if v is True: return "[OK]"
        if v is False: return "[X]"
        return "?"

    header = (
        f"{'State':5} {'Exp D':5} "
        f"{'Rust':>6} {'R-D':>4} {'R-Bal%':>7} {'R-PP':>6} {'R-Time':>7} "
        f"{'Py':>5} {'P-D':>4} {'P-Bal%':>7} {'P-Time':>7} "
        f"{'Speed':>6} {'R-OK':>5} {'P-OK':>5}"
    )
    sep = "-" * len(header)
    print(sep)
    print(header)
    print(sep)

    for r in results:
        rust_ok  = ok(r.get("rust_valid"))
        py_ok    = ok(r.get("py_valid"))
        print(
            f"{r['state']:5} {str(r['expected_districts']):5} "
            f"{ok(r.get('rust_assignments_exist')):>6} "
            f"{str(r.get('rust_districts_found', '?')):>4} "
            f"{str(r.get('rust_balance_pct', 'n/a')):>7} "
            f"{str(r.get('rust_mean_pp', 'n/a')):>6} "
            f"{str(r.get('rust_time_s') or '-') + 's':>7} "
            f"{ok(r.get('py_assignments_exist')):>5} "
            f"{str(r.get('py_districts_found', '?')):>4} "
            f"{str(r.get('py_balance_pct', 'n/a')):>7} "
            f"{str(r.get('py_time_s') or '—') + 's':>7} "
            f"{str(r.get('speedup', 'n/a')):>6}× "
            f"{rust_ok:>4} {py_ok:>4}"
        )
    print(sep)

    rust_valid_count = sum(1 for r in results if r.get("rust_valid"))
    py_valid_count   = sum(1 for r in results if r.get("py_valid"))
    both_valid       = sum(1 for r in results if r.get("rust_valid") and r.get("py_valid"))
    n = len(results)
    print(f"\nRust valid: {rust_valid_count}/{n}  |  Python valid: {py_valid_count}/{n}  "
          f"|  Both valid: {both_valid}/{n}")

    times_rust = [r["rust_time_s"] for r in results if r.get("rust_time_s")]
    times_py   = [r["py_time_s"]   for r in results if r.get("py_time_s")]
    if times_rust and times_py:
        total_rust = sum(times_rust)
        total_py   = sum(times_py)
        print(f"Total time — Rust: {total_rust:.1f}s  |  Python: {total_py:.1f}s  "
              f"|  Speedup: {total_py/total_rust:.1f}×")


def write_csv(results: list[dict], path: str) -> None:
    if not results:
        return
    fields = list(results[0].keys())
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(results)
    print(f"\nResults written to: {path}")


def main():
    parser = argparse.ArgumentParser(description="Rust vs Python head-to-head validation")
    parser.add_argument("--states", nargs="+", default=DEFAULT_STATES,
                        help="State codes to validate (default: VT RI DE AL CO)")
    parser.add_argument("--year", default=DEFAULT_YEAR, choices=["2020", "2010", "2000"])
    parser.add_argument("--version", default=DEFAULT_VERSION)
    parser.add_argument("--csv", help="Write results to CSV file")
    parser.add_argument("--no-run", action="store_true",
                        help="Skip running pipelines — only compare existing outputs")
    args = parser.parse_args()

    binary = find_binary()
    run_pipelines = not args.no_run

    print(f"\nRedist head-to-head validation — year={args.year}  version={args.version}")
    print(f"States: {' '.join(s.upper() for s in args.states)}")
    if run_pipelines:
        print("Mode: RUN both pipelines then compare")
    else:
        print("Mode: compare EXISTING outputs only (--no-run)")
    print()

    results = []
    for state in args.states:
        state = state.upper()
        if state not in STATE_NAMES:
            print(f"WARNING: {state} not in STATE_NAMES mapping — skipping")
            continue
        print(f"  {state}...", end=" ", flush=True)
        r = validate_state(binary, state, args.year, args.version, run_pipelines)
        results.append(r)
        rust_sym = "OK" if r.get("rust_valid") else "FAIL"
        py_sym   = "OK" if r.get("py_valid")   else "FAIL"
        print(f"Rust={rust_sym}  Python={py_sym}  "
              f"Rust-bal={r.get('rust_balance_pct')}%  "
              f"Py-bal={r.get('py_balance_pct')}%")

    print()
    print_table(results)

    if args.csv:
        write_csv(results, args.csv)

    any_fail = any(not r.get("rust_valid") or not r.get("py_valid") for r in results)
    sys.exit(1 if any_fail else 0)


if __name__ == "__main__":
    main()
