"""Read native States queue exports independently of the JavaScript writers."""
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile

sha = lambda value: hashlib.sha256(value).hexdigest()

def check(path, run):
    assert run['native_states_command']['workers'] == '100'
    assert run['workers']['requested'] == 100
    assert [row['state_code'] for row in run['states']] == ['VT', 'WY']
    with zipfile.ZipFile(path) as outer:
        assert outer.testzip() is None
        receipt = json.loads(outer.read('batch-receipt.json'))
        assert receipt['native_states_command'] == run['native_states_command']
        assert receipt['workers'] == run['workers']
        for row in run['states']:
            state = row['state_code']
            with zipfile.ZipFile(io.BytesIO(outer.read(state + '/prepared-run.zip'))) as archive:
                assert archive.testzip() is None
                names = archive.namelist()
                assert len(names) == len(set(names))
                assert all(not name.startswith('/') and '..' not in name.split('/') for name in names)
                recipe = json.loads(archive.read('states-flags/recipe.json'))
                identities = json.loads(archive.read('states-flags/identities.json'))
                assert recipe['command_kind'] == 'states'
                assert json.loads(recipe['command']['arguments_json']) == run['native_states_command']['arguments']
                assert sha(json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()) == identities['source_sha256']
                assert sha(archive.read('request.json')) == identities['request_sha256']
                assert archive.read('states-flags/arguments.json').decode() == recipe['command']['arguments_json']
                plan = json.loads(archive.read('plan.rplan'))
                assert archive.read('plan.rplan').decode() == row['result']['prepared']['export']['document_json']
                assert archive.read('context.rctx').decode() == row['result']['prepared']['export']['context_json']
                assert plan['plan']['units']['state'] == state and plan['plan']['k'] == 1
                assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
                assert [assignments[str(i)] for i in range(len(plan['plan']['assignment']))] == [d + 1 for d in plan['plan']['assignment']]
                options = json.loads(archive.read('native-state/engine-options.json'))
                assert options['cvd_iters'] == 3 and options['cvd_metric'] == 'geographic'

root = Path('target/native-states-queue')
check(root / 'accepted.zip', json.loads((root / 'accepted.json').read_text(encoding='utf-8')))
if '--skip-browser' not in sys.argv:
    browser = Path('target/native-states-queue-browser')
    project = json.loads((browser / 'complete.bisect').read_text(encoding='utf-8'))
    check(browser / 'queue.zip', project['result'])
    handoff = json.loads((browser / 'output-handoff.bisect').read_text(encoding='utf-8'))
    assert handoff['files']['nativeStateOutputInputs']['version'] == 'vQueue'
print('PASS native States queue ZIPs: master policy, exact original sources/flags/request identities, applied CVD options, native assignments, plan/context and browser output-version handoff when enabled')
