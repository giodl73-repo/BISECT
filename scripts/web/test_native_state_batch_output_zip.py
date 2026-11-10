"""Independent inspection of national state-folder archives and installation."""
from pathlib import Path
import json,zipfile,hashlib,base64
root=Path('target/native-state-batch-output');sha=lambda b:hashlib.sha256(b).hexdigest()
output=json.loads((root/'output.json').read_text(encoding='utf-8'));assert len(output['outputs'])==50
with zipfile.ZipFile(root/'national.zip') as z:
    assert z.testzip() is None
    paths=z.namelist();assert len(paths)==len(set(paths))
    for item in output['outputs']:
        prefix=item['target_namespace']+'/'
        assert prefix=='v2/2020/states/'+item['state_slug']+'/'
        for path,expected in item['file_sha256'].items():assert sha(z.read(prefix+path))==expected
        doc=json.loads(z.read(prefix+'plan.rplan'));index=json.loads(z.read(prefix+'unit-index.json'));assign=json.loads(z.read(prefix+'data/final_assignments.json'))
        assert index['unit_ids']==doc['plan']['units']['unit_ids']
        assert [assign[str(i)] for i in range(len(index['unit_ids']))]==[d+1 for d in doc['plan']['assignment']]
        scope=json.loads(z.read(prefix+'browser/output-scope.json'));assert scope['state_scope']['state_slug']==item['state_slug'];assert scope['original_export_label']==item['label']
        assert scope['state_scope']['historical_completion_paths'][0]=='2020/states/'+item['state'].lower()+'/data/final_assignments.json'
with zipfile.ZipFile(root/'years.zip') as z:
    assert z.testzip() is None
    assert all('v2/'+year+'/states/rhode_island/plan.rplan' in z.namelist() for year in ['2000','2010','2020'])
with zipfile.ZipFile(root/'human-state.zip') as z:
    assert z.testzip() is None
    doc=json.loads(z.read('v2/2020/states/rhode_island/plan.rplan'));assert doc['metadata']['label']=='A human readable plan'
with zipfile.ZipFile(root/'multiscale-state.zip') as z:
    assert z.testzip() is None
    doc=json.loads(z.read('v2/2020/states/rhode_island/plan.rplan'));assert len(doc['plan']['units']['unit_ids'])==8
with zipfile.ZipFile(root/'over-2000-files.zip') as z:assert len(z.namelist())==2010 and z.testzip() is None
installed=json.loads((root/'installed.json').read_text(encoding='utf-8'));base=Path(installed['directory']);receipt=json.loads((base/installed['checkpoint_path']).read_text(encoding='utf-8'));assert receipt['status']=='completed' and receipt['summary']['completed']==2
for state in receipt['states']:
    directory=base/state['target_namespace'];assert not (directory/'manifest.tmp').exists()
    for entry in state['receipt']['files']:
        data=(directory/entry['path']).read_bytes();assert len(data)==entry['byte_length'] and sha(data)==entry['sha256']
browser=Path('target/native-state-batch-output-browser');project=json.loads((browser/'complete.bisect').read_text(encoding='utf-8'));files=json.loads((browser/'installed.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(browser/'batch.zip') as z:
    assert z.testzip() is None
    for item in project['result']['outputs']:
        for path,expected in item['file_sha256'].items():
            full=item['target_namespace']+'/'+path;assert sha(z.read(full))==expected and sha(bytes(files[full]))==expected
print('PASS native batch output archives: 50 canonical state folders, three years, native indices/scope records, 2,010 entries and local/browser checkpoint readback')
