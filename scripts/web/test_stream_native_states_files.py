"""Independent ZIP/hash/assignment readback for shared native States source streams."""
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile

root = Path('target/stream-native-states')
sha = lambda value: hashlib.sha256(value).hexdigest()

def check(raw, state, run):
    with zipfile.ZipFile(io.BytesIO(raw)) as outer:
        assert outer.testzip() is None
        with zipfile.ZipFile(io.BytesIO(outer.read(state + '/prepared-run.zip'))) as archive:
            assert archive.testzip() is None
            paths = archive.namelist()
            assert len(paths) == len(set(paths))
            assert all(not p.startswith('/') and '..' not in p.split('/') for p in paths)
            recipe = json.loads(archive.read('states-flags/recipe.json'))
            identities = json.loads(archive.read('states-flags/identities.json'))
            assert recipe['command_kind'] == 'states'
            assert sha(json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()) == identities['source_sha256']
            assert recipe == run['native_states']['recipe']
            assert sha(run['request_source']['source']['text'].encode()) == identities['request_sha256']
            assert archive.read('states-flags/arguments.json').decode() == recipe['command']['arguments_json']
            assert json.loads(archive.read('states-flags/compilation.json'))['native_command']['workers'] == '4'
            assert archive.read('plan.rplan').decode() == run['prepared']['export']['document_json']
            assert archive.read('context.rctx').decode() == run['prepared']['export']['context_json']
            plan = json.loads(archive.read('plan.rplan'))
            assert plan['plan']['k'] == (2 if state == 'RI' else 1)
            assert len(plan['plan']['assignment']) == (250 if state == 'RI' else 2)
            assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
            assert [assignments[str(i)] for i in range(len(plan['plan']['assignment']))] == [d + 1 for d in plan['plan']['assignment']]
            opts = json.loads(archive.read('native-state/engine-options.json'))
            assert opts['cvd_iters'] == 3 and opts['cvd_metric'] == 'geographic'

for state in ['RI', 'VT']:
    run = json.loads((root / (state + '-execution.json')).read_text(encoding='utf-8'))['result']
    check((root / (state + '-output.zip')).read_bytes(), state, run)

if '--skip-browser' not in sys.argv:
    browser = Path('target/stream-native-states-browser')
    project = json.loads((browser / 'completed.bisect').read_text(encoding='utf-8'))
    files = json.loads((browser / 'installed-index.json').read_text(encoding='utf-8'))
    def read(name):
        target = (browser / 'captured' / name).resolve()
        assert target.is_relative_to((browser / 'captured').resolve())
        data = target.read_bytes()
        assert len(data) == files[name]['byte_length'] and sha(data) == files[name]['sha256']
        return data
    assert project['files']['nativeStateSourceSettings']['command_kind'] == 'states'
    for row in project['result']['states']:
        state = row['state']
        namespace = row['native_output']['target_namespace']
        assert namespace + '/manifest.tmp' not in files
        receipt = json.loads(read(namespace + '/browser/install-receipt.json'))
        for entry in receipt['files']:
            assert '..' not in entry['path'].split('/')
            data = read(namespace + '/' + entry['path'])
            assert len(data) == entry['byte_length'] and sha(data) == entry['sha256']
        run = json.loads(read(namespace + '/browser/accepted-run.json'))
        check(read(state + '/state-output.zip'), state, run)
        jobs = json.loads(read(state + '/state-run-jobs.json'))
        assert jobs[0]['native_state']['command_kind'] == 'states'
        assert jobs[0]['native_state']['command']['arguments_json'] == project['files']['statesCommandInputs']['arguments_json']
print('PASS native States stream archives: exact source/flag/request identities, original RI 250-tract plan, embedded congressional counts, CVD controls, native index assignments; browser native-layout files checked when enabled')
