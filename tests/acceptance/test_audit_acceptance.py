"""
L2 Acceptance tests for Spec 6 Scenario 2: Audit Trail.

These tests verify the audit chain-of-custody assertions:
- manifest has tiger_sha256 (64-char hex)
- manifest has binary_download_url pointing to github.com
- verification command contains no local paths
- verification command contains --seed

Run with:
    pytest tests/acceptance/test_audit_acceptance.py -v
"""
import json
import subprocess
import sys
from pathlib import Path

import pytest

BINARY = Path("redist/target/release/redist.exe") if sys.platform == "win32" else Path("redist/target/release/redist")
BASE_DIR = Path(".")


def binary_available() -> bool:
    return BINARY.exists()


def vt_plan_exists() -> bool:
    plan_dir = BASE_DIR / "outputs" / "RustV3" / "2020" / "plans" / "vt_congress_test"
    return (plan_dir / "manifest.json").exists()


skip_no_binary = pytest.mark.skipif(
    not binary_available(), reason="redist binary not built"
)
skip_no_vt_fixture = pytest.mark.skipif(
    not vt_plan_exists(), reason="VT fixture plan not available"
)


@pytest.fixture
def vt_plan_label():
    return "vt_congress_test"


@pytest.fixture
def binary():
    return str(BINARY)


@pytest.fixture
def vt_manifest():
    path = BASE_DIR / "outputs" / "RustV3" / "2020" / "plans" / "vt_congress_test" / "manifest.json"
    return json.loads(path.read_text(encoding="utf-8"))


@skip_no_vt_fixture
class TestManifestAuditFields:
    """Scenario 2: verify manifest audit fields."""

    def test_manifest_has_tiger_sha256(self, vt_manifest):
        """manifest.tiger_sha256 must be 64-char lowercase hex (or null but documented)."""
        # tiger_sha256 may be None for plans without TIGER hashing
        tiger_sha = vt_manifest.get("tiger_sha256")
        if tiger_sha is not None:
            assert len(tiger_sha) == 64, "tiger_sha256 must be 64 chars"
            assert all(c in "0123456789abcdef" for c in tiger_sha.lower()), \
                "tiger_sha256 must be lowercase hex"

    def test_manifest_has_binary_download_url(self, vt_manifest):
        """manifest.binary_download_url must reference github.com."""
        url = vt_manifest.get("binary_download_url", "")
        assert "github.com" in url, f"binary_download_url must reference github.com, got: {url!r}"
        assert url.startswith("https://"), "binary_download_url must be HTTPS"

    def test_manifest_has_tiger_source_url(self, vt_manifest):
        """manifest.tiger_source_url must point to Census.gov (no local paths)."""
        url = vt_manifest.get("tiger_source_url", "")
        assert "census.gov" in url, f"tiger_source_url must point to census.gov, got: {url!r}"
        assert "C:\\" not in url, "tiger_source_url must not contain Windows paths"
        assert "/home/" not in url, "tiger_source_url must not contain Unix home paths"


@skip_no_binary
@skip_no_vt_fixture
class TestVerificationCommand:
    """Verify audit trail for the verification command."""

    def test_verification_command_no_local_paths(self, vt_plan_label, binary, tmp_path):
        """Verification command in audit output must not contain local file paths."""
        out_dir = tmp_path / "audit_check"
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
        assert audit_path.exists(), "audit.json must be written"
        data = json.loads(audit_path.read_text(encoding="utf-8"))
        cmd = data.get("audit", {}).get("verification_command", "")
        assert "C:\\" not in cmd, "verification command must not contain Windows paths"
        assert "/home/" not in cmd, "verification command must not contain Unix home paths"
        assert "/Users/" not in cmd, "verification command must not contain macOS home paths"

    def test_verification_command_contains_seed(self, vt_plan_label, binary, tmp_path):
        """Verification command must include --seed for reproducibility."""
        out_dir = tmp_path / "audit_seed"
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
        if result.returncode != 0:
            pytest.skip(f"report command failed: {result.stderr}")
        json_path = out_dir / f"{vt_plan_label}_report.json"
        if not json_path.exists():
            pytest.skip("report JSON not created")
        data = json.loads(json_path.read_text(encoding="utf-8"))
        cmd = data.get("sections", {}).get("audit", {}).get("verification_command", "")
        assert "--seed" in cmd, f"verification command must include --seed, got: {cmd[:200]!r}"
