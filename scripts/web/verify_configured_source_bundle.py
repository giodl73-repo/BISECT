import base64
import hashlib
import json
from pathlib import Path
from zipfile import ZipFile
import sys

root=Path('target/configured-source-bundle')
for i in [0,1]:
    result=json.loads((root/f'case-{i}-result.json').read_text(encoding='utf-8'))
    project=json.loads(Path(f'target/native-demographic-adoption/case-{i}.bisect').read_text(encoding='utf-8'))
    with ZipFile(root/f'case-{i}.zip') as archive:
        assert archive.testzip() is None
        assert len(archive.namelist())==len(set(archive.namelist()))==9
        manifest=json.loads(archive.read('bundle.json'))
        assert set(archive.namelist())==set(manifest['files'])|{'bundle.json'}
        for name,entry in manifest['files'].items():
            data=archive.read(name)
            assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
        assert archive.read('plan.rplan')==result['practitioner_export']['document_json'].encode()
        assert archive.read('context.rctx')==result['practitioner_export']['context_json'].encode()
        for name,source in [('sources/config.yml',project['files']['configSource']),('sources/graph.json',project['files']['configRunInputs']['graph']),('sources/demographics.json',project['files']['configRunInputs']['demographics']),('sources/native-demographics.csv',project['files']['demographicSource']),('sources/native-demographic-adoption.json',project['files']['nativeDemographicReceipt'])]:
            assert archive.read(name)==base64.b64decode(source['base64'])
print('Two source ZIPs: exact native plan/context and five selected sources, CRCs, unique safe fixed paths, sizes and all manifest byte hashes verified independently.')
if '--browser' in sys.argv:
    browser=Path('target/native-demographic-adoption-browser')
    project=json.loads((browser/'sources-exported.bisect').read_text(encoding='utf-8'))
    with ZipFile(browser/'configured-plan-sources.zip') as archive:
        assert archive.testzip() is None
        manifest=json.loads(archive.read('bundle.json'))
        assert set(archive.namelist())==set(manifest['files'])|{'bundle.json'}
        for name,entry in manifest['files'].items():
            data=archive.read(name)
            assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
        assert archive.read('plan.rplan')==project['result']['practitioner_export']['document_json'].encode()
        assert archive.read('context.rctx')==project['result']['practitioner_export']['context_json'].encode()
        assert archive.read('sources/native-demographics.csv')==base64.b64decode(project['files']['demographicSource']['base64'])
        assert archive.read('sources/native-demographic-adoption.json')==base64.b64decode(project['files']['nativeDemographicReceipt']['base64'])
    print('Actual browser ZIP: exact native worker artifacts, original CSV/receipt, CRC and all manifest byte hashes verified independently.')
