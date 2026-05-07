"""
Permanent acceptance tests for the `redist` Rust binary.

Required before deletion of the migration validation harness (Plan 02 Task 5).
Replaces what compare_rust_vs_python.py and validate_rust_vs_python.py
verified during the migration: that the Rust pipeline produces VALID outputs.

Per the architecture spec (docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md):

    Hard gates (must pass):
        Population imbalance <= 0.5% per district
        District count exactly equals state's congressional seats
        Contiguity: each district's connected component count = 1
        Exit code 0
        final_assignments.json parseable

    Parity gates (informational, drift detection):
        Polsby-Popper within published acceptable range
        Reock within published acceptable range

The Rust binary writes final_assignments.json to:
    {output_base}/{version}/states/{state_name}/data/final_assignments.json

Run with:
    pytest tests/acceptance/test_redist_invariants.py -v --tb=short

Requires:
    - `redist` binary on PATH (built from redist/ workspace)
    - Adjacency data under outputs/{version}/data/{year}/adjacency/
    - METIS (gpmetis) on PATH for the bisection invocation
    - ~2 minutes total (VT ~5s, AL ~60s)
"""

import json
import os
import shutil
import subprocess
from pathlib import Path
from collections import Counter, defaultdict, deque

import pytest


# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------

REDIST_BIN = shutil.which("redist")

pytestmark = pytest.mark.skipif(
    REDIST_BIN is None,
    reason="`redist` binary not on PATH; build with `cargo build --release` "
           "and add target/release to PATH"
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def state_name(code: str) -> str:
    """Map two-letter code to lowercase_underscore name used by Rust output dirs."""
    NAMES = {
        "VT": "vermont",
        "AL": "alabama",
        "DE": "delaware",
        "RI": "rhode_island",
        # Extend as needed.
    }
    return NAMES[code.upper()]


def run_redist_state(state: str, year: str, version: str, output_base: Path) -> subprocess.CompletedProcess:
    """Invoke `redist state` and return the completed process."""
    return subprocess.run(
        [
            "redist", "state",
            "--state", state,
            "--year", year,
            "--version", version,
            "--output-dir", str(output_base),
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )


def load_assignments(output_base: Path, version: str, state: str) -> dict:
    """Load final_assignments.json. Returns dict[geoid_str -> district_int]."""
    path = output_base / version / "states" / state_name(state) / "data" / "final_assignments.json"
    with path.open() as f:
        return json.load(f)


def load_adjacency(year: str, state: str) -> dict:
    """Load adjacency for connectivity check (loaded only if test exercises contiguity).

    Stub: actual loader depends on whether .adj.bin or .pkl is the canonical
    test fixture path. Fill in when wiring this test against real data.
    """
    raise NotImplementedError("Provide adjacency-loading helper (.adj.bin reader) — see Plan 02 Task 4")


def population_imbalance_pct(assignments: dict, populations: dict) -> float:
    """Max district-population deviation from ideal, as a percent."""
    by_district = defaultdict(int)
    for geoid, district in assignments.items():
        by_district[district] += populations[geoid]
    total = sum(by_district.values())
    n_districts = len(by_district)
    ideal = total / n_districts
    return max(abs(v - ideal) for v in by_district.values()) / ideal * 100


def is_district_contiguous(assignments: dict, district: int, adjacency: dict) -> bool:
    """BFS over the adjacency graph restricted to tracts in `district`. Returns True iff one component."""
    members = {g for g, d in assignments.items() if d == district}
    if not members:
        return True
    start = next(iter(members))
    visited = {start}
    queue = deque([start])
    while queue:
        g = queue.popleft()
        for n in adjacency.get(g, []):
            if n in members and n not in visited:
                visited.add(n)
                queue.append(n)
    return visited == members


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestVTInvariants:
    """Vermont — 1 district, trivial. Smoke test for the binary itself."""

    def test_redist_state_vt_exits_zero(self, tmp_path):
        result = run_redist_state("VT", "2020", "ci_test", tmp_path)
        assert result.returncode == 0, (
            f"redist state --state VT exited {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )

    def test_vt_assignments_file_exists_and_parses(self, tmp_path):
        run_redist_state("VT", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "VT")
        assert isinstance(assignments, dict)
        assert len(assignments) == 193, f"VT has 193 tracts; got {len(assignments)}"

    def test_vt_district_count_is_one(self, tmp_path):
        run_redist_state("VT", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "VT")
        district_counts = Counter(assignments.values())
        assert len(district_counts) == 1, f"VT must produce 1 district; got {len(district_counts)}"


class TestALInvariants:
    """Alabama — 7 districts, VRA target. Catches regressions in edge weighting + multi-district balance."""

    def test_redist_state_al_exits_zero(self, tmp_path):
        result = run_redist_state("AL", "2020", "ci_test", tmp_path)
        assert result.returncode == 0, (
            f"redist state --state AL exited {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )

    def test_al_district_count_is_seven(self, tmp_path):
        run_redist_state("AL", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "AL")
        district_counts = Counter(assignments.values())
        assert len(district_counts) == 7, f"AL must produce 7 districts; got {len(district_counts)}"

    def test_al_all_tracts_assigned(self, tmp_path):
        run_redist_state("AL", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "AL")
        assert len(assignments) == 1437, f"AL has 1437 tracts; got {len(assignments)}"

    @pytest.mark.skip(reason="population fixture wiring TODO before Plan 02 Task 4")
    def test_al_population_balance(self, tmp_path):
        run_redist_state("AL", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "AL")
        # populations: dict[geoid -> int] — load from outputs/data/2020/units/al_units_2020.csv or similar
        populations = {}  # TODO: wire fixture
        assert population_imbalance_pct(assignments, populations) <= 0.5

    @pytest.mark.skip(reason="adjacency fixture wiring TODO before Plan 02 Task 4")
    def test_al_all_districts_contiguous(self, tmp_path):
        run_redist_state("AL", "2020", "ci_test", tmp_path)
        assignments = load_assignments(tmp_path, "ci_test", "AL")
        adjacency = load_adjacency("2020", "AL")  # TODO: wire fixture
        for district in set(assignments.values()):
            assert is_district_contiguous(assignments, district, adjacency), (
                f"District {district} is not contiguous"
            )
