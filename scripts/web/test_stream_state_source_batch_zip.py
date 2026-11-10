"""Independently inspect disk bytes and nested native outputs, not browser hashes."""
import base64
import hashlib
import io
import json
import sys
from pathlib import Path
import zipfile

root = Path('target/stream-state-source-batch')
receipt = json.loads((root / 'evidence.json').read_text(encoding='utf-8'))['completed']
assert receipt['summary']['succeeded'] == 50
for row in receipt['states']:
    directory = root / receipt['output_folder'] / row['state']
    for artifact in row['output']['artifacts']:
        data = (directory / artifact['name']).read_bytes()
        assert len(data) == artifact['byte_length']
        assert hashlib.sha256(data).hexdigest() == artifact['sha256']
    oracle = json.loads((root / 'native-oracles' / (row['state'] + '-1.json')).read_text(encoding='utf-8'))
    with zipfile.ZipFile(directory / 'state-output.zip') as output:
        assert output.testzip() is None
        assert len(output.namelist()) == len(set(output.namelist()))
        prefix = row['state'] + '/'
        with zipfile.ZipFile(io.BytesIO(output.read(prefix + 'native-run.zip'))) as native:
            assert native.testzip() is None
            assert json.loads(native.read('run-receipt.json')) == oracle
            assert native.read('plan.rplan') == oracle['practitioner_export']['document_json'].encode('utf-8')
            assert native.read('context.rctx') == oracle['practitioner_export']['context_json'].encode('utf-8')
            assert native.read('manifest.json') == oracle['report_manifest']['manifest_json'].encode('utf-8')
        with zipfile.ZipFile(io.BytesIO(output.read(prefix + 'native-workflow.zip'))) as workflow:
            assert workflow.testzip() is None
            for name, encoded in oracle['workflow']['files_base64'].items():
                assert workflow.read(name) == base64.b64decode(encoded, validate=True)
    receipts = json.loads((directory / 'acquisition-receipts.json').read_text(encoding='utf-8'))
    for acquisition in receipts:
        data = (directory / (acquisition['kind'] + '.zip')).read_bytes()
        assert len(data) == acquisition['byte_length']
        assert hashlib.sha256(data).hexdigest() == acquisition['source_sha256']
print('Streaming disk ZIPs: all 50 synthetic states, independent lengths/SHA/CRCs, exact native Run/plan/context/manifest and every workflow byte, original acquired ZIP hashes passed.')
if '--local-ri' in sys.argv:
    original = json.loads((root / 'original-ri-evidence.json').read_text(encoding='utf-8'))
    row = original['states'][0]
    directory = root / original['output_folder'] / 'RI'
    oracle = json.loads((root / 'native-oracles/RI-2.json').read_text(encoding='utf-8'))
    for artifact in row['output']['artifacts']:
        data = (directory / artifact['name']).read_bytes()
        assert len(data) == artifact['byte_length']
        assert hashlib.sha256(data).hexdigest() == artifact['sha256']
    with zipfile.ZipFile(directory / 'state-output.zip') as output:
        assert output.testzip() is None
        preparation = json.loads(output.read('RI/source-preparation.json'))
        assert preparation['units'] == 250 and preparation['population'] == '1097379'
        with zipfile.ZipFile(io.BytesIO(output.read('RI/native-run.zip'))) as native:
            assert native.testzip() is None
            assert json.loads(native.read('run-receipt.json')) == oracle
        with zipfile.ZipFile(io.BytesIO(output.read('RI/native-workflow.zip'))) as workflow:
            assert workflow.testzip() is None
            for name, encoded in oracle['workflow']['files_base64'].items():
                assert workflow.read(name) == base64.b64decode(encoded, validate=True)
    jobs = json.loads((directory / 'state-run-jobs.json').read_text(encoding='utf-8'))
    for kind, source in [('tiger','tiger'), ('pl94171','pl94')]:
        assert (directory / (kind + '.zip')).read_bytes() == Path(f'target/census-zip-acquisition/ri-{source}.zip').read_bytes()
    print('Original RI stream: 250 tracts/1097379 population, exact original source ZIPs, complete native two-seat Run/workflow and verified disk artifact bytes passed.')

if '--local-ca' in sys.argv:
    topology = '--topology' in sys.argv
    suffix = '-topology' if topology else ''
    original = json.loads((root / f'original-ca{suffix}-evidence.json').read_text(encoding='utf-8'))
    row = original['states'][0]
    directory = root / original['output_folder'] / 'CA'
    oracle = json.loads((root / f'native-oracles/CA-52{suffix}.json').read_text(encoding='utf-8'))
    if topology:
        legacy = json.loads((root / 'native-oracles/CA-52.json').read_text(encoding='utf-8'))
        assert oracle['practitioner_export']['document']['plan'] == legacy['practitioner_export']['document']['plan']
        assert oracle['practitioner_export']['prepared_graph_hash'] == legacy['practitioner_export']['prepared_graph_hash']
        for name in ['maps/districts.png', 'maps/districts.svg', 'analysis/compactness.json']:
            assert name in oracle['workflow']['files_base64']
        assert any(name.endswith('.html') for name in oracle['workflow']['files_base64'])
    for artifact in row['output']['artifacts']:
        data = (directory / artifact['name']).read_bytes()
        assert len(data) == artifact['byte_length']
        assert hashlib.sha256(data).hexdigest() == artifact['sha256']
    with zipfile.ZipFile(directory / 'state-output.zip') as output:
        assert output.testzip() is None
        prepared = json.loads(output.read('CA/source-preparation.json'))
        assert prepared['units'] == 9129 and prepared['population'] == '39538223'
        if topology:
            assert prepared['geometry_profile'] == 'display-topology'
            geometry = output.read('CA/prepared-geometry.json')
            # The native CLI adds its line terminator; the WASM JSON string does not.
            assert geometry == Path('target/original-state-acquisition/CA/topology-geometry.json').read_bytes().rstrip(b'\r\n')
        with zipfile.ZipFile(io.BytesIO(output.read('CA/native-run.zip'))) as native:
            assert native.testzip() is None
            assert json.loads(native.read('run-receipt.json')) == oracle
        with zipfile.ZipFile(io.BytesIO(output.read('CA/native-workflow.zip'))) as workflow:
            assert workflow.testzip() is None
            for name, encoded in oracle['workflow']['files_base64'].items():
                assert workflow.read(name) == base64.b64decode(encoded, validate=True)
    for kind in ['tiger', 'pl94171']:
        assert (directory / (kind + '.zip')).read_bytes() == Path(f'target/original-state-acquisition/CA/{kind}.zip').read_bytes()
    print('Original CA stream: 9129 tracts/39538223 population, exact original ZIPs, complete native 52-seat Run/workflow, file hashes and ZIP CRCs passed.')
