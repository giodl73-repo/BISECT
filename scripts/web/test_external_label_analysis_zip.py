"""Verify external analysis/report ZIPs independently of the JS exporter."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys
import zipfile

root=Path('target/external-label-analysis-browser' if '--browser' in sys.argv else 'target/external-label-analysis-wasm')
count=0
for archive in root.glob('*.zip'):
    native=root/(archive.stem+'-result.json')
    result=json.loads(native.read_text(encoding='utf-8')) if native.exists() else json.loads(archive.with_suffix('.bisect').read_text(encoding='utf-8'))['result']
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None
        assert len(zipped.infolist())==len(result['files'])
        assert set(zipped.namelist())==set(result['files'])==set(result['file_sha256'])
        for name,text in result['files'].items():
            p=PurePosixPath(name)
            assert not p.is_absolute() and '..' not in p.parts and '\\' not in name
            assert name=='.bisect' or name.startswith(('analysis/external_grid/','reports/external_grid/'))
            data=zipped.read(name)
            assert data==text.encode('utf-8')
            assert hashlib.sha256(data).hexdigest()==result['file_sha256'][name]
            count+=1
        indexes=[(result['index_json'],result)] if 'index_json' in result else [(item['result']['index_json'],item['result']) for item in result['years'] if item['status']=='completed']
        for index,stage in indexes:
            assert stage['files'][stage['index_path']]==index
            assert stage['index_sha256']==hashlib.sha256(index.encode()).hexdigest()
            record=json.loads(index)
            if stage['schema_version']=='bisect-label-analysis-v1':
                assert stage['external_import_verified'] is True
                assert all(value in ('one-based','zero-based') for value in stage['external_district_bases'].values())
                for name,text in stage['files'].items():
                    if name.endswith('/plan.rplan'):
                        plan=json.loads(text)
                        assert plan['provenance']['producer']=={'method':'external-label-import-context-alignment'}
                        assert plan['extensions']['org.bisect.label-import']['index_sha256']==record['run_index_sha256']
assert count>0
print(f"External label {'browser' if '--browser' in sys.argv else 'direct'} ZIPs: {count} exact native files, safe paths, CRCs, digests, external provenance and index chains passed.")
