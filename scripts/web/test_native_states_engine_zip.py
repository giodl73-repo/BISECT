"""Independent native States run/source and physical-export archive checks."""
import base64
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path('target/native-states-engine')
sha = lambda value: hashlib.sha256(value).hexdigest()
for mode in ['geosection', 'areasection']:
    run = json.loads((root / (mode + '.json')).read_text(encoding='utf-8'))
    origin = run['native_states']
    assert sha(json.dumps(origin['recipe'], ensure_ascii=False, separators=(',', ':')).encode()) == origin['source_sha256']
    assert sha(run['request_source']['source']['text'].encode()) == origin['request_sha256']
    with zipfile.ZipFile(root / (mode + '.zip')) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        assert archive.read('states-flags/arguments.json').decode() == origin['recipe']['command']['arguments_json']
        assert json.loads(archive.read('states-flags/recipe.json')) == origin['recipe']
        assert json.loads(archive.read('states-flags/compilation.json')) == origin['compilation']
        assert archive.read('plan.rplan').decode() == run['prepared']['export']['document_json']
        assert archive.read('context.rctx').decode() == run['prepared']['export']['context_json']
        plan = json.loads(archive.read('plan.rplan'))
        assignments = json.loads(archive.read('native-state/data/final_assignments.json'))
        assert [assignments[str(i)] for i in range(len(plan['plan']['assignment']))] == [d + 1 for d in plan['plan']['assignment']]
for name in ['multiscale.zip', 'multiscale-adaptive.zip']:
    with zipfile.ZipFile(root / name) as archive:
        assert archive.testzip() is None
        plan = json.loads(archive.read('plan.rplan'))
        assert plan['plan']['units']['unit_kind'] == 'block-group'
        assert len(plan['plan']['units']['unit_ids']) == 72
        assert 'states-flags/recipe.json' in archive.namelist()
with zipfile.ZipFile(root / 'native-output.zip') as archive:
    assert archive.testzip() is None
    prefix = 'vBulk/2020/states/rhode_island/'
    retained = json.loads(archive.read(prefix + 'browser/accepted-run.json'))
    assert retained['native_states']['recipe']['command']['arguments_json'] == json.loads((root / 'geosection.json').read_text(encoding='utf-8'))['native_states']['recipe']['command']['arguments_json']

if '--skip-browser' not in sys.argv:
    browser = Path('target/native-states-engine-browser')
    completed = json.loads((browser / 'completed.bisect').read_text(encoding='utf-8'))['result']
    with zipfile.ZipFile(browser / 'states-run.zip') as archive:
        assert archive.testzip() is None
        assert archive.read('plan.rplan').decode() == completed['prepared']['export']['document_json']
        assert archive.read('states-flags/arguments.json').decode() == completed['native_states']['recipe']['command']['arguments_json']
    output = json.loads((browser / 'native-output.bisect').read_text(encoding='utf-8'))['result']
    for name, encoded in output['files_base64'].items():
        assert sha(base64.b64decode(encoded)) == output['file_sha256'][name]
    mapped = json.loads((browser / 'mapped.bisect').read_text(encoding='utf-8'))['result']
    assert mapped['schema_version'] == 'bisect-native-district-map-v1'
print('PASS native States archives: exact flags/recipes/compilation hashes, ordinary and fine-unit physical exports, native layout, plus browser files when enabled')
