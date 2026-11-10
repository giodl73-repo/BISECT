"""Mutations of build prerequisites that previously failed cold parity builds."""
from pathlib import Path
import tempfile
import unittest

from check_build_gate_dependencies import FIXTURE_PRODUCERS, FIXTURE_EMISSIONS, STATE_SMC_MARKERS, check


class BuildGateDependencies(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'scripts/web').mkdir(parents=True)

    def script(self, name, text=None):
        if text is None:
            text = ''
            if name == 'test_smc_real_state_wasm.mjs':
                text = '\n'.join(STATE_SMC_MARKERS)
            if name in FIXTURE_EMISSIONS:
                directory, filename = FIXTURE_EMISSIONS[name]
                text = f"const root='{directory}';await fs.writeFile(root+'/{filename}','{{}}');"
        (self.root / 'scripts/web' / name).write_text(text, encoding='utf-8')

    def builder(self, text):
        (self.root / 'scripts/web/build_wasm.ps1').write_text(text, encoding='utf-8')
        return check(self.root)

    @staticmethod
    def reference(profile, name='examples/seed'):
        return 'run(' + repr(f'target/{profile}/{name}.exe') + ');'

    def test_both_native_profiles_must_be_built_before_use(self):
        self.script('native.mjs', self.reference('debug') + self.reference('release'))
        for omitted in ('debug', 'release'):
            lines = ['cargo build -p fixture --example seed' + (' --release' if profile == 'release' else '')
                     for profile in ('debug', 'release') if profile != omitted]
            result = self.builder('\n'.join(lines + ['node scripts/web/native.mjs']))
            self.assertEqual(len(result['errors']), 1)
            self.assertIn('/' + omitted + '/', result['errors'][0]['executable'])
        result = self.builder('cargo build -p fixture --example seed\n'
                              'cargo build -p fixture --release --example seed\n'
                              'node scripts/web/native.mjs\n')
        self.assertEqual(result['status'], 'passed')

    def test_later_build_does_not_satisfy_earlier_use(self):
        self.script('native.mjs', self.reference('debug'))
        result = self.builder('node scripts/web/native.mjs\ncargo build -p fixture --example seed\n')
        self.assertEqual(len(result['errors']), 1)

    def test_cargo_run_is_a_producer_and_comments_are_not(self):
        self.script('native.mjs', self.reference('debug'))
        result = self.builder('# cargo build -p fixture --example seed\nnode scripts/web/native.mjs\n')
        self.assertEqual(result['status'], 'failed')
        result = self.builder('cargo run -p fixture --example seed -- fixture.json\nnode scripts/web/native.mjs\n')
        self.assertEqual(result['status'], 'passed')

    def test_all_observed_fixture_dependencies_reject_missing_and_reversed_order(self):
        for consumer, producer in FIXTURE_PRODUCERS.items():
            with self.subTest(consumer=consumer):
                self.script(consumer)
                self.script(producer)
                self.assertEqual(self.builder(f'node scripts/web/{consumer}\n')['status'], 'failed')
                self.assertEqual(self.builder(f'node scripts/web/{consumer}\nnode scripts/web/{producer}\n')['status'], 'failed')
                self.assertEqual(self.builder(f'node scripts/web/{producer}\nnode scripts/web/{consumer}\n')['status'], 'passed')

    def test_current_producer_write_removal_is_rejected_before_runtime(self):
        producer = 'test_config_build_wasm.mjs'
        source = (Path(__file__).parent / producer).read_text(encoding='utf-8-sig')
        writer = "await fs.writeFile(root+'/browser-retry.bisect',JSON.stringify(fixture));"
        self.assertEqual(source.count(writer), 1)
        prefix = ('cargo build -p fixture --release --example execute_tool_request\n'
                  'cargo build -p fixture --example config_seed_reference\n')
        command = prefix + f'node scripts/web/{producer}\n'
        self.script(producer, source.replace(writer, ''))
        result = self.builder(command)
        self.assertEqual(len(result['errors']), 1)
        self.assertEqual(result['errors'][0]['artifact'],
                         'target/config-build-wasm/browser-retry.bisect')
        self.script(producer, source)
        self.assertEqual(self.builder(command)['status'], 'passed')

    def test_real_state_smc_output_writers_are_required(self):
        producer = 'test_smc_real_state_wasm.mjs'
        source = (Path(__file__).parent / producer).read_text(encoding='utf-8-sig')
        command = ('cargo build -p fixture --release --example execute_tool_request\n'
                   f'node scripts/web/{producer} dist/exact-site\n')
        for marker in STATE_SMC_MARKERS:
            with self.subTest(marker=marker):
                self.assertIn(marker, source)
                self.script(producer, source.replace(marker, '', 1))
                result = self.builder(command)
                self.assertEqual(len(result['errors']), 1)
                self.assertIn('SMC fixture loop or file writer', result['errors'][0]['error'])
        self.script(producer, source)
        self.assertEqual(self.builder(command)['status'], 'passed')

    def test_missing_invoked_source_is_an_error(self):
        result = self.builder('node scripts/web/missing.mjs\n')
        self.assertEqual(result['errors'][0]['error'], 'invoked script is missing')

    def test_package_default_bin_producer(self):
        crate = self.root / 'crates/fixture'
        (crate / 'src').mkdir(parents=True)
        (crate / 'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\n', encoding='utf-8')
        (crate / 'src/main.rs').write_text('fn main() {}', encoding='utf-8')
        self.script('native.mjs', self.reference('release', 'fixture'))
        result = self.builder('cargo build -p fixture --release\nnode scripts/web/native.mjs\n')
        self.assertEqual(result['status'], 'passed')


if __name__ == '__main__':
    unittest.main()
