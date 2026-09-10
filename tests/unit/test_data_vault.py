"""Portable vault safety and relocation tests; no Census/network/M: dependency."""
import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts import data_vault as vault


class VaultTests(unittest.TestCase):
    def test_catalog_has_unique_portable_paths_and_complete_contexts(self):
        data = json.loads(vault.CATALOG.read_text())
        rows = data["items"]
        self.assertEqual(len(rows), 350)
        self.assertEqual(len({r["id"] for r in rows}), 350)
        for year in (2000, 2010, 2020):
            self.assertEqual(sum(r["group"] == "contexts" and r["year"] == year for r in rows), 50)
        with tempfile.TemporaryDirectory() as temp:
            for row in rows:
                self.assertTrue(vault.inside(Path(temp), row["path"]).is_relative_to(temp))
                self.assertEqual(len(row["sha256"]), 64)

    def test_traversal_and_absolute_paths_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            for path in ("../escape", "/etc/passwd", "C:/escape", "foo\\bar", ""):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    vault.inside(Path(temp), path)

    def test_relocated_file_verifies_and_tampering_is_detected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "item").write_bytes(b"abc")
            rows = [{"id": "fixture", "path": "item", "bytes": 3, "sha256": hashlib.sha256(b"abc").hexdigest()}]
            self.assertEqual(vault.inspect(root, rows, True)[0]["status"], "verified")
            (root / "item").write_bytes(b"xyz")
            self.assertEqual(vault.inspect(root, rows, True)[0]["status"], "hash-mismatch")

    def test_link_dry_run_does_not_write(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo, data = root / "clone", root / "drive/project"
            repo.mkdir()
            (data / "derived-data/census/2020/certified").mkdir(parents=True)
            rows = vault.link_plan(data, repo)
            self.assertIn("would-link", {r["status"] for r in rows})
            self.assertFalse((repo / "data").exists())

    def test_link_never_replaces_local_data(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo, data = root / "clone", root / "drive/project"
            local = repo / "data/enacted_districts"
            local.mkdir(parents=True)
            (local / "keep").write_text("user data")
            rows = vault.link_plan(data, repo, apply=True)
            self.assertIn("existing-local-path-retained", {r["status"] for r in rows})
            self.assertEqual((local / "keep").read_text(), "user data")

    def test_fetch_dry_run_never_opens_network(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(vault.urllib.request, "urlopen", side_effect=AssertionError("network")):
            result = vault.fetch(Path(temp), [{"id": "fixture", "path": "raw/file", "url": "https://www2.census.gov/fixture"}])
            self.assertTrue(result["dry_run"])

    def test_derived_context_not_misrepresented_as_downloadable(self):
        with tempfile.TemporaryDirectory() as temp, self.assertRaisesRegex(ValueError, "rebuilt"):
            vault.fetch(Path(temp), [{"id": "context", "path": "data/context"}], True)

    def test_existing_destination_not_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            src, dst = root / "src", root / "dst"
            src.write_bytes(b"new")
            dst.write_bytes(b"old")
            with self.assertRaisesRegex(ValueError, "not overwritten"):
                vault.copy_checked(src, dst, hashlib.sha256(b"new").hexdigest(), 3)
            self.assertEqual(dst.read_bytes(), b"old")

    def test_bad_source_never_admitted(self):
        with tempfile.TemporaryDirectory() as temp:
            source, target = Path(temp) / "source", Path(temp) / "target"
            source.write_bytes(b"bad")
            with self.assertRaisesRegex(ValueError, "nothing admitted"):
                vault.copy_checked(source, target, hashlib.sha256(b"new").hexdigest(), 3)
            self.assertFalse(target.exists())

    def test_untrusted_download_host_rejected(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(vault.urllib.request, "urlopen", side_effect=AssertionError("network")):
            with self.assertRaisesRegex(ValueError, "official Census"):
                vault.fetch(Path(temp), [{"id": "fixture", "path": "raw/file", "url": "https://example.com/data"}], True)

    def test_builder_profile_pins_sources_and_line_endings(self):
        profile = json.loads((vault.ROOT / "configs/data-vault/builder-2020.json").read_text())
        self.assertEqual(len(profile["sources"]), 6)
        self.assertTrue(all(len(row["revision"]) == 40 and row["line_endings"] in ("lf", "crlf") for row in profile["sources"]))

    def test_vault_option_overrides_environment(self):
        with tempfile.TemporaryDirectory() as temp, patch.dict(vault.os.environ, {"BISECT_DATA_VAULT": str(Path(temp) / "wrong")}):
            self.assertEqual(vault.vault_root(str(Path(temp) / "right")), Path(temp).resolve() / "right")


if __name__ == "__main__":
    unittest.main()
