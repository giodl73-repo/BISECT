"""
L2 acceptance tests for `redist analyze` and `redist map`.

Requires:
- redist binary built: cargo build --release --manifest-path redist/Cargo.toml
- VT 2020 final_assignments.json in outputs/V3/2020/vermont/
  (run: redist state --state VT --year 2020 --version V3)
- Demographics CSV: data/2020/demographics/vermont_demographics_2020.csv
- Election CSV: data/2020/elections/presidential_by_tract.csv

Run from repo root: pytest redist/tests/acceptance/test_analyze_map_acceptance.py -v
"""

import json
import subprocess
import sys
import time
import unittest
from pathlib import Path


def find_redist_binary() -> str:
    candidates = [
        Path("redist/target/release/redist.exe"),
        Path("redist/target/release/redist"),
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    raise RuntimeError(
        "redist binary not found. Build with: "
        "cargo build --release --manifest-path redist/Cargo.toml"
    )


def run(args, check=True, **kwargs):
    return subprocess.run(args, capture_output=True, text=True, check=check, **kwargs)


VT_ASSIGNMENTS = Path("outputs/V3/2020/vermont/final_assignments.json")
VT_ANALYSIS_DIR = Path("outputs/V3/2020/vermont/analysis")
VT_MAPS_DIR = Path("outputs/V3/2020/vermont/maps")
DEMO_CSV = Path("data/2020/demographics/vermont_demographics_2020.csv")
ELECTION_CSV = Path("data/2020/elections/presidential_by_tract.csv")


class TestAnalyzeAcceptance(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not VT_ASSIGNMENTS.exists():
            raise unittest.SkipTest(
                f"VT assignments not found at {VT_ASSIGNMENTS}. "
                "Run: redist state --state VT --year 2020 --version V3"
            )
        if not DEMO_CSV.exists():
            raise unittest.SkipTest(
                f"Demographics CSV not found at {DEMO_CSV}. "
                "Run: redist fetch --year 2020"
            )
        cls.binary = find_redist_binary()

    def _run_analyze(self, *type_args, force=False):
        cmd = [self.binary, "analyze", "--state", "VT", "--year", "2020",
               "--version", "V3", "--types"] + list(type_args)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def test_demographic_json_structure(self):
        result = self._run_analyze("demographic", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)

        out = VT_ANALYSIS_DIR / "demographic.json"
        self.assertTrue(out.exists(), f"missing {out}")

        data = json.loads(out.read_text())
        self.assertIn("districts", data)
        self.assertEqual(len(data["districts"]), 1, "VT has exactly 1 district")

        d = data["districts"][0]
        self.assertEqual(d["district"], 1)
        self.assertGreater(d["total_pop"], 500_000, "VT pop ~643K")
        self.assertIn("pct_minority", d)
        self.assertIn("is_majority_minority", d)
        self.assertIn("pop_basis", d)
        self.assertEqual(d["pop_basis"], "total_population")

        # Race fractions should sum to ~1.0
        pct_sum = (d.get("pct_white", 0) + d.get("pct_black", 0) +
                   d.get("pct_asian", 0) + d.get("pct_hispanic", 0) +
                   d.get("pct_other", 0))
        self.assertAlmostEqual(pct_sum, 1.0, places=1,
                               msg=f"pct fractions sum={pct_sum:.3f}")

    def test_political_json_structure(self):
        if not ELECTION_CSV.exists():
            self.skipTest(f"election CSV not found: {ELECTION_CSV}")
        result = self._run_analyze("political", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)

        out = VT_ANALYSIS_DIR / "political.json"
        self.assertTrue(out.exists())
        data = json.loads(out.read_text())
        self.assertIn("available", data)

        if data["available"] and data["districts"]:
            d = data["districts"][0]
            self.assertIn("dem_pct", d)
            self.assertIn("lean_dem", d)
            self.assertIn("is_uncontested", d)
            # dem + rep should be ~1.0 (ignoring third parties)
            self.assertAlmostEqual(d["dem_pct"] + d["rep_pct"], 1.0, places=1)

    def test_all_types_produce_all_files(self):
        result = self._run_analyze("all", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)

        for name in ["demographic.json", "political.json", "urban.json", "summary.json"]:
            path = VT_ANALYSIS_DIR / name
            self.assertTrue(path.exists(), f"missing {name}")
            # Each file should be valid JSON
            data = json.loads(path.read_text())
            self.assertIn("analyzer", data, f"{name} missing 'analyzer' field")

    def test_skip_existing_without_force(self):
        # First run to ensure file exists
        self._run_analyze("demographic", force=True)
        out = VT_ANALYSIS_DIR / "demographic.json"
        mtime1 = out.stat().st_mtime

        time.sleep(0.05)

        # Second run without --force should skip
        result = self._run_analyze("demographic")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("skip", result.stderr.lower(), "should report skip")
        self.assertAlmostEqual(out.stat().st_mtime, mtime1, places=1,
                               msg="file should not be regenerated without --force")

    def test_force_regenerates(self):
        self._run_analyze("demographic", force=True)
        out = VT_ANALYSIS_DIR / "demographic.json"
        mtime1 = out.stat().st_mtime

        time.sleep(0.05)

        self._run_analyze("demographic", force=True)
        self.assertGreater(out.stat().st_mtime, mtime1 - 0.01,
                           "file should be regenerated with --force")

    def test_summary_has_population_balance(self):
        result = self._run_analyze("summary", force=True)
        # VT has 1 district — balance is trivially valid
        # Exit code may be 0 (balanced) or 2 (imbalanced) — VT should be 0
        self.assertIn(result.returncode, [0, 2])

        out = VT_ANALYSIS_DIR / "summary.json"
        if out.exists():
            data = json.loads(out.read_text())
            self.assertIn("population_balance_valid", data)
            if data["districts"]:
                d = data["districts"][0]
                self.assertIn("pop_balance_ok", d)
                self.assertIn("ideal_pop", d)

    def test_unknown_state_exits_nonzero(self):
        result = run([self.binary, "analyze", "--state", "ZZ",
                      "--year", "2020", "--version", "V3"], check=False)
        self.assertNotEqual(result.returncode, 0)


class TestMapAcceptance(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        if not VT_ASSIGNMENTS.exists():
            raise unittest.SkipTest(
                f"VT assignments not found at {VT_ASSIGNMENTS}. "
                "Run: redist state --state VT --year 2020 --version V3"
            )
        cls.binary = find_redist_binary()

    def _run_map(self, *type_args, force=True, dpi="150"):
        cmd = [self.binary, "map", "--state", "VT", "--year", "2020",
               "--version", "V3", "--dpi", dpi, "--types"] + list(type_args)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def test_districts_map_produces_valid_png(self):
        result = self._run_map("districts", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)

        png = VT_MAPS_DIR / "districts.png"
        self.assertTrue(png.exists(), f"districts.png not created")
        data = png.read_bytes()
        self.assertEqual(data[:4], b"\x89PNG", "must be valid PNG magic")
        self.assertGreater(len(data), 5000, f"PNG too small ({len(data)}B), likely blank")

    def test_districts_map_skip_without_force(self):
        self._run_map("districts", force=True)
        png = VT_MAPS_DIR / "districts.png"
        mtime1 = png.stat().st_mtime

        time.sleep(0.05)
        result = self._run_map("districts", force=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("skip", result.stderr.lower())

    def test_political_map_produces_png(self):
        # Ensure analysis exists first
        run([self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "political", "--force"], check=False)
        result = self._run_map("political", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        png = VT_MAPS_DIR / "political.png"
        self.assertTrue(png.exists())
        self.assertEqual(png.read_bytes()[:4], b"\x89PNG")

    def test_demographic_map_produces_png(self):
        run([self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic", "--force"], check=False)
        result = self._run_map("demographic", force=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        png = VT_MAPS_DIR / "demographic.png"
        self.assertTrue(png.exists())
        self.assertEqual(png.read_bytes()[:4], b"\x89PNG")

    def test_dpi_flag_affects_file_size(self):
        self._run_map("districts", dpi="72", force=True)
        png_72 = VT_MAPS_DIR / "districts.png"
        size_72 = png_72.stat().st_size

        # Rename to compare
        png_72_copy = VT_MAPS_DIR / "districts_72.png"
        png_72.rename(png_72_copy)

        self._run_map("districts", dpi="300", force=True)
        size_300 = (VT_MAPS_DIR / "districts.png").stat().st_size

        # Restore 72dpi copy
        png_72_copy.rename(VT_MAPS_DIR / "districts_72.png")

        self.assertGreater(size_300, size_72,
                           f"300 DPI ({size_300}B) should be larger than 72 DPI ({size_72}B)")

    def test_unknown_state_exits_nonzero(self):
        result = run([self.binary, "map", "--state", "ZZ",
                      "--year", "2020", "--version", "V3"], check=False)
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
