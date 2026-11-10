import os
import base64
import io
import json
import zipfile
from pathlib import Path

root=Path(os.environ.get('BISECT_PREPARED_OUTPUT','target/prepared-state-batch'))
jobs={j['scope']['state']:j for j in json.loads((root/'jobs.json').read_text(encoding='utf-8'))}
checked=0
for filename in ['outputs.zip','mixed-outputs.zip','no-export-outputs.zip']:
    with zipfile.ZipFile(root/filename) as outer:
        assert outer.testzip() is None
        assert len(outer.namelist())==len(set(outer.namelist()))
        assert all('..' not in name.split('/') and not name.startswith('/') for name in outer.namelist())
        receipt=json.loads(outer.read('batch-receipt.json'))
        for row in receipt['states']:
            if row['status']!='ok':continue
            state=row['state_code']
            if row['result']['schema_version']!='bisect-prepared-state-run-v1':
                assert state+'/native-run.zip' in outer.namelist()
                continue
            with zipfile.ZipFile(io.BytesIO(outer.read(state+'/prepared-run.zip'))) as inner:
                assert inner.testzip() is None
                data=row['result'];prepared=data['prepared']
                assert inner.read('request.json')==data['request_source']['source']['text'].encode('utf-8')
                assert inner.read('accepted-replay.json')==prepared['replay_artifact_json'].encode('utf-8')
                assert json.loads(inner.read('final_assignments.json'))==prepared['result']['assignments']
                assert json.loads(inner.read('prepared-run-receipt.json'))==data
                if prepared['export']:
                    assert inner.read('plan.rplan')==prepared['export']['document_json'].encode('utf-8')
                    assert inner.read('context.rctx')==prepared['export']['context_json'].encode('utf-8')
                else:
                    assert 'plan.rplan' not in inner.namelist() and 'context.rctx' not in inner.namelist()
                assert inner.read('display-geometry.geojson')==base64.b64decode(data['geometry']['base64'])
                if filename=='outputs.zip':
                    assert inner.read('request.json')==jobs[state]['prepared_engine']['source']['text'].encode('utf-8')
                    assert b'"seed":18446744073709551615' in inner.read('request.json')
                checked+=1
(root/'zip-evidence.json').write_text(json.dumps({'checked_prepared_state_archives':checked,'independent_python_zip_crc_and_path_checks':True,'exact_original_numeric_u64_request_bytes':True,'exact_native_plan_context_and_accepted_replay_bytes':True,'no_physical_export_fabricated':True},indent=2)+'\n',encoding='utf-8')
print(f'Prepared state ZIP: {checked} nested state archives, independent CRC/path checks, exact original/native/replay/geometry bytes and no fabricated physical export passed.')
