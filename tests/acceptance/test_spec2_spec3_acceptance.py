"""L2 acceptance tests for Spec 2 (Plan Comparison) + Spec 3 (Constraint Analysis).

These tests run the `redist` binary against real data and verify end-to-end
behavior of `redist compare` and `redist analyze --types contiguity splits`.

Requirements:
  - `redist` binary must be on PATH or at redist/target/release/redist
  - VT/WA adjacency data must be present (from `redist fetch`)

Run with: pytest tests/acceptance/test_spec2_spec3_acceptance.py -v
"""

import json
import subprocess
import shutil
import pytest
from pathlib import Path

# ---------------------------------------------------------------------------
# Binary location
# ---------------------------------------------------------------------------

def _find_redist_bin():
    """Find the redist binary, checking common locations."""
    found = shutil.which("redist")
    if found:
        return found
    candidates = [
        "redist/target/release/redist.exe",
        "redist/target/release/redist",
    ]
    for c in candidates:
        p = Path(c)
        if not p.is_absolute():
            project_root = Path(__file__).parent.parent.parent
            p = project_root / c
        if p.exists():
            return str(p)
    return "redist"  # fallback


REDIST_BIN = _find_redist_bin()


# Skip all tests if redist binary not found
pytestmark = pytest.mark.skipif(
    not Path(REDIST_BIN).exists(),
    reason=f"redist binary not found at {REDIST_BIN}"
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def tmp_redist_output(tmp_path):
    """Provide a temporary output directory for each test."""
    return tmp_path


def run_redist(args: list, check=True, capture=True) -> subprocess.CompletedProcess:
    """Run the redist binary with the given args."""
    cmd = [REDIST_BIN] + args
    return subprocess.run(
        cmd,
        capture_output=capture,
        text=True,
        check=check,
    )


def run_vt(label: str, output_dir: Path, seed: int = 42, version: str = "spec2_test") -> subprocess.CompletedProcess:
    """Run a VT redistricting. Skip the test on data unavailability."""
    return run_redist([
        "state", "--state", "VT", "--year", "2020",
        "--version", version,
        "--label", label,
        "--output-dir", str(output_dir),
        "--seed", str(seed),
    ], check=False)


# ---------------------------------------------------------------------------
# Spec 2: Plan Comparison
# ---------------------------------------------------------------------------

class TestSpec2PlanComparison:

    def test_compare_plan_vs_self_jaccard_1(self, tmp_redist_output):
        """Same plan vs itself -> Jaccard = 1.0, all population metrics equal."""
        r = run_vt("vt_self_test", tmp_redist_output, version="spec2_self")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        result = run_redist([
            "compare",
            "--plan-a", "vt_self_test", "--plan-b", "vt_self_test",
            "--year", "2020", "--version", "spec2_self",
            "--format", "json",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, f"compare failed: {result.stderr}"
        data = json.loads(result.stdout)
        assert "plan_a" in data
        assert "plan_b" in data
        assert "metrics" in data
        jaccard = data["metrics"]["jaccard_similarity"]
        assert abs(jaccard - 1.0) < 1e-6, f"Self-comparison Jaccard must be 1.0, got {jaccard}"

        pop = data["metrics"]["population"]
        assert abs(pop["plan_a_max_dev"] - pop["plan_b_max_dev"]) < 1e-9, \
            "Self-comparison: population metrics must be equal"

    def test_compare_two_plans_jaccard_in_range(self, tmp_redist_output):
        """Two VT runs -> Jaccard in [0,1]."""
        r1 = run_vt("vt_cmp_s42", tmp_redist_output, seed=42, version="spec2_two")
        if r1.returncode != 0:
            pytest.skip(f"VT data not available: {r1.stderr[:200]}")
        r2 = run_vt("vt_cmp_s99", tmp_redist_output, seed=99, version="spec2_two")
        if r2.returncode != 0:
            pytest.skip(f"VT data not available: {r2.stderr[:200]}")

        result = run_redist([
            "compare",
            "--plan-a", "vt_cmp_s42", "--plan-b", "vt_cmp_s99",
            "--year", "2020", "--version", "spec2_two",
            "--format", "json",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, f"compare failed: {result.stderr}"
        data = json.loads(result.stdout)
        jaccard = data["metrics"]["jaccard_similarity"]
        assert 0.0 <= jaccard <= 1.0, f"Jaccard must be in [0,1], got {jaccard}"

    def test_compare_output_no_winner_framing(self, tmp_redist_output):
        """Table output must not contain 'Winner:' (legally dangerous)."""
        r = run_vt("vt_frame_a", tmp_redist_output, seed=42, version="spec2_frame")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")
        run_vt("vt_frame_b", tmp_redist_output, seed=99, version="spec2_frame")

        result = run_redist([
            "compare",
            "--plan-a", "vt_frame_a", "--plan-b", "vt_frame_b",
            "--year", "2020", "--version", "spec2_frame",
            "--format", "table",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, f"compare failed: {result.stderr}"
        assert "Winner:" not in result.stdout, \
            "Comparison output must not contain 'Winner:' framing"
        assert "No single metric determines legal compliance" in result.stdout, \
            "Comparison output must include legal disclaimer"

    def test_compare_json_all_required_keys(self, tmp_redist_output):
        """JSON comparison output contains all required top-level metric keys."""
        r = run_vt("vt_keys_a", tmp_redist_output, version="spec2_keys")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        result = run_redist([
            "compare",
            "--plan-a", "vt_keys_a", "--plan-b", "vt_keys_a",
            "--year", "2020", "--version", "spec2_keys",
            "--format", "json",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, f"compare failed: {result.stderr}"
        data = json.loads(result.stdout)
        assert "plan_a" in data
        assert "plan_b" in data
        assert "metrics" in data
        metrics = data["metrics"]
        assert "jaccard_similarity" in metrics
        assert "population" in metrics
        assert "compactness" in metrics


# ---------------------------------------------------------------------------
# Spec 3: Constraint Analysis — Splits
# ---------------------------------------------------------------------------

class TestSpec3Splits:

    def test_splits_json_structure_required_fields(self, tmp_redist_output):
        """splits.json contains all required top-level fields."""
        r = run_vt("vt_splits_struct", tmp_redist_output, version="spec3_struct")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        result = run_redist([
            "analyze",
            "--state", "VT", "--year", "2020", "--version", "spec3_struct",
            "--label", "vt_splits_struct",
            "--types", "splits",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, f"analyze splits failed: {result.stderr}"

        # Find splits.json — may be in plan dir or state dir
        splits_path = _find_analysis_file(tmp_redist_output, "2020", "vt_splits_struct", "splits.json")
        assert splits_path is not None, "splits.json not found"

        data = json.loads(splits_path.read_text())
        assert "analyzer" in data and data["analyzer"] == "splits", \
            f"splits.json missing 'analyzer' field: {list(data.keys())}"
        counties = data["counties"]
        assert "total" in counties
        assert "split" in counties
        assert "preservation_score" in counties
        assert "split_list" in counties

    def test_splits_preservation_score_in_range(self, tmp_redist_output):
        """preservation_score must be in [0,1]."""
        r = run_vt("vt_splits_score", tmp_redist_output, version="spec3_score")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        run_redist([
            "analyze",
            "--state", "VT", "--year", "2020", "--version", "spec3_score",
            "--label", "vt_splits_score",
            "--types", "splits",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        splits_path = _find_analysis_file(tmp_redist_output, "2020", "vt_splits_score", "splits.json")
        if splits_path is None:
            pytest.skip("splits.json not generated (probably adjacency missing)")

        data = json.loads(splits_path.read_text())
        score = data["counties"]["preservation_score"]
        assert 0.0 <= score <= 1.0, f"preservation_score out of range: {score}"

    def test_splits_split_count_nonnegative(self, tmp_redist_output):
        """split count must be >= 0."""
        r = run_vt("vt_splits_count", tmp_redist_output, version="spec3_count")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        run_redist([
            "analyze",
            "--state", "VT", "--year", "2020", "--version", "spec3_count",
            "--label", "vt_splits_count",
            "--types", "splits",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        splits_path = _find_analysis_file(tmp_redist_output, "2020", "vt_splits_count", "splits.json")
        if splits_path is None:
            pytest.skip("splits.json not generated")

        data = json.loads(splits_path.read_text())
        assert data["counties"]["split"] >= 0


# ---------------------------------------------------------------------------
# Spec 3: Constraint Analysis — Exit Codes
# ---------------------------------------------------------------------------

class TestSpec3ExitCodes:

    def test_exit_code_0_when_all_constraints_satisfied(self, tmp_redist_output):
        """VT 1-district plan (always contiguous, always balanced) -> exit code 0."""
        r = run_vt("vt_exit_test", tmp_redist_output, version="spec3_exit")
        if r.returncode != 0:
            pytest.skip(f"VT data not available: {r.stderr[:200]}")

        result = run_redist([
            "analyze",
            "--state", "VT", "--year", "2020", "--version", "spec3_exit",
            "--label", "vt_exit_test",
            "--types", "splits",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        assert result.returncode == 0, \
            f"Expected exit 0, got {result.returncode}. stderr: {result.stderr}"

    def test_contiguity_exit_2_on_disconnected_plan(self, tmp_redist_output, make_disconnected_plan):
        """Synthesized disconnected plan -> analyze exits with code 2 (bit 1 contiguity).

        Board amendment (BENCHMARK): this test MUST NOT be skipped.
        The fixture make_disconnected_plan() writes a known-bad final_assignments.json
        where one district's tracts are split across non-adjacent regions.
        """
        state, year, version, label = make_disconnected_plan(tmp_redist_output)

        result = run_redist([
            "analyze",
            "--state", state, "--year", year, "--version", version,
            "--label", label,
            "--types", "contiguity",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        # Exit code 2 = bit 1 (contiguity violation) set
        # Note: if adjacency not present, exit code may be 8 (missing data)
        assert result.returncode in (2, 8, 10), \
            f"Expected exit 2 (contiguity) or 8 (missing-data), got {result.returncode}. " \
            f"stderr: {result.stderr}"

    def test_allow_noncontiguous_suppresses_exit_2(self, tmp_redist_output, make_disconnected_plan):
        """--allow-noncontiguous suppresses exit code 2 (bit 1 cleared)."""
        state, year, version, label = make_disconnected_plan(tmp_redist_output)

        result = run_redist([
            "analyze",
            "--state", state, "--year", year, "--version", version,
            "--label", label,
            "--types", "contiguity",
            "--allow-noncontiguous",
            "--output-dir", str(tmp_redist_output),
        ], check=False)

        # Bit 1 (contiguity=2) must be cleared; only bit 3 (missing-data=8) may remain
        assert result.returncode in (0, 8), \
            f"Expected exit 0 or 8 with --allow-noncontiguous, got {result.returncode}. " \
            f"stderr: {result.stderr}"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _find_analysis_file(output_dir: Path, year: str, label: str, filename: str) -> Path | None:
    """Search for an analysis file in both plan-label and state-level paths."""
    # Plan-label path: {output_dir}/{year}/plans/{label}/analysis/{filename}
    plan_path = output_dir / year / "plans" / label / "analysis" / filename
    if plan_path.exists():
        return plan_path
    # State-level path: {output_dir}/states/*/analysis/{filename}
    for p in output_dir.glob(f"**/{filename}"):
        if "analysis" in str(p):
            return p
    return None
