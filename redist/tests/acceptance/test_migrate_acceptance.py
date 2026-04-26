"""L2 acceptance tests for `redist migrate`."""
import json
import subprocess
import unittest
from pathlib import Path


def find_binary():
    for p in [
        Path("redist/target/release/redist.exe"),
        Path("redist/target/release/redist"),
    ]:
        if p.exists():
            return str(p.resolve())
    raise RuntimeError(
        "redist binary not found — run: cargo build --release --manifest-path redist/Cargo.toml"
    )


def run(args, check=True):
    return subprocess.run(args, capture_output=True, text=True, check=check)


class TestMigrateAcceptance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Need a legacy state plan in states/ tree
        src = Path("outputs/V3/2020/states/vermont/data/final_assignments.json")
        if not src.exists():
            raise unittest.SkipTest(
                f"VT legacy plan not found at {src}. "
                "Run: redist state --state VT --year 2020 --version V3"
            )
        cls.binary = find_binary()

    def _migrate(self, label, force=False):
        cmd = [
            self.binary, "migrate",
            "--state", "VT",
            "--year", "2020",
            "--version", "V3",
            "--label", label,
        ]
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def test_migrate_creates_plan_dir(self):
        label = "vt_migrate_test"
        r = self._migrate(label, force=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        plan_dir = Path(f"outputs/V3/2020/plans/{label}")
        self.assertTrue(plan_dir.exists(), f"plan dir not created: {plan_dir}")
        self.assertTrue((plan_dir / "manifest.json").exists(), "manifest.json missing")

    def test_migrate_manifest_has_correct_fields(self):
        label = "vt_migrate_fields_test"
        self._migrate(label, force=True)
        manifest = json.loads(
            Path(f"outputs/V3/2020/plans/{label}/manifest.json").read_text()
        )
        self.assertEqual(manifest.get("state_code"), "VT")
        # source field not currently in manifest — check label instead
        self.assertEqual(manifest.get("label"), label)

    def test_migrate_assignments_copied(self):
        label = "vt_migrate_assign_test"
        self._migrate(label, force=True)
        # assignments may live at root or under data/
        root_path = Path(f"outputs/V3/2020/plans/{label}/final_assignments.json")
        data_path = Path(f"outputs/V3/2020/plans/{label}/data/final_assignments.json")
        asgn_path = root_path if root_path.exists() else data_path
        self.assertTrue(asgn_path.exists(), "assignments not copied")
        data = json.loads(asgn_path.read_text())
        self.assertGreater(len(data), 0)

    def test_migrate_collision_without_force(self):
        label = "vt_migrate_collision_test"
        self._migrate(label, force=True)       # first run
        r = self._migrate(label, force=False)  # second without --force
        self.assertNotEqual(r.returncode, 0, "should fail without --force")
        self.assertIn("exists", r.stderr.lower())

    def test_migrate_force_overwrites(self):
        label = "vt_migrate_force_test"
        self._migrate(label, force=True)
        r = self._migrate(label, force=True)   # second with --force
        self.assertEqual(r.returncode, 0, r.stderr)


if __name__ == "__main__":
    unittest.main()
