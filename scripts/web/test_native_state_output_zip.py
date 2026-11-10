"""Independent native-layout package and verified-install inspection."""
from pathlib import Path
import json,zipfile,hashlib
sha=lambda data:hashlib.sha256(data).hexdigest()
root=Path('target/native-state-output')
for name in ['ordinary','multiscale','tui']:
    output=json.loads((root/(name+'-output.json')).read_text(encoding='utf-8'))
    with zipfile.ZipFile(root/(name+'.zip')) as archive:
        assert archive.testzip() is None
        names=archive.namelist();assert len(names)==len(set(names))
        prefix=output['target_namespace']+'/'
        assert all(n.startswith(prefix) and '\\' not in n and '..' not in n.split('/') for n in names)
        for path,expected in output['file_sha256'].items():assert sha(archive.read(prefix+path))==expected
        assert archive.read(prefix+output['label']+'.rplan')==archive.read(prefix+'plan.rplan')
        plan=json.loads(archive.read(prefix+'plan.rplan'));ids=json.loads(archive.read(prefix+'unit-index.json'))['unit_ids'];assign=json.loads(archive.read(prefix+'data/final_assignments.json'))
        assert ids==plan['plan']['units']['unit_ids']
        assert [assign[str(i)] for i in range(len(ids))]==[d+1 for d in plan['plan']['assignment']]
        manifest=json.loads(archive.read(prefix+'manifest.json'));assert manifest['submission_type']=='research' and manifest['source_tool']=='bisect-wasm'
        assert prefix+'data/provenance.json' not in names and prefix+'audit-certificate.json' not in names and prefix+'manifest.tmp' not in names
        if name=='multiscale':assert len(ids)==8
installed=json.loads((root/'installed.json').read_text(encoding='utf-8'));directory=Path(installed['directory'])/installed['target_namespace']
assert not (directory/'manifest.tmp').exists()
receipt=json.loads((directory/'browser/install-receipt.json').read_text(encoding='utf-8'))
for artifact in receipt['files']:
    data=(directory/artifact['path']).read_bytes();assert len(data)==artifact['byte_length'] and sha(data)==artifact['sha256']
assert sha((directory/'browser/install-receipt.json').read_bytes())==installed['receipt']['install_receipt_sha256']
browser=Path('target/native-state-output-browser');project=json.loads((browser/'complete.bisect').read_text(encoding='utf-8'));out=project['result']
with zipfile.ZipFile(browser/'native-layout.zip') as archive:
    assert archive.testzip() is None
    for path,expected in out['file_sha256'].items():assert sha(archive.read(out['target_namespace']+'/'+path))==expected
actual=json.loads((browser/'installed.json').read_text(encoding='utf-8'))
for path,expected in out['file_sha256'].items():assert sha(bytes(actual[path]))==expected
print('PASS native output artifacts: four ZIPs, original modern/labeled RPLAN, exact native-index assignments, fine units, research manifest and local/browser readback receipt hashes')
