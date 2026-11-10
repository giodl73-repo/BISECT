"""Run the frozen selected-byte replay regression against an immutable package.

This is a regression gate, not fresh native filesystem or full release acceptance.
All generated projects, browser profiles and receipts remain outside the package.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--package', type=Path, required=True)
parser.add_argument('--host-sequence', type=Path, required=True)
parser.add_argument('--native-map-reference', type=Path, required=True)
parser.add_argument('--native-map-adapter', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--c-sidecar', action='store_true')
parser.add_argument('--browser', action='store_true')
args = parser.parse_args()
scripts = Path(__file__).resolve().parent
package, host, output = args.package.resolve(), args.host_sequence.resolve(), args.output.resolve()
assert package.is_dir() and (package/'bisect_wasm.wasm').is_file()
assert host.is_file()
assert not output.is_relative_to(package) and not package.is_relative_to(output)
assert not output.exists(), 'Preserve existing evidence; choose a fresh output path.'
if args.browser:
    assert os.environ.get('BISECT_PLAYWRIGHT_PACKAGES')
if args.c_sidecar:
    assert (package/'c-metis/metis-sidecar.json').is_file()
output.mkdir(parents=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

def package_hashes():
    return {p.relative_to(package).as_posix(): sha(p)
            for p in sorted(package.rglob('*')) if p.is_file()}

original = package_hashes()
common = ['--wasm', str(package/'bisect_wasm.wasm')]
if args.c_sidecar:
    common += ['--c-sidecar', str(package/'c-metis')]
steps = [('runtime', ['node', str(scripts/'test_native_replay_export.mjs'),
                      *common, '--host-sequence', str(host), '--native-map-reference', str(args.native_map_reference.resolve()), '--native-map-adapter', str(args.native_map_adapter.resolve()), '--output', str(output/'runtime')])]
if args.browser:
    steps.append(('browser', ['node', str(scripts/'test_native_replay_export_browser.mjs'),
                              *common, '--site', str(package), '--output', str(output/'browser')]))
sources = [Path(__file__).resolve(), scripts/'test_native_replay_export.mjs',
           scripts/'test_native_replay_export_browser.mjs',
           scripts/'fixtures/native-recorded-replay.mjs', scripts/'fixtures/fine-unit-map.mjs',
           scripts/'workbench_project_open.mjs',
           *sorted((scripts/'fixtures/native-recorded-replay').glob('*.json')), *sorted((scripts/'fixtures/fine-unit-map').glob('*.json'))]
record = dict(schema='bisect-native-replay-export-package-v1', status='running',
              scope='Frozen selected-byte regression only; not fresh native filesystem, all methods/states, capacity or full release acceptance.',
              package=str(package), package_files=original, host_sha256=sha(host), native_map_reference_sha256=sha(args.native_map_reference.resolve()), native_map_adapter_sha256=sha(args.native_map_adapter.resolve()),
              sources={str(p): sha(p) for p in sources}, explicit_c=args.c_sidecar,
              browser_requested=args.browser, steps=[], full_parity=False)
try:
    for label, command in steps:
        with (output/(label+'.log')).open('wb') as log:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
        record['steps'].append(dict(name=label, command=command, exit_code=result.returncode,
                                    log_sha256=sha(output/(label+'.log'))))
        assert result.returncode == 0, f'{label} failed; retained log: {output/(label+".log")}'
    record['package_unchanged'] = package_hashes() == original
    assert record['package_unchanged']
    record['status'] = 'passed'
except BaseException as error:
    record.update(status='failed', error=repr(error))
    raise
finally:
    (output/'evidence.json').write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
print(json.dumps(dict(status=record['status'], root=str(output), browser_requested=args.browser)))
