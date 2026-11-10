"""Read downloaded context-origin bundles independently; do not execute payloads."""
from pathlib import Path
from zipfile import ZipFile
import json,hashlib,base64
root=Path('target/context-engine-input-browser');checked=0
for kind in ['tract','precinct','imported']:
 project=json.loads((root/(kind+'-exported.bisect')).read_text(encoding='utf-8'))
 with ZipFile(root/(kind+'-sources.zip')) as package:
  assert package.testzip() is None
  manifest=json.loads(package.read('bundle.json'));assert set(package.namelist())==set(manifest['files'])|{'bundle.json'}
  for name,identity in manifest['files'].items():
   assert not name.startswith('/') and '..' not in name.split('/');data=package.read(name);assert identity=={'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()};checked+=1
  original=package.read('sources/original-context.rctx');origin=json.loads(package.read('sources/context-engine-origin.json'));graph_bytes=package.read('sources/graph.json');graph=json.loads(graph_bytes);context=json.loads(original)
  assert original==base64.b64decode(project['files']['contextEngineInputs']['source']['base64'])
  assert origin['source_sha256']==hashlib.sha256(original).hexdigest() and origin['graph_sha256']==hashlib.sha256(graph_bytes).hexdigest()
  assert origin['context_hash']==context['context_hash'] and origin['unit_universe_hash']==context['units']['unit_universe_hash'];assert origin['physical_edge_weights_metres'] is True and origin['edge_policy']=='all-context-edges'
  assert graph['geoids']==context['units']['unit_ids'] and graph['population']==context['populations'];assert graph['unit_kind']==kind
  assert graph['areas']==graph['exterior_perimeters']==graph['centroids']==[]
  export=project['result']['practitioner_export'];assert package.read('plan.rplan').decode()==export['document_json'];assert package.read('context.rctx').decode()==export['context_json']
  if kind!='tract':assert graph['county_ids']==context['subdivisions']['county_ids']
Path('target/context-engine-input/zip-evidence.json').write_text(json.dumps({'archives':3,'hashed_files':checked,'original_rctx_graph_and_origin_exact':True,'native_plan_context_exact':True,'missing_metadata_not_fabricated':True,'crc_and_paths_verified':True,'payload_execution':False},indent=2)+'\n',encoding='utf-8')
print(f'RCTX source ZIPs: three archives, {checked} member hashes, original context/origin/graph and native outputs exact.')
