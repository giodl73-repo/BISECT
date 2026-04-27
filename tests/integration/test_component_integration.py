"""
L1 integration tests — component integration with synthetic data.

No real census data required. Tests interactions between data structures,
policy configuration, and algorithmic formulas using only pure Python
and the project's committed data files (state_policy.json).

Test groups:
  1. TestRplanRoundTrip      — RPLAN format contract validation
  2. TestStatePolicy         — state_policy.json structure and vocabulary
  3. TestGranularityFloor    — AP-08 granularity floor constraint maths

Run with:
    pytest tests/integration/test_component_integration.py -v --tb=short
"""

import json
import tempfile
import unittest
from pathlib import Path


# ---------------------------------------------------------------------------
# 1. RPLAN format round-trip
# ---------------------------------------------------------------------------


class TestRplanRoundTrip(unittest.TestCase):
    """RPLAN v0.1 format: write → read preserves all fields.

    These tests validate the RPLAN JSON contract without touching the Rust
    library — they verify the spec itself so that any future implementation
    can be checked against these invariants.
    """

    def _make_rplan(self, assignments=None, metadata=None):
        if assignments is None:
            assignments = {"50001020100": 1, "50001020200": 1, "50001020300": 1}
        if metadata is None:
            metadata = {
                "label": "test_plan",
                "state_fips": "50",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-01-01T00:00:00Z",
                "created_by": "test",
            }
        return {
            "rplan_version": "0.1",
            "metadata": metadata,
            "assignments": assignments,
            "geometry": None,
        }

    def test_write_read_preserves_assignments(self):
        rplan = self._make_rplan()
        with tempfile.NamedTemporaryFile(suffix=".rplan", mode="w", delete=False) as f:
            json.dump(rplan, f)
            path = f.name
        loaded = json.loads(Path(path).read_text(encoding="utf-8"))
        self.assertEqual(loaded["assignments"], rplan["assignments"])

    def test_write_read_preserves_metadata_chamber(self):
        rplan = self._make_rplan()
        with tempfile.NamedTemporaryFile(suffix=".rplan", mode="w", delete=False) as f:
            json.dump(rplan, f)
            path = f.name
        loaded = json.loads(Path(path).read_text(encoding="utf-8"))
        self.assertEqual(loaded["metadata"]["chamber"], "congressional")

    def test_rplan_version_at_top_level(self):
        rplan = self._make_rplan()
        self.assertIn("rplan_version", rplan)
        self.assertEqual(rplan["rplan_version"], "0.1")

    def test_rplan_version_not_in_metadata(self):
        """R3 amendment: rplan_version must be at top level only, not inside metadata."""
        rplan = self._make_rplan()
        self.assertNotIn("rplan_version", rplan["metadata"],
                         "rplan_version must NOT appear inside metadata (R3 amendment)")

    def test_geometry_is_none_when_not_embedded(self):
        rplan = self._make_rplan()
        self.assertIsNone(rplan["geometry"])

    def test_geoid_format_11_chars(self):
        """Census tract GEOIDs must be exactly 11 characters."""
        assignments = {
            "50001020100": 1,  # VT tract — 2-char state + 3-char county + 6-char tract
            "50001020200": 1,
        }
        for geoid in assignments:
            self.assertEqual(len(geoid), 11, f"GEOID {geoid!r} must be 11 chars (census tract)")

    def test_district_ids_are_1_based(self):
        """District IDs in assignments must start at 1 (not 0)."""
        assignments = {"50001020100": 1, "50001020200": 2, "50001020300": 3}
        values = list(assignments.values())
        self.assertTrue(all(v >= 1 for v in values),
                        "All district IDs must be >= 1 (1-based)")

    def test_district_ids_dont_exceed_num_districts(self):
        num_districts = 2
        assignments = {"50001020100": 1, "50001020200": 2, "50001020300": 2}
        for geoid, district in assignments.items():
            self.assertLessEqual(district, num_districts,
                                 f"District {district} exceeds num_districts={num_districts}")

    def test_metadata_required_fields_present(self):
        required_metadata_fields = [
            "label", "state_fips", "state_code", "year",
            "chamber", "num_districts", "population_source",
            "balance_tolerance_pct", "created_at",
        ]
        rplan = self._make_rplan()
        for field in required_metadata_fields:
            self.assertIn(field, rplan["metadata"],
                          f"RPLAN metadata must include '{field}'")

    def test_empty_assignments_still_valid_rplan(self):
        """RPLAN with no assignments is a valid (but empty) plan."""
        rplan = self._make_rplan(assignments={})
        self.assertEqual(len(rplan["assignments"]), 0)
        self.assertEqual(rplan["rplan_version"], "0.1")

    def test_rplan_json_serialisable(self):
        """RPLAN must round-trip through JSON without error."""
        rplan = self._make_rplan()
        serialised = json.dumps(rplan)
        deserialised = json.loads(serialised)
        self.assertEqual(deserialised["rplan_version"], "0.1")
        self.assertEqual(len(deserialised["assignments"]), 3)


# ---------------------------------------------------------------------------
# 2. State policy JSON structure and vocabulary
# ---------------------------------------------------------------------------


def _load_policy():
    """Load state_policy.json from the committed data file."""
    policy_path = Path("redist/data/state_policy.json")
    if not policy_path.exists():
        return None
    return json.loads(policy_path.read_text(encoding="utf-8"))


_POLICY = _load_policy()
_POLICY_MISSING = _POLICY is None


@unittest.skipIf(_POLICY_MISSING, "state_policy.json not found at redist/data/state_policy.json")
class TestStatePolicy(unittest.TestCase):
    """State policy JSON: vocabulary, required fields, and special-state contracts."""

    @classmethod
    def setUpClass(cls):
        cls.policy = _POLICY

    def test_all_50_states_present(self):
        all_codes = [
            "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
            "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
            "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NJ", "NM",
            "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD",
            "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
        ]
        for code in all_codes:
            self.assertIn(code, self.policy,
                          f"State {code} missing from state_policy.json")

    def test_la_subdivision_is_parish(self):
        la = self.policy["LA"]
        self.assertEqual(la["subdivision_term"], "parish",
                         "Louisiana subdivision_term must be 'parish'")
        self.assertEqual(la["subdivision_term_plural"], "parishes",
                         "Louisiana subdivision_term_plural must be 'parishes'")

    def test_ct_subdivision_is_town(self):
        ct = self.policy["CT"]
        self.assertEqual(ct["subdivision_term"], "town",
                         "Connecticut subdivision_term must be 'town'")

    def test_ak_subdivision_is_borough(self):
        ak = self.policy["AK"]
        self.assertEqual(ak["subdivision_term"], "borough",
                         "Alaska subdivision_term must be 'borough'")

    def test_va_has_independent_city_note(self):
        va = self.policy["VA"]
        self.assertIn("independent_city_note", va,
                      "Virginia must have independent_city_note field")
        self.assertIn("independent_cities", va,
                      "Virginia must have independent_cities list")
        self.assertIsInstance(va["independent_cities"], list)
        # VA has 38 independent cities; at least 30 must be listed
        self.assertGreater(len(va["independent_cities"]), 30,
                           f"VA must list 30+ independent cities, got {len(va['independent_cities'])}")

    def test_eldoria_test_state_accessible(self):
        """_TEST_EL (Eldoria) must be present with all unusual values intact."""
        self.assertIn("_TEST_EL", self.policy,
                      "Eldoria test state (_TEST_EL) must be in state_policy.json")
        el = self.policy["_TEST_EL"]
        self.assertEqual(el["subdivision_term"], "realm",
                         "Eldoria subdivision_term must be 'realm'")
        self.assertEqual(el["commission_type"], "grand_council_of_wizards")
        self.assertAlmostEqual(el["balance_tolerance_house_pct"], 7.5,
                               msg="Eldoria house tolerance must be 7.5")

    def test_eldoria_population_basis_is_vap(self):
        el = self.policy["_TEST_EL"]
        self.assertEqual(el["population_basis"], "vap",
                         "Eldoria must use 'vap' population basis")

    def test_ne_is_unicameral(self):
        ne = self.policy["NE"]
        self.assertEqual(ne["senate_districts"], 0,
                         "Nebraska unicameral: senate_districts must be 0")
        notes = ne.get("notes", "").lower()
        self.assertIn("unicameral", notes,
                      "Nebraska notes must mention 'unicameral'")

    def test_hi_permanent_resident_basis(self):
        hi = self.policy["HI"]
        self.assertEqual(hi["population_basis"], "permanent_resident",
                         "Hawaii must use 'permanent_resident' population basis")

    def test_nj_student_counting_college_address(self):
        nj = self.policy["NJ"]
        self.assertEqual(nj["student_counting"], "college_address",
                         "NJ must use 'college_address' student counting")

    def test_wa_nesting_ratio_2to1(self):
        wa = self.policy["WA"]
        self.assertEqual(wa.get("nesting_ratio"), "2:1",
                         "Washington must require 2:1 senate-in-house nesting")

    def test_all_states_have_required_fields(self):
        required = [
            "name", "fips", "house_districts", "senate_districts",
            "subdivision_term", "balance_tolerance_house_pct",
            "population_basis", "has_independent_commission",
        ]
        for code, state in self.policy.items():
            if code.startswith("_"):
                continue  # skip _version, _description, _TEST_EL etc
            for field in required:
                self.assertIn(field, state,
                              f"State {code} missing required field: '{field}'")

    def test_congressional_balance_tolerance_is_half_percent(self):
        """All US states (fips not null) must have 0.5% tolerance for congressional districts."""
        for code, state in self.policy.items():
            if code.startswith("_"):
                continue
            if state.get("fips") is None:
                continue  # Skip non-US/parliamentary entries
            tol = state.get("balance_tolerance_congressional_pct")
            if tol is not None:
                self.assertAlmostEqual(
                    tol, 0.5,
                    msg=f"{code} congressional tolerance must be 0.5%, got {tol}",
                )

    def test_fips_codes_are_zero_padded_strings(self):
        """FIPS codes must be strings (zero-padded, e.g. '01' not 1)."""
        for code, state in self.policy.items():
            if code.startswith("_"):
                continue
            fips = state.get("fips")
            if fips is not None:
                self.assertIsInstance(fips, str,
                                      f"{code} fips must be a string (got {type(fips).__name__})")
                self.assertGreaterEqual(len(fips), 2,
                                        f"{code} fips must be at least 2 chars: {fips!r}")

    def test_has_independent_commission_is_bool(self):
        """has_independent_commission must always be a boolean."""
        for code, state in self.policy.items():
            if code.startswith("_"):
                continue
            val = state.get("has_independent_commission")
            if val is not None:
                self.assertIsInstance(val, bool,
                                      f"{code} has_independent_commission must be bool, got {type(val).__name__}")

    def test_ireland_is_stv_multi_member(self):
        ie = self.policy.get("IE")
        self.assertIsNotNone(ie)
        self.assertEqual(ie["electoral_system"], "single_transferable_vote")
        self.assertEqual(ie["total_seats"], 174)

    def test_uk_registered_electors(self):
        uk = self.policy.get("GB-ENG")
        self.assertIsNotNone(uk)
        self.assertEqual(uk["population_basis"], "registered_electors")

    def test_germany_wahlkreis_vocabulary(self):
        de = self.policy.get("DE-WAHLKREIS")
        self.assertIsNotNone(de)
        self.assertEqual(de["subdivision_term"], "Wahlkreis")

    def test_canada_ridings(self):
        ca = self.policy.get("CA-PROV")
        self.assertIsNotNone(ca)
        self.assertEqual(ca["subdivision_term"], "riding")

    def test_malta_5_seats(self):
        mt = self.policy.get("MT-PARLIAMENT")
        self.assertIsNotNone(mt)
        self.assertEqual(mt["seats_per_district"], 5)


# ---------------------------------------------------------------------------
# 3. Granularity floor constraint maths (AP-08)
# ---------------------------------------------------------------------------


class TestGranularityFloor(unittest.TestCase):
    """AP-08 granularity floor: verify the maths for the resolution adequacy check.

    The granularity floor rule: a redistricting run at a given geographic
    resolution is adequate only if there are enough units per district that
    the algorithm has meaningful choices. The threshold used by the pipeline
    is >= 20 units/district.

    These tests use 2020 Census counts (tract and block-group counts are
    fixed public knowledge — not pipeline outputs).
    """

    _THRESHOLD = 20.0  # pipeline minimum: 20 tracts per district

    def _units_per_district(self, total_units: int, num_districts: int) -> float:
        return total_units / num_districts

    # Washington state — the motivating case for AP-08

    def test_wa_house_tract_below_threshold(self):
        """WA 98HD at tract resolution: ~18 tracts/district, below threshold."""
        upd = self._units_per_district(1784, 98)  # WA tract count 2020
        self.assertLess(
            upd, self._THRESHOLD,
            f"WA 98HD at tract: {upd:.1f}/district must be below {self._THRESHOLD} threshold",
        )

    def test_wa_house_block_group_above_threshold(self):
        """WA 98HD at block_group resolution: ~54 block groups/district, above threshold."""
        upd = self._units_per_district(5311, 98)  # WA block-group count 2020
        self.assertGreater(
            upd, self._THRESHOLD,
            f"WA 98HD at block_group: {upd:.1f}/district must be above {self._THRESHOLD} threshold",
        )

    def test_wa_congressional_tract_well_above_threshold(self):
        """WA 10CD at tract resolution: ~178 tracts/district, well above threshold."""
        upd = self._units_per_district(1784, 10)
        self.assertGreater(
            upd, 100.0,
            f"WA 10CD at tract: {upd:.1f}/district should be well above threshold",
        )

    # Texas — large state, adequate at tract level even for 150HD

    def test_tx_house_tract_adequate(self):
        """TX 150HD at tract: ~46 tracts/district, adequate."""
        upd = self._units_per_district(6896, 150)  # TX tract count 2020
        self.assertGreater(
            upd, self._THRESHOLD,
            f"TX 150HD at tract: {upd:.1f}/district must be above threshold",
        )

    # Vermont — 1 congressional district, always adequate

    def test_vt_congressional_tract_adequate(self):
        """VT 1CD at tract: 193 tracts/1 district, obviously adequate."""
        upd = self._units_per_district(193, 1)
        self.assertGreater(upd, self._THRESHOLD)
        self.assertAlmostEqual(upd, 193.0)

    # Impact formula: how much does one unit swap affect balance?

    def test_wa_house_tract_swap_impact_formula(self):
        """One tract swap at WA 98HD = ~5.5% of ideal district population.

        This makes a 5% tolerance very hard to guarantee — motivating block-group use.
        """
        total_pop = 7_705_281
        num_tracts = 1784
        num_districts = 98
        avg_tract_pop = total_pop / num_tracts
        ideal_district_pop = total_pop / num_districts
        impact_pct = avg_tract_pop / ideal_district_pop * 100
        # Should be ~5.5% — between 4% and 10%
        self.assertGreater(
            impact_pct, 4.0,
            f"WA tract swap impact {impact_pct:.1f}% should be >4%",
        )
        self.assertLess(
            impact_pct, 10.0,
            f"WA tract swap impact {impact_pct:.1f}% should be <10%",
        )

    def test_threshold_is_positive(self):
        self.assertGreater(self._THRESHOLD, 0)

    def test_upd_formula_is_monotone_in_units(self):
        """More units at fixed district count → higher units/district."""
        low = self._units_per_district(100, 10)
        high = self._units_per_district(200, 10)
        self.assertLess(low, high)

    def test_upd_formula_is_monotone_decreasing_in_districts(self):
        """More districts at fixed unit count → lower units/district."""
        few = self._units_per_district(1000, 5)
        many = self._units_per_district(1000, 50)
        self.assertGreater(few, many)

    def test_granularity_floor_boundary_condition(self):
        """Exactly 20 units/district is at the boundary — must not be less than threshold."""
        upd = self._units_per_district(20, 1)
        # Strictly: >= 20 passes; < 20 fails. 20.0 is on the boundary.
        self.assertGreaterEqual(upd, self._THRESHOLD)


if __name__ == "__main__":
    unittest.main()
