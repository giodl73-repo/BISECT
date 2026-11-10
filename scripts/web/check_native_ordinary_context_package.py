"""Immutable packaged ordinary replay/context regression; not full acceptance."""
from pathlib import Path
import argparse,base64,hashlib,json,os,subprocess
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--package',type=Path,required=True);parser.add_argument('--host',type=Path,required=True);parser.add_argument('--native-map-reference',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--c-sidecar',action='store_true');parser.add_argument('--browser',action='store_true');args=parser.parse_args()
scripts=Path(__file__).resolve().parent;package=args.package.resolve();output=args.output.resolve();host=args.host.resolve();reference=args.native_map_reference.resolve()
assert package.is_dir()and(package/'bisect_wasm.wasm').is_file()and host.is_file()and reference.is_file()
assert not output.exists()and not output.is_relative_to(package)and not package.is_relative_to(output)
if args.browser:assert os.environ.get('BISECT_PLAYWRIGHT_PACKAGES')
if args.c_sidecar:assert(package/'c-metis/metis-sidecar.json').is_file()
output.mkdir(parents=True);sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
def hashes():return{f.relative_to(package).as_posix():sha(f)for f in sorted(package.rglob('*'))if f.is_file()}
original=hashes();common=['--wasm',str(package/'bisect_wasm.wasm')]
if args.c_sidecar:common+=['--c-sidecar',str(package/'c-metis')]
steps=[('runtime',['node',str(scripts/'test_native_ordinary_context.mjs'),*common,'--host',str(host),'--output',str(output/'runtime')])]
if args.browser:steps.append(('browser',['node',str(scripts/'test_native_ordinary_context_browser.mjs'),*common,'--site',str(package),'--output',str(output/'browser')]))
sources=[Path(__file__).resolve(),scripts/'test_native_ordinary_context.mjs',scripts/'test_native_ordinary_context_browser.mjs',scripts/'fixtures/native-ordinary-context.mjs',scripts/'fixtures/native-ordinary-recorded-replay.json',*sorted((scripts/'fixtures/native-ordinary-recorded-replay').glob('*.json'))]
record=dict(schema='bisect-native-ordinary-context-package-v1',status='running',scope='Complete fixed native ordinary source fixtures, both declared feature profiles and optional actual browser/readback. Frozen actual native originals; not fresh native filesystem, real-state/capacity/all-method/publisher or full release acceptance.',package=str(package),package_files=original,host_sha256=sha(host),native_map_reference_sha256=sha(reference),sources={str(f):sha(f)for f in sources},explicit_c=args.c_sidecar,browser_requested=args.browser,steps=[],full_parity=False)
try:
 for name,command in steps:
  with(output/(name+'.log')).open('wb')as log:result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
  record['steps'].append(dict(name=name,command=command,exit_code=result.returncode,log_sha256=sha(output/(name+'.log'))));assert result.returncode==0,name
 if args.browser:
  evidence=list((output/'browser').glob('*/evidence.json'));assert len(evidence)==1;broot=evidence[0].parent;browser=json.loads(evidence[0].read_text(encoding='utf-8'));assert browser['status']=='passed-maintained-native-ordinary-context-browser'and len(browser['cases'])==11
  request=json.loads((broot/'map-request.json').read_text(encoding='utf-8'))['input'];plan=broot/'native-map-plan.rplan';geometry=broot/'native-map-geometry.json';plan.write_text(request['document_json'],encoding='utf-8');geometry.write_text(request['unit_geometry_json'],encoding='utf-8')
  mapped=subprocess.run([str(reference),str(plan),str(geometry),str(request['dpi']),str(broot/'native-map')],capture_output=True);(broot/'native-map.stdout').write_bytes(mapped.stdout);(broot/'native-map.stderr').write_bytes(mapped.stderr);assert mapped.returncode==0,mapped.stderr
  expected=json.loads((broot/'expected-map.json').read_text(encoding='utf-8'));assert sha(broot/'native-map.png')==expected['png_sha256']==sha(broot/'ordinary-map.png')
  source=json.loads((broot/'input.bisect').read_text(encoding='utf-8'));ran=json.loads((broot/'node-replay.json').read_text(encoding='utf-8'));assert(broot/'download-context').read_text(encoding='utf-8')==ran['replay']['export']['context_json']
  assert json.loads((broot/'download-plan').read_text(encoding='utf-8'))['plan']==ran['replay']['export']['document']['plan']
  for name in ['audit-handoff.bisect','map-handoff.bisect']:
   saved=json.loads((broot/name).read_text(encoding='utf-8'));assert saved['result']==ran
   for key in ['packageVerificationInputs','nativeReplaySources','nativeRecordedReplaySettings']:assert saved['files'][key]==source['files'][key]
   receipt=json.loads(base64.b64decode(saved['files']['nativeRecordedReplayReceipt']['base64']));assert receipt['context_sha256']==hashlib.sha256(ran['replay']['export']['context_json'].encode()).hexdigest()
  assert'nativeRecordedReplayReceipt'not in json.loads((broot/'stale-handoff.bisect').read_text(encoding='utf-8'))['files']
  record['independent_browser_readback']=dict(status='passed',native_png_sha256=expected['png_sha256'],exact_context_plan_and_source_retention=True,stale_receipt_absent=True)
 record['package_unchanged']=hashes()==original;assert record['package_unchanged'];record['status']='passed'
except BaseException as error:record.update(status='failed',error=repr(error));raise
finally:(output/'evidence.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(status=record['status'],root=str(output),browser_requested=args.browser)))
