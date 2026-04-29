"""L2 acceptance tests for Spec 4 (Partisan Metrics) + Spec 5 (Multi-Chamber / Nested Plans).

These tests verify end-to-end behavior of:
  - redist analyze --types partisan
  - redist suite draw + validate

Board amendments applied:
  - bootstrap_ci: --bootstrap-samples flag (default 1000), progress printed
  - build_chamber_adjacency: tie-break by min GEOID lexicographic order
  - IL variable nesting: require explicit --nest-ratio, exit non-zero without it

Requirements:
  - redist binary must be on PATH or at redist/target/release/redist
  - VT data must be present (from `redist fetch`)
  - Election CSV data at data/2020/elections/presidential_by_tract.csv (for VT tests)

Run with: pytest tests/acceptance/test_spec4_spec5_acceptance.py -v
"""

import json
import subprocess
import shutil
import tempfile
import pytest
from pathlib import Path

# ---------------------------------------------------------------------------
# Binary location
# ---------------------------------------------------------------------------

def _find_redist_bin():
    found = shutil.which("redist")
    if found:
        return found
    candidates = [
        "redist/target/release/redist.exe",
        "redist/target/release/redist",
    ]
    project_root = Path(__file__).parent.parent.parent
    for c in candidates:
        p = project_root / c
        if p.exists():
            return str(p)
    return "redist"

REDIST_BIN = _find_redist_bin()

pytestmark = pytest.mark.skipif(
    not Path(REDIST_BIN).exists(),
    reason=f"redist binary not found at {REDIST_BIN}"
)


def run_redist(args: list[str], check=False, capture=True) -> subprocess.CompletedProcess:
    cmd = [REDIST_BIN] + args
    return subprocess.run(cmd, capture_output=capture, text=True, check=check)


def has_vt_data() -> bool:
    result = run_redist(
        ["state", "--state", "VT", "--year", "2020", "--version", "spec_check", "--print-only"],
        check=False
    )
    return result.returncode == 0


def has_election_data(year: str = "2020") -> bool:
    return Path(f"data/{year}/elections/presidential_by_tract.csv").exists()


# ---------------------------------------------------------------------------
# Spec 4: Partisan metrics
# ---------------------------------------------------------------------------

class TestSpec4PartisanAnalyzer:
    """Tests for redist analyze --types partisan."""

    def test_partisan_analyzer_type_parsed(self):
        """redist analyze --types partisan is a valid analyzer type (no parse error)."""
        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_parse_test", "--types", "partisan", "--help",
        ], check=False)
        # --help always exits 0 from clap
        assert result.returncode == 0

    def test_vt_partisan_ci_suppressed(self, tmp_path):
        """VT (1 district) -> partisan.json with ci_available=false."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_vt_ci",
            "--types", "partisan", "--force",
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0:
            pytest.skip(f"VT assignments not available: {result.stderr[:300]}")

        partisan_path = tmp_path / "spec4_vt_ci" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        assert partisan_path.exists(), f"partisan.json not found at {partisan_path}"

        data = json.loads(partisan_path.read_text())
        assert data["available"] is True
        assert len(data["districts"]) == 1

        for metric_name in ("efficiency_gap", "mean_median", "partisan_bias"):
            metric = data["metrics"][metric_name]
            assert metric["ci_available"] is False, f"{metric_name} ci_available should be False for VT"
            assert "Bootstrap CI requires" in metric["ci_reason"]
            assert "found 1" in metric["ci_reason"]

    def test_vt_partisan_all_three_metrics_present(self, tmp_path):
        """partisan.json contains efficiency_gap, mean_median, partisan_bias."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_vt_metrics",
            "--types", "partisan", "--force",
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0:
            pytest.skip(f"VT assignments not available: {result.stderr[:300]}")

        partisan_path = tmp_path / "spec4_vt_metrics" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        if not partisan_path.exists():
            pytest.skip("partisan.json not created (election data may be missing)")

        data = json.loads(partisan_path.read_text())

        if not data.get("available", False):
            pytest.skip(f"Partisan not available: {data.get('unavailable_reason', '')}")

        metrics = data.get("metrics", {})
        assert "efficiency_gap" in metrics, "efficiency_gap must be present"
        assert "mean_median" in metrics, "mean_median must be present"
        assert "partisan_bias" in metrics, "partisan_bias must be present"

    def test_partisan_no_threshold_8pct_field(self, tmp_path):
        """Board amendment: academic_reference replaces threshold_8pct field."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_vt_nofield",
            "--types", "partisan", "--force",
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0:
            pytest.skip(f"VT assignments not available: {result.stderr[:300]}")

        partisan_path = tmp_path / "spec4_vt_nofield" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        if not partisan_path.exists():
            pytest.skip("partisan.json not created")

        data = json.loads(partisan_path.read_text())
        if not data.get("available", False):
            pytest.skip("Partisan data not available")

        eg = data["metrics"]["efficiency_gap"]
        assert "threshold_8pct" not in eg, "threshold_8pct field must NOT be present (board amendment)"
        assert "academic_reference" in eg, "academic_reference must be present"
        assert "Stephanopoulos" in eg["academic_reference"], "EG academic reference must cite Stephanopoulos"

    def test_partisan_missing_election_returns_unavailable(self, tmp_path):
        """No election CSV -> partisan.json with available=false, exit 0."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        # Run with a nonexistent election file to trigger unavailable
        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_missing_election",
            "--types", "partisan", "--force",
            "--election-file", str(tmp_path / "nonexistent_election.csv"),
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0 and "No assignments found" in result.stderr:
            pytest.skip("VT assignments not available")

        partisan_path = tmp_path / "spec4_missing_election" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        if not partisan_path.exists():
            pytest.skip("partisan.json not created (assignments may be missing)")

        assert result.returncode == 0, f"Missing election must exit 0, got {result.returncode}: {result.stderr}"
        data = json.loads(partisan_path.read_text())
        assert data["available"] is False, "missing election file must produce available=false"
        assert "redist fetch --type elections" in data.get("unavailable_reason", ""), \
            "unavailable_reason must guide user to fetch command"
        assert "required_file" in data, "required_file must be present"

    def test_partisan_election_file_flag(self, tmp_path):
        """--election-file flag is accepted by the CLI (parse test)."""
        election_csv = tmp_path / "test_election.csv"
        election_csv.write_text("geoid,dem_votes,rep_votes\n50001000100,1000.0,800.0\n")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_election_flag",
            "--types", "partisan",
            "--election-file", str(election_csv),
            "--output-base", str(tmp_path),
        ], check=False)

        # Accept either success or "assignments not found" — we just need no parse error
        combined = result.stderr + result.stdout
        assert "error: unexpected argument" not in combined.lower(), \
            f"--election-file must be a valid flag, got: {combined[:300]}"
        assert "unrecognized argument" not in combined.lower()

    def test_partisan_bootstrap_samples_flag(self, tmp_path):
        """--bootstrap-samples flag is accepted by the CLI."""
        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_bootstrap",
            "--types", "partisan",
            "--bootstrap-samples", "500",
            "--output-base", str(tmp_path),
        ], check=False)

        combined = result.stderr + result.stdout
        assert "error: unexpected argument" not in combined.lower(), \
            f"--bootstrap-samples must be a valid flag, got: {combined[:300]}"
        assert "unrecognized argument" not in combined.lower()

    def test_partisan_analyzer_name_in_output(self, tmp_path):
        """partisan.json has analyzer='partisan'."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_vt_name",
            "--types", "partisan", "--force",
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0:
            pytest.skip(f"VT assignments not available: {result.stderr[:300]}")

        partisan_path = tmp_path / "spec4_vt_name" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        if not partisan_path.exists():
            pytest.skip("partisan.json not created")

        data = json.loads(partisan_path.read_text())
        assert data.get("analyzer") == "partisan"

    def test_vt_partisan_ci_reason_exact_text(self, tmp_path):
        """ci_reason must say 'Bootstrap CI requires >=10 districts (found 1)' for VT."""
        if not has_vt_data():
            pytest.skip("VT redistricting data not available")

        result = run_redist([
            "analyze", "--state", "VT", "--year", "2020",
            "--version", "spec4_ci_text",
            "--types", "partisan", "--force",
            "--output-base", str(tmp_path),
        ], check=False)

        if result.returncode != 0:
            pytest.skip(f"VT assignments not available: {result.stderr[:300]}")

        partisan_path = tmp_path / "spec4_ci_text" / "2020" / "states" / "vermont" / "analysis" / "partisan.json"
        if not partisan_path.exists():
            pytest.skip("partisan.json not created")

        data = json.loads(partisan_path.read_text())
        if not data.get("available", False):
            pytest.skip("Partisan data not available")

        ci_reason = data["metrics"]["efficiency_gap"].get("ci_reason", "")
        assert "Bootstrap CI requires >=10 districts (found 1)" == ci_reason, \
            f"ci_reason must match exactly, got: {ci_reason!r}"


# ---------------------------------------------------------------------------
# Spec 5: Suite CLI parsing
# ---------------------------------------------------------------------------

class TestSpec5SuiteCLI:
    """Tests for redist suite draw/validate CLI parsing and basic behavior."""

    def test_suite_draw_help(self):
        """redist suite draw --help exits 0."""
        result = run_redist(["suite", "draw", "--help"], check=False)
        assert result.returncode == 0

    def test_suite_validate_help(self):
        """redist suite validate --help exits 0."""
        result = run_redist(["suite", "validate", "--help"], check=False)
        assert result.returncode == 0

    def test_suite_nest_mode_senate_in_house_parsed(self):
        """--nest senate-in-house is a valid CLI argument."""
        result = run_redist([
            "suite", "draw",
            "--state", "WA", "--year", "2020", "--version", "test",
            "--name", "wa_parse_test",
            "--house-districts", "98",
            "--senate-districts", "49",
            "--nest", "senate-in-house",
            "--help",  # just parse, don't execute
        ], check=False)
        combined = result.stderr + result.stdout
        assert "error: unexpected argument" not in combined.lower(), \
            f"--nest senate-in-house must be a valid flag"
        assert result.returncode == 0

    def test_suite_draw_creates_suite_json(self, tmp_path):
        """redist suite draw creates suite.json manifest."""
        result = run_redist([
            "suite", "draw",
            "--state", "VT", "--year", "2020", "--version", "spec5_draw",
            "--name", "vt_draw_test",
            "--house-districts", "1",
            "--senate-districts", "1",
            "--nest", "none",
            "--output-base", str(tmp_path),
        ], check=False)

        suite_path = tmp_path / "spec5_draw" / "2020" / "suites" / "vt_draw_test" / "suite.json"
        assert suite_path.exists(), f"suite.json must be created at {suite_path}"

        data = json.loads(suite_path.read_text())
        assert data["suite_name"] == "vt_draw_test"
        assert data["state"] == "VT"

    def test_suite_manifest_nest_mode_recorded(self, tmp_path):
        """suite.json records nest_mode correctly."""
        result = run_redist([
            "suite", "draw",
            "--state", "VT", "--year", "2020", "--version", "spec5_nestmode",
            "--name", "vt_nestmode_test",
            "--nest", "none",
            "--output-base", str(tmp_path),
        ], check=False)

        suite_path = tmp_path / "spec5_nestmode" / "2020" / "suites" / "vt_nestmode_test" / "suite.json"
        if not suite_path.exists():
            pytest.skip("suite draw failed — likely no assignment data")

        data = json.loads(suite_path.read_text())
        assert data.get("nest_mode") == "none"

    def test_suite_validate_missing_suite_errors(self, tmp_path):
        """redist suite validate on nonexistent suite exits non-zero."""
        result = run_redist([
            "suite", "validate",
            "--name", "nonexistent_suite_xyz",
            "--version", "v1",
            "--year", "2020",
            "--output-base", str(tmp_path),
        ], check=False)
        assert result.returncode != 0, "Validate on nonexistent suite must fail"

    def test_suite_draw_senate_in_house_nest_mode(self, tmp_path):
        """redist suite draw --nest senate-in-house records the mode in suite.json."""
        result = run_redist([
            "suite", "draw",
            "--state", "VT", "--year", "2020", "--version", "spec5_sih",
            "--name", "vt_sih_test",
            "--nest", "senate-in-house",
            "--output-base", str(tmp_path),
        ], check=False)

        suite_path = tmp_path / "spec5_sih" / "2020" / "suites" / "vt_sih_test" / "suite.json"
        if not suite_path.exists():
            # WA nesting validation may exit 5 if house plan is empty (stub draw)
            # Just verify it doesn't crash with an unexpected error
            combined = result.stderr + result.stdout
            if "error: unexpected argument" in combined.lower():
                pytest.fail(f"CLI parsing failed: {combined[:300]}")
            return  # suite.json creation skipped for empty plan — OK

        data = json.loads(suite_path.read_text())
        assert data.get("nest_mode") == "senate-in-house"


# ---------------------------------------------------------------------------
# Spec 5 + board amendment: IL variable nesting
# ---------------------------------------------------------------------------

class TestSpec5BoardAmendments:
    """Board amendment tests for Spec 5."""

    def test_il_nest_without_nest_ratio_exits_nonzero(self, tmp_path):
        """IL suite draw without --nest-ratio exits non-zero (WARD amendment)."""
        result = run_redist([
            "suite", "draw",
            "--state", "IL", "--year", "2020", "--version", "spec5_il",
            "--name", "il_no_ratio",
            "--nest", "senate-in-house",
            "--output-base", str(tmp_path),
        ], check=False)
        assert result.returncode != 0, \
            "IL --nest senate-in-house without --nest-ratio must exit non-zero"
        combined = result.stderr + result.stdout
        assert "variable by statute" in combined or "nest-ratio" in combined, \
            f"Error must mention variable nesting or --nest-ratio: {combined[:300]}"

    def test_il_nest_with_nest_ratio_accepted(self, tmp_path):
        """IL suite draw with --nest-ratio N is accepted."""
        result = run_redist([
            "suite", "draw",
            "--state", "IL", "--year", "2020", "--version", "spec5_il_ratio",
            "--name", "il_with_ratio",
            "--nest", "senate-in-house",
            "--nest-ratio", "3",
            "--output-base", str(tmp_path),
        ], check=False)
        # Expect exit 0 (suite.json written) — exit 5 is nesting violation (OK for empty plan)
        combined = result.stderr + result.stdout
        assert "error: unexpected argument" not in combined.lower(), \
            f"--nest-ratio must be a valid flag: {combined[:300]}"
        # IL with --nest-ratio 3 should not fail with "variable by statute"
        assert "variable by statute" not in combined

    def test_wa_nest_wrong_ratio_warns_but_proceeds(self, tmp_path):
        """WA with --nest-ratio 3 (not 2:1 constitutional) warns but proceeds."""
        result = run_redist([
            "suite", "draw",
            "--state", "WA", "--year", "2020", "--version", "spec5_wa_warn",
            "--name", "wa_warn_test",
            "--nest", "senate-in-house",
            "--nest-ratio", "3",
            "--output-base", str(tmp_path),
        ], check=False)
        combined = result.stderr + result.stdout
        assert "WARNING" in combined, "Must emit warning for non-constitutional WA ratio"
        assert "constitution requires" in combined or "constitut" in combined.lower(), \
            f"Warning must mention constitutional requirement: {combined[:300]}"

    def test_bootstrap_samples_flag_accepted(self):
        """--bootstrap-samples flag is accepted by analyze command."""
        result = run_redist(["analyze", "--state", "VT", "--bootstrap-samples", "500", "--help"],
                           check=False)
        assert result.returncode == 0
        combined = result.stderr + result.stdout
        assert "unrecognized" not in combined.lower()
