"""Fixed full native unit-identity corpus, optional actual browser and native PNG readback."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import subprocess

parser=argparse.ArgumentParser(description=__doc__)
for name in ('package','host','host-sequence','native-map-reference','output'):
 parser.add_argument('--'+name,type=Path,required=True)
parser.add_argument('--c-sidecar',action='store_true')
parser.add_argument('--browser',action='store_true')
args=parser.parse_args();scripts=Path(__file__).resolve().parent
package=args.package.resolve();output=args.output.resolve();host=args.host.resolve();sequence=args.host_sequence.resolve();reference=args.native_map_reference.resolve()
assert package.is_dir() and (package/'bisect_wasm.wasm').is_file() and host.is_file() and sequence.is_file() and reference.is_file()
output.mkdir(parents=True,exist_ok=False)
def sha(path):
 with path.open('rb') as stream:return hashlib.file_digest(stream,'sha256').hexdigest()
def read(path):return json.loads(path.read_text(encoding='utf-8'))
def hashes():return {p.relative_to(package).as_posix():sha(p) for p in sorted(package.rglob('*')) if p.is_file()}
before=hashes();rows=read(scripts/'fixtures/native-imported-recorded-replay.json');assert len(rows)==12 and len({r['name'] for r in rows})==12
record={'schema':'bisect-native-unit-identity-package-v1','status':'running','package':str(package),'package_files':before,'host_sha256':sha(host),'host_sequence_sha256':sha(sequence),'native_map_reference_sha256':sha(reference),'explicit_c':args.c_sidecar,'browser_requested':args.browser,'steps':[],'browser_readbacks':[],'full_parity':False,'scope':'Complete fixed twelve-case actual native source/context/plan corpus and seven portable refusals. Native/C-enabled images use metis-core here; separate existing ordinary C-FFI fixture remains required. Not real-state capacity, source authentication, release provenance or parent acceptance.'}
def run(name,command):
 result=subprocess.run(command,capture_output=True)
 log=output/(name+'.log');log.write_bytes(result.stdout+result.stderr)
 record['steps'].append({'name':name,'command':command,'exit_code':result.returncode,'log_sha256':sha(log)})
 assert result.returncode==0,(name,result.stderr[-2000:])
try:
 runtime=['node',str(scripts/'test_native_unit_identity.mjs'),'--wasm',str(package/'bisect_wasm.wasm'),'--host',str(host),'--host-sequence',str(sequence),'--output',str(output/'runtime')]
 if args.c_sidecar:runtime+=['--c-sidecar',str(package/'c-metis')]
 run('runtime',runtime)
 result=list((output/'runtime').glob('*/evidence.json'));assert len(result)==1
 r=read(result[0]);assert r['status']=='passed-maintained-native-unit-identity' and len(r['cases'])==12 and len(r['refusals'])==7
 if args.browser:
  for row in rows:
   name=row['name'];parent=output/'browser'/name
   command=['node',str(scripts/'test_native_unit_identity_browser.mjs'),'--case',name,'--wasm',str(package/'bisect_wasm.wasm'),'--site',str(package),'--output',str(parent)]
   if args.c_sidecar:command+=['--c-sidecar',str(package/'c-metis')]
   run('browser-'+name,command)
   found=list(parent.glob('*/evidence.json'));assert len(found)==1;root=found[0].parent;b=read(found[0])
   assert b['status']=='passed-maintained-native-unit-identity-browser' and b['unit_case']==name and len(b['cases'])==11
   assert b['package_bytes_unchanged'] and not b['page_errors'] and not b['external_requests']
   ran=read(root/'node-replay.json');source=read(root/'input.bisect');assert ran['status']=='match'
   assert (root/'download-context').read_bytes()==row['context_json'].encode()
   assert read(root/'download-plan')==ran['replay']['export']['document'] and read(root/'download-plan')['plan']==json.loads(row['document_json'])['plan']
   assert read(root/'download-evidence')==ran
   assert read(root/'ordinary-audit-reply.json')['result']==read(root/'expected-audit.json')
   for saved_name in ('audit-handoff.bisect','map-handoff.bisect'):
    saved=read(root/saved_name);assert saved['result']==ran
    for key in ('packageVerificationInputs','nativeReplaySources','nativeRecordedReplaySettings'):assert saved['files'][key]==source['files'][key]
    receipt=json.loads(base64.b64decode(saved['files']['nativeRecordedReplayReceipt']['base64']))
    assert receipt['context_sha256']==hashlib.sha256(row['context_json'].encode()).hexdigest()
    assert receipt['document_sha256']==hashlib.sha256(ran['replay']['export']['document_json'].encode()).hexdigest() and receipt['comparison']==ran['comparison']
   assert 'nativeRecordedReplayReceipt' not in read(root/'stale-handoff.bisect')['files']
   requested=read(root/'map-request.json')['input'];plan=root/'native-reference.rplan';geometry=root/'native-reference-geometry.json'
   plan.write_bytes(requested['document_json'].encode());geometry.write_bytes(requested['unit_geometry_json'].encode())
   native=subprocess.run([str(reference),str(plan),str(geometry),str(requested['dpi']),str(root/'native-reference-map')],capture_output=True)
   (root/'native-reference-map.stdout').write_bytes(native.stdout);(root/'native-reference-map.stderr').write_bytes(native.stderr);assert native.returncode==0,native.stderr
   png=sha(root/'native-reference-map.png');assert png==read(root/'expected-map.json')['png_sha256']==sha(root/'ordinary-map.png')
   for label,asset in b['served_assets'].items():assert sha(package/(label.lstrip('/') or 'index.html'))==asset['sha256']
   audit={'name':name,'status':'passed','source_context_plan_receipt_and_audit_exact':True,'stale_receipt_absent':True,'native_png_sha256':png,'served_assets_rechecked':True}
   (root/'independent-readback.json').write_bytes(json.dumps(audit,indent=2).encode());record['browser_readbacks'].append(audit)
  assert len(record['browser_readbacks'])==12
 record['package_unchanged']=hashes()==before;assert record['package_unchanged'];record['status']='passed'
except BaseException as error:
 record.update(status='failed',error=repr(error));raise
finally:(output/'evidence.json').write_bytes(json.dumps(record,indent=2).encode())
print(json.dumps({'status':record['status'],'runtime_cases':12,'refusals':7,'browser_workflows':len(record['browser_readbacks']),'root':str(output)}))
