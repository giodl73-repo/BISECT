"""Independent resumed-file, CRC and original per-year chain checks."""
import hashlib
import json
from pathlib import Path
import zipfile
root = Path('target/label-years-resume-wasm')
count = 0
for kind in ['analysis', 'report']:
    for prefix in [1, 2]:
        name = f'{kind}-{prefix}'
        native = json.loads((root / (name + '-native.json')).read_text(encoding='utf-8'))
        request = json.loads((root / (name + '-request.json')).read_text(encoding='utf-8'))
        with zipfile.ZipFile(root / (name + '.zip')) as archive:
            assert archive.testzip() is None
            assert len(archive.namelist()) == len(set(archive.namelist()))
            assert set(archive.namelist()) == set(native['files'])
            for path, text in native['files'].items():
                assert not path.startswith('/') and '..' not in path.split('/') and '\\' not in path
                data = archive.read(path)
                assert data == text.encode('utf-8')
                assert hashlib.sha256(data).hexdigest() == native['file_sha256'][path]
                count += 1
            for item in native['years']:
                result = item['result']
                index = json.loads(result['index_json'])
                field = 'build_index_json' if kind == 'analysis' else 'analysis_index_json'
                hash_field = 'run_index_sha256' if kind == 'analysis' else 'analysis_index_sha256'
                assert index[hash_field] == hashlib.sha256(request['input']['batches'][item['year']][field].encode('utf-8')).hexdigest()
                assert archive.read(result['index_path']) == result['index_json'].encode('utf-8')
print(f'Resumed year ZIPs: {count} exact native files, complete file sets, CRCs, digests and original per-year index chains passed.')
