"""
L2 acceptance tests for `redist state --partition-mode partisan-weighted`.

Two layers:

1. **Args + mutex guard** (always runs): exercises the CLI argument surface
   without needing data — validates that:
     - `--help` lists `partisan-weighted` as a partition-mode value
     - `--help` documents `--partisan-shares`, `--dem-threshold`, `--rep-threshold`
     - Invalid combinations (vra+shares, partisan-weighted-without-shares) fail
       cleanly with a recognisable error message

2. **End-to-end** (skipped unless data + adjacency are present): runs
   `redist state --state VT --partition-mode partisan-weighted` against
   a synthetic partisan-shares.tsv built from the producer script. Requires:
       - `redist` binary on PATH or under redist/target/release/
       - VT 2020 adjacency under outputs/V3/data/2020/adjacency/
       - The producer script (always present)

Run from repo root:
    pytest redist/tests/acceptance/test_partisan_weighted_acceptance.py -v
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Binary discovery
# ---------------------------------------------------------------------------

def find_redist_binary() -> str:
    """Return path to redist binary or None if not built."""
    if (b := shutil.which("redist")):
        return b
    for c in (
        Path("redist/target/release/redist.exe"),
        Path("redist/target/release/redist"),
        Path("redist/target/debug/redist.exe"),
        Path("redist/target/debug/redist"),
    ):
        if c.exists():
            return str(c)
    return None


REDIST = find_redist_binary()
HAVE_BINARY = REDIST is not None
VT_ADJ = Path("outputs/V3/data/2020/adjacency/vt_adjacency_2020.adj.bin")
HAVE_VT_DATA = VT_ADJ.exists()


# ---------------------------------------------------------------------------
# Layer 1: CLI surface validation (always runs)
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not HAVE_BINARY, reason="redist binary not built")
class TestCliSurface:
    def test_state_help_lists_partisan_weighted(self):
        result = subprocess.run(
            [REDIST, "state", "--help"], capture_output=True, text=True
        )
        assert result.returncode == 0
        assert "partisan-weighted" in result.stdout, (
            f"--help should list partisan-weighted as a partition-mode value:\n{result.stdout}"
        )

    def test_state_help_documents_partisan_shares_flag(self):
        result = subprocess.run(
            [REDIST, "state", "--help"], capture_output=True, text=True
        )
        assert "--partisan-shares" in result.stdout
        assert "--dem-threshold" in result.stdout
        assert "--rep-threshold" in result.stdout


# ---------------------------------------------------------------------------
# Layer 2: Mutex-guard error paths (requires binary + adjacency)
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not (HAVE_BINARY and HAVE_VT_DATA),
                    reason="redist binary or VT adjacency not available")
class TestMutexGuard:
    """Verify the runner rejects invalid partisan-mode combinations."""

    def test_metis_vra_with_partisan_shares_rejected(self, tmp_path):
        # Mutex violation: VRA + partisan = Callais p.36 disentanglement breach
        shares_file = tmp_path / "shares.tsv"
        shares_file.write_text("geoid\tdem_share\n50001000100\t0.5\n")

        result = subprocess.run([
            REDIST, "state", "--state", "VT", "--year", "2020", "--version", "test_mutex",
            "--partition-mode", "metis-vra",
            "--partisan-shares", str(shares_file),
            "--output-dir", str(tmp_path / "out"),
            "--print-only",  # don't actually run the bisection
        ], capture_output=True, text=True)
        assert result.returncode != 0, "expected non-zero exit on mutex violation"
        combined = result.stdout + result.stderr
        assert "partisan-shares" in combined, (
            f"error should mention partisan-shares: {combined!r}"
        )
        assert "metis-vra" in combined or "partisan-weighted" in combined, (
            f"error should explain the conflict: {combined!r}"
        )

    def test_partisan_weighted_without_shares_rejected(self, tmp_path):
        result = subprocess.run([
            REDIST, "state", "--state", "VT", "--year", "2020", "--version", "test_mutex",
            "--partition-mode", "partisan-weighted",
            "--output-dir", str(tmp_path / "out"),
            "--print-only",
        ], capture_output=True, text=True)
        assert result.returncode != 0
        combined = result.stdout + result.stderr
        assert "partisan-shares" in combined or "--partisan-shares" in combined, (
            f"error should explain the missing flag: {combined!r}"
        )


# ---------------------------------------------------------------------------
# Layer 3: End-to-end run on Vermont (1-district trivial smoke test)
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not (HAVE_BINARY and HAVE_VT_DATA),
                    reason="redist binary or VT adjacency not available")
class TestEndToEndVT:
    """Run partisan-weighted on VT (1 district, trivially balanced)."""

    @pytest.fixture
    def shares_file(self, tmp_path):
        # Generate a synthetic per-tract shares file via the producer.
        # 193 tracts in VT — we don't need real data; producer accepts any tract-level CSV.
        # Produce via direct mode with synthetic input.
        producer = Path("scripts/data/political/build_dem_shares.py")
        if not producer.exists():
            pytest.skip("producer script missing")

        # Build a fake tract-level results CSV: every tract gets 50/50 votes
        # (so every tract is swing — no boost — but file is well-formed).
        geoids_path = Path("outputs/V3/data/2020/adjacency/vt_adjacency_2020_geoids.json")
        if not geoids_path.exists():
            pytest.skip(f"VT geoid mapping not present at {geoids_path}")
        geoids = json.loads(geoids_path.read_text())

        input_csv = tmp_path / "vt_tract_results.csv"
        with input_csv.open("w", newline="") as f:
            f.write("geoid,dem_votes,rep_votes\n")
            for g in geoids.values():
                f.write(f"{g},500,500\n")

        out_tsv = tmp_path / "dem_shares.tsv"
        result = subprocess.run([
            sys.executable, str(producer),
            "--mode", "direct",
            "--state", "VT", "--year", "2020",
            "--input", str(input_csv),
            "--output", str(out_tsv),
        ], capture_output=True, text=True)
        assert result.returncode == 0, f"producer failed: {result.stderr}"
        return out_tsv

    def test_vt_partisan_weighted_run_succeeds(self, tmp_path, shares_file):
        result = subprocess.run([
            REDIST, "state", "--state", "VT", "--year", "2020",
            "--version", "test_partisan_weighted",
            "--partition-mode", "partisan-weighted",
            "--partisan-shares", str(shares_file),
            "--output-dir", str(tmp_path / "out"),
        ], capture_output=True, text=True, timeout=60)
        assert result.returncode == 0, (
            f"redist state failed:\nstdout: {result.stdout}\nstderr: {result.stderr}"
        )

        # Verify outputs exist and are valid
        final = tmp_path / "out" / "test_partisan_weighted" / "states" / "vermont" / "data" / "final_assignments.json"
        assert final.exists(), f"final_assignments.json missing at {final}"
        assignments = json.loads(final.read_text())
        assert len(assignments) == 193, f"VT must have 193 tracts, got {len(assignments)}"
        # All 193 tracts in 1 district
        district_counts = set(assignments.values())
        assert len(district_counts) == 1, f"VT must produce 1 district, got {len(district_counts)}"
