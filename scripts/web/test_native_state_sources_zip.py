"""Independent checks of raw-source recipes, scope bindings and saved outputs."""
import hashlib
import io
import json
from pathlib import Path
import zipfile

root = Path('target/native-state-sources')
paths = [root / 'native.zip', Path('target/native-state-sources-browser/source-output.zip')]
stream = json.loads((root / 'stream.json').read_text())
for row in stream['states']:
    directory = root / stream['output_folder'] / row['state']
    marker = json.loads((directory / 'completed.json').read_text())
    for artifact in marker['artifacts']:
        data = (directory / artifact['name']).read_bytes()
        assert len(data) == artifact['byte_length']
        assert hashlib.sha256(data).hexdigest() == artifact['sha256']
    source = json.loads((directory / 'source-project.bisect-workbench').read_text())
    # The small fixtures use the ordinary JSON workbench archive envelope.
    assert source['schema_version']
    paths.append(directory / 'state-output.zip')

checked = 0


def inspect(data, depth=0):
    global checked
    assert depth <= 4 and len(data) <= 128 * 1024 * 1024
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        assert all(not name.startswith('/') and '..' not in name.split('/') and '\\' not in name for name in archive.namelist())
        if 'state-flags/recipe.json' in archive.namelist():
            recipe = json.loads(archive.read('state-flags/recipe.json'))
            identity = json.loads(archive.read('state-flags/identities.json'))
            binding = json.loads(archive.read('state-flags/scope-binding.json'))
            compilation = json.loads(archive.read('state-flags/compilation.json'))
            assert 'source_inputs' in recipe
            assert hashlib.sha256(json.dumps(recipe, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest() == identity['source_sha256']
            assert hashlib.sha256(archive.read('request.json')).hexdigest() == identity['request_sha256']
            assert json.loads(archive.read('state-flags/arguments.json')) == binding['source_arguments']
            assert binding['arguments'] == compilation['native_command']['arguments']
            assert binding['algorithm_unchanged'] is True
            assert binding['native_algorithm_debug'] == compilation['native_command']['native_debug']['algorithm']
            receipt = json.loads(archive.read('prepared-run-receipt.json'))
            assert receipt['scope']['state'] == binding['scope']['state']
            assert receipt['scope']['districts'] == binding['scope']['districts']
            run = receipt['prepared']
            expected = {str(i): run['result']['assignments'][unit] for i, unit in enumerate(run['executed_request']['graph']['geoids'])}
            assert archive.read('native-state/data/final_assignments.json').decode() == json.dumps(expected, sort_keys=True, indent=2)
            checked += 1
        for name in archive.namelist():
            if name.endswith('.zip'):
                inspect(archive.read(name), depth + 1)


for path in paths:
    inspect(path.read_bytes())
assert checked == 4
print('PASS native State source archives: four exact recipes/scope bindings/native assignments, two local completion-marker artifact sets and CRC/path checks')
