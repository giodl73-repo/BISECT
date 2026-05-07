"""
Unit tests for scripts/data/elections/download_vest.py — VEST per-state
DOI dispatcher.

Doesn't actually download — just verifies the cache lookup, missing-entry
error path, and CLI dispatch.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from unittest import mock

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "elections"))

import download_vest as vest  # noqa: E402


def _fake_cache(tmp_path: Path, by_state_and_year: dict) -> Path:
    path = tmp_path / "vest_dois.json"
    path.write_text(json.dumps({
        "schema_version": 1,
        "description": "test",
        "by_state_and_year": by_state_and_year,
    }))
    return path


class TestLookup:
    def test_returns_entry_when_present(self):
        cache = {
            "by_state_and_year": {
                "WI": {
                    "2020": {"doi": "10.7910/DVN/AAAAAA", "filename": "wi_2020.zip"}
                }
            }
        }
        entry = vest.lookup(cache, "WI", "2020")
        assert entry is not None
        assert entry["doi"] == "10.7910/DVN/AAAAAA"

    def test_lookup_is_case_insensitive_for_state(self):
        cache = {"by_state_and_year": {"WI": {"2020": {"doi": "x", "filename": "y"}}}}
        assert vest.lookup(cache, "wi", "2020") is not None

    def test_returns_none_when_state_absent(self):
        cache = {"by_state_and_year": {"WI": {"2020": {"doi": "x", "filename": "y"}}}}
        assert vest.lookup(cache, "TX", "2020") is None

    def test_returns_none_when_year_absent(self):
        cache = {"by_state_and_year": {"WI": {"2020": {"doi": "x", "filename": "y"}}}}
        assert vest.lookup(cache, "WI", "2018") is None


class TestMainCli:
    def test_list_known_prints_table(self, tmp_path, capsys, monkeypatch):
        cache_path = _fake_cache(tmp_path, {
            "WI": {"2020": {"doi": "10.7910/DVN/X", "filename": "wi.zip"}},
            "CA": {"2020": {"doi": "10.7910/DVN/Y", "filename": "ca.zip"}},
            "_template": {"DUMMY": {}},  # should be skipped
        })
        monkeypatch.setattr(vest, "DOI_CACHE_PATH", cache_path)
        rc = vest.main(["--list-known", "--state", "X", "--year", "2020"])
        assert rc == 0
        out = capsys.readouterr().out
        assert "WI" in out
        assert "CA" in out
        assert "_template" not in out
        assert "DUMMY" not in out

    def test_missing_entry_returns_1_with_helpful_message(self, tmp_path, capsys, monkeypatch):
        cache_path = _fake_cache(tmp_path, {
            "WI": {"2020": {"doi": "x", "filename": "y"}}
        })
        monkeypatch.setattr(vest, "DOI_CACHE_PATH", cache_path)
        rc = vest.main(["--state", "TX", "--year", "2020"])
        assert rc == 1
        err = capsys.readouterr().err
        assert "TX" in err
        assert "2020" in err
        assert "vest_dois.json" in err

    def test_known_entry_dispatches_to_downloader(self, tmp_path, monkeypatch):
        cache_path = _fake_cache(tmp_path, {
            "WI": {"2020": {"doi": "10.7910/DVN/ABCDEF", "filename": "wi_2020.zip"}}
        })
        monkeypatch.setattr(vest, "DOI_CACHE_PATH", cache_path)
        with mock.patch("subprocess.call") as call_mock:
            call_mock.return_value = 0
            rc = vest.main(["--state", "WI", "--year", "2020", "--output-dir", str(tmp_path / "out")])
            assert rc == 0
            cmd = call_mock.call_args[0][0]
            assert "--doi" in cmd
            assert "10.7910/DVN/ABCDEF" in cmd
            assert "--filename" in cmd
            assert "wi_2020.zip" in cmd

    def test_dispatcher_passes_through_subprocess_returncode(self, tmp_path, monkeypatch):
        cache_path = _fake_cache(tmp_path, {
            "WI": {"2020": {"doi": "x", "filename": "y"}}
        })
        monkeypatch.setattr(vest, "DOI_CACHE_PATH", cache_path)
        with mock.patch("subprocess.call", return_value=42):
            rc = vest.main(["--state", "WI", "--year", "2020"])
            assert rc == 42


class TestRealDoiCache:
    """Smoke test: the shipped vest_dois.json parses and has the expected shape."""

    def test_shipped_cache_parses(self):
        cache = vest.load_doi_cache()
        assert cache.get("schema_version") == 1
        assert "by_state_and_year" in cache

    def test_shipped_cache_has_template_marker(self):
        # The empty-cache _template entry is documentation; should be present
        # so users know what shape to use.
        cache = vest.load_doi_cache()
        assert "_template" in cache.get("by_state_and_year", {})
