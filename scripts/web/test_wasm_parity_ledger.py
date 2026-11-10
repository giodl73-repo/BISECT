"""Portable closure-gate mutations; synthetic files are NOT engine parity evidence."""
import copy
import hashlib
import json
import lzma
from pathlib import Path
import tempfile
import unittest

from wasm_parity_ledger import (audit_inventory, initialize, pin, validate_inventory,
                               SOURCE_CATALOGS, validate_current_source_catalogs)


class ClosureGate(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ('declaration.rs', 'consumer.rs', 'browser.js', 'test.py', 'fixture.json',
                     'native.exe', 'engine.wasm', 'native-output.json', 'browser-output.json', 'comparison.json'):
            (self.root / name).write_text('synthetic guard witness: ' + name, encoding='utf-8')
        self.data = {'entries': [{'id': 'sample.Run', 'source': 'declaration.rs'}]}
        initialize(self.root, self.data)
        self.row = self.data['entries'][0]['parity_record']
        self.row.update(implementation_state='implemented', evidence_state='verified',
                        classification='browser-adaptation', open_issue=None)
        self.row['native_authority']['consumer'] = pin(self.root, 'consumer.rs')
        self.contract = {
            'contract_id': 'sample.Run/selected-fixture', 'scope': {'fixture': 'synthetic'},
            'comparison': {'version': 1, 'mode': 'exact-bytes', 'allowlist': []},
            'backend': 'metis-core', 'claim_kind': 'workflow', 'required_for_full_parity': True,
            'successful_support': True, 'implementation_state': 'implemented', 'evidence_state': 'verified',
            'native_consumer': pin(self.root, 'consumer.rs'),
            'browser_action': {'source': pin(self.root, 'browser.js'), 'operation': 'run', 'ui_hook': '#run'},
        }
        self.bundle = {key: copy.deepcopy(self.contract[key]) for key in
                       ('contract_id', 'scope', 'comparison', 'backend', 'claim_kind')}
        self.bundle.update(schema='bisect-parity-acceptance-v1', status='passed', outcome='accepted-output',
                           reference_kind='native-cli', browser_exercised=True, reproducible_fixture=True,
                           source_files=[pin(self.root, 'consumer.rs'), pin(self.root, 'browser.js')],
                           test_sources=[pin(self.root, 'test.py')], fixture_inputs=[pin(self.root, 'fixture.json')],
                           raw_outputs=[pin(self.root, 'native-output.json'), pin(self.root, 'browser-output.json')],
                           comparison_results=[pin(self.root, 'comparison.json')], checks=[{'status': 'passed'}])
        self.bundle['build'] = dict(native_executable=pin(self.root, 'native.exe'), wasm=pin(self.root, 'engine.wasm'),
                                    source_revision='synthetic-test-only', toolchain='test', dependencies='test',
                                    build_settings='test', browser_runtime='test', original_arguments=['run'],
                                    resolved_arguments=['run'], seed='18446744073709551615', profile='test',
                                    integer_width=32, threading='single', floating_point='f64', backend='metis-core')
        self.row['contracts'] = [self.contract]
        self.write_bundle()

    def write_bundle(self):
        (self.root / 'bundle.json').write_text(json.dumps(self.bundle), encoding='utf-8')
        self.contract['acceptance_bundle'] = pin(self.root, 'bundle.json')

    def check(self, full=True):
        validate_inventory(self.root, self.data, full)

    def test_valid_scoped_contract_and_full_gate(self):
        self.check()
        initialize(self.root, self.data)
        self.check()  # Initialization must not erase reviewed contracts.
        self.assertEqual(audit_inventory(self.root, self.data), [])

    def test_unverified_contract_still_checks_related_browser_source(self):
        self.row.update(evidence_state='partial', open_issue='Browser workflow not yet verified')
        self.contract['evidence_state'] = 'not-run'
        worker = self.root / 'worker.js'
        worker.write_text('original adapter', encoding='utf-8')
        self.contract['browser_action']['related_sources'] = [pin(self.root, 'worker.js')]
        self.check(full=False)
        worker.write_text('changed adapter', encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'Stale evidence: worker.js'):
            self.check(full=False)

    def test_encoded_wasm_binds_the_actual_bytes_in_every_build_profile(self):
        image = b'\x00asm\x01\x00\x00\x00'
        (self.root / 'engine.wasm.xz').write_bytes(lzma.compress(image))
        encoded = dict(pin(self.root, 'engine.wasm.xz'), encoding=dict(codec='xz',
                       decoded_sha256=hashlib.sha256(image).hexdigest(), decoded_byte_length=len(image)))
        self.bundle['build']['wasm'] = encoded
        self.bundle['build']['additional_profiles'] = {'c-enabled': dict(
            native_executable=self.bundle['build']['native_executable'], wasm=copy.deepcopy(encoded))}
        self.write_bundle()
        self.check()
        self.bundle['build']['additional_profiles']['c-enabled']['wasm']['encoding']['decoded_sha256'] = '0' * 64
        self.write_bundle()
        with self.assertRaisesRegex(ValueError, 'Decoded WASM identity differs'):
            self.check()

    def test_encoded_wasm_rejects_trailing_streams_length_and_wrong_file_type(self):
        image = b'\x00asm\x01\x00\x00\x00'
        cases = ((lzma.compress(image) + lzma.compress(image), image, len(image), 'Trailing encoded WASM'),
                 (lzma.compress(image + b'x'), image, len(image), 'exceeds declared length'),
                 (lzma.compress(image)[:-1], image, len(image), 'Truncated encoded WASM'),
                 (lzma.compress(b'not-wasm'), b'not-wasm', 8, 'not a WASM module'),
                 (lzma.compress(image), image, 65 * 1024 * 1024, 'Invalid encoded WASM limits'))
        for blob, original, length, diagnostic in cases:
            with self.subTest(diagnostic=diagnostic):
                (self.root / 'engine.wasm.xz').write_bytes(blob)
                self.bundle['build']['wasm'] = dict(pin(self.root, 'engine.wasm.xz'), encoding=dict(codec='xz',
                    decoded_sha256=hashlib.sha256(original).hexdigest(), decoded_byte_length=length))
                self.write_bundle()
                with self.assertRaisesRegex(ValueError, diagnostic):
                    self.check()

    def test_audit_lists_every_parent_and_required_contract_that_blocks_closure(self):
        pending = copy.deepcopy(self.contract)
        pending.update(contract_id='sample.Run/pending', evidence_state='partial',
                       scope={'remaining': 'Check the saved workspace with the native CLI'})
        optional = copy.deepcopy(pending)
        optional.update(contract_id='sample.Run/optional', required_for_full_parity=False)
        self.row.update(implementation_state='partial', evidence_state='partial', open_issue='Workspace incomplete')
        self.row['contracts'].extend((pending, optional))
        self.data['entries'].append({'id': 'sample.Other', 'source': 'declaration.rs'})
        initialize(self.root, self.data)

        run, other = audit_inventory(self.root, self.data)
        self.assertEqual([run['id'], other['id']], ['sample.Run', 'sample.Other'])
        self.assertEqual([c['code'] for c in run['conditions']],
                         ['parent-evidence-unverified', 'parent-implementation-unresolved',
                          'required-contracts-unverified', 'open-issue'])
        self.assertEqual([c['id'] for c in run['unverified_required_contracts']], ['sample.Run/pending'])
        self.assertEqual(run['unverified_required_contracts'][0]['remaining'], pending['scope']['remaining'])
        self.assertIn('missing-native-consumer', [c['code'] for c in other['conditions']])
        self.assertIn('missing-required-contracts', [c['code'] for c in other['conditions']])
        with self.assertRaisesRegex(ValueError, '^sample.Run: Parent evidence has not been verified'):
            self.check()

    def test_audit_refuses_stale_evidence_instead_of_reporting_a_valid_snapshot(self):
        self.row.update(evidence_state='partial', open_issue='Another workflow is pending')
        (self.root / 'browser-output.json').write_text('different result', encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'Stale evidence: browser-output.json'):
            audit_inventory(self.root, self.data)

    def test_bad_execution_evidence(self):
        mutations = (
            ('inspector', lambda b: b.update(reference_kind='inspection')),
            ('shared-adapter-only', lambda b: b.update(reference_kind='shared-adapter')),
            ('not-run', lambda b: b.update(status='not-run')),
            ('no-browser', lambda b: b.update(browser_exercised=False)),
            ('required-capacity-skipped', lambda b: b['checks'].append({'status': 'not-run'})),
            ('substituted-backend', lambda b: b['build'].update(backend='c-ffi')),
            ('different-scope', lambda b: b.update(scope={'fixture': 'different'})),
            ('matching-failure-not-success', lambda b: b.update(outcome='expected-native-failure')),
            ('missing-toolchain', lambda b: b['build'].pop('toolchain')),
            ('unbound-consumer', lambda b: b.update(source_files=[pin(self.root, 'browser.js')])),
        )
        original = copy.deepcopy(self.bundle)
        for name, mutate in mutations:
            with self.subTest(name=name):
                self.bundle = copy.deepcopy(original)
                mutate(self.bundle)
                self.write_bundle()
                with self.assertRaises(ValueError):
                    self.check()

    def test_unfinalized_comparison_cannot_close_even_when_bundle_matches(self):
        invalid = (
            None, '', 'exact outputs', {}, {'mode': 'exact-bytes', 'allowlist': []},
            *({'version': value, 'mode': 'exact-bytes', 'allowlist': []}
              for value in (0, -1, True, '1')),
            *({'version': 1, 'mode': value, 'allowlist': []}
              for value in (None, 1, '', ' exact-bytes ', 'contract-not-yet-finalized',
                            'pending-actual-native-workflow-and-browser-acceptance',
                            'unreviewed', 'todo', 'tbd', 'not-run')),
            {'version': 1, 'mode': 'exact-bytes'},
            {'version': 1, 'mode': 'exact-bytes', 'allowlist': {}},
            {'version': 1, 'mode': 'exact-bytes', 'allowlist': None},
        )
        baseline = copy.deepcopy(self.bundle)
        for comparison in invalid:
            with self.subTest(comparison=comparison):
                self.contract['comparison'] = copy.deepcopy(comparison)
                self.bundle = copy.deepcopy(baseline)
                self.bundle['comparison'] = copy.deepcopy(comparison)
                self.write_bundle()
                for full in (False, True):
                    with self.assertRaisesRegex(ValueError, 'comparison'):
                        self.check(full=full)

    def test_unfinalized_partial_contract_remains_an_explicit_gap(self):
        self.row.update(evidence_state='partial', open_issue='Comparison remains unfinished')
        self.contract.update(implementation_state='partial', evidence_state='partial')
        for comparison in (None, 'unversioned pending review',
                           {'version': 1, 'mode': 'contract-not-yet-finalized', 'allowlist': []}):
            with self.subTest(comparison=comparison):
                self.contract['comparison'] = comparison
                self.check(full=False)
                with self.assertRaisesRegex(ValueError, 'Parent evidence has not been verified'):
                    self.check(full=True)

    def test_contract_specific_native_authority_is_enforced(self):
        self.contract['comparison']['reference_kinds'] = ['native-cli']
        self.bundle['comparison'] = copy.deepcopy(self.contract['comparison'])
        self.write_bundle()
        self.check()
        self.bundle['reference_kind'] = 'independent-native-kernel'
        self.write_bundle()
        for full in (False, True):
            with self.assertRaisesRegex(ValueError, 'comparison-required native authority'):
                self.check(full=full)
        for kinds in ([], 'native-cli', ['shared-adapter'], ['native-cli', 'native-cli']):
            with self.subTest(reference_kinds=kinds):
                self.contract['comparison']['reference_kinds'] = kinds
                self.bundle['comparison'] = copy.deepcopy(self.contract['comparison'])
                self.bundle['reference_kind'] = 'native-cli'
                self.write_bundle()
                with self.assertRaisesRegex(ValueError, 'comparison native authority'):
                    self.check()

    def test_all_declared_comparison_stages_are_required(self):
        required = ['native-cli-orchestration', 'browser-run-artifacts']
        self.contract['comparison']['required_check_ids'] = required
        self.bundle['comparison'] = copy.deepcopy(self.contract['comparison'])
        for checks in ([{'status': 'passed'}],
                       [{'id': required[0], 'status': 'passed'}],
                       [{'id': required[0], 'status': 'passed'}, {'id': required[0], 'status': 'passed'},
                        {'id': required[1], 'status': 'passed'}]):
            with self.subTest(checks=checks):
                self.bundle['checks'] = checks
                self.write_bundle()
                for full in (False, True):
                    with self.assertRaisesRegex(ValueError, 'comparison-required stage'):
                        self.check(full=full)
        self.bundle['checks'] = [{'id': name, 'status': 'passed'} for name in required]
        self.write_bundle()
        self.check()
        for ids in ([], 'native-cli-orchestration', ['duplicate', 'duplicate'], ['']):
            with self.subTest(required_check_ids=ids):
                self.contract['comparison']['required_check_ids'] = ids
                self.bundle['comparison'] = copy.deepcopy(self.contract['comparison'])
                self.write_bundle()
                with self.assertRaisesRegex(ValueError, 'comparison required stage'):
                    self.check()

    def test_stale_bytes(self):
        for name in ('consumer.rs', 'browser.js', 'fixture.json', 'engine.wasm', 'native.exe', 'bundle.json'):
            with self.subTest(path=name):
                path = self.root / name
                original = path.read_bytes()
                path.write_bytes(original + b'altered')
                with self.assertRaises(ValueError):
                    self.check()
                path.write_bytes(original)

    def test_preview_preserves_unresolved_parent(self):
        pending = {'id': 'sample.Missing', 'source': 'declaration.rs'}
        self.data['entries'].append(pending)
        initialize(self.root, self.data)
        self.check(full=False)
        with self.assertRaisesRegex(ValueError, 'sample.Missing'):
            self.check(full=True)

    def test_cannot_drop_required_contract_or_mark_pending_parent_verified(self):
        self.row['contracts'] = []
        with self.assertRaises(ValueError):
            self.check()
        self.row['contracts'] = [self.contract]
        self.contract['required_for_full_parity'] = False
        with self.assertRaises(ValueError):
            self.check()

    def test_expected_failure_is_explicit(self):
        self.bundle['outcome'] = 'expected-native-failure'
        self.contract['successful_support'] = False
        self.write_bundle()
        self.check()

    def test_partial_parent_does_not_pass_full_gate(self):
        self.row.update(evidence_state='partial', open_issue='Remaining reconciliation')
        self.check(full=False)
        with self.assertRaises(ValueError):
            self.check(full=True)

    def test_paths_cannot_escape_or_repin_stale_sources_on_initialize(self):
        self.row['native_authority']['declaration']['path'] = '../outside'
        with self.assertRaises(ValueError):
            self.check()
        self.row['native_authority']['declaration'] = pin(self.root, 'declaration.rs')
        (self.root / 'declaration.rs').write_text('changed', encoding='utf-8')
        initialize(self.root, self.data)
        with self.assertRaisesRegex(ValueError, 'Stale'):
            self.check()


class CurrentSourceCatalogGate(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'browser.js').write_bytes(b'current synthetic browser source')
        # True equals 1 in Python: a one-byte fixture makes the strict int guard observable.
        (self.root / 'native.rs').write_bytes(b'N')
        self.catalogs = {}
        for relative, schema in SOURCE_CATALOGS:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            refs = []
            for name in ('native.rs', 'browser.js'):
                ref = pin(self.root, name)
                ref['byte_length'] = (self.root / name).stat().st_size
                refs.append(ref)
            row = ({'native_declaration': 'native.rs', 'native_dispatch': {'path': 'native.rs'},
                    'native_kernel_sources': ['native.rs'], 'browser_sources': ['browser.js']}
                   if schema == 'bisect-practitioner-actions-v1' else
                   {'native_authority': ['native.rs'], 'browser_sources': ['browser.js'],
                    'consumer_sources': ['native.rs']})
            self.catalogs[relative] = {'schema_version': schema, 'authority': refs, 'rows': [row]}
        self.write_catalogs()

    def write_catalogs(self):
        for relative, data in self.catalogs.items():
            (self.root / relative).write_text(json.dumps(data), encoding='utf-8')

    def test_both_current_source_catalogs_validate(self):
        validate_current_source_catalogs(self.root)

    def test_stale_source_is_rejected_without_refresh(self):
        before = {relative: (self.root / relative).read_bytes() for relative in self.catalogs}
        (self.root / 'browser.js').write_bytes(b'changed source')
        with self.assertRaisesRegex(ValueError, 'Stale'):
            validate_current_source_catalogs(self.root)
        self.assertEqual(before, {relative: (self.root / relative).read_bytes()
                                  for relative in self.catalogs})

    def test_each_catalog_is_required_and_checked(self):
        for relative in self.catalogs:
            with self.subTest(catalog=relative):
                self.write_catalogs()
                (self.root / relative).unlink()
                with self.assertRaisesRegex(ValueError, 'Missing current source catalog'):
                    validate_current_source_catalogs(self.root)
        self.write_catalogs()

    def test_catalog_metadata_and_paths_cannot_bypass_validation(self):
        original = copy.deepcopy(self.catalogs)
        for relative in self.catalogs:
            for mutation in ('schema', 'root-list', 'empty', 'duplicate', 'length', 'boolean-length',
                             'unsafe', 'digest', 'missing-reference', 'missing-group'):
                with self.subTest(catalog=relative, mutation=mutation):
                    self.catalogs = copy.deepcopy(original)
                    data = self.catalogs[relative]
                    if mutation == 'schema':
                        data['schema_version'] = 'unknown'
                    elif mutation == 'root-list':
                        self.catalogs[relative] = []
                    elif mutation == 'empty':
                        data['authority'] = []
                    elif mutation == 'duplicate':
                        data['authority'] *= 2
                    elif mutation == 'length':
                        data['authority'][0]['byte_length'] += 1
                    elif mutation == 'boolean-length':
                        data['authority'][0]['byte_length'] = True
                    elif mutation == 'unsafe':
                        data['authority'][0]['path'] = '../outside.js'
                    elif mutation == 'digest':
                        data['authority'][0]['sha256'] = '0' * 64
                    elif mutation == 'missing-reference':
                        data['authority'].pop()
                    elif mutation == 'missing-group':
                        data['rows'][0].pop('browser_sources')
                    self.write_catalogs()
                    with self.assertRaises(ValueError):
                        validate_current_source_catalogs(self.root)


if __name__ == '__main__':
    unittest.main()
