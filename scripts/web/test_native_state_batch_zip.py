"""Independently check source recipes and native assignments in batch exports."""
import hashlib
import io
import json
from pathlib import Path
import zipfile

root = Path('target/native-state-batch')
archives = [root / name for name in ['complete.zip', 'multiscale.zip', 'years.zip']]
browser = Path('target/native-state-batch-browser/state-flags.zip')
if browser.exists():
    archives.append(browser)
recipes = 0


def inspect(data, depth=0):
    global recipes
    assert depth <= 4 and len(data) <= 128 * 1024 * 1024
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        for name in archive.namelist():
            assert not name.startswith('/') and '..' not in name.split('/') and '\\' not in name
            assert archive.getinfo(name).file_size <= 128 * 1024 * 1024
        if 'state-flags/recipe.json' in archive.namelist():
            recipe = json.loads(archive.read('state-flags/recipe.json'))
            identity = json.loads(archive.read('state-flags/identities.json'))
            receipt = json.loads(archive.read('prepared-run-receipt.json'))
            canonical = json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()
            assert hashlib.sha256(canonical).hexdigest() == identity['source_sha256']
            assert hashlib.sha256(archive.read('request.json')).hexdigest() == identity['request_sha256']
            assert archive.read('state-flags/arguments.json').decode() == recipe['command']['arguments_json']
            assert receipt['native_state']['recipe'] == recipe
            run = receipt['prepared']
            request = run['executed_request']
            fine = run['run_kind'] == 'multiscale' and request['fine_level'] == 'bg'
            ids = request['block_groups']['geoids'] if fine else (
                request['request'] if run['run_kind'] == 'multiscale' else request)['graph']['geoids']
            expected = {str(i): run['result']['assignments'][unit] for i, unit in enumerate(ids)}
            assert archive.read('native-state/data/final_assignments.json').decode() == json.dumps(expected, sort_keys=True, indent=2)
            assert json.loads(archive.read('native-state/unit-index.json'))['unit_ids'] == ids
            if fine:
                assert json.loads(archive.read('native-state/multiscale-settings.json'))['fine_level'] == 'bg'
            recipes += 1
        for name in archive.namelist():
            if name.endswith('.zip'):
                inspect(archive.read(name), depth + 1)


for filename in archives:
    inspect(filename.read_bytes())
assert recipes >= 8
print(f'PASS native State batch archives: {recipes} exact source/flag/request identities and native unit-index assignments, including multiscale and all three years')
