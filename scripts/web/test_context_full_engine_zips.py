"""Independently verify queued context/source, request, native output and report bytes."""
from pathlib import Path
from zipfile import ZipFile
import hashlib,io,json
root=Path('target/context-full-engine');archives=0;files=0;artifacts=0
def check(package):
 global archives,files,artifacts
 archives+=1;assert package.testzip() is None;names=package.namelist();assert len(names)==len(set(names)) and all(not n.startswith('/') and '..' not in n.split('/') for n in names)
 row=json.loads(package.read('prepared-run-receipt.json'));native=row['prepared'];attachment=native['context_engine_source'];source=attachment['source'];origin=attachment['origin'];original=package.read('context-inputs/original.rctx');assert original==source['context_json'].encode('utf-8');assert json.loads(package.read('context-inputs/origin.json'))==origin;assert origin['source_sha256']==hashlib.sha256(original).hexdigest();assert source['physical_edge_weights_metres'] is True
 assert row['request_source']['context_source']==source
 request=package.read('request.json').decode();assert request==row['request_source']['source']['text'];requested=json.loads(request);assert (requested['request'] if 'request' in requested else requested['input']['request'] if 'input' in requested else requested)['options']['seed']==18446744073709551615
 graph=(requested['request'] if 'request' in requested else requested['input']['request'] if 'input' in requested else requested)['graph'];context=json.loads(original);assert graph['geoids']==context['units']['unit_ids'] and graph['population']==context['populations'];assert graph['areas']==graph['exterior_perimeters']==graph['centroids']==[];assert origin['context_hash']==context['context_hash'] and origin['unit_universe_hash']==context['units']['unit_universe_hash']
 expected_graph=json.loads(json.loads((root/(graph['unit_kind']+'-prepared.json')).read_text(encoding='utf-8'))['graph_json']);assert graph==expected_graph;assert origin['graph_sha256']==hashlib.sha256(json.loads((root/(graph['unit_kind']+'-prepared.json')).read_text(encoding='utf-8'))['graph_json'].encode()).hexdigest()
 assert package.read('plan.rplan').decode()==native['export']['document_json'];assert package.read('context.rctx').decode()==native['export']['context_json'];assert package.read('accepted-replay.json').decode()==native['replay_artifact_json'];assert json.loads(package.read('final_assignments.json'))==native['result']['assignments'];files+=8
 if 'ilp_report_capture' in native['result']:
  packet=native['result']['ilp_report_capture']
  for name,text in packet['files'].items():assert packet['file_sha256'][name]==hashlib.sha256(text.encode()).hexdigest();artifacts+=1
for kind in ['tract','precinct']:
 for mode in ['standard','compact','ratio','ilp']:
  with ZipFile(root/(kind+'-'+mode+'.zip')) as package:check(package)
for mode in ['ordinary','multiscale','multiscale-build']:
 with ZipFile(root/('tract-'+mode+'.zip')) as package:check(package)
with ZipFile('target/context-full-engine-browser/batch.zip') as outer:
 archives+=1;assert outer.testzip() is None;receipt=json.loads(outer.read('batch-receipt.json'));row=receipt['states'][0];assert row['status']=='ok'
 with ZipFile(io.BytesIO(outer.read('RI/prepared-run.zip'))) as nested:
  check(nested);saved=json.loads(nested.read('prepared-run-receipt.json'));assert saved==row['result']
(root/'zip-evidence.json').write_text(json.dumps({'archives_including_nested':archives,'exact_native_or_source_files':files,'captured_artifact_hashes':artifacts,'raw_max_u64_request_seed_preserved':True,'original_context_origin_and_graph_bound':True,'native_plan_context_replay_exact':True,'crc_safe_paths_verified':True,'script_or_model_execution':False},indent=2)+'\n',encoding='utf-8');print(f'RCTX full-engine ZIPs: {archives} archives, {files} exact files and {artifacts} captured artifact hashes; original context/u64/native outputs preserved.')
