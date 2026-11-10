"""Seal State diagnostic orchestration and test both unchanged engine shells."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]


def pin(p):
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(p.read_bytes()).hexdigest(), byte_length=p.stat().st_size)


def main():
    overlays = ['native-state-diagnostics.js', 'native-state-workflow.js', 'native-state-job.js', 'state-run-batch.js', 'workbench.js']
    tests = ['test_state_diagnostic_flags.mjs', 'test_state_diagnostic_flags_browser.mjs', 'test_native_state_workflow.mjs', 'test_native_states_workers.mjs', 'test_native_fetch_install_browser.mjs']
    sources = [ROOT / 'web/lab' / n for n in overlays] + [ROOT / 'scripts/web' / n for n in tests + ['build_state_diagnostic_preview.py']] + [ROOT / n for n in ['scripts/web/fixtures/native-state-source.json', 'crates/bisect-cli/src/status.rs', 'crates/bisect-cli/src/runner/mod.rs', 'crates/bisect-cli/src/main.rs', 'crates/bisect-runner/src/state_args.rs', 'target/release/bisect.exe', 'target/debug/examples/geometric_state_reference.exe']]
    before = [pin(p) for p in sources]
    output = ROOT / 'target/state-diagnostic-flags' / ('package-' + str(time.time_ns()))
    output.mkdir(parents=True)
    record = dict(status='running', checks=[], packages={}, source_files=before, deployed=False, acceptance_v1=False)

    def run(name, command, env):
        print(name, flush=True)
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = output / (name + '.log')
        log.write_text(result.stdout + result.stderr, encoding='utf-8')
        record['checks'].append(dict(name=name, exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)

    try:
        for profile in ['default', 'c-sidecar']:
            base = ROOT / 'dist' / ('wasm-fetch-outcome-history-' + profile + '-preview')
            package = ROOT / 'dist' / ('wasm-state-diagnostic-' + profile + '-preview')
            assert base.is_dir() and not package.exists(), package
            assert not any(p.is_symlink() for p in base.rglob('*'))
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            shutil.copytree(base, package, copy_function=os.link)
            for relative in overlays + ['toolkit/' + n for n in overlays] + ['offline-manifest.json']:
                target = package / relative
                temporary = target.with_name(target.name + '.diagnostic-tmp')
                shutil.copyfile(target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / target.name, temporary)
                os.replace(temporary, target)
            unit = output / (profile + '-unit')
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'), BISECT_STATE_DIAGNOSTIC_OUTPUT=str(unit), BISECT_STATE_WORKFLOW_OUTPUT=str(output / (profile + '-legacy-workflow')), BISECT_FETCH_INSTALL_OUTPUT=str(output / (profile + '-fetch-browser')))
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)], env)
            assert all(pin(ROOT / p['path']) == p for p in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').glob('*.js'):
                for directory in [package, package / 'toolkit']:
                    assert pin(directory / source.name)['sha256'] == pin(source)['sha256'], source
            sealed = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            record['packages'][profile] = sealed
            for label, command in [
                ('assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')]),
                ('offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)]),
                ('unit', ['node', 'scripts/web/test_state_diagnostic_flags.mjs']),
                ('legacy-workflow', ['node', 'scripts/web/test_native_state_workflow.mjs', '--core-only']),
                ('browser', ['node', 'scripts/web/test_state_diagnostic_flags_browser.mjs', '--package', str(package / 'toolkit'), '--fixtures', str(unit), '--output', str(output / (profile + '-browser'))]),
                ('fetch-browser', ['node', 'scripts/web/test_native_fetch_install_browser.mjs', '--package', str(package / 'toolkit')]),
                ('batch-workers', ['node', 'scripts/web/test_native_states_workers.mjs']),
            ]:
                run(profile + '-' + label, command, env)
            assert all(pin(ROOT / p['path']) == p for p in sealed), 'Tested package changed'
        record['status'] = 'passed'
    except Exception as error:
        record.update(status='failed', error=str(error))
        raise
    finally:
        record['source_unchanged'] = [pin(p) for p in sources] == before
        destination = output / 'evidence.json'
        destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)
        assert record['source_unchanged'], 'Source changed during gates'


if __name__ == '__main__':
    main()
