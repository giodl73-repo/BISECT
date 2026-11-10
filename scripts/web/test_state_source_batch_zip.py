import sys
import base64,io,json,zipfile
from pathlib import Path
root=Path('target/state-source-batch')
for filename in ['source.zip','failed-prepared.zip']+(['real-ri.zip'] if '--real' in sys.argv else []):
 with zipfile.ZipFile(root/filename) as outer:
  assert outer.testzip() is None
  receipt=json.loads(outer.read('batch-receipt.json'))
  for row in receipt['states']:
   prefix=row['state_code']+'/'
   prepared=row['source_preparation']
   assert outer.read(prefix+'prepared-graph.json').decode()==prepared['graph_json']
   assert outer.read(prefix+'prepared-geometry.json').decode()==prepared['geometry_json']
   assert json.loads(outer.read(prefix+'source-preparation.json'))==prepared
   if row['status']=='ok':
    with zipfile.ZipFile(io.BytesIO(outer.read(prefix+'native-run.zip'))) as inner:
     assert inner.testzip() is None
     assert inner.read('plan.rplan').decode()==row['result']['practitioner_export']['document_json']
     assert inner.read('context.rctx').decode()==row['result']['practitioner_export']['context_json']
    with zipfile.ZipFile(io.BytesIO(outer.read(prefix+'native-workflow.zip'))) as workflow:
     assert workflow.testzip() is None
     for name,value in row['result']['workflow']['files_base64'].items():assert workflow.read(name)==base64.b64decode(value)
   else:assert prefix+'native-run.zip' not in outer.namelist()
print('Source batch ZIPs: exact prepared graph/geometry/evidence, native plan/context and all analysis/map/report bytes/CRCs; refused generation exports preparation without a fabricated Run.')
