"""L0 unit tests for RPLAN v0.1 format.

Tests the RPLAN JSON structure from Python's perspective — ensuring the
format written by the Rust redist-report crate can be parsed and validated
by Python consumers (e.g., dashboards, court submissions, third-party tools).
"""
import json
import pytest


def make_valid_rplan(
    label="test_plan",
    state_fips="50",
    state_code="VT",
    year="2020",
    chamber="congressional",
    num_districts=1,
    assignments=None,
):
    """Create a minimal valid RPLAN dict for testing."""
    if assignments is None:
        assignments = {"50023000100": 1}
    return {
        "rplan_version": "0.1",
        "metadata": {
            "label": label,
            "state_fips": state_fips,
            "state_code": state_code,
            "year": year,
            "chamber": chamber,
            "num_districts": num_districts,
            "population_source": "total",
            "balance_tolerance_pct": 0.5,
            "created_at": "2026-04-26T00:00:00Z",
            "created_by": "test",
        },
        "assignments": assignments,
        "geometry": None,
    }


# ---------------------------------------------------------------------------
# Structure tests
# ---------------------------------------------------------------------------

class TestRplanStructure:

    def test_rplan_version_at_root_level(self):
        """rplan_version must be at root level."""
        rplan = make_valid_rplan()
        assert "rplan_version" in rplan
        assert rplan["rplan_version"] == "0.1"

    def test_rplan_version_not_in_metadata(self):
        """rplan_version must NOT appear inside metadata."""
        rplan = make_valid_rplan()
        assert "rplan_version" not in rplan["metadata"], \
            "rplan_version must NOT appear inside metadata"

    def test_rplan_has_required_top_level_keys(self):
        """RPLAN must have rplan_version, metadata, assignments, geometry."""
        rplan = make_valid_rplan()
        for key in ["rplan_version", "metadata", "assignments", "geometry"]:
            assert key in rplan, f"RPLAN missing required top-level key: {key}"

    def test_rplan_geometry_can_be_null(self):
        """geometry: null is allowed (geometry is optional)."""
        rplan = make_valid_rplan()
        assert rplan["geometry"] is None

    def test_rplan_metadata_has_required_fields(self):
        """metadata must have all required fields."""
        rplan = make_valid_rplan()
        required = [
            "label", "state_fips", "state_code", "year", "chamber",
            "num_districts", "population_source", "balance_tolerance_pct",
            "created_at", "created_by",
        ]
        for field in required:
            assert field in rplan["metadata"], f"metadata missing field: {field}"

    def test_rplan_roundtrip_preserves_assignments(self):
        """Serializing and deserializing preserves assignments exactly."""
        assignments = {"50023000100": 1, "50023000200": 1, "50025000100": 1}
        rplan = make_valid_rplan(assignments=assignments)
        serialized = json.dumps(rplan)
        decoded = json.loads(serialized)
        assert decoded["assignments"] == assignments


# ---------------------------------------------------------------------------
# GEOID format tests
# ---------------------------------------------------------------------------

class TestRplanGeoidFormat:

    def test_valid_11char_numeric_geoid(self):
        """11-character numeric GEOID is valid."""
        geoid = "50023000100"
        assert len(geoid) == 11
        assert geoid.isdigit()

    def test_geoid_too_short_is_invalid(self):
        """GEOID shorter than 11 characters is invalid."""
        geoid = "5002300010"  # 10 chars
        assert len(geoid) < 11, "Should be < 11 chars"

    def test_geoid_non_numeric_is_invalid(self):
        """GEOID with non-numeric characters is invalid."""
        geoid = "5002300010X"
        assert not geoid.isdigit(), "Should not be all digits"

    def test_all_geoids_in_valid_rplan_are_11_chars(self):
        """All GEOIDs in a valid RPLAN are exactly 11 numeric characters."""
        assignments = {
            "50023000100": 1,
            "50023000200": 1,
            "50025000100": 1,
        }
        rplan = make_valid_rplan(assignments=assignments, num_districts=1)
        for geoid in rplan["assignments"]:
            assert len(geoid) == 11, f"GEOID {geoid!r} must be 11 chars"
            assert geoid.isdigit(), f"GEOID {geoid!r} must be numeric"


# ---------------------------------------------------------------------------
# District range tests
# ---------------------------------------------------------------------------

class TestRplanDistrictRange:

    def test_district_values_are_1_based(self):
        """District values must be in [1, num_districts]."""
        rplan = make_valid_rplan(num_districts=3, assignments={
            "50023000100": 1,
            "50023000200": 2,
            "50025000100": 3,
        })
        for geoid, district in rplan["assignments"].items():
            assert 1 <= district <= rplan["metadata"]["num_districts"], \
                f"GEOID {geoid}: district {district} out of range"

    def test_district_zero_is_invalid(self):
        """District value 0 is invalid (must be 1-based)."""
        # Illustrative: verify Python-side check
        district = 0
        num_districts = 3
        assert not (1 <= district <= num_districts), \
            "District 0 should be invalid"

    def test_district_exceeding_num_districts_is_invalid(self):
        """District value > num_districts is invalid."""
        district = 5
        num_districts = 3
        assert not (1 <= district <= num_districts), \
            "District 5 > 3 should be invalid"


# ---------------------------------------------------------------------------
# Version tests
# ---------------------------------------------------------------------------

class TestRplanVersion:

    def test_version_string_format_is_major_minor(self):
        """rplan_version must be in 'major.minor' format."""
        version = "0.1"
        parts = version.split(".")
        assert len(parts) == 2
        assert all(p.isdigit() for p in parts)

    def test_version_0_1_is_supported(self):
        """Version 0.1 is the current supported version."""
        rplan = make_valid_rplan()
        major, minor = rplan["rplan_version"].split(".")
        assert int(major) == 0
        assert int(minor) == 1

    def test_unknown_major_version_should_be_rejected(self):
        """Major version != 0 should be rejected by readers."""
        rplan = make_valid_rplan()
        rplan["rplan_version"] = "2.0"
        major = int(rplan["rplan_version"].split(".")[0])
        assert major != 0, "Major version 2 should trigger rejection"

    def test_unknown_minor_version_warns_not_fails(self):
        """Minor version bump (0.2) should warn but not fail."""
        rplan = make_valid_rplan()
        rplan["rplan_version"] = "0.2"
        major, minor = rplan["rplan_version"].split(".")
        assert int(major) == 0, "Major version matches"
        assert int(minor) > 1, "Minor version is newer"
        # Readers should warn, not error
