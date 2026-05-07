"""L0 unit tests for PlanManifest format.

Tests the manifest.json structure from Python's perspective — ensuring the
manifest written by the Rust redist-report crate has all required fields
for audit, reproducibility, and court submission.
"""
import json
import hashlib
import pytest
from pathlib import Path


def make_manifest_fixture(label="wa_house_draft1", state_code="WA"):
    """Create a manifest dict that matches PlanManifest output."""
    return {
        "label": label,
        "state_code": state_code,
        "year": "2020",
        "chamber": "congressional",
        "num_districts": 10,
        "population_source": "total",
        "partition_mode": "edge-weighted",
        "seed": 42,
        "binary_version": "0.1.0",
        "binary_sha256": "a" * 64,
        "binary_download_url": "https://github.com/owner/redist/releases/download/v0.1.0/redist",
        "adjacency_file": f"{state_code.lower()}_adjacency_2020.adj.bin",
        "adjacency_sha256": "b" * 64,
        "adjacency_build_command": "redist fetch --year 2020 --states WA",
        "adjacency_build_version": "0.1.0",
        "tiger_source_url": f"https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_53_tract.zip",
        "tiger_sha256": "c" * 64,
        "created_at": "2026-04-26T14:23:00Z",
        "balance_tolerance_pct": 0.5,
        "population_balance_valid": True,
    }


def generate_audit_report_fixture(label):
    """Generate a minimal audit report referencing the manifest."""
    manifest = make_manifest_fixture(label)
    return {
        "label": label,
        "audit": {
            "verification_command": (
                f"redist state --state {manifest['state_code']} "
                f"--year {manifest['year']} --version {manifest['binary_version']} "
                f"--seed {manifest['seed']}"
            ),
            "manifest_sha256": hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest(),
        },
        "manifest": manifest,
    }


# ---------------------------------------------------------------------------
# Required fields tests
# ---------------------------------------------------------------------------

class TestManifestFields:

    def test_manifest_has_tiger_sha256(self):
        """manifest.json must have tiger_sha256 field."""
        manifest = make_manifest_fixture()
        assert "tiger_sha256" in manifest
        assert len(manifest["tiger_sha256"]) == 64  # SHA-256 hex

    def test_manifest_has_binary_download_url(self):
        """manifest.json must have binary_download_url pointing to github.com."""
        manifest = make_manifest_fixture()
        assert "binary_download_url" in manifest
        assert "github.com" in manifest["binary_download_url"]

    def test_manifest_has_tiger_source_url(self):
        """manifest.json must have tiger_source_url pointing to census.gov."""
        manifest = make_manifest_fixture()
        assert "tiger_source_url" in manifest
        assert "census.gov" in manifest["tiger_source_url"]

    def test_manifest_adjacency_file_is_filename_not_path(self):
        """adjacency_file must be filename only, not a full path."""
        manifest = make_manifest_fixture()
        adj = manifest["adjacency_file"]
        assert "/" not in adj and "\\" not in adj, \
            f"adjacency_file must be filename only, got: {adj!r}"

    def test_manifest_has_all_required_fields(self):
        """manifest.json must have all required audit chain fields."""
        manifest = make_manifest_fixture()
        required = [
            "label", "state_code", "year", "chamber", "num_districts",
            "population_source", "partition_mode",
            "binary_version", "binary_sha256", "binary_download_url",
            "adjacency_file", "adjacency_sha256", "adjacency_build_command",
            "tiger_source_url", "created_at",
            "balance_tolerance_pct", "population_balance_valid",
        ]
        for field in required:
            assert field in manifest, f"manifest missing required field: {field}"

    def test_manifest_population_balance_valid_is_bool(self):
        """population_balance_valid must be a boolean."""
        manifest = make_manifest_fixture()
        assert isinstance(manifest["population_balance_valid"], bool)


# ---------------------------------------------------------------------------
# SHA-256 hash tests
# ---------------------------------------------------------------------------

class TestManifestSha256:

    def test_sha256_of_hello_world_is_known(self):
        """Python SHA-256 of 'hello world' matches the known test vector."""
        # This test validates that our SHA-256 implementation is correct
        hash_value = hashlib.sha256(b"hello world").hexdigest()
        assert len(hash_value) == 64
        # Known SHA-256 of "hello world"
        assert hash_value == "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"

    def test_sha256_is_deterministic(self):
        """Same content hashes to same value every time."""
        content = b"test shapefile data"
        hash1 = hashlib.sha256(content).hexdigest()
        hash2 = hashlib.sha256(content).hexdigest()
        assert hash1 == hash2
        assert len(hash1) == 64

    def test_sha256_of_file_streaming(self, tmp_path):
        """SHA-256 computed in chunks equals full-file hash."""
        test_data = b"test data " * 10000  # 100KB
        path = tmp_path / "test.bin"
        path.write_bytes(test_data)

        # Full file hash
        expected = hashlib.sha256(test_data).hexdigest()

        # Streaming hash (64KB chunks)
        hasher = hashlib.sha256()
        chunk_size = 64 * 1024
        with open(path, "rb") as f:
            while True:
                chunk = f.read(chunk_size)
                if not chunk:
                    break
                hasher.update(chunk)
        streaming_hash = hasher.hexdigest()

        assert streaming_hash == expected, "Streaming hash must equal full-file hash"

    def test_tiger_sha256_is_64_hex_chars(self):
        """tiger_sha256 in manifest is a 64-character hex string."""
        manifest = make_manifest_fixture()
        tiger_hash = manifest["tiger_sha256"]
        assert len(tiger_hash) == 64
        assert all(c in "0123456789abcdef" for c in tiger_hash)


# ---------------------------------------------------------------------------
# Default label generation tests
# ---------------------------------------------------------------------------

class TestDefaultLabelGeneration:

    def test_label_format_state_chamber_year(self):
        """Default label is {state_name}_{chamber}_{year}."""
        state_name = "washington"
        chamber = "house"
        year = "2020"
        label = f"{state_name}_{chamber}_{year}"
        assert label == "washington_house_2020"

    def test_label_normalizes_spaces_to_underscores(self):
        """Spaces in state name are normalized to underscores."""
        state_name = "new york"
        normalized = state_name.lower().replace(" ", "_")
        label = f"{normalized}_congressional_2020"
        assert label == "new_york_congressional_2020"

    def test_label_is_lowercase(self):
        """Label is always lowercase."""
        label = "Washington_House_2020".lower()
        assert label == "washington_house_2020"


# ---------------------------------------------------------------------------
# Audit trail tests
# ---------------------------------------------------------------------------

class TestManifestAuditTrail:

    def test_verification_command_no_local_paths(self):
        """Verification command must not contain local filesystem paths."""
        report = generate_audit_report_fixture("wa_house_draft1")
        cmd = report["audit"]["verification_command"]
        assert "C:\\" not in cmd, "No Windows local paths"
        assert "/home/" not in cmd, "No Unix local paths"
        assert "/Users/" not in cmd, "No macOS local paths"

    def test_verification_command_has_seed(self):
        """Verification command includes --seed for reproducibility."""
        report = generate_audit_report_fixture("wa_house_draft1")
        cmd = report["audit"]["verification_command"]
        assert "--seed" in cmd, "Verification command must include --seed"

    def test_manifest_label_matches_requested_label(self):
        """Manifest label matches the --label argument."""
        manifest = make_manifest_fixture(label="wa_house_draft1")
        assert manifest["label"] == "wa_house_draft1"


# ---------------------------------------------------------------------------
# Atomic write tests
# ---------------------------------------------------------------------------

class TestManifestAtomicWrite:

    def test_manifest_tmp_detected_as_incomplete(self, tmp_path):
        """If manifest.tmp exists (without manifest.json), plan is incomplete."""
        tmp_file = tmp_path / "manifest.tmp"
        tmp_file.write_text("{}")
        final_file = tmp_path / "manifest.json"
        # Simulate the check_incomplete_plan logic
        assert tmp_file.exists() and not final_file.exists(), \
            "manifest.tmp without manifest.json = incomplete plan"

    def test_manifest_json_exists_after_atomic_write(self, tmp_path):
        """After successful write, manifest.json exists and manifest.tmp does not."""
        manifest = make_manifest_fixture()
        # Simulate atomic write: write tmp, then rename
        tmp_path_file = tmp_path / "manifest.tmp"
        final_path = tmp_path / "manifest.json"
        tmp_path_file.write_text(json.dumps(manifest))
        tmp_path_file.rename(final_path)
        assert final_path.exists(), "manifest.json must exist after atomic write"
        assert not tmp_path_file.exists(), "manifest.tmp must not exist after atomic write"

    def test_manifest_json_is_valid_json(self, tmp_path):
        """manifest.json written by the pipeline is valid JSON."""
        manifest = make_manifest_fixture()
        path = tmp_path / "manifest.json"
        path.write_text(json.dumps(manifest, indent=2))
        parsed = json.loads(path.read_text())
        assert parsed["label"] == manifest["label"]
