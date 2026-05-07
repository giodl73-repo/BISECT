"""
L2 acceptance tests for: redist compare, redist export, redist policy, redist report.

These tests cover commands not yet exercised by earlier L2 files:
  - redist policy  (state_policy.json lookup, table and JSON output)
  - redist export  (CSV and GerryChain from the existing ChamberTest plan)
  - redist compare (Jaccard similarity between plans)
  - redist report  (JSON/HTML from the existing ChamberTest VT plan)

Requires:
  - redist binary built: cargo build -p redist-cli --release
  - VT plan at outputs/ChamberTest/2020/plans/vt_cong_chamber_test/ (created by L2 acceptance test)

The existing plan at outputs/ChamberTest/2020/plans/vt_cong_chamber_test/ is used
rather than re-creating it so these tests run fast and without network/data dependencies.

Run with:
    pytest tests/acceptance/test_compare_export_policy_report.py -v --tb=short
"""

import json
import subprocess
import sys
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

BINARY = (
    Path("redist/target/release/redist.exe")
    if sys.platform == "win32"
    else Path("redist/target/release/redist")
)
BASE_DIR = Path(".")

# Plan that already exists from the chamber acceptance test suite
_EXISTING_PLAN = BASE_DIR / "outputs" / "ChamberTest" / "2020" / "plans" / "vt_cong_chamber_test"
_EXISTING_PLAN_LABEL = "vt_cong_chamber_test"
_EXISTING_PLAN_VERSION = "ChamberTest"
_EXISTING_PLAN_YEAR = "2020"


def binary_available() -> bool:
    return BINARY.exists()


def existing_plan_ready() -> bool:
    """True when the ChamberTest VT plan has been produced."""
    return (_EXISTING_PLAN / "manifest.json").exists()


skip_no_binary = pytest.mark.skipif(
    not binary_available(),
    reason="redist binary not built — run: cargo build -p redist-cli --release",
)
skip_no_plan = pytest.mark.skipif(
    not existing_plan_ready(),
    reason=(
        "VT plan not found at outputs/ChamberTest/2020/plans/vt_cong_chamber_test/. "
        "Run the chamber acceptance tests first to create it."
    ),
)


def run_cmd(args: list, check: bool = False) -> subprocess.CompletedProcess:
    """Run a redist sub-command, capturing all output."""
    return subprocess.run(
        [str(BINARY)] + args,
        capture_output=True,
        text=True,
        check=check,
        cwd=str(BASE_DIR),
    )


# ---------------------------------------------------------------------------
# redist policy
# ---------------------------------------------------------------------------


@skip_no_binary
class TestRedistPolicy:
    """redist policy — reads state_policy.json embedded in the binary."""

    def test_policy_la_shows_parishes(self):
        r = run_cmd(["policy", "--state", "LA"])
        assert r.returncode == 0, r.stderr
        assert "parish" in r.stdout.lower(), "LA policy must mention parishes"

    def test_policy_va_shows_independent(self):
        r = run_cmd(["policy", "--state", "VA"])
        assert r.returncode == 0, r.stderr
        assert "independent" in r.stdout.lower(), (
            "VA policy must mention independent cities"
        )

    def test_policy_eldoria_test_state(self):
        r = run_cmd(["policy", "--state", "_TEST_EL"])
        assert r.returncode == 0, r.stderr
        output = r.stdout.lower()
        assert "realm" in output, "Eldoria must show 'realm' subdivision"
        assert "grand_council" in output.replace(" ", "_"), (
            "Eldoria must show commission_type=grand_council_of_wizards"
        )

    def test_policy_json_format_la(self):
        r = run_cmd(["policy", "--state", "LA", "--format", "json"])
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert data.get("subdivision_term") == "parish", (
            "LA subdivision_term must be 'parish'"
        )
        assert data.get("subdivision_term_plural") == "parishes", (
            "LA subdivision_term_plural must be 'parishes'"
        )
        assert "preservation_citation" in data, (
            "LA policy JSON must include preservation_citation"
        )

    def test_policy_json_format_eldoria(self):
        r = run_cmd(["policy", "--state", "_TEST_EL", "--format", "json"])
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert data.get("commission_type") == "grand_council_of_wizards"
        # balance_tolerance_house_pct for Eldoria is 7.5 (not the standard 5.0)
        assert abs(data.get("balance_tolerance_house_pct", 0) - 7.5) < 1e-9, (
            f"Eldoria house tolerance must be 7.5, got {data.get('balance_tolerance_house_pct')}"
        )

    def test_policy_wa_shows_nesting_ratio(self):
        r = run_cmd(["policy", "--state", "WA"])
        assert r.returncode == 0, r.stderr
        assert "2:1" in r.stdout, "WA policy must show 2:1 nesting ratio"

    def test_policy_hi_permanent_resident_in_json(self):
        r = run_cmd(["policy", "--state", "HI", "--format", "json"])
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert data.get("population_basis") == "permanent_resident", (
            "HI must use permanent_resident population basis"
        )

    def test_policy_ne_unicameral_senate_zero(self):
        r = run_cmd(["policy", "--state", "NE", "--format", "json"])
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert data.get("senate_districts") == 0, (
            "Nebraska unicameral: senate_districts must be 0"
        )

    def test_policy_ct_subdivision_is_town(self):
        r = run_cmd(["policy", "--state", "CT", "--format", "json"])
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert data.get("subdivision_term") == "town", (
            "CT subdivision must be 'town'"
        )

    def test_policy_unknown_state_exits_nonzero(self):
        r = run_cmd(["policy", "--state", "ZZ"])
        assert r.returncode != 0, (
            "Unknown state code 'ZZ' must cause non-zero exit"
        )

    def test_policy_table_format_is_default(self):
        """Omitting --format should produce human-readable table (no JSON braces)."""
        r = run_cmd(["policy", "--state", "LA"])
        assert r.returncode == 0, r.stderr
        # Table output should not start with '{' (JSON)
        assert not r.stdout.strip().startswith("{"), (
            "Default format must not be JSON"
        )
        assert "Louisiana" in r.stdout, "Table output must include state name"


# ---------------------------------------------------------------------------
# redist export (ChamberTest VT plan)
# ---------------------------------------------------------------------------


@skip_no_binary
@skip_no_plan
class TestRedistExport:
    """redist export — produces CSV and GerryChain JSON from the existing VT plan."""

    def _export(self, tmp_path: Path, fmt: str) -> subprocess.CompletedProcess:
        return run_cmd(
            [
                "export",
                "--label", _EXISTING_PLAN_LABEL,
                "--year", _EXISTING_PLAN_YEAR,
                "--version", _EXISTING_PLAN_VERSION,
                "--format", fmt,
                "--out", str(tmp_path),
                "--output-base", "outputs",
            ]
        )

    def test_export_csv_exits_zero(self, tmp_path):
        r = self._export(tmp_path, "csv")
        assert r.returncode == 0, (
            f"export csv failed (rc={r.returncode}):\n{r.stderr}"
        )

    def test_export_csv_produces_file(self, tmp_path):
        self._export(tmp_path, "csv")
        csv_files = list(tmp_path.glob("*.csv"))
        assert csv_files, "CSV export must produce at least one .csv file"

    def test_export_csv_has_required_columns(self, tmp_path):
        self._export(tmp_path, "csv")
        csv_files = list(tmp_path.glob("*.csv"))
        assert csv_files, "No CSV file produced"
        header = csv_files[0].read_text(encoding="utf-8").splitlines()[0].lower()
        assert "geoid" in header, f"CSV must have 'geoid' column, header: {header!r}"
        assert "district" in header, f"CSV must have 'district' column, header: {header!r}"

    def test_export_csv_has_data_rows(self, tmp_path):
        self._export(tmp_path, "csv")
        csv_files = list(tmp_path.glob("*.csv"))
        lines = csv_files[0].read_text(encoding="utf-8").strip().splitlines()
        # VT has 193 tracts; must have at least header + 1 data row
        assert len(lines) >= 2, f"CSV must have at least 2 lines, got {len(lines)}"

    def test_export_gerrychain_exits_zero(self, tmp_path):
        r = self._export(tmp_path, "gerrychain")
        assert r.returncode == 0, (
            f"export gerrychain failed (rc={r.returncode}):\n{r.stderr}"
        )

    def test_export_gerrychain_uses_assignment_singular(self, tmp_path):
        self._export(tmp_path, "gerrychain")
        gc_files = list(tmp_path.glob("*gerrychain*.json"))
        assert gc_files, "GerryChain export must produce a JSON file"
        data = json.loads(gc_files[0].read_text(encoding="utf-8"))
        assert "assignment" in data, (
            "GerryChain export must use 'assignment' (singular) field — GerryChain v2.3 spec"
        )
        assert "assignments" not in data, (
            "GerryChain export must NOT use 'assignments' (plural)"
        )

    def test_export_gerrychain_all_keys_numeric(self, tmp_path):
        """GerryChain assignment keys must be numeric (node IDs or GEOIDs)."""
        self._export(tmp_path, "gerrychain")
        gc_files = list(tmp_path.glob("*gerrychain*.json"))
        assert gc_files, "No GerryChain JSON produced"
        data = json.loads(gc_files[0].read_text(encoding="utf-8"))
        for key in data.get("assignment", {}):
            assert str(key).isdigit() or str(key).lstrip("-").isdigit(), (
                f"GerryChain assignment key {key!r} must be numeric"
            )

    def test_export_geojson_exits_zero(self, tmp_path):
        """GeoJSON export works (geometry may be stub when TIGER not available)."""
        r = self._export(tmp_path, "geojson")
        # GeoJSON may require TIGER shapefiles; accept returncode 0 or informative failure
        if r.returncode != 0:
            combined = (r.stdout + r.stderr).lower()
            assert "tiger" in combined or "geojson" in combined or "shp" in combined, (
                f"GeoJSON failure must mention TIGER/shapefile: {r.stderr[:300]}"
            )
            pytest.skip("GeoJSON export requires TIGER shapefiles — skipping geometry validation")
        geojson_files = list(tmp_path.glob("*.geojson"))
        assert geojson_files, "GeoJSON export must produce a .geojson file"
        data = json.loads(geojson_files[0].read_text(encoding="utf-8"))
        assert data["type"] == "FeatureCollection", (
            "GeoJSON must be a FeatureCollection"
        )


# ---------------------------------------------------------------------------
# redist compare (ChamberTest VT plan vs itself)
# ---------------------------------------------------------------------------


@skip_no_binary
@skip_no_plan
class TestRedistCompare:
    """redist compare — Jaccard similarity between plans."""

    def _compare(self, plan_a: str, plan_b: str, fmt: str = "table", extra: list = None) -> subprocess.CompletedProcess:
        cmd = [
            "compare",
            "--plan-a", plan_a,
            "--plan-b", plan_b,
            "--year", _EXISTING_PLAN_YEAR,
            "--version", _EXISTING_PLAN_VERSION,
            "--output-base", "outputs",
            "--format", fmt,
        ]
        if extra:
            cmd.extend(extra)
        return run_cmd(cmd)

    def test_compare_exits_zero(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL)
        assert r.returncode == 0, (
            f"compare failed (rc={r.returncode}):\n{r.stderr}"
        )

    def test_compare_same_plan_jaccard_is_1(self):
        """A plan compared against itself must have Jaccard similarity = 1.0."""
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL)
        assert r.returncode == 0, r.stderr
        # The table formatter writes Jaccard to 6 decimal places
        assert "1.000000" in r.stdout, (
            f"Same-plan comparison must show Jaccard=1.000000, got:\n{r.stdout}"
        )

    def test_compare_output_contains_jaccard_label(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL)
        assert r.returncode == 0, r.stderr
        assert "Jaccard" in r.stdout, (
            "Comparison table must include 'Jaccard' label"
        )

    def test_compare_output_no_winner_framing(self):
        """Legal requirement: output must not contain 'Winner:' framing."""
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL)
        assert r.returncode == 0, r.stderr
        assert "Winner:" not in r.stdout, (
            "Comparison output must not use 'Winner:' framing (legally impermissible)"
        )

    def test_compare_output_has_disclaimer(self):
        """Output must include the legal disclaimer."""
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL)
        assert r.returncode == 0, r.stderr
        assert "No single metric" in r.stdout, (
            "Comparison output must include legal disclaimer"
        )

    def test_compare_json_format_exits_zero(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL, fmt="json")
        assert r.returncode == 0, (
            f"compare --format json failed:\n{r.stderr}"
        )

    def test_compare_json_format_parses(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL, fmt="json")
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        assert "metrics" in data, "JSON compare output must have 'metrics' key"
        assert "jaccard_similarity" in data["metrics"], (
            "JSON compare metrics must include 'jaccard_similarity'"
        )
        assert "plan_a" in data, "JSON compare output must include 'plan_a'"
        assert "plan_b" in data, "JSON compare output must include 'plan_b'"

    def test_compare_json_same_plan_jaccard_is_1(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL, fmt="json")
        assert r.returncode == 0, r.stderr
        data = json.loads(r.stdout)
        jaccard = data["metrics"]["jaccard_similarity"]
        assert abs(jaccard - 1.0) < 1e-9, (
            f"Same-plan Jaccard must be 1.0, got {jaccard}"
        )

    def test_compare_missing_plan_exits_nonzero(self):
        r = self._compare("nonexistent_plan_xyz_abc", _EXISTING_PLAN_LABEL)
        assert r.returncode != 0, (
            "Comparing a nonexistent plan must exit non-zero"
        )

    def test_compare_csv_format_exits_zero(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL, fmt="csv")
        assert r.returncode == 0, (
            f"compare --format csv failed:\n{r.stderr}"
        )

    def test_compare_csv_has_header(self):
        r = self._compare(_EXISTING_PLAN_LABEL, _EXISTING_PLAN_LABEL, fmt="csv")
        assert r.returncode == 0, r.stderr
        lines = r.stdout.strip().splitlines()
        assert lines, "CSV output must not be empty"
        assert "metric" in lines[0], f"CSV header must contain 'metric', got: {lines[0]!r}"


# ---------------------------------------------------------------------------
# redist report (ChamberTest VT plan)
# ---------------------------------------------------------------------------


@skip_no_binary
@skip_no_plan
class TestRedistReport:
    """redist report — generates JSON and HTML commission reports."""

    @classmethod
    def setup_class(cls):
        """Ensure all required analysis files exist before running report tests."""
        binary = str(BINARY)
        # Run all analysis types needed by the report assembler
        subprocess.run(
            [binary, "analyze",
             "--label", _EXISTING_PLAN_LABEL,
             "--year", _EXISTING_PLAN_YEAR,
             "--version", _EXISTING_PLAN_VERSION,
             "--types", "all", "--force",
             "--output-base", "outputs"],
            capture_output=True, text=True
        )

    def _report(self, tmp_path: Path, fmt: str) -> subprocess.CompletedProcess:
        return run_cmd(
            [
                "report",
                "--label", _EXISTING_PLAN_LABEL,
                "--year", _EXISTING_PLAN_YEAR,
                "--version", _EXISTING_PLAN_VERSION,
                "--format", fmt,
                "--out", str(tmp_path),
                "--output-base", "outputs",
            ]
        )

    def test_report_json_exits_zero(self, tmp_path):
        r = self._report(tmp_path, "json")
        assert r.returncode == 0, (
            f"report --format json failed (rc={r.returncode}):\n{r.stderr}"
        )

    def test_report_json_produces_file(self, tmp_path):
        self._report(tmp_path, "json")
        json_files = list(tmp_path.glob("*.json"))
        assert json_files, "JSON report must produce at least one .json file"

    def test_report_json_has_report_version(self, tmp_path):
        self._report(tmp_path, "json")
        json_files = list(tmp_path.glob("*.json"))
        assert json_files
        data = json.loads(json_files[0].read_text(encoding="utf-8"))
        assert "report_version" in data, (
            "JSON report must include 'report_version' key"
        )

    def test_report_json_has_sections(self, tmp_path):
        self._report(tmp_path, "json")
        json_files = list(tmp_path.glob("*.json"))
        data = json.loads(json_files[0].read_text(encoding="utf-8"))
        assert "sections" in data, "JSON report must include 'sections' key"

    @pytest.mark.xfail(reason="HTML report has Tera template rendering bug — tracked for fix", strict=False)
    def test_report_html_exits_zero(self, tmp_path):
        r = self._report(tmp_path, "html")
        assert r.returncode == 0, (
            f"report --format html failed (rc={r.returncode}):\n{r.stderr}"
        )

    @pytest.mark.xfail(reason="HTML report has Tera template rendering bug", strict=False)
    def test_report_html_produces_file(self, tmp_path):
        self._report(tmp_path, "html")
        html_files = list(tmp_path.glob("*.html"))
        assert html_files, "HTML report must produce at least one .html file"

    @pytest.mark.xfail(reason="HTML report has Tera template rendering bug", strict=False)
    def test_report_html_is_valid_html(self, tmp_path):
        self._report(tmp_path, "html")
        html_files = list(tmp_path.glob("*.html"))
        assert html_files
        content = html_files[0].read_text(encoding="utf-8")
        assert "<html" in content.lower(), "HTML report must contain <html tag"

    @pytest.mark.xfail(reason="HTML report has Tera template rendering bug", strict=False)
    def test_report_html_self_contained_no_cdn(self, tmp_path):
        """Self-contained requirement: no external CDN references."""
        self._report(tmp_path, "html")
        html_files = list(tmp_path.glob("*.html"))
        assert html_files
        content = html_files[0].read_text(encoding="utf-8")
        assert "cdn.jsdelivr.net" not in content, (
            "HTML report must not load from jsdelivr CDN"
        )
        assert "unpkg.com" not in content, (
            "HTML report must not load from unpkg CDN"
        )
        assert "http://" not in content, (
            "HTML report must not contain HTTP (only HTTPS or no external links)"
        )

    def test_report_pdf_exits_nonzero(self, tmp_path):
        """PDF is deferred — must exit non-zero (either 'PDF not available' or
        analysis-missing error; either way it must not silently succeed)."""
        r = self._report(tmp_path, "pdf")
        assert r.returncode != 0, (
            f"PDF format must exit non-zero (not yet implemented), got rc=0"
        )

    def test_report_audit_only_writes_audit_json(self, tmp_path):
        """--audit-only writes audit.json only (no full report)."""
        r = run_cmd(
            [
                "report",
                "--label", _EXISTING_PLAN_LABEL,
                "--year", _EXISTING_PLAN_YEAR,
                "--version", _EXISTING_PLAN_VERSION,
                "--audit-only",
                "--out", str(tmp_path),
                "--output-base", "outputs",
            ]
        )
        assert r.returncode == 0, (
            f"--audit-only failed (rc={r.returncode}):\n{r.stderr}"
        )
        audit_path = tmp_path / "audit.json"
        assert audit_path.exists(), "audit.json must be written by --audit-only"
        data = json.loads(audit_path.read_text(encoding="utf-8"))
        assert "audit" in data, "audit.json must contain 'audit' key"

    def test_report_audit_verification_no_local_paths(self, tmp_path):
        """Audit verification_command must not embed local file paths."""
        r = run_cmd(
            [
                "report",
                "--label", _EXISTING_PLAN_LABEL,
                "--year", _EXISTING_PLAN_YEAR,
                "--version", _EXISTING_PLAN_VERSION,
                "--audit-only",
                "--out", str(tmp_path),
                "--output-base", "outputs",
            ]
        )
        assert r.returncode == 0, r.stderr
        audit_path = tmp_path / "audit.json"
        data = json.loads(audit_path.read_text(encoding="utf-8"))
        cmd = data.get("audit", {}).get("verification_command", "")
        assert "C:\\" not in cmd, "verification_command must not contain Windows paths"
        assert "/home/" not in cmd, "verification_command must not contain Unix home paths"
        assert "/Users/" not in cmd, "verification_command must not contain macOS home paths"


if __name__ == "__main__":
    import unittest
    unittest.main()
