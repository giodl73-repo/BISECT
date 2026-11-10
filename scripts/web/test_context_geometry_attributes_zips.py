"""Independently read source/context/attribute archives; no payload execution."""
from pathlib import Path
from zipfile import ZipFile
import base64,hashlib,io,json
root=Path('target/context-geometry-attributes');browser=Path('target/context-geometry-attributes-browser');archives=0;files=0;claims=0
hashbytes=lambda b:hashlib.sha256(b).hexdigest()
def check_source(package,project=None):
 global archives,files
 archives+=1;assert package.testzip() is None;manifest=json.loads(package.read('bundle.json'));assert set(package.namelist())==set(manifest['files'])|{'bundle.json'}
 for name,identity in manifest['files'].items():
  assert not name.startswith('/') and '..' not in name.split('/');b=package.read(name);assert identity=={'bytes':len(b),'sha256':hashbytes(b)};files+=1
 context_bytes=package.read('sources/original-context.rctx');attributes=package.read('sources/context-attributes.json');origin=json.loads(package.read('sources/context-engine-origin.json'));graph_bytes=package.read('sources/graph.json');context=json.loads(context_bytes);doc=json.loads(attributes);graph=json.loads(graph_bytes)
 assert origin['source_sha256']==hashbytes(context_bytes) and origin['graph_sha256']==hashbytes(graph_bytes);assert origin['attributes_source']['source_sha256']==hashbytes(attributes) and origin['attributes_source']['bytes']==len(attributes)
 assert doc['context_hash']==origin['context_hash']==context['context_hash'];assert doc['unit_ids']==context['units']['unit_ids']==graph['geoids'];assert doc['unit_kind']==context['units']['unit_kind']==graph['unit_kind'];assert doc['area_units']=='m2' and doc['length_units']=='m' and doc['centroid_crs']=='WGS84'
 for k in ['areas','exterior_perimeters','centroids']:assert doc[k]==graph[k] and len(graph[k])==len(graph['geoids'])
 if project:
  assert attributes==base64.b64decode(project['files']['contextEngineInputs']['attributes']['base64']);assert context_bytes==base64.b64decode(project['files']['contextEngineInputs']['source']['base64']);assert package.read('plan.rplan').decode()==project['result']['practitioner_export']['document_json'];assert package.read('context.rctx').decode()==project['result']['practitioner_export']['context_json']
def check_state(package):
 global archives,files,claims
 archives+=1;assert package.testzip() is None;names=package.namelist();assert len(names)==len(set(names)) and all(not n.startswith('/') and '..' not in n.split('/') for n in names)
 receipt=json.loads(package.read('prepared-run-receipt.json'));native=receipt['prepared'];source=native['context_engine_source']['source'];origin=native['context_engine_source']['origin'];context=package.read('context-inputs/original.rctx');attrs=package.read('context-inputs/attributes.json');assert context==source['context_json'].encode();assert attrs==source['attributes_source']['attributes_json'].encode();assert json.loads(package.read('context-inputs/origin.json'))==origin;assert origin['source_sha256']==hashbytes(context);assert origin['attributes_source']=={'source_name':source['attributes_source']['source_name'],'source_sha256':hashbytes(attrs),'bytes':len(attrs)}
 doc=json.loads(attrs);graph=native['executed_request']['graph'];assert doc['context_hash']==json.loads(context)['context_hash'];assert doc['unit_ids']==graph['geoids'];assert graph['population']==json.loads(context)['populations']
 for k in ['areas','exterior_perimeters','centroids']:assert graph[k]==doc[k]
 assert package.read('request.json').decode()==receipt['request_source']['source']['text'];assert package.read('plan.rplan').decode()==native['export']['document_json'];assert package.read('context.rctx').decode()==native['export']['context_json'];assert package.read('accepted-replay.json').decode()==native['replay_artifact_json'];assert json.loads(package.read('final_assignments.json'))==native['result']['assignments'];files+=9
 if native['executed_request']['options']['structure']=='centroidal-voronoi':assert native['native_build'] is False and native['result']['metrics']['within_requested_tolerance'] is False;claims+=1
for kind in ['tract','precinct','imported']:
 with ZipFile(root/(kind+'-sources.zip')) as package:check_source(package)
for kind in ['tract','precinct']:
 for mode in ['area','area-moving','moving-knife','geographic-cvd']:
  with ZipFile(root/(kind+'-'+mode+'.zip')) as package:check_state(package)
project=json.loads((browser/'exported.bisect').read_text(encoding='utf-8'))
with ZipFile(browser/'sources.zip') as package:check_source(package,project)
with ZipFile(browser/'batch.zip') as outer:
 archives+=1;assert outer.testzip() is None;row=json.loads(outer.read('batch-receipt.json'))['states'][0];assert row['status']=='ok'
 with ZipFile(io.BytesIO(outer.read('RI/prepared-run.zip'))) as package:check_state(package);assert json.loads(package.read('prepared-run-receipt.json'))==row['result']
(root/'zip-evidence.json').write_text(json.dumps({'archives_including_nested':archives,'exact_source_or_native_files':files,'original_attribute_bytes_and_context_bound':True,'scope_units_unit_order_and_vectors_exact':True,'cvd_unbalanced_candidates_not_certified':claims,'crc_paths_hashes_verified':True,'script_or_engine_execution':False},indent=2)+'\n',encoding='utf-8');print(f'RCTX attribute ZIPs: {archives} archives, {files} exact files; original context/attributes/native outputs and {claims} explicitly unbalanced CVD candidates checked.')
