"""Verify multi-year Run exports with Python's independent ZIP reader."""
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile


def check(path, run):
    with zipfile.ZipFile(path) as outer:
        assert outer.testzip() is None
        receipt = json.loads(outer.read('run-years-receipt.json'))
        assert receipt['native_run_policy'] == run['native_run_policy']
        assert [row['year'] for row in receipt['years']] == ['2020', '2010', '2000']
        for year in run['years']:
            with zipfile.ZipFile(io.BytesIO(outer.read(year['year'] + '/state-outputs.zip'))) as batch:
                assert batch.testzip() is None
                child = year['result']
                assert json.loads(batch.read('batch-receipt.json'))['native_run_command'] == child['native_run_command']
                for row in child['states']:
                    with zipfile.ZipFile(io.BytesIO(batch.read(row['state_code'] + '/prepared-run.zip'))) as archive:
                        assert archive.testzip() is None
                        names = archive.namelist()
                        assert len(names) == len(set(names))
                        assert all(not n.startswith('/') and '..' not in n.split('/') for n in names)
                        recipe = json.loads(archive.read('run-flags/recipe.json'))
                        identities = json.loads(archive.read('run-flags/identities.json'))
                        assert recipe['command_kind'] == 'run'
                        assert recipe['source_inputs']['preparation']['input']['year'] == year['year']
                        assert json.loads(recipe['command']['arguments_json']) == child['native_run_command']['arguments']
                        assert archive.read('run-flags/arguments.json').decode() == recipe['command']['arguments_json']
                        encoded = json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()
                        assert hashlib.sha256(encoded).hexdigest() == identities['source_sha256']
                        assert hashlib.sha256(archive.read('request.json')).hexdigest() == identities['request_sha256']
                        prepared = row['result']['prepared']
                        assert archive.read('plan.rplan').decode() == prepared['export']['document_json']
                        assert archive.read('context.rctx').decode() == prepared['export']['context_json']
                        plan = json.loads(archive.read('plan.rplan'))['plan']
                        assert plan['units']['state'] == row['state_code'] and plan['k'] == 1
                        assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
                        assert [assignments[str(i)] for i in range(len(plan['assignment']))] == [d + 1 for d in plan['assignment']]


root = Path('target/native-run-queue')
check(root / 'accepted.zip', json.loads((root / 'accepted.json').read_text(encoding='utf-8')))
if '--skip-browser' not in sys.argv:
    browser = Path('target/native-run-queue-browser')
    check(browser / 'queue.zip', json.loads((browser / 'complete.bisect').read_text(encoding='utf-8'))['result'])
print('PASS native Run multi-year ZIPs: original sources, flags, source/request hashes, plans, contexts and native assignment indices')
