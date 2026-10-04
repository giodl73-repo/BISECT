"""Failure-path tests for source custody; no GIS installation or vault needed."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import subprocess
import os
import zipfile
from unittest.mock import patch
import workspace as w
from package_map import extract


class CustodyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); self.source = self.root / 'source'; self.source.write_bytes(b'canonical')
        self.digest = w.sha(self.source); self.target = self.root / 'copy'

    def test_bad_source_never_admitted(self):
        with self.assertRaises(ValueError): w.checked_copy(self.source, self.target, '0'*64, 9)
        self.assertFalse(self.target.exists()); self.assertFalse(list(self.root.glob('*.partial')))

    def test_existing_mismatch_preserved(self):
        self.target.write_bytes(b'keep')
        with self.assertRaises(ValueError): w.checked_copy(self.source, self.target, self.digest, 9)
        self.assertEqual(self.target.read_bytes(), b'keep')

    def test_interrupted_copy_retry_and_no_shared_source_inode(self):
        with patch.object(w.shutil, 'copyfileobj', side_effect=OSError('interrupted')):
            with self.assertRaises(OSError): w.checked_copy(self.source, self.target, self.digest, 9)
        self.assertFalse(self.target.exists())
        w.checked_copy(self.source, self.target, self.digest, 9)
        self.target.write_bytes(b'edited'); self.assertEqual(self.source.read_bytes(), b'canonical')

    def test_cache_reuse_without_source(self):
        w.checked_copy(self.source, self.target, self.digest, 9); self.source.unlink()
        w.checked_copy(self.source, self.target, self.digest, 9)

    def test_traversal_rejected(self):
        for name in ['../secret', '/root', 'C:/path', 'a\\b', 'inputs/../secret']:
            with self.assertRaises(ValueError): w.relative(name)

    def test_linked_workspace_rejected(self):
        link = self.root / 'link'
        try: link.symlink_to(self.root, target_is_directory=True)
        except OSError:
            if os.name != 'nt': raise
            subprocess.run(['cmd', '/c', 'mklink', '/J', str(link), str(self.root)], check=True, capture_output=True)
        with patch.object(w, 'REPO', self.root):
            with self.assertRaises(ValueError): w.local_directory(link / 'work')

    def test_missing_archive_drive_preserves_local(self):
        package = self.root / 'release.zip'; package.write_bytes(b'package')
        w.write_json(package.with_suffix('.verification.json'), {'status': 'verified', 'zip_sha256': w.sha(package)})
        w.archive(package, self.root / 'absent-drive', 'release-1')
        self.assertTrue(package.exists()); self.assertFalse((self.root / 'absent-drive').exists())
        self.assertEqual(json.loads(package.with_suffix('.archive.json').read_text())['status'], 'verified-local-awaiting-archive')

    def test_zip_traversal_refused_before_extraction(self):
        archive = self.root / 'bad.zip'
        with zipfile.ZipFile(archive, 'w') as z: z.writestr('../escape', 'bad')
        target = self.root / 'extracted'
        with self.assertRaises(ValueError): extract(archive, target)
        self.assertFalse(target.exists())

    def test_duplicate_zip_member_refused(self):
        archive = self.root / 'bad.zip'
        with zipfile.ZipFile(archive, 'w') as z:
            z.writestr('A', 'one'); z.writestr('a', 'two')
        with self.assertRaises(ValueError): extract(archive, self.root / 'extracted')

    def test_insufficient_space_and_duplicate_run(self):
        selection = self.root / 'selection.json'
        w.write_json(selection, {'schema':'bisect-publication-selection-v1','files':[{'source':'source','path':'inputs/source','bytes':9,'sha256':self.digest}]})
        with patch.object(w, 'REPO', self.root), patch.object(w.shutil, 'disk_usage', return_value=type('Space', (), {'free': 1})()):
            with self.assertRaises(OSError): w.stage(selection, 'test', self.root, True)
        run = self.root / '.work/publication/runs/test'; run.mkdir()
        with patch.object(w, 'REPO', self.root):
            with self.assertRaises(FileExistsError): w.stage(selection, 'test', self.root, True)


if __name__ == '__main__': unittest.main()
