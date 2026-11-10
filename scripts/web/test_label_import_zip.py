"""Independently inspect imported assignment ZIP bytes, CRCs and source provenance."""
import base64
import hashlib
import json
from pathlib import Path
import sys
import zipfile
browser = '--browser' in sys.argv
root = Path('target/label-import-browser' if browser else 'target/label-import-wasm')
names = ['csv', 'geojson', 'rplan', 'shapefile'] if browser else ['csv', 'geojson', 'rplan', 'shapefile', 'shapefile-no-shx', 'reversed', 'mixed', 'forced-uppercase']
count = 0
for name in names:
    result = json.loads((root / (name + '-result.json')).read_text(encoding='utf-8'))
    request = json.loads((root / (name + '-request.json')).read_text(encoding='utf-8'))['input']
    with zipfile.ZipFile(root / (name + '.zip')) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        assert set(archive.namelist()) == set(result['files'])
        for path, text in result['files'].items():
            assert not path.startswith('/') and '..' not in path.split('/') and '\\' not in path
            data = archive.read(path)
            assert data == text.encode('utf-8')
            assert hashlib.sha256(data).hexdigest() == result['file_sha256'][path]
            assert archive.getinfo(path).external_attr >> 16 & 0o111 == 0
            count += 1
        index = json.loads(archive.read(result['index_path']))
        assert index['algorithm']['structure'] == 'external'
        assert index['algorithm']['source_file'] == request['source']['name']
        assert index['algorithm']['source'] == hashlib.sha256(base64.b64decode(request['source']['base64'])).hexdigest()
        assert index['created'] == request['created']
        units = 0
        for state, row in index['states'].items():
            assignments = json.loads(archive.read(f"runs/{index['label']}/{index['year']}/{state}/assignments.json"))
            assert len(assignments) == row['tracts'] and row['status'] == 'ok'
            assert all(type(value) is int and 0 <= value <= 4294967295 for value in assignments.values())
            units += len(assignments)
        assert units == index['summary']['total']
        registry = json.loads(archive.read('.bisect'))
        assert index['year'] in registry[index['label']]['built']
        for key in ['dbf', 'shx']:
            if request[key] is None:
                assert result[key] is None
            else:
                data = base64.b64decode(request[key]['base64'])
                assert result[key]['bytes'] == len(data)
                assert result[key]['sha256'] == hashlib.sha256(data).hexdigest()
print(f"Label import {'browser' if browser else 'direct'} ZIPs: {count} exact files, CRCs, digests, state counts and original source/sidecar provenance passed.")
