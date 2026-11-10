"""Independent inspection of native State automatic follow-up packages."""
from pathlib import Path
import io,json,zipfile,hashlib,base64
sha=lambda b:hashlib.sha256(b).hexdigest()
def safe(z):
    assert z.testzip() is None
    names=z.namelist();assert len(names)==len(set(names))
    assert all(not n.startswith(('/', '\\')) and '..' not in n.split('/') and '\\' not in n for n in names)
def workflow(z,w):
    safe(z)
    for name,expected in w['file_sha256'].items():
        assert sha(z.read(name))==expected
        assert z.read(name)==base64.b64decode(w['files_base64'][name])
    assert z.read('plan.rplan')==base64.b64decode(w['files_base64']['plan.rplan'])
    assert sha(z.read('plan.rplan'))==w['accepted_plan_source_sha256']
    assert sha(z.read('context.rctx'))==w['accepted_context_source_sha256']
    assert 'effective-config.yml' not in z.namelist()
paths=[Path('target/native-state-followup/batch.zip'),Path('target/native-state-followup-browser/complete.zip')]
stream=json.loads(Path('target/native-state-sources-followup/stream.json').read_text(encoding='utf-8'))
base=Path('target/native-state-sources-followup')/stream['output_folder']
for row in stream['states']:
    directory=base/row['state'];marker=json.loads((directory/'completed.json').read_text(encoding='utf-8'))
    assert marker['followup']==row['followup']
    for artifact in marker['artifacts']:
        data=(directory/artifact['name']).read_bytes();assert len(data)==artifact['byte_length'] and sha(data)==artifact['sha256']
    paths.append(directory/'state-output.zip')
for path in paths:
    with zipfile.ZipFile(path) as z:
        safe(z);row=json.loads(z.read('batch-receipt.json'))['states'][0];run=row['result'];state=row['state_code'];w=run['workflow']
        with zipfile.ZipFile(io.BytesIO(z.read(state+'/native-workflow.zip'))) as nested:workflow(nested,w)
        with zipfile.ZipFile(io.BytesIO(z.read(state+'/prepared-run.zip'))) as prepared:
            safe(prepared)
            assert prepared.read('plan.rplan')==base64.b64decode(w['files_base64']['plan.rplan'])
            recipe=json.loads(prepared.read('state-flags/recipe.json'))
            assert recipe['followup']['settings']['analysis'] is True
            ids=json.loads(prepared.read('state-flags/identities.json'));assert sha(json.dumps(recipe,separators=(',',':'),ensure_ascii=False).encode())==ids['source_sha256']
            assert b'18446744073709551615' in prepared.read('state-flags/arguments.json')
with zipfile.ZipFile('target/native-state-followup/multiscale-workflow.zip') as z:
    safe(z);w=json.loads(z.read('workflow-receipt.json'))
    for name,expected in w['file_sha256'].items():assert sha(z.read(name))==expected
    assert len(json.loads(z.read('plan.rplan'))['plan']['units']['unit_ids'])==8
print('PASS native State follow-up archives: four nested batch/local packages, fine-unit multiscale files, original recipe identities and two streamed completion artifact sets')
