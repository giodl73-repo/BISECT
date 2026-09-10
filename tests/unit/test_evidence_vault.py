import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts import evidence_vault as e


class EvidenceTests(unittest.TestCase):
    def test_dry_run_and_verified_hydration_preserve_existing(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            vault, repo = root / "vault", root / "repo"
            vault.mkdir()
            (vault / "source").write_bytes(b"evidence")
            entry = {"repo_path": "docs/evidence", "vault_path": "source", "bytes": 8, "sha256": hashlib.sha256(b"evidence").hexdigest()}
            with patch.object(e, "entries", return_value=[entry]):
                e.hydrate(vault, repo)
                self.assertFalse(repo.exists())
                e.hydrate(vault, repo, True)
                self.assertEqual((repo / "docs/evidence").read_bytes(), b"evidence")
                (repo / "docs/evidence").write_bytes(b"user")
                with self.assertRaises(ValueError):
                    e.hydrate(vault, repo, True)
                self.assertEqual((repo / "docs/evidence").read_bytes(), b"user")

    def test_git_fallback_validates_bytes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            entry = {"repo_path": "docs/evidence", "vault_path": "source", "bytes": 8, "sha256": hashlib.sha256(b"evidence").hexdigest(), "revision": "a" * 40}
            def git(command, **kwargs):
                kwargs["stdout"].write(b"evidence")
            with patch.object(e, "entries", return_value=[entry]), patch.object(e.subprocess, "run", side_effect=git):
                e.hydrate(root / "vault", root / "repo", True)
                self.assertEqual((root / "vault/source").read_bytes(), b"evidence")


if __name__ == "__main__":
    unittest.main()
