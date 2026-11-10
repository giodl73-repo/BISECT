"""Stage and check the bounded local release-file import on existing engine profiles."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, digest


def pin(path):
    path = Path(path)
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=digest(path), byte_length=path.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suffix', choices=['preview', 'preview-v2'], default='preview')
    args = parser.parse_args()
    source_names = ['web/lab/workbench.js', 'web/lab/native-adjacency-bundle.js', 'web/lab/native-adjacency-files.js',
                    'scripts/web/test_native_adjacency_bundle.mjs', 'scripts/web/test_native_adjacency_bundle_browser.mjs',
                    'scripts/web/fixtures/native-workspace-case.mjs', 'scripts/web/build_native_adjacency_bundle_preview.py',
                    'crates/bisect-cli/src/fetch.rs', 'crates/bisect-wasm/src/native_adjacency.rs']
    source_pins = [pin(ROOT / name) for name in source_names]
    # Validation-host reserve, not a product input limit. Browsers use ephemeral
    # contexts here; no persistent Chrome profile or engine rebuild is needed.
    assert shutil.disk_usage(ROOT).free > 200 * 1024**2, 'Insufficient scope-check reserve'
    output = ROOT / 'target/native-adjacency-bundle' / ('package-' + str(time.time_ns()))
    output.mkdir(parents=True)
    checks, packages = [], {}
    def run(name, command, env=None):
        print(name, flush=True)
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                                text=True, encoding='utf-8', errors='replace')
        log = output / (name + '.log')
        log.write_text(result.stdout + result.stderr, encoding='utf-8')
        checks.append(dict(name=name, command=command, exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)
    record = dict(status='running', checks=checks, packages=packages, deployed=False, acceptance_v1=False)
    try:
        for profile in ['default', 'c-sidecar']:
            base = ROOT / 'dist' / ('wasm-install-outcome-' + profile + '-durable-preview')
            package = ROOT / 'dist' / ('wasm-native-adjacency-bundle-' + profile + '-' + args.suffix)
            assert base.is_dir() and not package.exists()
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            assert not any(p.is_symlink() for p in base.rglob('*'))
            shutil.copytree(base, package, copy_function=os.link)
            for relative in ['workbench.js', 'toolkit/workbench.js', 'native-adjacency-bundle.js',
                             'toolkit/native-adjacency-bundle.js', 'offline-manifest.json']:
                target = package / relative
                temporary = target.with_name(target.name + '.bundle-tmp')
                source = target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / target.name
                shutil.copyfile(source, temporary)
                os.replace(temporary, target)
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)])
            assert all(pin(ROOT / r['path']) == r for r in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').iterdir():
                if source.is_file() and source.suffix in ['.js', '.css']:
                    for directory in [package, package / 'toolkit']:
                        assert digest(directory / source.name) == digest(source), source
            packages[profile] = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'))
            run(profile + '-assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')])
            run(profile + '-offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)])
            unit = output / (profile + '-unit')
            run(profile + '-unit', ['node', 'scripts/web/test_native_adjacency_bundle.mjs'],
                dict(env, BISECT_BUNDLE_TEST_OUTPUT=str(unit)))
            browser = output / (profile + '-browser')
            run(profile + '-browser', ['node', 'scripts/web/test_native_adjacency_bundle_browser.mjs', '--package', str(package / 'toolkit')],
                dict(env, BISECT_BUNDLE_TEST_OUTPUT=str(browser)))
            assert all(pin(ROOT / r['path']) == r for r in packages[profile]), 'Tested package changed'
        record['status'] = 'passed'
    except Exception as error:
        record.update(status='failed', error=str(error))
        raise
    finally:
        assert [pin(ROOT / name) for name in source_names] == source_pins, 'Source changed during validation'
        record['source_files'] = source_pins
        record['native_host'] = pin(Path(os.environ.get('BISECT_TEST_NATIVE_TOOL', str(ROOT / 'target/release/examples/execute_tool_request.exe'))))
        destination = output / 'evidence.json'
        destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)


if __name__ == '__main__':
    main()
