import base64,hashlib,io,json,zipfile
from pathlib import Path
root=Path('target/large-source')
prepared=json.loads((root/'prepared.json').read_text(encoding='utf-8'))
run=json.loads((root/'run.json').read_text(encoding='utf-8'))
with zipfile.ZipFile(root/'batch.zip') as outer:
 assert outer.testzip() is None
 receipt=json.loads(outer.read('batch-receipt.json'))
 assert receipt['receipt_storage']=='separate-large-source-files-v1'
 row=receipt['states'][0];assert row['state_code']=='RI' and row['status']=='ok'
 assert 'result' not in row and 'geometry_json' not in row['source_preparation']
 graph=outer.read(row['source_preparation']['graph_path']).decode()
 geometry=outer.read(row['source_preparation']['geometry_path']).decode()
 assert graph==prepared['graph_json'] and geometry==prepared['geometry_json']
 metadata=json.loads(outer.read('RI/source-preparation.json'))
 assert metadata.pop('graph_path')=='prepared-graph.json'
 assert metadata.pop('geometry_path')=='prepared-geometry.json'
 assert {**metadata,'graph_json':graph,'geometry_json':geometry}==prepared
 assert hashlib.sha256(geometry.encode()).hexdigest()==prepared['geometry_sha256']
 with zipfile.ZipFile(io.BytesIO(outer.read(row['native_run_archive']))) as native:
  assert native.testzip() is None
  assert json.loads(native.read(row['native_run_receipt']))==run
  assert native.read('plan.rplan').decode()==run['practitioner_export']['document_json']
  assert native.read('context.rctx').decode()==run['practitioner_export']['context_json']
 with zipfile.ZipFile(io.BytesIO(outer.read('RI/native-workflow.zip'))) as workflow:
  assert workflow.testzip() is None
  for name,value in run['workflow']['files_base64'].items():
   payload=workflow.read(name);assert payload==base64.b64decode(value)
   assert hashlib.sha256(payload).hexdigest()==run['workflow']['file_sha256'][name]
  assert json.loads(workflow.read('workflow-receipt.json'))=={k:v for k,v in run['workflow'].items() if k!='files_base64'}
 assert len(outer.read('batch-receipt.json'))<100_000
 assert len(outer.read('RI/source-preparation.json'))<100_000
print(f'Large-source ZIP: CRCs, exact graph/geometry, reconstructable preparation, full native Run receipt, plan/context and every workflow byte/hash passed ({(root/"batch.zip").stat().st_size} bytes).')
