"""Independently inspect actual browser label-pipeline ZIP downloads."""
import hashlib
import json
from pathlib import Path
import zipfile

root = Path('target/label-pipeline-browser')
count = 0
for oracle_name, zip_name in [('build-analysis', 'build-analysis.zip'), ('build-report', 'build-reports.zip'), ('analysis', 'analysis.zip'), ('selected-analysis', 'selected-analysis.zip'), ('typed-analysis', 'typed-analysis.zip'), ('report-html', 'reports-html.zip'), ('report-json', 'reports-json.zip')]:
    native = json.loads((root / (oracle_name + '-native.json')).read_text(encoding='utf-8'))
    with zipfile.ZipFile(root / zip_name) as archive:
        assert archive.testzip() is None
        assert len(archive.namelist()) == len(set(archive.namelist()))
        assert set(archive.namelist()) == set(native['files'])
        for name, text in native['files'].items():
            assert not name.startswith('/') and '..' not in name.split('/') and '\\' not in name
            data = archive.read(name)
            assert data == text.encode('utf-8')
            assert hashlib.sha256(data).hexdigest() == native['file_sha256'][name]
            count += 1
    index = json.loads(native['index_json'])
    if oracle_name.startswith('report-') or oracle_name == 'build-report':
        analysis_name = 'build-analysis' if oracle_name == 'build-report' else 'analysis'
        analysis = json.loads((root / (analysis_name + '-native.json')).read_text(encoding='utf-8'))
        assert index['analysis_index_sha256'] == hashlib.sha256(analysis['index_json'].encode('utf-8')).hexdigest()
    else:
        request = json.loads((root / (oracle_name + '-request.json')).read_text(encoding='utf-8'))
        assert index['run_index_sha256'] == hashlib.sha256(request['input']['build_index_json'].encode('utf-8')).hexdigest()
print(f'Actual browser label pipeline ZIPs: {count} exact native files, complete file sets, safe paths, CRCs, digests and raw-byte index chains passed.')
