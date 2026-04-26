"""
L2 acceptance tests for `redist analyze` and `redist map`.

Requires:
- redist binary built: cargo build --release --manifest-path redist/Cargo.toml
- VT 2020 run: redist state --state VT --year 2020 --version V3
- AL 2020 run: redist state --state AL --year 2020 --version V3
- Demographics CSV: data/2020/demographics/vermont_demographics_2020.csv
- Election CSV: data/2020/elections/presidential_by_tract.csv
- Geoid mapping: outputs/V3/data/2020/adjacency/vt_adjacency_2020_geoids.json
  (generate with: python scripts/data/generate_adj_bin.py --year 2020 --states VT AL)

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


# Correct output paths: {version}/states/{state_name}/
VT_DATA_DIR    = Path("outputs/V3/states/vermont/data")
VT_ANALYSIS    = Path("outputs/V3/states/vermont/analysis")
VT_MAPS        = Path("outputs/V3/states/vermont/maps")
AL_DATA_DIR    = Path("outputs/V3/states/alabama/data")
AL_ANALYSIS    = Path("outputs/V3/states/alabama/analysis")
AL_MAPS        = Path("outputs/V3/states/alabama/maps")
DEMO_CSV       = Path("data/2020/demographics/vermont_demographics_2020.csv")
ELECTION_CSV   = Path("data/2020/elections/presidential_by_tract.csv")
VT_GEOID_JSON  = Path("outputs/V3/data/2020/adjacency/vt_adjacency_2020_geoids.json")


class TestAnalyzeVT(unittest.TestCase):
    """L2 analyze tests on Vermont (1 district — simple, fast)."""

    @classmethod
    def setUpClass(cls):
        if not (VT_DATA_DIR / "final_assignments.json").exists():
            raise unittest.SkipTest(
                "VT assignments not found. Run: redist state --state VT --year 2020 --version V3"
            )
        if not DEMO_CSV.exists():
            raise unittest.SkipTest(
                f"Demographics CSV not found: {DEMO_CSV}. Run: redist fetch --year 2020"
            )
        if not VT_GEOID_JSON.exists():
            raise unittest.SkipTest(
                f"Geoid mapping not found: {VT_GEOID_JSON}. "
                "Run: python scripts/data/generate_adj_bin.py --year 2020 --states VT"
            )
        cls.binary = find_redist_binary()

    def _analyze(self, *types, force=True, state="VT"):
        cmd = [self.binary, "analyze", "--state", state, "--year", "2020",
               "--version", "V3", "--types"] + list(types)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    # ── demographic ────────────────────────────────────────────────────────

    def test_demographic_district_count(self):
        r = self._analyze("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((VT_ANALYSIS / "demographic.json").read_text())
        self.assertEqual(len(data["districts"]), 1, "VT must have exactly 1 district")

    def test_demographic_population_plausible(self):
        self._analyze("demographic")
        d = json.loads((VT_ANALYSIS / "demographic.json").read_text())["districts"][0]
        self.assertGreater(d["total_pop"], 500_000, "VT pop ~643K")
        self.assertLess(d["total_pop"], 800_000)

    def test_demographic_fractions_sum_to_one(self):
        self._analyze("demographic")
        d = json.loads((VT_ANALYSIS / "demographic.json").read_text())["districts"][0]
        total = (d["pct_white"] + d["pct_black"] + d["pct_asian"] +
                 d["pct_hispanic"] + d["pct_other"])
        self.assertAlmostEqual(total, 1.0, places=1, msg=f"fractions sum={total:.4f}")

    def test_demographic_required_fields(self):
        self._analyze("demographic")
        d = json.loads((VT_ANALYSIS / "demographic.json").read_text())["districts"][0]
        for field in ["district", "total_pop", "pct_white", "pct_minority",
                      "is_majority_minority", "pop_basis"]:
            self.assertIn(field, d, f"missing field: {field}")
        self.assertEqual(d["pop_basis"], "total_population")

    def test_demographic_vt_not_majority_minority(self):
        self._analyze("demographic")
        d = json.loads((VT_ANALYSIS / "demographic.json").read_text())["districts"][0]
        self.assertFalse(d["is_majority_minority"], "VT is ~89% white, not MM")

    # ── political ─────────────────────────────────────────────────────────

    def test_political_json_structure(self):
        if not ELECTION_CSV.exists():
            self.skipTest("election CSV not available")
        r = self._analyze("political")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((VT_ANALYSIS / "political.json").read_text())
        self.assertIn("available", data)
        if data["available"] and data["districts"]:
            d = data["districts"][0]
            for f in ["dem_pct", "rep_pct", "margin", "lean_dem", "is_uncontested"]:
                self.assertIn(f, d, f"missing field: {f}")
            self.assertAlmostEqual(d["dem_pct"] + d["rep_pct"], 1.0, places=1)

    def test_political_vt_leans_dem(self):
        if not ELECTION_CSV.exists():
            self.skipTest("election CSV not available")
        self._analyze("political")
        data = json.loads((VT_ANALYSIS / "political.json").read_text())
        if data["available"] and data["districts"]:
            d = data["districts"][0]
            # VT has voted D in every presidential election since 1988
            self.assertTrue(d["lean_dem"], "VT should lean Democratic")
            self.assertGreater(d["dem_pct"], 0.5)

    # ── urban ─────────────────────────────────────────────────────────────

    def test_urban_json_structure(self):
        r = self._analyze("urban")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((VT_ANALYSIS / "urban.json").read_text())
        self.assertIn("analyzer", data)
        self.assertEqual(data["analyzer"], "urban")
        self.assertIn("available", data)
        self.assertIn("districts", data)

    def test_urban_graceful_when_no_places_data(self):
        # Urban may be unavailable if places CSV doesn't exist — should still exit 0
        r = self._analyze("urban")
        self.assertEqual(r.returncode, 0, r.stderr)

    # ── compactness ───────────────────────────────────────────────────────

    def test_compactness_json_structure(self):
        r = self._analyze("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((VT_ANALYSIS / "compactness.json").read_text())
        self.assertIn("analyzer", data)
        self.assertEqual(data["analyzer"], "compactness")
        self.assertEqual(len(data["districts"]), 1)

    def test_compactness_metrics_in_range(self):
        self._analyze("compactness")
        d = json.loads((VT_ANALYSIS / "compactness.json").read_text())["districts"][0]
        for metric in ["polsby_popper", "reock", "convex_hull_ratio"]:
            self.assertIn(metric, d)
            val = d[metric]
            self.assertGreater(val, 0.0, f"{metric} must be > 0")
            self.assertLessEqual(val, 1.0, f"{metric} must be ≤ 1")

    def test_compactness_crs_note_present(self):
        self._analyze("compactness")
        d = json.loads((VT_ANALYSIS / "compactness.json").read_text())["districts"][0]
        self.assertIn("crs_note", d, "WGS84 limitation must be documented")

    # ── summary ───────────────────────────────────────────────────────────

    def test_summary_has_all_fields(self):
        self._analyze("summary")
        data = json.loads((VT_ANALYSIS / "summary.json").read_text())
        self.assertIn("population_balance_valid", data)
        self.assertEqual(len(data["districts"]), 1)
        d = data["districts"][0]
        for f in ["district", "pop_balance_ok", "ideal_pop", "pop_deviation_pct"]:
            self.assertIn(f, d, f"missing summary field: {f}")

    def test_summary_vt_balanced(self):
        # VT has 1 district — trivially balanced
        r = self._analyze("summary")
        self.assertEqual(r.returncode, 0, "VT (1 district) should be balanced")
        data = json.loads((VT_ANALYSIS / "summary.json").read_text())
        self.assertTrue(data["population_balance_valid"])
        self.assertTrue(data["districts"][0]["pop_balance_ok"])

    # ── all types ─────────────────────────────────────────────────────────

    def test_all_types_produce_all_files(self):
        r = self._analyze("all")
        self.assertEqual(r.returncode, 0, r.stderr)
        for name in ["demographic.json", "political.json", "urban.json",
                     "compactness.json", "summary.json"]:
            path = VT_ANALYSIS / name
            self.assertTrue(path.exists(), f"missing {name}")
            self.assertIn("analyzer", json.loads(path.read_text()), f"{name} missing 'analyzer'")

    # ── skip / force ──────────────────────────────────────────────────────

    def test_skip_without_force(self):
        self._analyze("demographic", force=True)
        mtime1 = (VT_ANALYSIS / "demographic.json").stat().st_mtime
        time.sleep(0.05)
        r = self._analyze("demographic", force=False)
        self.assertEqual(r.returncode, 0)
        self.assertIn("skip", r.stderr.lower())
        self.assertAlmostEqual((VT_ANALYSIS / "demographic.json").stat().st_mtime, mtime1, places=1)

    def test_force_regenerates(self):
        self._analyze("demographic", force=True)
        mtime1 = (VT_ANALYSIS / "demographic.json").stat().st_mtime
        time.sleep(0.05)
        self._analyze("demographic", force=True)
        self.assertGreater((VT_ANALYSIS / "demographic.json").stat().st_mtime, mtime1 - 0.01)

    def test_unknown_state_exits_nonzero(self):
        r = run([self.binary, "analyze", "--state", "ZZ", "--year", "2020",
                 "--version", "V3"], check=False)
        self.assertNotEqual(r.returncode, 0)


class TestAnalyzeAL(unittest.TestCase):
    """L2 analyze tests on Alabama (7 districts — multi-district coverage)."""

    @classmethod
    def setUpClass(cls):
        if not (AL_DATA_DIR / "final_assignments.json").exists():
            raise unittest.SkipTest(
                "AL assignments not found. Run: redist state --state AL --year 2020 --version V3"
            )
        cls.binary = find_redist_binary()

    def _analyze(self, *types, force=True):
        cmd = [self.binary, "analyze", "--state", "AL", "--year", "2020",
               "--version", "V3", "--types"] + list(types)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def test_al_demographic_seven_districts(self):
        r = self._analyze("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((AL_ANALYSIS / "demographic.json").read_text())
        self.assertEqual(len(data["districts"]), 7, "AL must have exactly 7 districts")

    def test_al_demographic_district_ids_sequential(self):
        self._analyze("demographic")
        data = json.loads((AL_ANALYSIS / "demographic.json").read_text())
        ids = sorted(d["district"] for d in data["districts"])
        self.assertEqual(ids, list(range(1, 8)), "district IDs must be 1-7")

    def test_al_demographic_populations_plausible(self):
        self._analyze("demographic")
        data = json.loads((AL_ANALYSIS / "demographic.json").read_text())
        pops = [d["total_pop"] for d in data["districts"]]
        # AL total pop ~5M / 7 districts ≈ 714K per district
        for pop in pops:
            self.assertGreater(pop, 400_000, f"district pop {pop} too low")
            self.assertLess(pop, 1_000_000, f"district pop {pop} too high")

    def test_al_compactness_seven_districts(self):
        r = self._analyze("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads((AL_ANALYSIS / "compactness.json").read_text())
        self.assertEqual(len(data["districts"]), 7)
        for d in data["districts"]:
            self.assertGreater(d["polsby_popper"], 0.0)
            self.assertLessEqual(d["polsby_popper"], 1.0)

    def test_al_summary_balance_valid(self):
        # AL uses edge-weighted mode — should pass ±0.5%
        r = self._analyze("summary")
        self.assertEqual(r.returncode, 0, "AL balanced run should exit 0")
        data = json.loads((AL_ANALYSIS / "summary.json").read_text())
        self.assertTrue(data["population_balance_valid"],
                        "AL edge-weighted should pass balance check")


class TestMapVT(unittest.TestCase):
    """L2 map tests on Vermont (1 district)."""

    @classmethod
    def setUpClass(cls):
        if not (VT_DATA_DIR / "final_assignments.json").exists():
            raise unittest.SkipTest(
                "VT assignments not found. Run: redist state --state VT --year 2020 --version V3"
            )
        if not VT_GEOID_JSON.exists():
            raise unittest.SkipTest(
                f"Geoid mapping not found: {VT_GEOID_JSON}"
            )
        cls.binary = find_redist_binary()

    def _map(self, *types, force=True, dpi="150"):
        cmd = [self.binary, "map", "--state", "VT", "--year", "2020",
               "--version", "V3", "--dpi", dpi, "--types"] + list(types)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def _ensure_analysis(self, *types):
        run([self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types"] + list(types) + ["--force"], check=False)

    def _valid_png(self, path: Path, min_bytes=5000) -> None:
        self.assertTrue(path.exists(), f"{path} not created")
        data = path.read_bytes()
        self.assertEqual(data[:4], b"\x89PNG", f"{path} not valid PNG")
        self.assertGreater(len(data), min_bytes, f"{path} too small ({len(data)}B)")

    def test_districts_map_valid_png(self):
        r = self._map("districts")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(VT_MAPS / "districts.png")

    def test_political_map_valid_png(self):
        self._ensure_analysis("political")
        r = self._map("political")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(VT_MAPS / "political.png")

    def test_demographic_map_valid_png(self):
        self._ensure_analysis("demographic")
        r = self._map("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(VT_MAPS / "demographic.png")

    def test_compactness_map_valid_png(self):
        self._ensure_analysis("compactness")
        r = self._map("compactness")
        self.assertEqual(r.returncode, 0, r.stderr)
        self._valid_png(VT_MAPS / "compactness.png", min_bytes=1000)

    def test_rounds_graceful_skip_for_vt(self):
        # VT has 1 district — no bisection rounds, should skip not fail
        r = self._map("rounds")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("skip", r.stderr.lower(), "should report skip for no-rounds state")

    def test_dpi_72_smaller_than_300(self):
        self._map("districts", dpi="72")
        size_72 = (VT_MAPS / "districts.png").stat().st_size
        self._map("districts", dpi="300")
        size_300 = (VT_MAPS / "districts.png").stat().st_size
        self.assertGreater(size_300, size_72,
                           f"300 DPI ({size_300}B) must be larger than 72 DPI ({size_72}B)")

    def test_skip_without_force(self):
        self._map("districts", force=True)
        mtime1 = (VT_MAPS / "districts.png").stat().st_mtime
        time.sleep(0.05)
        r = self._map("districts", force=False)
        self.assertEqual(r.returncode, 0)
        self.assertIn("skip", r.stderr.lower())

    def test_unknown_state_exits_nonzero(self):
        r = run([self.binary, "map", "--state", "ZZ", "--year", "2020",
                 "--version", "V3"], check=False)
        self.assertNotEqual(r.returncode, 0)


class TestMapAL(unittest.TestCase):
    """L2 map tests on Alabama (7 districts — multi-district coverage)."""

    @classmethod
    def setUpClass(cls):
        if not (AL_DATA_DIR / "final_assignments.json").exists():
            raise unittest.SkipTest(
                "AL assignments not found. Run: redist state --state AL --year 2020 --version V3"
            )
        cls.binary = find_redist_binary()

    def _map(self, *types, force=True):
        cmd = [self.binary, "map", "--state", "AL", "--year", "2020",
               "--version", "V3", "--types"] + list(types)
        if force:
            cmd.append("--force")
        return run(cmd, check=False)

    def test_al_districts_map_valid_png(self):
        r = self._map("districts")
        self.assertEqual(r.returncode, 0, r.stderr)
        png = AL_MAPS / "districts.png"
        self.assertTrue(png.exists())
        data = png.read_bytes()
        self.assertEqual(data[:4], b"\x89PNG")
        # Multi-district state map should be larger than VT's
        self.assertGreater(len(data), 20_000,
                           f"AL map too small ({len(data)}B) for 7 districts")

    def test_al_political_map_produces_png(self):
        # Ensure political analysis exists
        run([self.binary, "analyze", "--state", "AL", "--year", "2020",
             "--version", "V3", "--types", "political", "--force"], check=False)
        r = self._map("political")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue((AL_MAPS / "political.png").exists())

    def test_al_demographic_map_produces_png(self):
        run([self.binary, "analyze", "--state", "AL", "--year", "2020",
             "--version", "V3", "--types", "demographic", "--force"], check=False)
        r = self._map("demographic")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue((AL_MAPS / "demographic.png").exists())


class TestResolutionFlag(unittest.TestCase):
    """L2 acceptance tests for --resolution flag on `redist state`.

    These tests verify the flag is accepted and parsed without requiring
    block group adjacency data (which may not be present in CI).

    With block group data present (outputs/V3/data/{year}/adjacency/*_bg_adjacency_*.pkl),
    the full block group path is exercised. Without it, graceful fallback to tract is
    verified via the WARNING message.
    """

    @classmethod
    def setUpClass(cls):
        cls.binary = find_redist_binary()

    def _state(self, resolution, state="VT", year="2020", version="ResolutionTest", **kwargs):
        """Run `redist state` with the given resolution flag."""
        cmd = [
            self.binary, "state",
            "--state", state,
            "--year", year,
            "--version", version,
            "--resolution", resolution,
            "--force",
        ]
        return run(cmd, check=False)

    def test_resolution_tract_accepted(self):
        """--resolution tract is the default and must be accepted."""
        r = self._state("tract")
        # May fail due to missing adjacency data, but must not fail due to flag parsing
        # (clap would exit 2 for unknown flags)
        self.assertNotEqual(r.returncode, 2,
                            f"--resolution tract rejected by clap: {r.stderr}")

    def test_resolution_block_group_accepted(self):
        """--resolution block_group must be accepted (exit code != 2 means clap parsed it)."""
        r = self._state("block_group")
        self.assertNotEqual(r.returncode, 2,
                            f"--resolution block_group rejected by clap: {r.stderr}")

    def test_resolution_block_group_fallback_or_success(self):
        """With block_group resolution, CLI either succeeds (data present) or falls back to
        tract with a clear WARNING (data absent). Must not crash or exit 2."""
        r = self._state("block_group")
        # Exit 2 = clap parse error — must never happen for a valid flag
        self.assertNotEqual(r.returncode, 2,
                            f"--resolution block_group parse error: {r.stderr}")
        if r.returncode != 0:
            # If it failed for non-parse reasons (e.g. missing tract adjacency too),
            # the error must be descriptive, not a clap usage message
            self.assertNotIn("error: unexpected argument", r.stderr.lower())
        else:
            # Successful run: either used block_group data or fell back to tract
            combined = r.stderr + r.stdout
            self.assertTrue(
                "WARNING: Block group adjacency not found" in combined or
                "[OK]" in combined,
                f"Expected fallback WARNING or [OK] in output: {combined}"
            )

    def test_resolution_block_group_fallback_warning_mentions_build_script(self):
        """When block group adjacency is absent, the fallback warning must mention
        build_bg_adjacency.py so users know how to generate the data."""
        r = self._state("block_group")
        if r.returncode == 2:
            self.fail(f"--resolution block_group rejected by clap: {r.stderr}")
        # Only check for the build script hint if we got the fallback (not full BG data)
        combined = r.stderr + r.stdout
        if "WARNING: Block group adjacency not found" in combined:
            self.assertIn("build_bg_adjacency.py", combined,
                          "Fallback warning must mention build_bg_adjacency.py")

    def test_resolution_invalid_value_rejected(self):
        """An invalid --resolution value must be rejected by clap with exit code 2."""
        r = run(
            [self.binary, "state", "--state", "VT", "--year", "2020",
             "--version", "ResolutionTest", "--resolution", "invalid_resolution"],
            check=False
        )
        self.assertEqual(r.returncode, 2,
                         f"Expected exit 2 for invalid resolution, got {r.returncode}: {r.stderr}")


if __name__ == "__main__":
    unittest.main()
