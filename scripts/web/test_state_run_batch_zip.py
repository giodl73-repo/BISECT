"""Inspect outer/inner native batch ZIPs independently of the JS writer."""
import base64,io,json,zipfile
from pathlib import Path
root=Path('target/state-run-batch')
for zip_name,project_name,expected_states in [('complete.zip','browser.bisect',3),('cancelled.zip',None,1),('workflow.zip','workflow.bisect',1)]:
 with zipfile.ZipFile(root/zip_name) as outer:
  assert outer.testzip() is None
  receipt=json.loads(outer.read('batch-receipt.json'))
  if project_name:
   project=json.loads((root/project_name).read_text(encoding='utf-8'))
   assert receipt==project['result']
  successful=[r for r in receipt['states'] if r['status']=='ok']
  assert len(successful)==expected_states
  for row in successful:
   with zipfile.ZipFile(io.BytesIO(outer.read(row['state_code']+'/native-run.zip'))) as inner:
    assert inner.testzip() is None
    assert inner.read('plan.rplan').decode()==row['result']['practitioner_export']['document_json']
    assert inner.read('context.rctx').decode()==row['result']['practitioner_export']['context_json']
    assert json.loads(inner.read('run-receipt.json'))==row['result']
   if 'workflow' in row['result']:
    with zipfile.ZipFile(io.BytesIO(outer.read(row['state_code']+'/native-workflow.zip'))) as workflow:
     assert workflow.testzip() is None
     for name,text in row['result']['workflow']['files_base64'].items():
      assert workflow.read(name)==base64.b64decode(text)
print('State batch ZIPs: full/cancelled/workflow CRCs, exact native plan/context/receipts and every analysis/map/report file passed.')
