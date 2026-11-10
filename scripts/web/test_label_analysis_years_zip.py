"""Independently verify native multi-year analysis and selected-year report exports."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

browser = '--browser' in sys.argv
root = Path('target/label-pipeline-browser' if browser else 'target/label-analysis-years-wasm')
pairs = [('years-analysis', 'years-analysis.zip'), ('years-selected', 'years-selected.zip'), ('years-report', 'years-report.zip'), ('years-cancelled', 'years-cancelled.zip'), ('years-resumed', 'years-resumed.zip')] if browser else [(name, name + '.zip') for name in ['all', 'selected', 'failed-2010', 'failed-2020', 'checkpoint']]
count = 0
for name, zipname in pairs:
    native = json.loads((root / (name + ('-native.json' if browser else '-result.json'))).read_text(encoding='utf-8'))
    request = json.loads((root / (('all' if name == 'checkpoint' else name) + '-request.json')).read_text(encoding='utf-8'))
    with zipfile.ZipFile(root / zipname) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        assert set(archive.namelist()) == set(native['files'])
        for path, text in native['files'].items():
            assert not path.startswith('/') and '..' not in path.split('/') and '\\' not in path
            data = archive.read(path)
            assert data == text.encode('utf-8')
            assert hashlib.sha256(data).hexdigest() == native['file_sha256'][path]
            count += 1
        if native['schema_version'] == 'bisect-label-analysis-years-v1':
            receipt = json.loads(archive.read('analysis/' + native['label'] + '/years.json'))
            assert receipt['status'] == native['status']
            assert receipt['requested_years'] == native['requested_years']
            if native.get('execution_status') == 'cancelled':
                assert set(native['requested_years']) - {row['year'] for row in native['years']} == {'2020'}
                assert json.loads(native['registry_json'])[native['label']]['analyzed'] == ['2010']
            assert [row['year'] for row in receipt['years']] == [row['year'] for row in native['years']]
            for row, item in zip(receipt['years'], native['years']):
                assert row['status'] == item['status']
                if item['status'] == 'failed':
                    assert row['error'] == item['error']
                    assert row['index_sha256'] is None
                    continue
                child = item['result']
                assert archive.read(child['index_path']) == child['index_json'].encode('utf-8')
                assert row['index_sha256'] == hashlib.sha256(child['index_json'].encode('utf-8')).hexdigest()
                assert json.loads(child['index_json'])['run_index_sha256'] == hashlib.sha256(request['input']['batches'][item['year']]['build_index_json'].encode('utf-8')).hexdigest()
                for path, text in child['files'].items():
                    if path != '.bisect':
                        assert archive.read(path) == text.encode('utf-8')
            assert archive.read('.bisect') == native['registry_json'].encode('utf-8')
        else:
            assert json.loads(native['index_json'])['analysis_index_sha256'] == hashlib.sha256(request['input']['analysis_index_json'].encode('utf-8')).hexdigest()
print(f'Multi-year {"browser" if browser else "direct"} ZIPs: {count} exact native files, complete sets, CRCs, digests, per-year original index chains and retained failure receipts passed.')
