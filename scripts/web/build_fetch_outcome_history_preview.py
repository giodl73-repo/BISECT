"""Seal and test the passive Fetch history overlay against both accepted folder shells."""
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
    overlay = ['installation-outcome-store.js', 'fetch-install-outcome-store.js', 'native-fetch-install-ui.js']
    tests = ['test_installation_outcome_store.mjs', 'test_fetch_install_outcome_store.mjs', 'test_native_fetch_install.mjs', 'test_native_fetch_install_browser.mjs', 'test_fetch_install_outcome_browser.mjs']
    sources = [ROOT / 'web/lab' / n for n in overlay] + [ROOT / 'scripts/web' / n for n in tests + ['build_fetch_outcome_history_preview.py']]
    before = [pin(p) for p in sources]
    output = ROOT / 'target/fetch-outcome-history' / ('package-' + str(time.time_ns()))
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
            base = ROOT / 'dist' / ('wasm-native-fetch-install-' + profile + '-preview-v2')
            package = ROOT / 'dist' / ('wasm-fetch-outcome-history-' + profile + '-preview')
            assert base.is_dir() and not package.exists(), package
            assert not any(p.is_symlink() for p in base.rglob('*'))
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            shutil.copytree(base, package, copy_function=os.link)
            for relative in overlay + ['toolkit/' + n for n in overlay] + ['offline-manifest.json']:
                target = package / relative
                temporary = target.with_name(target.name + '.history-tmp')
                shutil.copyfile(target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / target.name, temporary)
                os.replace(temporary, target)
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'), BISECT_FETCH_INSTALL_OUTPUT=str(output / (profile + '-folder-unit')), BISECT_FETCH_HISTORY_OUTPUT=str(output / (profile + '-history-browser')))
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)], env)
            assert all(pin(ROOT / p['path']) == p for p in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').glob('*.js'):
                for location in [package, package / 'toolkit']:
                    assert pin(location / source.name)['sha256'] == pin(source)['sha256'], source
            sealed = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            record['packages'][profile] = sealed
            for label, command in [
                ('assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')]),
                ('offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)]),
                ('old-history-unit', ['node', 'scripts/web/test_installation_outcome_store.mjs']),
                ('fetch-history-unit', ['node', 'scripts/web/test_fetch_install_outcome_store.mjs']),
                ('folder-unit', ['node', 'scripts/web/test_native_fetch_install.mjs']),
                ('folder-browser', ['node', 'scripts/web/test_native_fetch_install_browser.mjs', '--package', str(package / 'toolkit')]),
                ('history-browser', ['node', 'scripts/web/test_fetch_install_outcome_browser.mjs', '--package', str(package / 'toolkit')]),
            ]:
                run(profile + '-' + label, command, dict(env, BISECT_FETCH_INSTALL_OUTPUT=str(output / (profile + '-' + label))))
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
        assert record['source_unchanged'], 'Source changed during tests'


if __name__ == '__main__':
    main()
