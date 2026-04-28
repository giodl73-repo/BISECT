"""
L2 acceptance tests — pipeline regression guards.

Require the redist binary built at:
  redist/target/release/redist.exe (Windows) or redist/target/release/redist (Linux/Mac)

And adjacency data at:
  outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl  (VT)
  outputs/V3/data/2020/adjacency/wa_adjacency_2020.pkl  (WA)

Skip if binary or data missing.
"""
import json
import subprocess
import sys
import unittest
from pathlib import Path


def find_binary():
    for p in [
        Path("redist/target/release/redist.exe"),
        Path("redist/target/release/redist"),
    ]:
        if p.exists():
            return str(p.resolve())
    return None


def run(args, check=False):
    return subprocess.run(args, capture_output=True, text=True, check=check)


class TestVTPipeline(unittest.TestCase):
    """Task 197: VT congressional full pipeline — state + analyze + report."""

    @classmethod
    def setUpClass(cls):
        binary = find_binary()
        if not binary:
            raise unittest.SkipTest("redist binary not found — build with: cargo build --release -p redist-cli")
        cls.binary = binary

        vt_adj = Path("outputs/V3/data/2020/adjacency/vt_adjacency_2020.pkl")
        if not vt_adj.exists():
            raise unittest.SkipTest(
                f"VT adjacency not found at {vt_adj}. "
                "Run: redist fetch --type adjacency --states VT --year 2020"
            )

    def _run(self, args, check=False):
        return run([self.binary] + args, check=check)

    def test_01_vt_state_runs(self):
        r = self._run([
            "state", "--state", "VT", "--year", "2020",
            "--version", "l2_test", "--label", "vt_l2_test", "--force"
        ])
        self.assertEqual(r.returncode, 0, f"state failed: {r.stderr}")
        plan_dir = Path("outputs/l2_test/2020/plans/vt_l2_test")
        self.assertTrue(plan_dir.exists(), "plan directory not created")
        self.assertTrue((plan_dir / "manifest.json").exists(), "manifest.json missing")

    def test_02_vt_analyze_uses_plan_manifest_districts(self):
        # KEY REGRESSION TEST: analyze must use manifest num_districts (1 for VT),
        # not the global congressional count. If this returns 10, the bug is back.
        r = self._run([
            "analyze", "--label", "vt_l2_test",
            "--year", "2020", "--version", "l2_test",
            "--types", "summary", "contiguity", "compactness", "--force"
        ])
        self.assertEqual(r.returncode, 0, f"analyze failed: {r.stderr}")

        summary_path = Path("outputs/l2_test/2020/plans/vt_l2_test/analysis/summary.json")
        self.assertTrue(summary_path.exists(), "summary.json not written")
        summary = json.loads(summary_path.read_text())

        districts = summary.get("districts", [])
        self.assertEqual(len(districts), 1,
            f"VT congressional must have 1 district in summary, got {len(districts)}")

    def test_03_vt_report_generates_html(self):
        r = self._run([
            "report", "--label", "vt_l2_test",
            "--year", "2020", "--version", "l2_test",
            "--format", "html"
        ])
        self.assertEqual(r.returncode, 0, f"report failed: {r.stderr}")
        html_path = Path("reports/vt_l2_test/vt_l2_test_report.html")
        self.assertTrue(html_path.exists(), "HTML report not written")
        html = html_path.read_text()
        self.assertIn("Vermont", html, "HTML report must mention Vermont")

    def test_04_vt_report_json_has_correct_chamber(self):
        json_path = Path("reports/vt_l2_test/vt_l2_test_report.json")
        if not json_path.exists():
            # Generate if not already done
            self._run([
                "report", "--label", "vt_l2_test",
                "--year", "2020", "--version", "l2_test",
                "--format", "json"
            ])
        if json_path.exists():
            report = json.loads(json_path.read_text())
            exec_summary = report.get("sections", {}).get("executive_summary", {})
            self.assertEqual(exec_summary.get("chamber"), "congressional",
                "report executive summary must show correct chamber")


class TestWAHousePipelineRegression(unittest.TestCase):
    """Task 198: WA house 98D — the specific regression guard for the num_districts bug."""

    @classmethod
    def setUpClass(cls):
        binary = find_binary()
        if not binary:
            raise unittest.SkipTest("redist binary not found")
        cls.binary = binary

        wa_adj = Path("outputs/V3/data/2020/adjacency/wa_adjacency_2020.pkl")
        if not wa_adj.exists():
            raise unittest.SkipTest(
                f"WA adjacency not found at {wa_adj}. "
                "Run: redist fetch --type adjacency --states WA --year 2020"
            )

    def _run(self, args, check=False):
        return run([self.binary] + args, check=check)

    def test_01_wa_house_state_runs(self):
        r = self._run([
            "state", "--state", "WA", "--chamber", "house",
            "--year", "2020", "--version", "l2_test",
            "--label", "wa_house_l2_test", "--force"
        ])
        self.assertEqual(r.returncode, 0, f"state failed: {r.stderr}")
        manifest = json.loads(
            Path("outputs/l2_test/2020/plans/wa_house_l2_test/manifest.json").read_text()
        )
        self.assertEqual(manifest["num_districts"], 98,
            f"manifest must record 98 districts, got {manifest['num_districts']}")
        self.assertEqual(manifest["chamber"], "house")

    def test_02_wa_house_analyze_has_98_districts_not_10(self):
        """REGRESSION: before the PlanContext fix, analyze returned 10 districts (congressional)."""
        r = self._run([
            "analyze", "--label", "wa_house_l2_test",
            "--year", "2020", "--version", "l2_test",
            "--types", "summary", "--force"
        ])
        self.assertEqual(r.returncode, 0, f"analyze failed: {r.stderr}")

        summary_path = Path("outputs/l2_test/2020/plans/wa_house_l2_test/analysis/summary.json")
        self.assertTrue(summary_path.exists())
        summary = json.loads(summary_path.read_text())

        districts = summary.get("districts", [])
        self.assertEqual(len(districts), 98,
            f"WA house analyze must produce 98 districts, got {len(districts)}. "
            f"If this is 10, the num_districts regression has returned.")

    def test_03_wa_house_balance_valid(self):
        summary_path = Path("outputs/l2_test/2020/plans/wa_house_l2_test/analysis/summary.json")
        if not summary_path.exists():
            self.skipTest("summary.json not found — run test_02 first")
        summary = json.loads(summary_path.read_text())
        self.assertTrue(summary.get("population_balance_valid", False),
            "WA house plan must pass balance check after analyze")


if __name__ == "__main__":
    unittest.main()
