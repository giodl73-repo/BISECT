"""Run prepared-state native, ZIP and optional browser checks on one package.

Portable Rust paths only, including when the package also offers a C backend.
All inputs are maintained fixtures; generated outputs stay outside the package.
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
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--native', type=Path, default=Path('target/release/examples'))
parser.add_argument('--browser', action='store_true')
args = parser.parse_args()
scripts = Path(__file__).resolve().parent
package, output, native = args.package.resolve(), args.output.resolve(), args.native.resolve()
assert package.is_dir() and (package/'index.html').is_file()
assert (package/'bisect_wasm.wasm').is_file()
assert not output.is_relative_to(package) and not package.is_relative_to(output)
assert not output.exists(), 'Preserve existing evidence; choose a fresh output path.'
if args.browser:
    assert os.environ.get('BISECT_PLAYWRIGHT_PACKAGES'), 'Set BISECT_PLAYWRIGHT_PACKAGES before browser checks.'
for name in ['execute_tool_request.exe','execute_tool_sequence.exe']:
    assert (native/name).is_file(), name
output.mkdir(parents=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
def package_hashes():
    return {p.relative_to(package).as_posix():sha(p) for p in sorted(package.rglob('*')) if p.is_file()}
original = package_hashes()
env = os.environ.copy()
env.update(BISECT_PREPARED_PACKAGE=str(package), BISECT_PREPARED_NATIVE=str(native),
           BISECT_PREPARED_OUTPUT=str(output/'batch'),
           BISECT_PREPARED_BROWSER_OUTPUT=str(output/'browser'),
           BISECT_PREPARED_FIXTURES=str(scripts/'fixtures/prepared-state'))
steps = [('batch',['node',str(scripts/'test_prepared_state_batch.mjs')]),
         ('zip',[sys.executable,str(scripts/'test_prepared_state_zip.py')])]
if args.browser:
    steps.append(('browser',['node',str(scripts/'test_prepared_state_browser.mjs')]))
record = {'scope':'Packaged portable Rust prepared-state workflows; no explicit C backend or full parity acceptance',
          'package':str(package), 'wasm_sha256':original['bisect_wasm.wasm'],
          'package_files':original, 'steps':[], 'browser_requested':args.browser,
          'inputs':{str(p):sha(p) for p in sorted((scripts/'fixtures/prepared-state').glob('*'))},
          'native':{name:sha(native/name) for name in ['execute_tool_request.exe','execute_tool_sequence.exe']},
          'sources':{str(p):sha(p) for p in [Path(__file__).resolve(), scripts/'test_prepared_state_batch.mjs',scripts/'test_prepared_state_browser.mjs',scripts/'test_prepared_state_zip.py']}}
try:
    for label, command in steps:
        with (output/(label+'.log')).open('wb') as log:
            result = subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT)
        record['steps'].append({'name':label,'command':command,'exit_code':result.returncode,
                                'log_sha256':sha(output/(label+'.log'))})
        assert result.returncode == 0, f'{label} failed; retained log: {output/(label+".log")}'
    record['package_unchanged'] = package_hashes() == original
    assert record['package_unchanged']
    record['status'] = 'passed'
except BaseException as error:
    record.update(status='failed',error=repr(error))
    raise
finally:
    (output/'evidence.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':record['status'],'root':str(output),'browser_requested':args.browser}))
