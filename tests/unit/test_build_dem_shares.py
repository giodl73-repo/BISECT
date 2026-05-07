"""
Unit tests for scripts/data/political/build_dem_shares.py — the per-tract
Democratic vote share producer for partisan-weighted bisection mode.

The producer is invoked offline before `redist state --partition-mode partisan-weighted`.
Tests cover:
  - Direct mode: tract-level CSV → TSV
  - VAP-disaggregation mode: county results + tract demographics → TSV
  - GEOID normalization (leading zeros)
  - safe_dem_share zero-vote handling
  - Output format conformance with docs/file-formats/partisan-shares.md
"""

import csv
import sys
from pathlib import Path

import pytest

# Make the script importable
PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT / "scripts" / "data" / "political"))

import build_dem_shares  # noqa: E402


# ---------------------------------------------------------------------------
# Pure-function tests
# ---------------------------------------------------------------------------

class TestNormalizeGeoid:
    def test_pads_with_leading_zeros(self):
        assert build_dem_shares.normalize_geoid("1001020100") == "01001020100"

    def test_already_11_chars_unchanged(self):
        assert build_dem_shares.normalize_geoid("01001020100") == "01001020100"

    def test_rejects_non_digit(self):
        with pytest.raises(ValueError):
            build_dem_shares.normalize_geoid("01001A20100")

    def test_rejects_too_long(self):
        with pytest.raises(ValueError):
            build_dem_shares.normalize_geoid("010010201000000")


class TestSafeDemShare:
    def test_normal_split(self):
        assert build_dem_shares.safe_dem_share(60.0, 40.0) == 0.6

    def test_zero_total_returns_swing(self):
        assert build_dem_shares.safe_dem_share(0.0, 0.0) == 0.5

    def test_dem_only(self):
        assert build_dem_shares.safe_dem_share(100.0, 0.0) == 1.0

    def test_rep_only(self):
        assert build_dem_shares.safe_dem_share(0.0, 100.0) == 0.0


# ---------------------------------------------------------------------------
# Direct-mode tests
# ---------------------------------------------------------------------------

class TestDirectMode:
    def _write_csv(self, tmp_path, rows):
        path = tmp_path / "tract_results.csv"
        with path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=["geoid", "dem_votes", "rep_votes"])
            writer.writeheader()
            for r in rows:
                writer.writerow(r)
        return path

    def test_basic_three_tracts(self, tmp_path):
        path = self._write_csv(tmp_path, [
            {"geoid": "01001020100", "dem_votes": 600, "rep_votes": 400},
            {"geoid": "01001020200", "dem_votes": 200, "rep_votes": 800},
            {"geoid": "01001020300", "dem_votes": 0, "rep_votes": 0},
        ])
        shares = build_dem_shares.build_from_tract_results(path)
        assert shares["01001020100"] == 0.6
        assert shares["01001020200"] == 0.2
        assert shares["01001020300"] == 0.5  # zero-vote → swing

    def test_pads_short_geoid(self, tmp_path):
        path = self._write_csv(tmp_path, [
            {"geoid": "1001020100", "dem_votes": 100, "rep_votes": 100},
        ])
        shares = build_dem_shares.build_from_tract_results(path)
        assert "01001020100" in shares
        assert shares["01001020100"] == 0.5

    def test_rejects_negative_votes(self, tmp_path):
        path = self._write_csv(tmp_path, [
            {"geoid": "01001020100", "dem_votes": -1, "rep_votes": 100},
        ])
        with pytest.raises(ValueError, match="negative votes"):
            build_dem_shares.build_from_tract_results(path)

    def test_rejects_missing_columns(self, tmp_path):
        path = tmp_path / "bad.csv"
        path.write_text("geoid,dem_votes\n01001020100,100\n")
        with pytest.raises(ValueError, match="header missing columns"):
            build_dem_shares.build_from_tract_results(path)


# ---------------------------------------------------------------------------
# VAP-disaggregation tests
# ---------------------------------------------------------------------------

class TestVapDisaggregation:
    def _setup(self, tmp_path):
        county_path = tmp_path / "county_results.csv"
        county_path.write_text(
            "county_fips,dem_votes,rep_votes\n"
            "22001,4000,6000\n"   # 40% D
            "22003,7000,3000\n"   # 70% D
        )

        demo_path = tmp_path / "demographics.csv"
        demo_path.write_text(
            "GEOID,vap\n"
            "22001950100,5000\n"   # county 22001
            "22001950200,3000\n"   # county 22001
            "22003020100,2000\n"   # county 22003
        )
        return county_path, demo_path

    def test_assigns_county_share_to_all_tracts(self, tmp_path):
        cp, dp = self._setup(tmp_path)
        shares = build_dem_shares.build_from_county_disaggregation(cp, dp)
        # County 22001 was 40% D → all its tracts get 0.4
        assert shares["22001950100"] == 0.4
        assert shares["22001950200"] == 0.4
        # County 22003 was 70% D
        assert shares["22003020100"] == 0.7

    def test_unmapped_county_returns_swing(self, tmp_path):
        county_path = tmp_path / "county_results.csv"
        county_path.write_text("county_fips,dem_votes,rep_votes\n22001,1000,1000\n")
        demo_path = tmp_path / "demographics.csv"
        demo_path.write_text("GEOID,vap\n22099950100,1000\n")  # county 22099 not in results
        shares = build_dem_shares.build_from_county_disaggregation(county_path, demo_path)
        assert shares["22099950100"] == 0.5

    def test_accepts_voting_age_pop_alias(self, tmp_path):
        county_path = tmp_path / "county_results.csv"
        county_path.write_text("county_fips,dem_votes,rep_votes\n22001,500,500\n")
        demo_path = tmp_path / "demographics.csv"
        demo_path.write_text("GEOID,voting_age_pop\n22001950100,1000\n")
        shares = build_dem_shares.build_from_county_disaggregation(county_path, demo_path)
        assert shares["22001950100"] == 0.5


# ---------------------------------------------------------------------------
# Output format tests
# ---------------------------------------------------------------------------

class TestOutputFormat:
    def test_tsv_round_trips_through_loader(self, tmp_path):
        """Writer output is consumable by the Rust loader (validated via format spec)."""
        out = tmp_path / "shares.tsv"
        shares = {"01001020100": 0.42, "01001020200": 0.71}
        build_dem_shares.write_tsv(shares, out, {"producer": "test"})

        content = out.read_text(encoding="utf-8")
        # Must have provenance comments
        assert content.startswith("# producer: test\n")
        # Must have header row
        assert "geoid\tdem_share" in content
        # Must have data rows in sorted geoid order
        lines = [l for l in content.splitlines() if l and not l.startswith("#") and not l.startswith("geoid")]
        assert lines == ["01001020100\t0.420000", "01001020200\t0.710000"]

    def test_writer_rejects_out_of_range(self, tmp_path):
        out = tmp_path / "shares.tsv"
        with pytest.raises(ValueError, match="out of"):
            build_dem_shares.write_tsv({"01001020100": 1.5}, out, {})

    def test_creates_parent_directory(self, tmp_path):
        out = tmp_path / "nested" / "deep" / "shares.tsv"
        build_dem_shares.write_tsv({"01001020100": 0.5}, out, {})
        assert out.exists()
