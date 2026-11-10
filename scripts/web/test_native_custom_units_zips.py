"""Independently check source bundles, original companions and native exports; no execution."""
from pathlib import Path
from zipfile import ZipFile
import base64,hashlib,io,json
browser=Path('target/native-custom-units-browser');fixtures=Path('target/native-custom-units');count=0;archives=0
def digest(b):return hashlib.sha256(b).hexdigest()
def check_prepared(package,expected_kind):
 global count,archives
 archives+=1;assert package.testzip() is None
 names=package.namelist();assert len(names)==len(set(names));assert all(not n.startswith('/') and '..' not in n.split('/') for n in names)
 recipe=json.loads(package.read('native-inputs/recipe.json'));assert recipe['unit_kind']==expected_kind and recipe['year']=='2020'
 for key,name in [('adjacency','adjacency.bin'),('geoids','geoids.json'),('centroids','centroids.json'),('county_ids','county-ids.json')]:
  assert package.read('native-inputs/'+name)==base64.b64decode(recipe['sources'][key]['base64']);count+=1
 companion=json.loads(package.read('native-inputs/county-ids.json'));mapping=json.loads(package.read('native-inputs/geoids.json'));assert companion['unit_ids']==[mapping[str(i)] for i in range(len(mapping))]
 receipt=json.loads(package.read('prepared-run-receipt.json'));export=receipt['prepared']['export'];assert package.read('plan.rplan').decode()==export['document_json'];assert package.read('context.rctx').decode()==export['context_json'];assert package.read('accepted-replay.json').decode()==receipt['prepared']['replay_artifact_json'];count+=3
 plan=json.loads(package.read('plan.rplan'));context=json.loads(package.read('context.rctx'));assert plan['plan']['units']['unit_kind']==context['units']['unit_kind']==expected_kind;assert context['subdivisions']['county_ids']==companion['county_ids'];assert context['units']['unit_ids']==companion['unit_ids'];assert plan['plan']['units']['unit_ids']==companion['unit_ids']
for kind,year in [('precinct','2020'),('imported','2024')]:
 project=json.loads((browser/(kind+'-exported.bisect')).read_text(encoding='utf-8'));graph_source=base64.b64decode(project['files']['configRunInputs']['graph']['base64'])
 with ZipFile(browser/(kind+'-sources.zip')) as package:
  archives+=1;assert package.testzip() is None
  manifest=json.loads(package.read('bundle.json'));assert set(package.namelist())==set(manifest['files'])|{'bundle.json'}
  for name,identity in manifest['files'].items():
   data=package.read(name);assert identity=={'bytes':len(data),'sha256':digest(data)};count+=1
  assert package.read('sources/graph.json')==graph_source;graph=json.loads(graph_source);assert graph['unit_kind']==kind and graph['year']==year
  companion=json.loads(base64.b64decode(project['files']['nativeAdjacencyInputs']['county_ids']['base64']));assert graph['geoids']==companion['unit_ids'] and graph['county_ids']==companion['county_ids'];assert graph['geoids'][:2]==['A','é/2']
for kind in ['precinct','imported']:
 with ZipFile(fixtures/(kind+'-prepared.zip')) as package:check_prepared(package,kind)
with ZipFile(browser/'batch.zip') as package:
 archives+=1;assert package.testzip() is None
 receipt=json.loads(package.read('batch-receipt.json'));row=receipt['states'][0];assert row['status']=='ok'
 with ZipFile(io.BytesIO(package.read('RI/prepared-run.zip'))) as nested:check_prepared(nested,'precinct')
 native_source=row['result']['native_input']['sources']['county_ids'];assert digest(base64.b64decode(native_source['base64']))==row['source_preparation']['sources']['county_ids']['sha256']
 graph=json.loads(package.read('RI/prepared-graph.json'));companion=json.loads(base64.b64decode(native_source['base64']));assert graph['geoids']==companion['unit_ids'] and graph['county_ids']==companion['county_ids']
record={'archives_including_nested':archives,'exact_source_or_native_files':count,'source_bundle_hashes_and_crc_verified':True,'original_county_companion_bytes_preserved':True,'exact_order_and_context_county_ids':True,'explicit_custom_year_preserved':True,'script_or_engine_execution':False}
(fixtures/'zip-evidence.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8');print(f'Custom native ZIPs: {archives} archives, {count} exact files; county companions, custom IDs, year and native exports preserved.')
