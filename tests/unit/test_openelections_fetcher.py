"""
Unit tests for scripts/data/elections/download_openelections.py.

Doesn't actually clone — just verifies the script's argument parsing,
URL construction, and state-code validation. Real clone is exercised
manually or in a separate integration test.
"""

from __future__ import annotations

import sys
from pathlib import Path
from unittest import mock

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "elections"))

import download_openelections as oe  # noqa: E402


class TestRepoUrl:
    def test_lowercases_state_code(self):
        assert oe.repo_url("CA") == "https://github.com/openelections/openelections-data-ca.git"

    def test_handles_already_lowercase(self):
        assert oe.repo_url("ca") == "https://github.com/openelections/openelections-data-ca.git"


class TestKnownStates:
    def test_includes_all_50_state_postal_codes(self):
        # Spot-check the three smallest + three largest. The full set is asserted
        # to have 51 entries (50 states + DC).
        for code in ("VT", "DE", "WY", "CA", "TX", "NY"):
            assert code in oe.KNOWN_STATES, f"missing state: {code}"
        assert len(oe.KNOWN_STATES) == 51, f"expected 51 codes (50 states + DC), got {len(oe.KNOWN_STATES)}"

    def test_excludes_obvious_non_states(self):
        # OpenElections doesn't have territories
        for code in ("PR", "GU", "VI"):
            assert code not in oe.KNOWN_STATES


class TestCloneStateMockedSubprocess:
    def test_skips_existing_dir_without_force(self, tmp_path, capsys, monkeypatch):
        existing = tmp_path / f"{oe.REPO_PREFIX}vt"
        existing.mkdir()
        rc = oe.clone_state("VT", tmp_path, force=False)
        assert rc == 0
        out = capsys.readouterr().out
        assert "[skip]" in out

    def test_clones_when_dir_missing(self, tmp_path, monkeypatch):
        with mock.patch("subprocess.run") as run:
            run.return_value.returncode = 0
            rc = oe.clone_state("VT", tmp_path)
            assert rc == 0
            args, _ = run.call_args
            cmd = args[0]
            assert cmd[0] == "git"
            assert cmd[1] == "clone"
            assert "https://github.com/openelections/openelections-data-vt.git" in cmd

    def test_clone_failure_propagates_returncode(self, tmp_path, monkeypatch, capsys):
        with mock.patch("subprocess.run") as run:
            run.return_value.returncode = 128
            rc = oe.clone_state("VT", tmp_path)
            assert rc == 128
            assert "[FAIL]" in capsys.readouterr().out

    def test_unknown_state_prints_warning_but_attempts_clone(self, tmp_path, capsys, monkeypatch):
        with mock.patch("subprocess.run") as run:
            run.return_value.returncode = 0
            rc = oe.clone_state("XX", tmp_path)
            assert rc == 0
            captured = capsys.readouterr().out
            assert "WARNING" in captured
            assert "XX" in captured

    def test_force_reclones_existing_dir(self, tmp_path, monkeypatch):
        existing = tmp_path / f"{oe.REPO_PREFIX}vt"
        existing.mkdir()
        (existing / "stale.txt").write_text("old data")
        with mock.patch("subprocess.run") as run:
            run.return_value.returncode = 0
            rc = oe.clone_state("VT", tmp_path, force=True)
            assert rc == 0
            # The original directory should have been removed before clone
            run.assert_called_once()


class TestMainCli:
    def test_all_keyword_expands_to_known_states(self, monkeypatch, tmp_path):
        with mock.patch("subprocess.run") as run, mock.patch("shutil.which", return_value="/usr/bin/git"):
            run.return_value.returncode = 0
            rc = oe.main(["--states", "ALL", "--output-dir", str(tmp_path)])
            assert rc == 0
            # subprocess.run should have been invoked once per state
            assert run.call_count == len(oe.KNOWN_STATES)

    def test_explicit_states_only_those(self, monkeypatch, tmp_path):
        with mock.patch("subprocess.run") as run, mock.patch("shutil.which", return_value="/usr/bin/git"):
            run.return_value.returncode = 0
            rc = oe.main(["--states", "VT", "DE", "--output-dir", str(tmp_path)])
            assert rc == 0
            assert run.call_count == 2

    def test_missing_git_returns_2(self, capsys, tmp_path):
        with mock.patch("shutil.which", return_value=None):
            rc = oe.main(["--states", "VT", "--output-dir", str(tmp_path)])
            assert rc == 2
            assert "git not found" in capsys.readouterr().err
