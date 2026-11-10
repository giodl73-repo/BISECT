"""Independent readback of actual native Run source/flag/plan exports."""
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile

sha = lambda data: hashlib.sha256(data).hexdigest()
root = Path('target/native-run-command')

def check(path, run):
    native = run['native_run']
    recipe = native['recipe']
    assert sha(json.dumps(recipe,ensure_ascii=False,separators=(',',':')).encode()) == native['source_sha256']
    assert sha(run['request_source']['source']['text'].encode()) == native['request_sha256']
    with zipfile.ZipFile(path) as archive:
        assert archive.testzip() is None
        names = archive.namelist()
        assert len(names) == len(set(names))
        assert all(not p.startswith('/') and '..' not in p.split('/') and '\\' not in p for p in names)
        assert json.loads(archive.read('run-flags/recipe.json')) == recipe
        assert archive.read('run-flags/arguments.json').decode() == recipe['command']['arguments_json']
        assert json.loads(archive.read('run-flags/compilation.json')) == native['compilation']
        assert sha(archive.read('request.json')) == native['request_sha256']
        assert archive.read('plan.rplan').decode() == run['prepared']['export']['document_json']
        assert archive.read('context.rctx').decode() == run['prepared']['export']['context_json']
        plan = json.loads(archive.read('plan.rplan'))
        assert plan['plan']['k'] == 2 and len(plan['plan']['assignment']) == 64
        assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
        assert [assignments[str(i)] for i in range(64)] == [d + 1 for d in plan['plan']['assignment']]
        assert sum(d == 0 for d in plan['plan']['assignment']) == 32
        options = json.loads(archive.read('native-state/engine-options.json'))
        assert options['balance_tolerance'] == .5
        assert native['compilation']['native_command']['years'] == ['2020','2010','2000']

for mode in ['edge-weighted','geosection','centroidal-voronoi','template']:
    check(root/(mode+'.zip'), json.loads((root/(mode+'.json')).read_text(encoding='utf-8')))
failed = json.loads((root/'cvd-grid-failure.json').read_text(encoding='utf-8'))
assert 'exhausted after 51 attempts' in failed['error']
if '--skip-browser' not in sys.argv:
    browser = Path('target/native-run-command-browser')
    project = json.loads((browser/'complete.bisect').read_text(encoding='utf-8'))
    check(browser/'run.zip',project['result'])
    output = json.loads((browser/'output.bisect').read_text(encoding='utf-8'))
    assert output['files']['nativeStateOutputInputs']['version'] == 'vActualRun'
    mapped = json.loads((browser/'map.bisect').read_text(encoding='utf-8'))
    assert mapped['result']['dpi'] == 72
    inert = json.loads((browser/'print-only.bisect').read_text(encoding='utf-8'))
    assert inert['result']['native_run_print_only'] and not inert['result']['execution_performed']
print('PASS actual native Run archives: original flags/source/request identities, native index assignments, exact balanced plans/context, retained failure and browser output/map/print-only handoffs when enabled')
