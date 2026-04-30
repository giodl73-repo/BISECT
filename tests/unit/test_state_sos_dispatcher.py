"""
Unit tests for scripts/data/elections/state_sos/scrape_state_sos.py.

The actual download + parse paths are not exercised (they require live
SOS URLs that are marked verified=false in the shipped config). What IS
tested: cycle lookup, the verified-false guard, --list output, and the
parser-stub registration.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from unittest import mock

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "elections" / "state_sos"))

import scrape_state_sos as sos  # noqa: E402


def _fake_cycles(tmp_path: Path, states: dict) -> Path:
    path = tmp_path / "cycles.json"
    path.write_text(json.dumps({
        "schema_version": 1,
        "description": "test",
        "states": states,
    }))
    return path


# ---------------------------------------------------------------------------
# Lookup
# ---------------------------------------------------------------------------

class TestLookupCycle:
    def test_returns_entry_when_present(self):
        cache = {
            "states": {
                "LA": {
                    "cycles": {
                        "2020-presidential-primary": {
                            "url": "https://example.com",
                            "format": "excel",
                            "verified": True,
                        }
                    }
                }
            }
        }
        entry = sos.lookup_cycle(cache, "LA", "2020-presidential-primary")
        assert entry is not None
        assert entry["url"] == "https://example.com"

    def test_state_lookup_is_case_insensitive(self):
        cache = {"states": {"LA": {"cycles": {"x": {"url": "u"}}}}}
        assert sos.lookup_cycle(cache, "la", "x") is not None

    def test_returns_none_when_state_absent(self):
        cache = {"states": {"LA": {"cycles": {"x": {"url": "u"}}}}}
        assert sos.lookup_cycle(cache, "TX", "x") is None

    def test_returns_none_when_cycle_absent(self):
        cache = {"states": {"LA": {"cycles": {"x": {"url": "u"}}}}}
        assert sos.lookup_cycle(cache, "LA", "y") is None


# ---------------------------------------------------------------------------
# CLI: --list
# ---------------------------------------------------------------------------

class TestList:
    def test_list_includes_all_configured_cycles(self, tmp_path, capsys, monkeypatch):
        cache_path = _fake_cycles(tmp_path, {
            "LA": {
                "cycles": {
                    "2020-pp": {"url": "x", "format": "excel", "resolution": "parish",
                                "verified": True},
                    "2024-pp": {"url": "y", "format": "excel", "resolution": "parish",
                                "verified": False},
                }
            },
            "GA": {
                "cycles": {
                    "2020-pp": {"url": "z", "format": "clarity-xml",
                                "resolution": "precinct", "verified": False},
                }
            }
        })
        monkeypatch.setattr(sos, "CYCLES_PATH", cache_path)
        rc = sos.main(["--list"])
        assert rc == 0
        out = capsys.readouterr().out
        assert "LA" in out
        assert "GA" in out
        assert "2020-pp" in out
        assert "yes" in out  # at least one verified entry shown
        assert "no" in out   # at least one unverified entry shown


# ---------------------------------------------------------------------------
# Verified-false guard
# ---------------------------------------------------------------------------

class TestVerifiedGuard:
    def test_unverified_cycle_refuses_without_override(self, tmp_path, capsys, monkeypatch):
        cache_path = _fake_cycles(tmp_path, {
            "LA": {"cycles": {"2020-pp": {
                "url": "https://example.com/file.xlsx",
                "format": "excel",
                "verified": False,
            }}}
        })
        monkeypatch.setattr(sos, "CYCLES_PATH", cache_path)
        rc = sos.main(["--state", "LA", "--cycle", "2020-pp"])
        assert rc == 1
        err = capsys.readouterr().err
        assert "verified=false" in err
        assert "--allow-unverified" in err

    def test_unverified_cycle_proceeds_with_override(self, tmp_path, monkeypatch):
        cache_path = _fake_cycles(tmp_path, {
            "LA": {"cycles": {"2020-pp": {
                "url": "https://example.com/file.xlsx",
                "format": "excel",
                "verified": False,
            }}}
        })
        monkeypatch.setattr(sos, "CYCLES_PATH", cache_path)
        with mock.patch("scrape_state_sos.download", return_value=0):
            rc = sos.main([
                "--state", "LA", "--cycle", "2020-pp",
                "--output-dir", str(tmp_path),
                "--allow-unverified",
            ])
            # download was called; parser is a stub so we exit 0 with a warning
            assert rc == 0


# ---------------------------------------------------------------------------
# TODO-URL guard inside download()
# ---------------------------------------------------------------------------

class TestTodoUrlGuard:
    def test_todo_placeholder_url_returns_1(self, capsys, tmp_path):
        rc = sos.download("TODO: verify at sos.la.gov", tmp_path / "out.bin")
        assert rc == 1
        err = capsys.readouterr().err
        assert "TODO placeholder" in err


# ---------------------------------------------------------------------------
# Missing state/cycle path
# ---------------------------------------------------------------------------

class TestMissingArgs:
    def test_neither_list_nor_state_cycle(self, capsys, tmp_path, monkeypatch):
        cache_path = _fake_cycles(tmp_path, {})
        monkeypatch.setattr(sos, "CYCLES_PATH", cache_path)
        rc = sos.main([])
        assert rc == 2
        err = capsys.readouterr().err
        assert "--state and --cycle are required" in err

    def test_unknown_state(self, capsys, tmp_path, monkeypatch):
        cache_path = _fake_cycles(tmp_path, {"LA": {"cycles": {"x": {"url": "u"}}}})
        monkeypatch.setattr(sos, "CYCLES_PATH", cache_path)
        rc = sos.main(["--state", "TX", "--cycle", "x"])
        assert rc == 1
        err = capsys.readouterr().err
        assert "TX" in err


# ---------------------------------------------------------------------------
# Real shipped config sanity checks
# ---------------------------------------------------------------------------

class TestRealCycles:
    def test_shipped_cycles_parses(self):
        cache = sos.load_cycles()
        assert cache.get("schema_version") == 1
        states = cache.get("states", {})
        assert "LA" in states
        assert "AL" in states
        assert "GA" in states

    def test_all_shipped_cycles_marked_unverified(self):
        # Every shipped cycle MUST be verified=false until a real human confirms.
        # If this test fails, someone is claiming verification without doing the work.
        cache = sos.load_cycles()
        for state, sdata in cache.get("states", {}).items():
            for cycle_id, cdata in sdata.get("cycles", {}).items():
                assert cdata.get("verified") is False, (
                    f"{state}/{cycle_id} is marked verified=true in the shipped config; "
                    f"verification requires running against the live URL and confirming the parse."
                )

    def test_parser_stubs_registered_for_all_shipped_formats(self):
        cache = sos.load_cycles()
        for state, sdata in cache.get("states", {}).items():
            for cycle_id, cdata in sdata.get("cycles", {}).items():
                fmt = cdata.get("format")
                assert fmt in sos.PARSERS, (
                    f"{state}/{cycle_id} uses format '{fmt}' which has no parser stub. "
                    f"Add it to PARSERS in scrape_state_sos.py."
                )
