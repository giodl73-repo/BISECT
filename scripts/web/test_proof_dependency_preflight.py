from pathlib import Path
import hashlib
import json
import os
import shutil
import time
from proof_dependency_preflight import CHECKER_SOURCES, validate_checker, validate_solver

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / 'target/proof-preflight-tests' / ('run-' + str(time.time_ns()))
OUT.mkdir(parents=True, exist_ok=False)
CHECKER = REPO / 'target/proof-checker-wasm-build/wasm32-unknown-unknown/release'
SOLVER = REPO / 'target/roundingsat-wasm'
rows = []


def fixture(name, kind, omit=None):
    destination = OUT / name
    destination.mkdir()
    source = CHECKER if kind == 'checker' else SOLVER
    names = ['proof-checker-build.json', 'bisect_proof_checker.wasm'] if kind == 'checker' else ['build-receipt.json', 'roundingsat.mjs', 'roundingsat.wasm']
    for file in names:
        if file != omit:
            shutil.copy2(source / file, destination / file)
    return destination


def changed_receipt(directory, kind, mutate):
    p = directory / ('proof-checker-build.json' if kind == 'checker' else 'build-receipt.json')
    value = json.loads(p.read_text(encoding='utf-8'))
    mutate(value)
    p.write_text(json.dumps(value), encoding='utf-8')


def refuses(name, call, reason):
    try:
        call()
    except (ValueError, OSError) as error:
        assert reason in str(error), (name, error)
        rows.append({'case': name, 'refused': True, 'reason': str(error)})
    else:
        raise AssertionError(name + ' unexpectedly accepted')


protected = [CHECKER / 'proof-checker-build.json', CHECKER / 'bisect_proof_checker.wasm', SOLVER / 'build-receipt.json', SOLVER / 'roundingsat.wasm', SOLVER / 'roundingsat.mjs']
before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in protected}
baseline = {'checker': validate_checker(REPO), 'solver': validate_solver(REPO)}
for field in CHECKER_SOURCES:
    directory = fixture('stale-' + field, 'checker')
    changed_receipt(directory, 'checker', lambda r, field=field: r.update({field: '0' * 64}))
    refuses(field, lambda: validate_checker(REPO, directory), 'Stale checker source')
for kind in ['checker', 'solver']:
    validate = validate_checker if kind == 'checker' else validate_solver
    directory = fixture(kind + '-schema', kind)
    changed_receipt(directory, kind, lambda r: r.update(schema='unsupported'))
    refuses(kind + '-schema', lambda: validate(REPO, directory), 'Unsupported')
    directory = fixture(kind + '-revision', kind)
    changed_receipt(directory, kind, lambda r, kind=kind: r.update({('upstream_commit' if kind == 'checker' else 'commit'): '0' * 40}))
    refuses(kind + '-revision', lambda: validate(REPO, directory), 'revision mismatch')
    artifact = 'bisect_proof_checker.wasm' if kind == 'checker' else 'roundingsat.wasm'
    directory = fixture(kind + '-missing-artifact', kind, omit=artifact)
    refuses(kind + '-missing-artifact', lambda: validate(REPO, directory), artifact)
    directory = fixture(kind + '-corrupt-artifact', kind)
    file = directory / artifact
    data = bytearray(file.read_bytes())
    data[-1] ^= 1
    file.write_bytes(data)
    refuses(kind + '-corrupt-artifact', lambda: validate(REPO, directory), 'artifact mismatch')
directory = fixture('checker-provenance', 'checker')
changed_receipt(directory, 'checker', lambda r: r.update(vendor_provenance_sha256='0' * 64))
refuses('checker-provenance', lambda: validate_checker(REPO, directory), 'provenance mismatch')
directory = fixture('solver-glue-corruption', 'solver')
with (directory / 'roundingsat.mjs').open('ab') as f:
    f.write(b'\n// altered\n')
refuses('solver-glue-corruption', lambda: validate_solver(REPO, directory), 'artifact mismatch')
directory = fixture('solver-unknown-member', 'solver')
changed_receipt(directory, 'solver', lambda r: r['files'].update({'../escape.wasm': {}}))
refuses('solver-unknown-member', lambda: validate_solver(REPO, directory), 'file inventory')
# Git status can refresh its index even when the checkout is clean.
# Copy the real pinned solver so the deliberate stat and source changes cannot
# touch the configured solver or rely on the caller disabling optional writes.
solver_relative = Path('target/proof-toolchain/roundingsat')
scratch = OUT / 'readonly-solver-repo'
scratch_solver = scratch / solver_relative
shutil.copytree(REPO / solver_relative, scratch_solver)
boost = Path('target/proof-toolchain/boost/LICENSE_1_0.txt')
(scratch / boost).parent.mkdir(parents=True)
shutil.copy2(REPO / boost, scratch / boost)
index = scratch_solver / '.git/index'
index_before = hashlib.sha256(index.read_bytes()).hexdigest()
license_file = scratch_solver / 'LICENSE'
stamp = license_file.stat()
os.utime(license_file, ns=(stamp.st_atime_ns, stamp.st_mtime_ns + 10_000_000_000))
saved_git_locks = os.environ.get('GIT_OPTIONAL_LOCKS')
try:
    os.environ['GIT_OPTIONAL_LOCKS'] = '1'
    assert validate_solver(scratch, SOLVER) == baseline['solver']
    assert hashlib.sha256(index.read_bytes()).hexdigest() == index_before, 'Read-only validator changed the solver index'
    with license_file.open('ab') as f:
        f.write(b'\n')
    refuses('solver-dirty-checkout', lambda: validate_solver(scratch, SOLVER), 'Solver source checkout is dirty')
    assert hashlib.sha256(index.read_bytes()).hexdigest() == index_before, 'Refusal changed the solver index'
finally:
    if saved_git_locks is None:
        os.environ.pop('GIT_OPTIONAL_LOCKS', None)
    else:
        os.environ['GIT_OPTIONAL_LOCKS'] = saved_git_locks
after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in protected}
assert before == after
assert len(rows) == 20
report = {'baseline': baseline, 'refusals': rows, 'protected_inputs_unchanged': before, 'scratch_solver_index_sha256': index_before, 'scope': 'Read-only prerequisite and refusal checks; not compilation or formal parity acceptance'}
(OUT / 'preflight-test-evidence.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(f'Candidate preflight: both real prerequisites pass; {len(rows)} mutation/missing-input refusals; five real artifacts and scratch solver index unchanged.')
