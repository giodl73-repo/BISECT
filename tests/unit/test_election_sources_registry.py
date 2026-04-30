"""
Unit tests for scripts/data/elections/fetch_elections.py — the election-data
source registry + fetcher dispatcher.

Verifies:
  - sources.json parses and conforms to the schema we promise users
  - filter_sources respects each filter dimension
  - get_source / show / list paths return what we expect
  - fetch dispatch refuses sources without a fetcher
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "elections"))

import fetch_elections  # noqa: E402


# ---------------------------------------------------------------------------
# Registry shape
# ---------------------------------------------------------------------------

def test_registry_loads_and_has_sources():
    reg = fetch_elections.load_registry()
    assert reg.get("schema_version") == 1
    sources = reg.get("sources", [])
    assert len(sources) >= 3, "registry should have at least 3 sources"


def test_each_source_has_required_fields():
    reg = fetch_elections.load_registry()
    required = {"id", "name", "license", "resolution", "level", "years", "states", "format"}
    for s in reg["sources"]:
        missing = required - set(s.keys())
        assert not missing, f"source {s.get('id')} missing fields: {missing}"


def test_source_ids_are_unique():
    reg = fetch_elections.load_registry()
    ids = [s["id"] for s in reg["sources"]]
    assert len(ids) == len(set(ids)), f"duplicate source ids: {ids}"


def test_fetcher_paths_resolve_when_set():
    """If a source declares a fetcher script, the script must exist on disk."""
    reg = fetch_elections.load_registry()
    for s in reg["sources"]:
        fetcher = s.get("fetcher")
        if fetcher:
            full = PROJECT_ROOT / fetcher
            assert full.exists(), f"source {s['id']}: fetcher script missing at {full}"


# ---------------------------------------------------------------------------
# Filtering
# ---------------------------------------------------------------------------

class TestFilterSources:
    def setup_method(self):
        self.reg = fetch_elections.load_registry()
        self.sources = self.reg["sources"]

    def test_no_filters_returns_all(self):
        out = fetch_elections.filter_sources(self.sources)
        assert len(out) == len(self.sources)

    def test_resolution_filter_matches_substring(self):
        out = fetch_elections.filter_sources(self.sources, resolution="tract")
        assert any(s["id"] == "harvard-fekrazad-2020" for s in out)
        # MIT EDSL president has resolution "county | state | congressional-district"
        # — should not match the "tract" filter.
        assert not any(s["id"] == "mit-edsl-president-1976-2022" for s in out)

    def test_year_filter(self):
        out_2020 = fetch_elections.filter_sources(self.sources, year=2020)
        assert all(2020 in s["years"] for s in out_2020 if isinstance(s["years"], list))
        out_1976 = fetch_elections.filter_sources(self.sources, year=1976)
        # MIT EDSL covers 1976 (multiple variants now: president + senate-primary + house-primary)
        assert any("mit-edsl" in s["id"] for s in out_1976)
        assert not any(s["id"] == "harvard-fekrazad-2020" for s in out_1976)

    def test_license_filter(self):
        out = fetch_elections.filter_sources(self.sources, license_="CC0")
        assert any(s["id"] == "harvard-fekrazad-2020" for s in out)

    def test_automated_only(self):
        out = fetch_elections.filter_sources(self.sources, automated_only=True)
        assert all(s.get("fetcher") for s in out)
        # We should have at least the Harvard Dataverse one
        assert any(s["id"] == "harvard-fekrazad-2020" for s in out)


# ---------------------------------------------------------------------------
# get_source
# ---------------------------------------------------------------------------

def test_get_source_known_id():
    reg = fetch_elections.load_registry()
    s = fetch_elections.get_source(reg, "harvard-fekrazad-2020")
    assert s is not None
    assert s["id"] == "harvard-fekrazad-2020"


def test_get_source_unknown_id_returns_none():
    reg = fetch_elections.load_registry()
    s = fetch_elections.get_source(reg, "nonexistent-source")
    assert s is None


# ---------------------------------------------------------------------------
# Fetch dispatch
# ---------------------------------------------------------------------------

class TestFetchDispatch:
    def test_no_fetcher_returns_1_and_prints_url(self, capsys):
        # Pick a source we know has fetcher: null
        reg = fetch_elections.load_registry()
        s = fetch_elections.get_source(reg, "mggg-openprecincts")
        assert s is not None
        assert s.get("fetcher") is None
        rc = fetch_elections.fetch(s, [])
        assert rc == 1
        captured = capsys.readouterr().out
        assert "no automated fetcher" in captured.lower()
        assert s["url"] in captured

    def test_missing_fetcher_script_returns_2(self, capsys, tmp_path, monkeypatch):
        # Synthesize a source whose fetcher path doesn't exist
        fake_source = {
            "id": "fake-source",
            "name": "fake",
            "license": "CC0-1.0",
            "resolution": "tract",
            "level": "presidential",
            "years": [2020],
            "states": "all-50",
            "format": "csv",
            "fetcher": "scripts/data/elections/__nonexistent__.py",
        }
        # Run from project root so the path-relative check fires
        monkeypatch.chdir(PROJECT_ROOT)
        rc = fetch_elections.fetch(fake_source, [])
        assert rc == 2
        captured = capsys.readouterr()
        assert "not found" in captured.err.lower()


# ---------------------------------------------------------------------------
# Print helpers don't crash on any registered source
# ---------------------------------------------------------------------------

def test_print_table_handles_all_sources(capsys):
    reg = fetch_elections.load_registry()
    fetch_elections.print_table(reg["sources"])
    out = capsys.readouterr().out
    for s in reg["sources"]:
        assert s["id"][:27] in out


def test_print_detail_handles_all_sources(capsys):
    reg = fetch_elections.load_registry()
    for s in reg["sources"]:
        fetch_elections.print_detail(s)
    # Just verify nothing raised; output content varies
    out = capsys.readouterr().out
    assert "id:" in out
    assert "license:" in out
