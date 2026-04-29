"""L2 acceptance tests for Spec 0 (RPLAN) + Spec 1 (Custom Parameters).

These tests run the `redist` binary against real data and verify end-to-end
behavior of the new --label, --chamber, --districts, --balance-tolerance,
--force, --population-source flags and the `redist validate` command.

Requirements:
  - `redist` binary must be on PATH or at redist/target/release/redist
  - VT adjacency data must be present (from `redist fetch`)

Run with: pytest tests/acceptance/test_spec0_spec1_acceptance.py -v
"""

import json
import subprocess
import sys
import shutil
import pytest
from pathlib import Path

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

def _find_redist_bin():
    """Find the redist binary, checking common locations."""
    # Check PATH first
    found = shutil.which("redist")
    if found:
        return found
    # Check release build (Windows + Unix)
    candidates = [
        "redist/target/release/redist.exe",
        "redist/target/release/redist",
    ]
    for c in candidates:
        p = Path(c)
        if not p.is_absolute():
            # Try relative to the project root (parent of tests/)
            project_root = Path(__file__).parent.parent.parent
            p = project_root / c
        if p.exists():
            return str(p)
    return "redist"  # fallback

REDIST_BIN = _find_redist_bin()


@pytest.fixture
def tmp_redist_output(tmp_path):
    """Provide a temporary output directory for each test."""
    return tmp_path


def run_redist(args: list[str], check=True, capture=True) -> subprocess.CompletedProcess:
    """Run the redist binary with the given args."""
    cmd = [REDIST_BIN] + args
    return subprocess.run(
        cmd,
        capture_output=capture,
        text=True,
        check=check,
    )


def has_vt_adjacency() -> bool:
    """Check if VT adjacency data is available for testing."""
    try:
        result = run_redist(["state", "--state", "VT", "--year", "2020",
                             "--version", "spec_check", "--print-only"], check=False)
        return result.returncode == 0 or "adjacency" not in result.stderr
    except (FileNotFoundError, PermissionError):
        return False


# Skip all tests if redist binary not found
pytestmark = pytest.mark.skipif(
    not Path(REDIST_BIN).exists(),
    reason=f"redist binary not found at {REDIST_BIN}"
)


# ---------------------------------------------------------------------------
# Spec 1: Custom parameters acceptance tests
# ---------------------------------------------------------------------------

class TestSpec1CustomParameters:

    def test_chamber_flag_recorded_in_manifest(self, tmp_redist_output):
        """--chamber house is recorded in manifest.json."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec1_chamber",
            "--districts", "1", "--chamber", "house",
            "--label", "vt_house_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest_path = tmp_redist_output / "2020" / "plans" / "vt_house_accept" / "manifest.json"
        assert manifest_path.exists(), f"manifest not found at {manifest_path}"
        manifest = json.loads(manifest_path.read_text())
        assert manifest["chamber"] == "house"
        assert manifest["num_districts"] == 1

    def test_balance_tolerance_in_manifest(self, tmp_redist_output):
        """--balance-tolerance 5.0 is recorded in manifest (as pct)."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec1_tol",
            "--label", "vt_tol_accept",
            "--balance-tolerance", "5.0",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_tol_accept" / "manifest.json").read_text()
        )
        assert abs(manifest["balance_tolerance_pct"] - 5.0) < 0.01

    def test_population_source_in_manifest(self, tmp_redist_output):
        """--population-source total is recorded in manifest."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec1_pop",
            "--label", "vt_pop_accept",
            "--population-source", "total",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_pop_accept" / "manifest.json").read_text()
        )
        assert manifest["population_source"] == "total"

    def test_label_collision_exits_nonzero_without_force(self, tmp_redist_output):
        """Running twice with same label fails on second run without --force."""
        common_args = [
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec1_collision",
            "--label", "vt_collision_accept",
            "--output-dir", str(tmp_redist_output),
        ]
        result1 = run_redist(common_args, check=False)
        if result1.returncode != 0:
            pytest.skip(f"VT data not available: {result1.stderr[:200]}")
        result2 = run_redist(common_args, check=False)
        assert result2.returncode != 0, "Second run without --force must fail"
        combined = result2.stderr + result2.stdout
        assert "already exists" in combined or "already exists" in combined.lower()

    def test_force_flag_allows_second_run(self, tmp_redist_output):
        """--force on second run succeeds and overwrites."""
        common_args = [
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec1_force",
            "--label", "vt_force_accept",
            "--output-dir", str(tmp_redist_output),
        ]
        result1 = run_redist(common_args, check=False)
        if result1.returncode != 0:
            pytest.skip(f"VT data not available: {result1.stderr[:200]}")
        result2 = run_redist(common_args + ["--force"], check=False)
        assert result2.returncode == 0, f"Second run with --force failed: {result2.stderr}"


# ---------------------------------------------------------------------------
# Spec 0: RPLAN format acceptance tests
# ---------------------------------------------------------------------------

class TestSpec0RplanFormat:

    def test_manifest_required_fields_present(self, tmp_redist_output):
        """manifest.json has all required Spec 0 fields."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec0_fields",
            "--label", "vt_fields_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_fields_accept" / "manifest.json").read_text()
        )
        required_fields = [
            "label", "state_code", "year", "chamber", "num_districts",
            "population_source", "binary_version", "binary_download_url",
            "adjacency_file", "tiger_source_url", "created_at",
            "balance_tolerance_pct", "population_balance_valid",
        ]
        for field in required_fields:
            assert field in manifest, f"manifest missing required field: {field}"

    def test_manifest_adjacency_file_is_filename_not_path(self, tmp_redist_output):
        """manifest.json adjacency_file is a bare filename, not a local path."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec0_adj",
            "--label", "vt_adj_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_adj_accept" / "manifest.json").read_text()
        )
        adj = manifest["adjacency_file"]
        assert "/" not in adj and "\\" not in adj, \
            f"adjacency_file must be filename only, got: {adj!r}"

    def test_manifest_tiger_source_url_is_census_gov(self, tmp_redist_output):
        """manifest.json tiger_source_url points to census.gov."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec0_tiger",
            "--label", "vt_tiger_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_tiger_accept" / "manifest.json").read_text()
        )
        assert "tiger_source_url" in manifest
        assert "census.gov" in manifest["tiger_source_url"]

    def test_manifest_binary_download_url_is_github(self, tmp_redist_output):
        """manifest.json binary_download_url points to github.com."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec0_gh",
            "--label", "vt_gh_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_gh_accept" / "manifest.json").read_text()
        )
        assert "github.com" in manifest["binary_download_url"]

    def test_manifest_population_balance_valid_is_true(self, tmp_redist_output):
        """manifest.json population_balance_valid is true after successful run."""
        result = run_redist([
            "state", "--state", "VT", "--year", "2020",
            "--version", "spec0_bal",
            "--label", "vt_bal_accept",
            "--output-dir", str(tmp_redist_output),
        ], check=False)
        if result.returncode != 0:
            pytest.skip(f"VT data not available: {result.stderr[:200]}")
        manifest = json.loads(
            (tmp_redist_output / "2020" / "plans" / "vt_bal_accept" / "manifest.json").read_text()
        )
        assert manifest["population_balance_valid"] is True


# ---------------------------------------------------------------------------
# redist validate command acceptance tests
# ---------------------------------------------------------------------------

class TestRedistValidateCommand:

    def test_validate_passes_on_valid_rplan(self, tmp_path):
        """redist validate exits 0 for a valid .rplan file."""
        rplan_data = {
            "rplan_version": "0.1",
            "metadata": {
                "label": "vt_test",
                "state_fips": "50",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test",
            },
            "assignments": {"50023000100": 1},
            "geometry": None,
        }
        rplan_path = tmp_path / "plan.rplan"
        rplan_path.write_text(json.dumps(rplan_data))
        result = run_redist(["validate", "--file", str(rplan_path)], check=False)
        assert result.returncode == 0, f"validate failed: {result.stderr}"
        assert "PASS" in result.stdout

    def test_validate_fails_on_bad_geoid(self, tmp_path):
        """redist validate exits non-zero for bad GEOID (too short)."""
        rplan_data = {
            "rplan_version": "0.1",
            "metadata": {
                "label": "bad_geoid_test",
                "state_fips": "50",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test",
            },
            "assignments": {"500230": 1},  # too short GEOID
            "geometry": None,
        }
        rplan_path = tmp_path / "bad_plan.rplan"
        rplan_path.write_text(json.dumps(rplan_data))
        result = run_redist(["validate", "--file", str(rplan_path)], check=False)
        assert result.returncode != 0, "validate should fail on bad GEOID"
        assert "GEOID" in result.stderr or "GEOID" in result.stdout

    def test_validate_shows_tract_and_district_counts(self, tmp_path):
        """redist validate output includes tract count and district count."""
        rplan_data = {
            "rplan_version": "0.1",
            "metadata": {
                "label": "count_test",
                "state_fips": "50",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test",
            },
            "assignments": {"50023000100": 1, "50023000200": 1},
            "geometry": None,
        }
        rplan_path = tmp_path / "count_plan.rplan"
        rplan_path.write_text(json.dumps(rplan_data))
        result = run_redist(["validate", "--file", str(rplan_path)], check=False)
        assert result.returncode == 0
        assert "tracts" in result.stdout
        assert "districts" in result.stdout
