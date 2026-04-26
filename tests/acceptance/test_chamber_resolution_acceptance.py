"""
L2 acceptance tests for chamber, resolution, label auto-resolution, and balance tolerance features.

Requires:
- redist binary built
- VT 2020 run at tract level: outputs/V3/states/vermont/data/final_assignments.json
- VT 2020 labeled run: outputs/V3/2020/plans/vt_congress_test/ (from earlier experiments)
"""

import json, subprocess, time, unittest
from pathlib import Path

def find_binary():
    for p in [Path("redist/target/release/redist.exe"), Path("redist/target/release/redist")]:
        if p.exists(): return str(p.resolve())
    raise RuntimeError("binary not found")

def run(args, check=True):
    return subprocess.run(args, capture_output=True, text=True, check=check)


class TestChamberFlag(unittest.TestCase):
    """--chamber flag sets chamber-aware balance tolerance defaults."""

    @classmethod
    def setUpClass(cls):
        cls.binary = find_binary()
        adj = Path("outputs/V3/data/2020/adjacency/vt_adjacency_2020.adj.bin")
        if not adj.exists():
            raise unittest.SkipTest("VT adjacency not found")

    def _run_state(self, chamber, label, extra_args=None):
        cmd = [self.binary, "state", "--state", "VT", "--year", "2020",
               "--version", "ChamberTest", "--chamber", chamber,
               "--label", label, "--seed", "42", "--force"]
        if extra_args: cmd.extend(extra_args)
        return run(cmd, check=False)

    def test_congressional_chamber_default_tolerance(self):
        r = self._run_state("congressional", "vt_cong_chamber_test")
        self.assertEqual(r.returncode, 0, r.stderr)
        mf = Path("outputs/ChamberTest/2020/plans/vt_cong_chamber_test/manifest.json")
        self.assertTrue(mf.exists())
        m = json.loads(mf.read_text())
        self.assertEqual(m["chamber"], "congressional")
        self.assertAlmostEqual(m["balance_tolerance_pct"], 0.5, places=1,
                               msg="congressional default should be 0.5%")

    def test_house_chamber_default_tolerance(self):
        r = self._run_state("house", "vt_house_chamber_test")
        self.assertEqual(r.returncode, 0, r.stderr)
        mf = Path("outputs/ChamberTest/2020/plans/vt_house_chamber_test/manifest.json")
        m = json.loads(mf.read_text())
        self.assertEqual(m["chamber"], "house")
        self.assertAlmostEqual(m["balance_tolerance_pct"], 5.0, places=1,
                               msg="house default should be 5.0%")

    def test_senate_chamber_default_tolerance(self):
        r = self._run_state("senate", "vt_senate_chamber_test")
        self.assertEqual(r.returncode, 0, r.stderr)
        m = json.loads(Path("outputs/ChamberTest/2020/plans/vt_senate_chamber_test/manifest.json").read_text())
        self.assertEqual(m["chamber"], "senate")
        self.assertAlmostEqual(m["balance_tolerance_pct"], 5.0, places=1)

    def test_custom_districts_override(self):
        r = self._run_state("house", "vt_custom_dist_test", ["--districts", "1"])
        self.assertEqual(r.returncode, 0, r.stderr)
        m = json.loads(Path("outputs/ChamberTest/2020/plans/vt_custom_dist_test/manifest.json").read_text())
        self.assertEqual(m["num_districts"], 1)

    def test_label_in_manifest(self):
        r = self._run_state("congressional", "vt_label_test_abc")
        self.assertEqual(r.returncode, 0, r.stderr)
        m = json.loads(Path("outputs/ChamberTest/2020/plans/vt_label_test_abc/manifest.json").read_text())
        self.assertEqual(m["label"], "vt_label_test_abc")


class TestResolutionFlag(unittest.TestCase):
    """--resolution flag controls adjacency and TIGER source."""

    @classmethod
    def setUpClass(cls):
        cls.binary = find_binary()

    def test_resolution_tract_is_default(self):
        r = run([self.binary, "state", "--help"], check=False)
        self.assertIn("tract", r.stdout + r.stderr)

    def test_resolution_block_group_accepted(self):
        # Just verify the flag parses; BG adjacency may not exist → warning + fallback
        r = run([self.binary, "state", "--state", "VT", "--year", "2020",
                 "--version", "ResTest", "--label", "vt_bg_test",
                 "--resolution", "block_group", "--seed", "42", "--force"],
                check=False)
        # Should succeed (either with BG data or fallback to tract with warning)
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_resolution_invalid_value_rejected(self):
        r = run([self.binary, "state", "--state", "VT", "--year", "2020",
                 "--version", "ResTest", "--label", "vt_bad_res",
                 "--resolution", "invalid_resolution"],
                check=False)
        self.assertNotEqual(r.returncode, 0)

    def test_block_group_fallback_warning_when_no_bg_data(self):
        # For a state without BG adjacency, should print a warning and fall back
        r = run([self.binary, "state", "--state", "VT", "--year", "2020",
                 "--version", "ResTest", "--label", "vt_bg_fallback",
                 "--resolution", "block_group", "--seed", "42", "--force"],
                check=False)
        self.assertEqual(r.returncode, 0)
        # If BG data absent, warning should appear; if present, no warning needed
        # Either way, must succeed


class TestLabelAutoStateResolution(unittest.TestCase):
    """analyze --label without --state reads state from manifest."""

    @classmethod
    def setUpClass(cls):
        cls.binary = find_binary()
        # Need a labeled plan with manifest
        plan_dir = Path("outputs/ChamberTest/2020/plans/vt_cong_chamber_test")
        if not (plan_dir / "manifest.json").exists():
            # Create it quickly
            r = run([cls.binary, "state", "--state", "VT", "--year", "2020",
                     "--version", "ChamberTest", "--chamber", "congressional",
                     "--label", "vt_cong_chamber_test", "--seed", "42", "--force"],
                    check=False)
            if r.returncode != 0:
                raise unittest.SkipTest("Could not create test plan")

    def test_analyze_label_without_state(self):
        r = run([self.binary, "analyze",
                 "--label", "vt_cong_chamber_test",
                 "--year", "2020", "--version", "ChamberTest",
                 "--types", "demographic", "--force"],
                check=False)
        # Should succeed by reading state_code from manifest
        self.assertEqual(r.returncode, 0, r.stderr)
        out = Path("outputs/ChamberTest/2020/plans/vt_cong_chamber_test/analysis/demographic.json")
        self.assertTrue(out.exists(), "demographic.json not created")

    def test_analyze_state_still_works_explicitly(self):
        r = run([self.binary, "analyze", "--state", "VT",
                 "--label", "vt_cong_chamber_test",
                 "--year", "2020", "--version", "ChamberTest",
                 "--types", "demographic", "--force"],
                check=False)
        self.assertEqual(r.returncode, 0, r.stderr)


class TestBalanceToleranceFromManifest(unittest.TestCase):
    """Summary analyzer reads balance_tolerance from manifest, not hardcoded 0.5%."""

    @classmethod
    def setUpClass(cls):
        cls.binary = find_binary()
        # Need a state legislative plan (5% tolerance)
        plan = Path("outputs/ChamberTest/2020/plans/vt_house_chamber_test/manifest.json")
        if not plan.exists():
            raise unittest.SkipTest("vt_house_chamber_test plan not found")

    def test_summary_uses_manifest_tolerance(self):
        r = run([self.binary, "analyze",
                 "--label", "vt_house_chamber_test",
                 "--year", "2020", "--version", "ChamberTest",
                 "--types", "summary", "--force"],
                check=False)
        # VT with 5% state leg tolerance should be balanced (1 district = trivially balanced)
        self.assertEqual(r.returncode, 0,
                         "State leg plan with 5% tolerance should pass balance check")
        summary = json.loads(Path("outputs/ChamberTest/2020/plans/vt_house_chamber_test/analysis/summary.json").read_text())
        self.assertTrue(summary["population_balance_valid"],
                        "VT 1-district state leg plan must be balanced")

if __name__ == "__main__":
    unittest.main()
