"""Read actual CVD exports independently of the browser package writers."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path('target/bulk-cvd-controls')
sha = lambda data: hashlib.sha256(data).hexdigest()

def check(archive_path, run, iterations):
    recipe = run['native_states']['recipe']
    assert sha(json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()) == run['native_states']['source_sha256']
    assert sha(run['request_source']['source']['text'].encode()) == run['native_states']['request_sha256']
    assert run['prepared']['executed_request']['options']['cvd_iters'] == iterations
    assert run['prepared']['executed_request']['options']['cvd_metric'] == 'geographic'
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        paths = archive.namelist()
        assert len(paths) == len(set(paths)) and all('..' not in p.split('/') and not p.startswith('/') for p in paths)
        assert archive.read('states-flags/arguments.json').decode() == recipe['command']['arguments_json']
        assert json.loads(archive.read('states-flags/recipe.json')) == recipe
        assert archive.read('plan.rplan').decode() == run['prepared']['export']['document_json']
        assert archive.read('context.rctx').decode() == run['prepared']['export']['context_json']
        plan = json.loads(archive.read('plan.rplan'))
        assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
        assert [assignments[str(i)] for i in range(len(plan['plan']['assignment']))] == [d + 1 for d in plan['plan']['assignment']]
        options = json.loads(archive.read('native-state/engine-options.json'))
        assert options['cvd_iters'] == iterations and options['cvd_metric'] == 'geographic'

run = json.loads((root / 'accepted.json').read_text(encoding='utf-8'))
check(root / 'accepted.zip', run, 3)
if (root / 'before-execution.json').exists():
    before = json.loads((root / 'before-execution.json').read_text(encoding='utf-8'))
    assert before['options']['cvd_iters'] == 20 and before['options']['cvd_metric'] == 'graph-distance'
    assert before['assignments_sha256'] != sha(json.dumps(run['prepared']['result']['assignments'], separators=(',', ':')).encode())
if '--skip-browser' not in sys.argv:
    browser = Path('target/bulk-cvd-controls-browser')
    first = json.loads((browser / 'complete.bisect').read_text(encoding='utf-8'))['result']
    check(browser / 'cvd-run.zip', first, 3)
    changed = json.loads((browser / 'changed.bisect').read_text(encoding='utf-8'))['result']
    assert changed['prepared']['executed_request']['options']['cvd_iters'] == 1
    assert changed['prepared']['result']['assignments'] != first['prepared']['result']['assignments']
print('PASS bulk CVD archives: requested controls, exact original flags/recipe hashes, native plan/context/assignment bytes and browser iteration-change evidence when enabled')
