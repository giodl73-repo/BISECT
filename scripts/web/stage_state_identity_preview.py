"""Stage an owned naming-review preview; this is not full workflow acceptance."""
import argparse
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
    raw = p.read_bytes()
    return dict(path=p.relative_to(ROOT).as_posix(), sha256=hashlib.sha256(raw).hexdigest(), byte_length=len(raw))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--wasm', required=True)
    parser.add_argument('--profile', choices=['default', 'c-sidecar'], default='default')
    parser.add_argument('--workflow', action='store_true')
    args = parser.parse_args()
    wasm = (ROOT / args.wasm).resolve()
    assert any(wasm.is_relative_to(ROOT / folder) for folder in ['target/state-identity', 'target/state-name-workflow', 'target/state-report-workflow', 'target/cffi-legacy-gates']) and wasm.is_file()
    authority = ROOT / 'target/partition-timing/package-1793046626258235600/evidence.json'
    package_record = json.loads(authority.read_text(encoding='utf-8'))
    base = ROOT / 'dist' / ('wasm-partition-timing-' + args.profile + '-preview-v2')
    original = package_record['packages'][args.profile]
    assert all(pin(ROOT / r['path']) == r for r in original)
    assert not any(p.is_symlink() for p in base.rglob('*'))
    output = ROOT / ('target/state-name-workflow' if args.workflow else 'target/state-identity') / ('preview-' + str(time.time_ns()))
    output.mkdir()
    package = output / 'package'
    shutil.copytree(base, package, copy_function=os.link)

    def replace(source, destination):
        temp = destination.with_name(destination.name + '.identity-tmp')
        shutil.copyfile(source, temp)
        os.replace(temp, destination)

    def write(destination, value):
        temp = destination.with_name(destination.name + '.identity-tmp')
        temp.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
        os.replace(temp, destination)

    for directory in [package, package / 'toolkit']:
        for source in (ROOT / 'web/lab').glob('*.js'):
            replace(source, directory / source.name)
        replace(wasm, directory / 'bisect_wasm.wasm')
        catalog = directory / 'catalog.json'
        if catalog.is_file():
            value = json.loads(catalog.read_text(encoding='utf-8'))
            value['wasm_sha256'] = pin(wasm)['sha256']
            write(catalog, value)
        proof = directory / 'native-proof-build.json'
        if proof.is_file():
            value = json.loads(proof.read_text(encoding='utf-8'))
            for name, expected in value['wasm_dependencies'].items():
                if name == 'bisect_wasm.wasm':
                    value['wasm_dependencies'][name] = pin(wasm)['sha256']
                else:
                    assert pin(directory / name)['sha256'] == expected
            write(proof, value)
    replace(ROOT / 'web/lab/workbench.html', package / 'toolkit/index.html')
    replace(package / 'offline-manifest.json', package / 'offline-manifest.json')
    checks = []
    for name, command in [('offline-build', [sys.executable, 'scripts/web/build_offline_manifest.py', str(package)]), ('assets', ['node', 'scripts/web/test_toolkit_assets.mjs', str(package / 'toolkit')]), ('offline', ['node', 'scripts/web/test_offline_package.mjs', str(package)])]:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        log = output / (name + '.log')
        log.write_text(result.stdout + result.stderr, encoding='utf-8')
        checks.append(dict(name=name, exit_code=result.returncode, log=pin(log)))
        assert result.returncode == 0, str(log)
    assert all(pin(ROOT / r['path']) == r for r in original), 'Accepted base changed'
    record = dict(status='passed-stage-build', profile=args.profile, base_authority=pin(authority), wasm=pin(wasm), checks=checks, package=package.relative_to(ROOT).as_posix(), package_files=[pin(p) for p in sorted(package.rglob('*')) if p.is_file()], deployed=False, acceptance_v1=False, scope=('Owned selected-binary State name workflow preview; full registered, source/method/host and release acceptance remain open.' if args.workflow else 'Owned naming-review preview with assets/offline closure. Native/browser workflow execution, full registered gates and release acceptance remain open.'))
    (output / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(output, flush=True)


if __name__ == '__main__':
    main()
