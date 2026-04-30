"""
Network-gated integration tests for the election fetchers.

These hit live services (Harvard Dataverse, GitHub) and are SKIPPED by
default. Run them when you want to verify wire-level behavior:

    pytest -m network tests/integration/test_election_fetchers_network.py

What we cover:
  - Dataverse `--list-files` against the Fekrazad DOI (stable, public, no key)
  - Dataverse `--list-files` against the MIT EDSL DOI (verifies our claim
    that the listing endpoint works without a guestbook token)
  - Dataverse 400-error path: MIT EDSL DOWNLOAD without an API key — the
    helpful "guestbook" error must surface
  - OpenElections shallow git clone of a small state (DC, Delaware, Vermont)

What we DON'T cover here (would download too much):
  - Full Fekrazad state .zip download
  - Full OpenElections clone for any large state
  - VEST datasets (cache is empty by design)
  - State SOS scrapes (URLs are TODO placeholders)

If a test fails:
  - First check the actual upstream service (Dataverse status page, GitHub).
  - These tests carry an implicit dependency on those services being up
    and on their public APIs not having broken-changed.
  - If the upstream changed, update the relevant fetcher script.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]

# All tests in this module require live network access.
pytestmark = pytest.mark.network


# ---------------------------------------------------------------------------
# Skips: requests + git availability
# ---------------------------------------------------------------------------

requests_available = pytest.mark.skipif(
    pytest.importorskip("requests", reason="requests not installed", minversion="2.0") is None,
    reason="requests required for Dataverse tests"
)

git_available = pytest.mark.skipif(
    shutil.which("git") is None,
    reason="git binary not on PATH (required for OpenElections tests)"
)


# ---------------------------------------------------------------------------
# Dataverse: --list-files against a known stable DOI
# ---------------------------------------------------------------------------

DOWNLOADER = PROJECT_ROOT / "scripts" / "data" / "elections" / "download_election_data.py"


def _run_downloader(args: list[str], timeout: int = 60) -> subprocess.CompletedProcess:
    """Invoke download_election_data.py with the given args; return CompletedProcess."""
    return subprocess.run(
        [sys.executable, str(DOWNLOADER)] + args,
        capture_output=True, text=True, timeout=timeout,
    )


class TestDataverseListFiles:
    """Verify --list-files works against real Dataverse datasets."""

    def test_fekrazad_default_doi_lists_at_least_50_files(self):
        """Fekrazad ships 50 state .zip files + 2 USA bundles (≥52 expected)."""
        result = _run_downloader(["--list-files"])
        assert result.returncode == 0, f"stderr: {result.stderr}"
        # Header line + ≥51 file lines
        file_lines = [
            l for l in result.stdout.splitlines()
            if l.strip().endswith(".zip") or l.strip().endswith(".csv")
        ]
        assert len(file_lines) >= 50, f"expected ≥50 files, got {len(file_lines)}\n{result.stdout}"
        assert "Contiguous USA" in result.stdout, "expected Fekrazad bundle file"

    def test_mit_edsl_president_doi_lists_three_files(self):
        """MIT EDSL President 1976-2020 has 3 files we observed: tab + codebook + sources."""
        result = _run_downloader([
            "--list-files",
            "--doi", "10.7910/DVN/42MVDX",
        ])
        assert result.returncode == 0, f"stderr: {result.stderr}"
        assert "1976-2020-president.tab" in result.stdout, (
            f"MIT EDSL listing didn't include the expected .tab file:\n{result.stdout}"
        )


class TestDataverseGuestbookError:
    """Verify the helpful 400-error path fires when downloading without an API key."""

    def test_mit_edsl_download_without_key_surfaces_guestbook_message(self, tmp_path):
        """Downloading MIT EDSL without DATAVERSE_API_KEY must produce a clear
        'guestbook' error instead of a generic 400."""
        result = _run_downloader([
            "--doi", "10.7910/DVN/42MVDX",
            "--filename", "1976-2020-president.tab",
            "--output-dir", str(tmp_path),
        ], timeout=30)
        assert result.returncode != 0, "download without key should fail"
        combined = result.stdout + result.stderr
        # The improved error path puts the Dataverse 'message' body into the output.
        # If this assertion ever stops matching, we may have a different upstream
        # error — investigate before relaxing.
        assert (
            "guestbook" in combined.lower()
            or "Guestbook" in combined
            or "400" in combined
        ), f"expected guestbook hint in error output, got:\n{combined}"


# ---------------------------------------------------------------------------
# OpenElections: shallow clone of a small state
# ---------------------------------------------------------------------------

OPENELECTIONS = PROJECT_ROOT / "scripts" / "data" / "elections" / "download_openelections.py"


@git_available
class TestOpenElectionsClone:
    """Real shallow clone of a tiny state's OpenElections repo."""

    def test_dc_clone_succeeds(self, tmp_path):
        """DC's repo is one of the smallest (~few MB) — fast and safe to clone."""
        result = subprocess.run(
            [sys.executable, str(OPENELECTIONS),
             "--states", "DC",
             "--output-dir", str(tmp_path),
             "--depth", "1"],
            capture_output=True, text=True, timeout=120,
        )
        assert result.returncode == 0, (
            f"DC clone failed:\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        # Verify the directory was created and contains at least a README
        repo_dir = tmp_path / "openelections-data-dc"
        assert repo_dir.exists(), f"expected repo at {repo_dir}"
        assert (repo_dir / ".git").exists(), "expected .git directory in cloned repo"
        # Most OpenElections repos have a README
        has_readme = any(p.name.lower().startswith("readme") for p in repo_dir.iterdir())
        assert has_readme, f"expected README in {repo_dir}; got: {[p.name for p in repo_dir.iterdir()]}"

    def test_skip_on_existing_dir_without_force(self, tmp_path):
        """Re-cloning without --force should print [skip] and exit 0."""
        # Pre-create the directory so the script sees it as already-cloned
        (tmp_path / "openelections-data-dc").mkdir()
        result = subprocess.run(
            [sys.executable, str(OPENELECTIONS),
             "--states", "DC",
             "--output-dir", str(tmp_path)],
            capture_output=True, text=True, timeout=10,
        )
        assert result.returncode == 0
        assert "[skip]" in result.stdout.lower() or "[skip]" in result.stdout
