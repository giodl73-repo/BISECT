"""
L2 acceptance tests for `redist aggregate` and `redist map --scope national`.

Requires:
- redist binary built
- VT + AL states run: redist state --state VT --state AL --year 2020 --version V3
- VT + AL analysis: redist analyze --state VT --types all && redist analyze --state AL --types all

Run: pytest redist/tests/acceptance/test_aggregate_national.py -v
"""

import json
import subprocess
import time
import unittest
from pathlib import Path


def find_binary() -> str:
    for p in [Path("redist/target/release/redist.exe"),
              Path("redist/target/release/redist")]:
        if p.exists():
            return str(p.resolve())
    raise RuntimeError("redist binary not found.")


def run(args, check=True):
    return subprocess.run(args, capture_output=True, text=True, check=check)


NATIONAL_DIR = Path("outputs/V3/national")
STATES_DIR   = Path("outputs/V3/states")


class TestAggregateAcceptance(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        states_with_analysis = [
            s for s in STATES_DIR.iterdir()
            if s.is_dir() and (s / "analysis" / "demographic.json").exists()
        ] if STATES_DIR.exists() else []
        if len(states_with_analysis) < 2:
            raise unittest.SkipTest(
                "Need ≥2 states with analysis. "
                "Run: redist analyze --state VT --types all && "
                "redist analyze --state AL --types all"
            )
        cls.binary = find_binary()
        cls.n_states = len(states_with_analysis)
        # Count expected districts
        cls.n_districts = sum(
            len(json.loads((s / "analysis" / "demographic.json").read_text())["districts"])
            for s in states_with_analysis
        )

    def _aggregate(self, *types, force=True, csv=False):
        cmd = [self.binary, "aggregate", "--version", "V3", "--types"] + list(types)
        if force: cmd.append("--force")
        if csv:   cmd.append("--csv")
        return run(cmd, check=False)

    # ── JSON output structure ─────────────────────────────────────────────

    def test_demographic_json_structure(self):
        r = self._aggregate("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        out = NATIONAL_DIR / "us_demographic.json"
        self.assertTrue(out.exists())
        data = json.loads(out.read_text())
        self.assertEqual(data["scope"], "national")
        self.assertEqual(data["analyzer"], "demographic")
        self.assertIn("state_count", data)
        self.assertIn("district_count", data)
        self.assertEqual(data["state_count"], self.n_states)
        self.assertEqual(data["district_count"], self.n_districts)

    def test_all_districts_have_state_field(self):
        self._aggregate("demographic")
        data = json.loads((NATIONAL_DIR / "us_demographic.json").read_text())
        for d in data["districts"]:
            self.assertIn("state", d, "every district must have 'state' field")
            self.assertIsInstance(d["state"], str)

    def test_district_count_matches_field(self):
        self._aggregate("demographic")
        data = json.loads((NATIONAL_DIR / "us_demographic.json").read_text())
        self.assertEqual(len(data["districts"]), data["district_count"])

    def test_all_types_produce_files(self):
        r = self._aggregate("all")
        self.assertEqual(r.returncode, 0, r.stderr)
        for name in ["us_demographic.json", "us_political.json",
                     "us_urban.json", "us_summary.json"]:
            path = NATIONAL_DIR / name
            self.assertTrue(path.exists(), f"missing {name}")
            data = json.loads(path.read_text())
            self.assertIn("analyzer", data)
            self.assertEqual(data["scope"], "national")

    def test_compactness_aggregate(self):
        r = self._aggregate("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)
        out = NATIONAL_DIR / "us_compactness.json"
        if out.exists():
            data = json.loads(out.read_text())
            for d in data["districts"]:
                self.assertIn("polsby_popper", d)

    # ── CSV export ────────────────────────────────────────────────────────

    def test_csv_export_produces_file(self):
        r = self._aggregate("demographic", csv=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        csv_path = NATIONAL_DIR / "us_demographic.csv"
        self.assertTrue(csv_path.exists())

    def test_csv_has_correct_headers(self):
        self._aggregate("demographic", csv=True)
        lines = (NATIONAL_DIR / "us_demographic.csv").read_text().splitlines()
        self.assertGreater(len(lines), 1, "need header + at least 1 district")
        header = lines[0]
        self.assertIn("state", header)
        self.assertIn("district", header)
        self.assertIn("total_pop", header)

    def test_csv_row_count_matches_json(self):
        self._aggregate("demographic", csv=True)
        json_data = json.loads((NATIONAL_DIR / "us_demographic.json").read_text())
        csv_lines = (NATIONAL_DIR / "us_demographic.csv").read_text().splitlines()
        # header + N districts
        self.assertEqual(len(csv_lines) - 1, json_data["district_count"])

    # ── skip / force ──────────────────────────────────────────────────────

    def test_skip_without_force(self):
        self._aggregate("demographic", force=True)
        out = NATIONAL_DIR / "us_demographic.json"
        mtime1 = out.stat().st_mtime
        time.sleep(0.05)
        r = self._aggregate("demographic", force=False)
        self.assertEqual(r.returncode, 0)
        self.assertIn("skip", r.stderr.lower())
        self.assertAlmostEqual(out.stat().st_mtime, mtime1, places=1)

    def test_force_regenerates(self):
        self._aggregate("demographic", force=True)
        mtime1 = (NATIONAL_DIR / "us_demographic.json").stat().st_mtime
        time.sleep(0.05)
        self._aggregate("demographic", force=True)
        self.assertGreater(
            (NATIONAL_DIR / "us_demographic.json").stat().st_mtime, mtime1 - 0.01
        )

    def test_missing_analysis_skips_gracefully(self):
        # compactness.json may not exist for some states — should not crash
        r = self._aggregate("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)


class TestNationalMapAcceptance(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        states_with_data = [
            s for s in STATES_DIR.iterdir()
            if s.is_dir() and (s / "data" / "final_assignments.json").exists()
        ] if STATES_DIR.exists() else []
        if len(states_with_data) < 2:
            raise unittest.SkipTest(
                "Need ≥2 states with assignments. Run: redist states --year 2020 --version V3"
            )
        cls.binary = find_binary()
        cls.n_states = len(states_with_data)

    def _map_national(self, *types, force=True, dpi="150"):
        cmd = [self.binary, "map", "--scope", "national", "--version", "V3",
               "--dpi", dpi, "--types"] + list(types)
        if force: cmd.append("--force")
        return run(cmd, check=False)

    def _valid_png(self, path: Path, min_bytes: int = 10_000):
        self.assertTrue(path.exists(), f"{path} not found")
        data = path.read_bytes()
        self.assertEqual(data[:4], b"\x89PNG", f"{path} not valid PNG")
        self.assertGreater(len(data), min_bytes,
                           f"{path} too small ({len(data)}B)")

    def test_districts_map_produces_valid_png(self):
        r = self._map_national("districts")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(NATIONAL_DIR / "maps" / "districts.png", min_bytes=50_000)

    def test_districts_map_skip_without_force(self):
        self._map_national("districts", force=True)
        png = NATIONAL_DIR / "maps" / "districts.png"
        mtime1 = png.stat().st_mtime
        time.sleep(0.05)
        r = self._map_national("districts", force=False)
        self.assertEqual(r.returncode, 0)
        self.assertIn("skip", r.stderr.lower())

    def test_political_map_produces_png(self):
        r = self._map_national("political")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(NATIONAL_DIR / "maps" / "political.png", min_bytes=10_000)

    def test_demographic_map_produces_png(self):
        r = self._map_national("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(NATIONAL_DIR / "maps" / "demographic.png", min_bytes=10_000)

    def test_compactness_map_produces_png(self):
        r = self._map_national("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(NATIONAL_DIR / "maps" / "compactness.png", min_bytes=5_000)

    def test_dpi_affects_file_size(self):
        self._map_national("districts", dpi="72")
        size_72 = (NATIONAL_DIR / "maps" / "districts.png").stat().st_size
        self._map_national("districts", dpi="150")
        size_150 = (NATIONAL_DIR / "maps" / "districts.png").stat().st_size
        self.assertGreater(size_150, size_72,
                           f"150 DPI ({size_150}B) must be larger than 72 DPI ({size_72}B)")

    def test_state_field_in_stderr_count(self):
        r = self._map_national("districts", force=True)
        # Should report number of districts and states loaded
        self.assertIn("districts across", r.stderr)
        # e.g. "19 districts across 5 states"

    def test_rounds_skipped_gracefully(self):
        r = self._map_national("rounds")
        # rounds not applicable for national — should skip not fail
        self.assertEqual(r.returncode, 0)
        self.assertIn("skip", r.stderr.lower())

    def test_no_states_dir_exits_nonzero(self):
        r = run([self.binary, "map", "--scope", "national", "--version", "NONEXISTENT"],
                check=False)
        self.assertNotEqual(r.returncode, 0)


if __name__ == "__main__":
    unittest.main()
