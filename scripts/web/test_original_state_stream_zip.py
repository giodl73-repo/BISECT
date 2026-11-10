"""Independent original-source identities/populations and exported native bytes."""
import base64
from collections import defaultdict
import hashlib
import gzip
import io
import json
from pathlib import Path
import re
import struct
import sys
import zipfile

root = Path('target/stream-state-source-batch')
for state in sys.argv[1:]:
    assert re.fullmatch('[A-Z]{2}', state)
    receipt = json.loads((root / f'original-{state.lower()}-topology-evidence.json').read_text(encoding='utf-8'))
    assert receipt['summary']['succeeded'] == 1 and receipt['summary']['failed'] == 0
    assert re.fullmatch(r'bisect-run-[A-Za-z0-9-]{1,80}', receipt['output_folder'])
    row = receipt['states'][0]
    assert row['state'] == state and row['status'] == 'ok'
    directory = root / receipt['output_folder'] / state
    districts = receipt['selection']['states'][0]['districts']
    native = json.loads((root / f'native-oracles/{state}-{districts}-topology.json').read_text(encoding='utf-8'))
    for artifact in row['output']['artifacts']:
        assert '/' not in artifact['name'] and '\\' not in artifact['name']
        data = (directory / artifact['name']).read_bytes()
        assert len(data) == artifact['byte_length']
        assert hashlib.sha256(data).hexdigest() == artifact['sha256']
    for kind in ['tiger', 'pl94171']:
        original = Path('target/original-state-acquisition') / state / (kind + '.zip')
        assert (directory / (kind + '.zip')).read_bytes() == original.read_bytes()
        acquisition = json.loads(Path(str(original) + '.receipt.json').read_text(encoding='utf-8'))
        assert acquisition['sha256'] == hashlib.sha256(original.read_bytes()).hexdigest()
    recipe_name = next(artifact['name'] for artifact in row['output']['artifacts'] if artifact['name'] in ['state-run-jobs.json', 'state-source-recipe.json', 'state-source-recipe.json.gz'])
    if recipe_name.endswith('.gz'):
        with gzip.open(directory / recipe_name, 'rb') as stream:
            recipe_bytes = stream.read(256 * 1024 * 1024 + 1)
        assert 128 * 1024 * 1024 < len(recipe_bytes) <= 256 * 1024 * 1024
    else:
        recipe_bytes = (directory / recipe_name).read_bytes()
    jobs = json.loads(recipe_bytes)
    assert len(jobs) == 1 and jobs[0]['form']['location'] == state
    assert jobs[0]['form']['seed'] == '18446744073709551615'
    project_bytes = (directory / 'source-project.bisect-workbench').read_bytes()
    if project_bytes[:2] == b'\x1f\x8b':
        with gzip.GzipFile(fileobj=io.BytesIO(project_bytes)) as stream:
            project_bytes = stream.read(256 * 1024 * 1024 + 1)
        assert len(project_bytes) <= 256 * 1024 * 1024
    project = json.loads(project_bytes)
    if project['schema_version'] == 'bisect-workbench-archive-v2':
        for reference in project['shared_string_refs']:
            parent = project
            for key in reference['path'][:-1]:
                parent = parent[key]
            key = reference['path'][-1]
            assert parent[key] is None
            parent[key] = project['shared_strings'][reference['string_index']]
    assert project['operation'] == 'prepare-census-engine-input' and project['result'] is None
    assert project['files']['tigerInputs'] == jobs[0]['source_preparation']['tiger']
    assert project['files']['pl94Inputs'] == jobs[0]['source_preparation']['pl94']
    assert project['files']['configRunInputs'] == jobs[0]['configuration']['inputs']
    assert project['files']['configSource'] == jobs[0]['configuration']['source']
    assert project['files']['configRunSettings']['districts'] == districts
    assert project['files']['tuiRunSettings']['form'] == jobs[0]['form']
    with zipfile.ZipFile(directory / 'state-output.zip') as output:
        assert output.testzip() is None
        assert len(output.namelist()) == len(set(output.namelist()))
        prepared = json.loads(output.read(state + '/source-preparation.json'))
        assert prepared['state'] == state and prepared['geometry_profile'] == 'display-topology'
        graph = json.loads(output.read(state + '/prepared-graph.json'))
        assert graph['state'] == state and graph['year'] == '2020'
        assert prepared['units'] == len(graph['geoids']) == len(graph['population'])
        assert int(prepared['population']) == sum(graph['population'])
        with zipfile.ZipFile(io.BytesIO(output.read(state + '/native-run.zip'))) as archive:
            assert archive.testzip() is None
            assert json.loads(archive.read('run-receipt.json')) == native
            assert archive.read('plan.rplan') == native['practitioner_export']['document_json'].encode('utf-8')
            assert archive.read('context.rctx') == native['practitioner_export']['context_json'].encode('utf-8')
            assert archive.read('manifest.json') == native['report_manifest']['manifest_json'].encode('utf-8')
        with zipfile.ZipFile(io.BytesIO(output.read(state + '/native-workflow.zip'))) as archive:
            assert archive.testzip() is None
            for name, encoded in native['workflow']['files_base64'].items():
                assert archive.read(name) == base64.b64decode(encoded, validate=True)
    # Independent DBF identity reader; no reuse of the Rust preparation parser.
    with zipfile.ZipFile(directory / 'tiger.zip') as archive:
        data = archive.read(next(name for name in archive.namelist() if name.lower().endswith('.dbf')))
    count = struct.unpack_from('<I', data, 4)[0]
    header, width = struct.unpack_from('<HH', data, 8)
    fields, offset, position = {}, 1, 32
    while data[position] != 13:
        name = data[position:position+11].split(b'\0')[0].decode('ascii')
        size = data[position+16]
        fields[name] = (offset, size)
        offset += size
        position += 32
    start, size = fields.get('GEOID', fields.get('GEOID20'))
    ids = []
    for index in range(count):
        record = data[header+index*width:header+(index+1)*width]
        if record[0] != 42:
            ids.append(record[start:start+size].strip().decode('ascii'))
    assert sorted(ids) == graph['geoids'] and len(ids) == len(set(ids))
    # Independent streaming PL join by LOGRECNO and tract prefix, from raw ZIP.
    with zipfile.ZipFile(directory / 'pl94171.zip') as archive:
        blocks = {}
        geo = next(name for name in archive.namelist() if name.lower().endswith('geo2020.pl'))
        population = next(name for name in archive.namelist() if name.lower().endswith('000012020.pl'))
        with archive.open(geo) as stream:
            for line in stream:
                fields = line.rstrip(b'\r\n').split(b'|')
                if len(fields) > 8 and fields[2].strip() == b'750':
                    key, encoded = fields[7].strip(), fields[8].strip()
                    assert key not in blocks and encoded.startswith(b'7500000US')
                    geoid = encoded[9:]
                    assert len(geoid) == 15 and geoid.isdigit()
                    blocks[key] = geoid[:11].decode('ascii')
        sums = defaultdict(int)
        with archive.open(population) as stream:
            for line in stream:
                fields = line.rstrip(b'\r\n').split(b'|')
                if len(fields) > 5:
                    tract = blocks.pop(fields[4].strip(), None)
                    if tract is not None:
                        value = int(fields[5].strip())
                        assert value >= 0
                        sums[tract] += value
        assert not blocks and set(sums) <= set(graph['geoids'])
    assert [sums[geoid] for geoid in graph['geoids']] == graph['population']
    assert sum(sums.values()) == int(prepared['population'])
    print(f'Original {state}: all {len(ids)} native TIGER identities and {sum(sums.values())} PL population independently match; {districts}-district native Run/workflow bytes, source copies, hashes and ZIP CRCs pass.')
if len(sys.argv) == 1:
    raise SystemExit('Supply one or more original state codes, e.g. TX MD.')
