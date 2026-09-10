"""Real admission/validation logic with tiny controlled external boundaries."""
import hashlib
import io
import json
import subprocess
import tempfile
import unittest
import zipfile
from contextlib import ExitStack, redirect_stdout, redirect_stderr
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import data_vault as v


def digest(data):
    return hashlib.sha256(data).hexdigest()


def row(data=b"abc", member=None):
    result = {"id": "fixture", "path": "raw/file", "bytes": len(data), "sha256": digest(data), "url": "https://www2.census.gov/fixture"}
    if member:
        result["member"] = member
    return result


class Response(io.BytesIO):
    url = "https://www2.census.gov/fixture"


class FailureTests(unittest.TestCase):
    def test_interrupted_admission_retries_without_partial_destination(self):
        with tempfile.TemporaryDirectory() as temp:
            src, dst = Path(temp) / "src", Path(temp) / "dst"
            src.write_bytes(b"complete")
            def interrupt(inp, out):
                out.write(inp.read(2))
                raise OSError("interrupted")
            with patch.object(v.shutil, "copyfileobj", side_effect=interrupt), self.assertRaises(OSError):
                v.copy_checked(src, dst, digest(b"complete"), 8)
            self.assertFalse(dst.exists())
            v.copy_checked(src, dst, digest(b"complete"), 8)
            self.assertEqual(dst.read_bytes(), b"complete")

    def test_racing_publication_never_clobbers(self):
        with tempfile.TemporaryDirectory() as temp:
            src, dst = Path(temp) / "src", Path(temp) / "dst"
            src.write_bytes(b"abc")
            publish = v.publish_new
            def race(staged, target):
                target.write_bytes(b"user")
                publish(staged, target)
            with patch.object(v, "publish_new", side_effect=race), self.assertRaises(FileExistsError):
                v.copy_checked(src, dst, digest(b"abc"), 3)
            self.assertEqual(dst.read_bytes(), b"user")

    def test_download_success_and_idempotence(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(v, "open_download", return_value=Response(b"abc")) as network:
            root = Path(temp)
            self.assertEqual(v.fetch(root, [row()], True)["results"][0]["status"], "copied-verified")
            self.assertEqual(v.fetch(root, [row()], True)["results"], [])
            self.assertEqual(network.call_count, 1)

    def test_bad_download_and_limit_never_admitted(self):
        for content, limit in ((b"xyz", 3), (b"abcd", 3)):
            with self.subTest(content=content), tempfile.TemporaryDirectory() as temp, patch.object(v, "open_download", return_value=Response(content)):
                with self.assertRaises(ValueError):
                    v.fetch(Path(temp), [row()], True, max_bytes=limit)
                self.assertFalse((Path(temp) / "raw/file").exists())

    def test_zip_members_success_duplicate_missing_and_corrupt(self):
        for names, data, succeeds in ((["folder/pop"], b"abc", True), (["pop", "folder/pop"], b"abc", False), (["other"], b"abc", False), (["pop"], b"xyz", False)):
            with self.subTest(names=names, data=data), tempfile.TemporaryDirectory() as temp:
                archive = io.BytesIO()
                with zipfile.ZipFile(archive, "w") as z:
                    for name in names:
                        z.writestr(name, data)
                with patch.object(v, "open_download", return_value=Response(archive.getvalue())):
                    if succeeds:
                        v.fetch(Path(temp), [row(member="pop")], True, max_bytes=4096)
                        self.assertEqual((Path(temp) / "raw/file").read_bytes(), b"abc")
                    else:
                        with self.assertRaises(ValueError):
                            v.fetch(Path(temp), [row(member="pop")], True, max_bytes=4096)
                        self.assertFalse((Path(temp) / "raw/file").exists())

    def test_redirect_rejected_before_following(self):
        for url in ("https://example.com/evil", "http://www2.census.gov/plain"):
            with self.assertRaises(ValueError):
                v.CensusRedirect().redirect_request(None, None, 302, "", {}, url)

    def test_deadline_and_space_limits(self):
        with patch.object(v.time, "monotonic", return_value=2), self.assertRaisesRegex(ValueError, "deadline"):
            v.bounded_copy(io.BytesIO(b"abc"), io.BytesIO(), 3, 1)
        with patch.object(v.shutil, "disk_usage", return_value=SimpleNamespace(free=1)), self.assertRaisesRegex(ValueError, "free space"):
            v.require_space(Path.cwd(), 2)

    def test_cli_rejects_inapplicable_options_before_io(self):
        for args in (("build-2020", "--year", "2010", "--state", "RI", "--run-name", "test"), ("ri-replay", "--state", "TX", "--run-name", "test"), ("status", "--apply")):
            with self.subTest(args=args), patch.object(v.sys, "argv", ["vault", *args]), patch.object(v, "vault_root", side_effect=AssertionError("I/O")), redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
                v.main()
            self.assertEqual(error.exception.code, 2)

    def test_mixed_unknown_state_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown States"):
            v.selected(SimpleNamespace(state=["RI", "TYPO"], year=2020, group="contexts"))

    def test_incomplete_links_exit_nonzero(self):
        with patch.object(v.sys, "argv", ["vault", "link", "--apply"]), patch.object(v, "vault_root", return_value=Path.cwd()), patch.object(v, "link_plan", return_value=[{"status": "missing-vault-directory"}]), redirect_stdout(io.StringIO()):
            self.assertEqual(v.main(), 1)

    def test_real_link_relocation_and_local_directory_preserved(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            repo = root / "repo"
            repo.mkdir()
            old, new = root / "old", root / "new"
            remote = "derived-data/census/2020/certified"
            for drive, content in ((old, "old"), (new, "new")):
                (drive / remote).mkdir(parents=True)
                (drive / remote / "proof").write_text(content)
            local = repo / "data/enacted_districts"
            local.mkdir(parents=True)
            (local / "keep").write_text("user")
            target = repo / "data/2020/certified"
            try:
                v.link_plan(old, repo, apply=True)
                self.assertEqual((target / "proof").read_text(), "old")
                self.assertIn("existing-local-path-retained", {r["status"] for r in v.link_plan(new, repo)})
                v.link_plan(new, repo, apply=True, repair=True)
                self.assertEqual((target / "proof").read_text(), "new")
                self.assertEqual((local / "keep").read_text(), "user")
                self.assertEqual((old / remote / "proof").read_text(), "old")
            finally:
                if v.os.path.lexists(target):
                    if target.is_symlink():
                        target.unlink()
                    else:
                        v.os.rmdir(target)

    def test_versions_and_catalog_integrity(self):
        original = json.loads(v.CATALOG.read_text())
        for mutation in ("schema", "duplicate", "hash", "coverage"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temp:
                data = json.loads(json.dumps(original))
                if mutation == "schema": data["schema"] = "future"
                if mutation == "duplicate": data["items"].append(data["items"][0])
                if mutation == "hash": data["items"][0]["sha256"] = "z" * 64
                if mutation == "coverage": data["items"].pop(0)
                path = Path(temp) / "catalog.json"
                path.write_text(json.dumps(data))
                with patch.object(v, "CATALOG", path), self.assertRaises(ValueError):
                    v.load_catalog()
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "local.json"
            path.write_text('{"schema":"future","vault":"bad"}')
            with patch.object(v, "LOCAL", path), patch.dict(v.os.environ, {"BISECT_DATA_VAULT": ""}), self.assertRaises(ValueError):
                v.vault_root()

    def test_replay_mismatch_no_success_receipt(self):
        # Real baseline/context checks and result comparisons, controlled engine.
        import sys
        sys.path.insert(0, str(v.ROOT / "scripts/research"))
        import run_ri_adjacent_swaps as engine
        baseline = json.loads((v.ROOT / "docs/experiments/ri-boundary-budget-2020/analysis.json").read_text())
        expected = json.loads((v.ROOT / "docs/experiments/ri-adjacent-swaps-2020/analysis.json").read_text())
        for field in (None, "final", "status", "assignment_sha256"):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                context = root / "context"
                context.write_text("{}")
                results = json.loads(json.dumps(expected["results"]))
                if field: results[0][field] = "different"
                fixture = {"items": [{"id": "context-2020-RI", "path": "context", "sha256": v.sha256(context)}]}
                with patch.object(v, "load_catalog", return_value=fixture), patch.object(engine, "improve_swaps", side_effect=results):
                    if field:
                        with self.assertRaisesRegex(ValueError, "differs"):
                            v.ri_replay(root, "run")
                        self.assertFalse((root / "runs/portable-replay/run/analysis.json").exists())
                    else:
                        self.assertEqual(v.ri_replay(root, "run")["arms"], len(baseline["results"]))
                        with self.assertRaisesRegex(ValueError, "new run"):
                            v.ri_replay(root, "run")

    def test_builder_admission_failures_and_complete_receipt(self):
        for mode in ("success", "failed", "wrong-hash", "timeout", "missing-manifest"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temp, ExitStack() as stack:
                root = Path(temp)
                archive = root / "blocks.zip"
                with zipfile.ZipFile(archive, "w") as z:
                    z.writestr("blocks.shp", b"shape")
                (root / "population").write_bytes(b"pop")
                candidate_bytes = b'{"fixture":true}'
                sources = [{"id": "blocks", "group": "blocks", "state": "RI", "year": 2020, "path": "blocks.zip", "bytes": archive.stat().st_size, "sha256": v.sha256(archive)},
                           {"id": "pop", "group": "population", "state": "RI", "year": 2020, "path": "population", "bytes": 3, "sha256": digest(b"pop")}]
                context = {"id": "context-2020-RI", "group": "contexts", "state": "RI", "year": 2020, "path": "derived/context", "bytes": len(candidate_bytes), "sha256": digest(candidate_bytes)}
                stack.enter_context(patch.object(v, "load_catalog", return_value={"items": sources + [context]}))
                stack.enter_context(patch.object(v, "link_directory"))
                profile = v.load_profile()
                for source in profile["sources"]:
                    source["line_endings"] = "lf"
                    source["sha256"] = digest(b"# fixture\n")
                stack.enter_context(patch.object(v, "load_profile", return_value=profile))
                stack.enter_context(patch.object(v.subprocess, "check_output", return_value=b"# fixture\n"))
                def builder(command, **kwargs):
                    work = kwargs["cwd"]
                    if mode == "timeout": raise subprocess.TimeoutExpired(command, 1)
                    if mode == "failed": return SimpleNamespace(returncode=1)
                    (work / "candidate.rctx").write_bytes(b"wrong" if mode == "wrong-hash" else candidate_bytes)
                    (work / "report.json").write_text("{}")
                    if mode != "missing-manifest": (work / "manifest.json").write_text("{}")
                    return SimpleNamespace(returncode=0)
                stack.enter_context(patch.object(v.subprocess, "run", side_effect=builder))
                if mode != "success":
                    with self.assertRaises((ValueError, OSError)):
                        v.build_2020(root, "RI", "build", True)
                    self.assertFalse((root / "derived/context").exists())
                    self.assertFalse((root / "runs/context-builds/build/admission.json").exists())
                else:
                    self.assertEqual(v.build_2020(root, "RI", "build", True)["status"], "rebuilt-and-verified")
                    receipt = json.loads((root / "runs/context-builds/build/admission.json").read_text())
                    for key in ("schema", "catalog_sha256", "driver_sha256", "helper_sha256", "runtime", "inputs", "command", "resolved_state", "artifacts", "builder_sources"):
                        self.assertIn(key, receipt)
                    self.assertEqual(receipt["artifacts"].keys(), {"manifest.json", "report.json", "build.log"})


if __name__ == "__main__":
    unittest.main()
