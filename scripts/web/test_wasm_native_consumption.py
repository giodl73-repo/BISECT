"""Source-contract regressions, distinct from actual native/WASM execution."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from wasm_native_consumption import ROOT, FAMILIES, MAIN, INVENTORY, body, code_mask, traces


class NativeConsumption(unittest.TestCase):
    def test_mask_preserves_lines_and_ignores_noncode(self):
        source = '''// args.comment
/* args.outer /* args.inner */ */
let retained = r###"args.raw { }"###;
let other = br"args.bytes";
let message = "escaped \\" args.string";
let character = '}';
let value = args.sa_t0_factor;
let wrapped = args
    .output_dir;
'''
        masked = code_mask(source)
        self.assertEqual(len(source), len(masked))
        self.assertEqual(source.count('\n'), masked.count('\n'))
        for absent in ('args.comment', 'args.outer', 'args.inner', 'args.raw', 'args.bytes', 'args.string'):
            self.assertNotIn(absent, masked)
        self.assertIn('args.sa_t0_factor', masked)
        self.assertIn('.output_dir', masked)

    def test_balancing_ignores_literal_braces(self):
        source = 'fn example() { let x = "}"; /* { */ if true { args.field; } } fn next() {}'
        start, end, masked = body(source, 'fn example()')
        self.assertEqual(source[end:].strip(), 'fn next() {}')
        self.assertIn('args.field', masked)
        self.assertEqual(start, 0)

    def test_current_traces_and_changed_unused_behavior(self):
        entries = json.loads((ROOT / INVENTORY).read_text(encoding='utf-8'))['entries']
        current = traces(ROOT, entries)
        self.assertEqual(len(current), 152)
        self.assertEqual(sum(t['finding'] == 'no-direct-read-in-reviewed-boundary' for t in current.values()), 22)
        for entry in entries:
            if entry['id'] in current:
                self.assertEqual(entry['parity_record']['native_consumption'], current[entry['id']])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for source in {MAIN, *(family[1] for family in FAMILIES.values())}:
                path = root / source
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((ROOT / source).read_bytes())
            run = root / FAMILIES['cli.run'][1]
            original = run.read_text(encoding='utf-8')
            run.write_text(original.replace('Self{debug:args.debug', 'Self{debug:args.print_only', 1), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'reviewed unused set changed'):
                traces(root, entries)
            run.write_text(original + '\n// source revision witness\n', encoding='utf-8')
            self.assertNotEqual(current['cli.run.year'], traces(root, entries)['cli.run.year'])

    def test_removing_field_cannot_hide_an_existing_read(self):
        entries = json.loads((ROOT / INVENTORY).read_text(encoding='utf-8'))['entries']
        entries = [e for e in copy.deepcopy(entries) if e['id'] != 'cli.state.sa_t0_factor']
        with self.assertRaisesRegex(ValueError, 'unrecognized direct argument read sa_t0_factor'):
            traces(ROOT, entries)


if __name__ == '__main__':
    unittest.main()
