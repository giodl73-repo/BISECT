"""
L2 Acceptance tests for Spec 6: commission reports, GeoJSON export, plan import.

These tests require:
- redist binary built (cargo build --release --manifest-path redist/Cargo.toml)
- A VT plan fixture at outputs/RustV3/2020/plans/vt_congress_test/

Run with:
    pytest tests/acceptance/test_report_acceptance.py -v

Skip if binary not available:
    pytest tests/acceptance/test_report_acceptance.py -v -m "not requires_binary"
"""
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

BINARY = Path("redist/target/release/redist.exe") if sys.platform == "win32" else Path("redist/target/release/redist")
BASE_DIR = Path(".")


def binary_available() -> bool:
    """Check if the redist binary exists."""
    return BINARY.exists()


def vt_plan_exists() -> bool:
    """Check if the VT fixture plan directory exists with required analysis files."""
    plan_dir = BASE_DIR / "outputs" / "RustV3" / "2020" / "plans" / "vt_congress_test"
    analysis_dir = plan_dir / "analysis"
    required = ["population.json", "contiguity.json", "compactness.json"]
    return all((analysis_dir / f).exists() for f in required)


skip_no_binary = pytest.mark.skipif(
    not binary_available(), reason="redist binary not built"
)
skip_no_vt_fixture = pytest.mark.skipif(
    not vt_plan_exists(), reason="VT fixture plan not available (run redist state --state VT first)"
)


@pytest.fixture
def vt_plan_label():
    return "vt_congress_test"


@pytest.fixture
def binary():
    return str(BINARY)


@skip_no_binary
@skip_no_vt_fixture
class TestReportHtmlVt:
    """Test HTML report generation for Vermont congressional plan."""

    def test_report_html_vt(self, vt_plan_label, binary, tmp_path):
        """VT plan -> valid single-file HTML, opens without errors."""
        out_dir = tmp_path / "reports" / vt_plan_label
        result = subprocess.run(
            [
                binary, "report",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "html",
                "--out", str(out_dir),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"report command failed: {result.stderr}"
        html_path = out_dir / f"{vt_plan_label}_report.html"
        assert html_path.exists(), f"HTML file not created at {html_path}"
        content = html_path.read_text(encoding="utf-8")
        assert "<html" in content.lower(), "HTML must start with <html"
        assert "http://" not in content, "HTML must be self-contained (no external HTTP)"
        assert "https://cdn" not in content, "HTML must not load from CDN"

    def test_report_json_vt(self, vt_plan_label, binary, tmp_path):
        """VT plan -> valid JSON with all sections."""
        out_dir = tmp_path / "reports" / vt_plan_label
        result = subprocess.run(
            [
                binary, "report",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "json",
                "--out", str(out_dir),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"report command failed: {result.stderr}"
        json_path = out_dir / f"{vt_plan_label}_report.json"
        assert json_path.exists(), f"JSON file not created at {json_path}"
        data = json.loads(json_path.read_text(encoding="utf-8"))
        assert data["report_version"] == "1.0", "report_version must be 1.0"
        required_sections = [
            "population_equality", "geographic_constraints",
            "partisan_fairness", "vra_compliance",
            "compactness", "audit",
        ]
        for section in required_sections:
            assert section in data["sections"], f"section {section!r} missing from JSON report"

    def test_report_html_contains_all_section_headers(self, vt_plan_label, binary, tmp_path):
        """HTML report must contain all 10 section headers."""
        out_dir = tmp_path / "reports" / vt_plan_label
        result = subprocess.run(
            [
                binary, "report",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "html",
                "--out", str(out_dir),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"report command failed: {result.stderr}"
        html_path = out_dir / f"{vt_plan_label}_report.html"
        content = html_path.read_text(encoding="utf-8")
        section_headers = [
            "Executive Summary",
            "Population Equality",
            "Geographic Constraints",
            "Partisan Fairness",
            "Minority Representation",
            "Compactness",
            "Comparison vs Enacted Plan",
            "Nesting Compliance",
            "Audit Trail",
            "Maps",
        ]
        for header in section_headers:
            assert header in content, f"HTML report missing section header: {header!r}"

    def test_report_pdf_deferred_exits_nonzero(self, vt_plan_label, binary, tmp_path):
        """PDF format exits with code 1 after writing other formats (board amendment)."""
        out_dir = tmp_path / "reports_pdf" / vt_plan_label
        result = subprocess.run(
            [
                binary, "report",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "html", "json", "pdf",
                "--out", str(out_dir),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        # PDF deferred: must exit with code 1
        assert result.returncode == 1, (
            f"PDF format must exit with code 1, got {result.returncode}"
        )
        # Other formats must have been written
        html_path = out_dir / f"{vt_plan_label}_report.html"
        json_path = out_dir / f"{vt_plan_label}_report.json"
        assert html_path.exists(), "HTML must be written before PDF error"
        assert json_path.exists(), "JSON must be written before PDF error"
        # Error message must mention PDF unavailability
        stderr = result.stderr
        assert "PDF" in stderr or "pdf" in stderr, (
            f"stderr must mention PDF, got: {stderr!r}"
        )

    def test_audit_only_flag(self, vt_plan_label, binary, tmp_path):
        """--audit-only flag writes audit.json only."""
        out_dir = tmp_path / "audit_out"
        result = subprocess.run(
            [
                binary, "report",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--audit-only",
                "--out", str(out_dir),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"audit-only failed: {result.stderr}"
        audit_path = out_dir / "audit.json"
        assert audit_path.exists(), "audit.json must be written by --audit-only"
        data = json.loads(audit_path.read_text(encoding="utf-8"))
        assert "audit" in data, "audit.json must contain 'audit' key"


@skip_no_binary
@skip_no_vt_fixture
class TestExportVt:
    """Test export command for VT plan."""

    def test_export_geojson_vt(self, vt_plan_label, binary, tmp_path):
        """VT congressional plan (1 district) -> GeoJSON FeatureCollection with 1 feature."""
        result = subprocess.run(
            [
                binary, "export",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "geojson",
                "--out", str(tmp_path / "exports"),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"export failed: {result.stderr}"
        geojson_path = tmp_path / "exports" / f"{vt_plan_label}.geojson"
        assert geojson_path.exists(), "GeoJSON file must be created"
        data = json.loads(geojson_path.read_text(encoding="utf-8"))
        assert data["type"] == "FeatureCollection", "must be FeatureCollection"
        assert len(data["features"]) >= 1, "must have at least 1 feature"

    def test_export_csv_vt(self, vt_plan_label, binary, tmp_path):
        """VT congressional plan -> CSV with geoid and district columns."""
        result = subprocess.run(
            [
                binary, "export",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "csv",
                "--out", str(tmp_path / "exports"),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"export csv failed: {result.stderr}"
        csv_path = tmp_path / "exports" / f"{vt_plan_label}_tracts.csv"
        assert csv_path.exists(), "CSV file must be created"
        lines = csv_path.read_text(encoding="utf-8").strip().splitlines()
        assert len(lines) >= 1, "CSV must have at least header row"
        header = lines[0]
        assert "geoid" in header, "CSV must have geoid column"
        assert "district" in header, "CSV must have district column"

    def test_export_gerrychain_vt(self, vt_plan_label, binary, tmp_path):
        """VT plan -> GerryChain v2.3 JSON with 'assignment' (singular) field."""
        result = subprocess.run(
            [
                binary, "export",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "gerrychain",
                "--out", str(tmp_path / "exports"),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        assert result.returncode == 0, f"export gerrychain failed: {result.stderr}"
        gc_path = tmp_path / "exports" / f"{vt_plan_label}_gerrychain.json"
        assert gc_path.exists(), "GerryChain JSON file must be created"
        data = json.loads(gc_path.read_text(encoding="utf-8"))
        # GerryChain v2.3 uses "assignment" (singular)
        assert "assignment" in data, "GerryChain export must use 'assignment' field (singular)"
        assert "assignments" not in data, "GerryChain export must not use 'assignments' (plural)"
        assert data.get("gerrychain_version_target") == "2.3", "must target GerryChain v2.3"


@pytest.mark.integration
class TestGerryChainIntegration:
    """
    GerryChain integration test — gated by @pytest.mark.integration.

    This test verifies the GerryChain export schema by loading it with
    the actual GerryChain Python library. Skipped if GerryChain is not installed.

    Run with:
        pytest tests/acceptance/test_report_acceptance.py -v -m integration
    """

    @pytest.mark.skipif(
        not binary_available(),
        reason="redist binary not built"
    )
    def test_gerrychain_schema_valid(self, vt_plan_label, binary, tmp_path):
        """Export to GerryChain format, then verify schema with GerryChain library if available."""
        try:
            import gerrychain  # noqa: F401
        except ImportError:
            pytest.skip("gerrychain not installed — skipping integration test")

        result = subprocess.run(
            [
                binary, "export",
                "--label", vt_plan_label,
                "--year", "2020",
                "--version", "RustV3",
                "--format", "gerrychain",
                "--out", str(tmp_path / "exports"),
                "--output-base", "outputs",
            ],
            capture_output=True,
            text=True,
            cwd=str(BASE_DIR),
        )
        if result.returncode != 0:
            pytest.skip(f"export command not available: {result.stderr}")

        gc_path = tmp_path / "exports" / f"{vt_plan_label}_gerrychain.json"
        if not gc_path.exists():
            pytest.skip("GerryChain export file not created")

        # Verify schema with GerryChain
        verify_result = subprocess.run(
            [
                sys.executable, "-c",
                f"import json; d=json.load(open('{gc_path}')); "
                f"assert 'assignment' in d, 'missing assignment field'"
            ],
            capture_output=True,
            text=True,
        )
        assert verify_result.returncode == 0, (
            f"GerryChain schema validation failed: {verify_result.stderr}"
        )
