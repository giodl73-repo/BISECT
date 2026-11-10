"""Stage full native Fetch release ZIP transport without changing engine bytes."""
import json
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from evidence_blob_store import ROOT, digest

def pin(path):
    p = Path(path)
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=digest(p), byte_length=p.stat().st_size)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suffix', choices=['preview', 'preview-v2'], default='preview')
    args = parser.parse_args()
    overlays = ['adjacency-release.js', 'adjacency-release-ui.js', 'native-fetch-release-package.js']
    names = ['web/lab/' + n for n in overlays] + [
        'cloudflare/adjacency-release-bridge.mjs', 'cloudflare/source-worker.mjs',
        'scripts/web/test_native_fetch_release_package.mjs', 'scripts/web/test_adjacency_release_browser.mjs',
        'scripts/web/build_native_fetch_release_preview.py', 'crates/bisect-cli/src/fetch.rs',
        'crates/bisect-core/src/fetch_plan.rs', 'target/release/bisect.exe', 'data/manifest.json']
    source_pins = [pin(ROOT / n) for n in names]
    output = ROOT / 'target/adjacency-release-bundle' / ('package-' + str(time.time_ns()))
    output.mkdir(parents=True)
    checks, packages = [], {}
    record = dict(status='running', checks=checks, packages=packages, deployed=False, acceptance_v1=False)
    def run(name, command, env=None):
        print(name, flush=True)
        r = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = output / (name + '.log'); log.write_text(r.stdout + r.stderr, encoding='utf-8')
        checks.append(dict(name=name, command=command, exit_code=r.returncode, log=pin(log)))
        assert r.returncode == 0, str(log)
    try:
        for profile in ['default', 'c-sidecar']:
            base = ROOT / 'dist' / ('wasm-election-source-format-' + profile + '-preview')
            package = ROOT / 'dist' / ('wasm-native-fetch-release-' + profile + '-' + args.suffix)
            assert base.is_dir() and not package.exists()
            original = [pin(p) for p in sorted(base.rglob('*')) if p.is_file()]
            assert not any(p.is_symlink() for p in base.rglob('*'))
            shutil.copytree(base, package, copy_function=os.link)
            for relative in overlays + ['toolkit/' + n for n in overlays] + ['offline-manifest.json']:
                target = package / relative
                tmp = target.with_name(target.name + '.fetch-tmp')
                shutil.copyfile(target if relative == 'offline-manifest.json' else ROOT / 'web/lab' / target.name, tmp)
                os.replace(tmp, target)
            run(profile + '-offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)])
            assert all(pin(ROOT / p['path']) == p for p in original), 'Accepted base changed'
            for source in (ROOT / 'web/lab').iterdir():
                if source.is_file() and source.suffix in ['.js', '.css']:
                    for directory in [package, package / 'toolkit']:
                        assert digest(directory / source.name) == digest(source), source
            packages[profile] = [pin(p) for p in sorted(package.rglob('*')) if p.is_file()]
            env = dict(os.environ, BISECT_TEST_WASM=str(package / 'toolkit/bisect_wasm.wasm'))
            run(profile + '-assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')])
            run(profile + '-offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)])
            run(profile + '-native-wildcard', ['node', 'scripts/web/test_native_fetch_release_package.mjs'],
                dict(env, BISECT_FETCH_BUNDLE_OUTPUT=str(output / (profile + '-native-wildcard'))))
            for label, script in [('release-input-unit', 'test_adjacency_release.mjs'), ('release-bridge-regression', 'test_adjacency_release_bridge.mjs')]:
                run(profile + '-' + label, ['node', 'scripts/web/' + script], dict(env, BISECT_RELEASE_TEST_OUTPUT=str(output / (profile + '-' + label))))
            run(profile + '-browser', ['node', 'scripts/web/test_adjacency_release_browser.mjs', '--native-bundle', '--package', str(package / 'toolkit')],
                dict(env, BISECT_RELEASE_TEST_OUTPUT=str(output / (profile + '-browser'))))
            run(profile + '-election-browser-regression', ['node', 'scripts/web/test_election_source_format_browser.mjs', '--package', str(package / 'toolkit')],
                dict(env, BISECT_ELECTION_FORMAT_OUTPUT=str(output / (profile + '-election-browser'))))
            assert all(pin(ROOT / p['path']) == p for p in packages[profile]), 'Tested package changed'
        record['status'] = 'passed'
    except Exception as error:
        record.update(status='failed', error=str(error)); raise
    finally:
        record['source_files'] = source_pins
        record['source_unchanged'] = [pin(ROOT / n) for n in names] == source_pins
        destination = output / 'evidence.json'; destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
        print(destination, flush=True)
        assert record['source_unchanged'], 'Source changed during validation'

if __name__ == '__main__':
    main()
