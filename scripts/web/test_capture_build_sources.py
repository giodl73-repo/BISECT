from pathlib import Path
import subprocess
import tempfile
import unittest
from capture_build_sources import collect

class SourceCaptureTests(unittest.TestCase):
    def test_fixture_extensions_and_explicit_ignored_proof_are_retained(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
            def write(name, data):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            write('.gitignore', b'target/\n')
            write('docs/fixtures/import-label/positive/vermont_two_tracts.csv', b'geoid,district\n50001000100,1\n')
            write('docs/fixtures/source.shp', b'\x00\x01binary fixture')
            write('src/main.rs', b'fn main() {}\n')
            write('target/proof/oracle.mjs', b'export const retained = true;\n')
            write('target/unlisted-cache.bin', b'not a declared source')
            rows = collect(root, [{'path': 'target/proof/oracle.mjs'}])
            names = {row['path'] for row in rows}
            self.assertIn('docs/fixtures/import-label/positive/vermont_two_tracts.csv', names)
            self.assertIn('docs/fixtures/source.shp', names)
            self.assertIn('src/main.rs', names)
            self.assertIn('target/proof/oracle.mjs', names)
            self.assertNotIn('target/unlisted-cache.bin', names)
            for row in rows:
                self.assertEqual(row['bytes'], (root / row['path']).stat().st_size)
                self.assertEqual(len(row['sha256']), 64)

    def test_missing_prior_input_refuses_instead_of_shrinking_capture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
            with self.assertRaisesRegex(ValueError, 'Prior captured source is missing'):
                collect(root, [{'path': 'docs/fixtures/required.csv'}])

if __name__ == '__main__':
    unittest.main()
