"""Independent byte/CRC/file-set/index-chain checks for multi-year report ZIPs."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile
browser = '--browser' in sys.argv
root = Path('target/label-pipeline-browser' if browser else 'target/label-report-years-wasm')
names = ['report-years-html', 'report-years-json', 'report-years-selected', 'report-years-cancelled', 'report-years-resumed'] if browser else ['html', 'json', 'selected', 'failed-2010', 'failed-2020', 'checkpoint']
count = 0
for name in names:
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
        record = json.loads(archive.read('reports/' + native['label'] + '/years.json'))
        assert record['requested_years'] == native['requested_years']
        assert record['status'] == native['status']
        assert [row['year'] for row in record['years']] == [row['year'] for row in native['years']]
        for row, item in zip(record['years'], native['years']):
            assert row['status'] == item['status']
            if item['status'] == 'failed':
                assert row['error'] == item['error']
                continue
            child = item['result']
            assert row['index_sha256'] == hashlib.sha256(child['index_json'].encode('utf-8')).hexdigest()
            assert json.loads(child['index_json'])['analysis_index_sha256'] == hashlib.sha256(request['input']['batches'][item['year']]['analysis_index_json'].encode('utf-8')).hexdigest()
            for path, text in child['files'].items():
                if path != '.bisect':
                    assert archive.read(path) == text.encode('utf-8')
        if native.get('execution_status') == 'cancelled':
            assert set(native['requested_years']) - {row['year'] for row in native['years']} == {'2020'}
            assert json.loads(native['registry_json'])[native['label']]['reported'] == ['2010']
print(f'Multi-year report {"browser" if browser else "direct"} ZIPs: {count} exact native files, complete sets, CRCs, digests, raw analysis-index chains and retained failure/cancellation receipts passed.')
