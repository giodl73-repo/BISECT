import assert from 'node:assert/strict';import fs from 'node:fs/promises';import path from 'node:path';import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';
import {stateRunBatchZip} from '../../web/lab/state-run-batch.js';import {compressedStateBatchZip} from '../../web/lab/state-batch-zip.js';import {preflightMatrixPrerequisites,preflightMatrixCapacity} from './fixtures/matrix-prerequisites.mjs';
const reference=path.resolve(process.argv[2]||'target/national-matrix/wasm-1792827814125'),root=path.resolve('target/national-matrix/zip-'+Date.now());await fs.mkdir(root,{recursive:true});if(process.platform==='win32')assert.equal(spawnSync('compact.exe',['/C','/I','/Q',root],{windowsHide:true}).status,0);
if(await preflightMatrixPrerequisites(root,[path.join(reference,'evidence.json')])&&await preflightMatrixCapacity(root,2*1024**3)){
 const evidence=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));assert.equal(evidence.status,'passed-national-wasm-tier');assert.equal(evidence.outcomes.length,50);const states=[];
 for(const row of evidence.outcomes)states.push({state_code:row.state,status:'ok',result:JSON.parse(await fs.readFile(path.join(reference,row.state,'wasm-reply.json'),'utf8'))});
 await assert.rejects(()=>compressedStateBatchZip([['../escape',new Uint8Array([1])]]),/Invalid ZIP/);await assert.rejects(()=>compressedStateBatchZip([['same',new Uint8Array()],['same',new Uint8Array()]]),/Invalid ZIP/);
 const small=await compressedStateBatchZip([['empty',new Uint8Array()],['utf8.txt',new TextEncoder().encode('Exact UTF-8: café; zero\0; newline\n')]]);await fs.writeFile(path.join(root,'small.zip'),small);
 console.log('Export complete 50-state native-checked receipts');const result={schema_version:'bisect-state-run-batch-v1',status:'completed',selection:{year:'2020'},summary:{total:50,completed:50,succeeded:50,failed:0},states},zip=await stateRunBatchZip(result),file=path.join(root,'national.zip');await fs.writeFile(file,zip);
 const check=spawnSync('python',['-c',`import sys,zipfile,json,io,hashlib,pathlib
root=pathlib.Path(sys.argv[1]); ref=pathlib.Path(sys.argv[2])
with zipfile.ZipFile(root/'small.zip') as z:
 assert z.testzip() is None
 assert z.read('empty')==b''
 assert z.read('utf8.txt').decode('utf-8')=='Exact UTF-8: café; zero\\0; newline\\n'
with zipfile.ZipFile(root/'national.zip') as z:
 assert z.testzip() is None
 r=json.loads(z.read('batch-receipt.json')); assert r['schema_version']=='bisect-state-run-batch-export-v2'
 assert len(r['states'])==50; assert len(z.namelist())==51
 for row in r['states']:
  a=row['result_archive']; b=z.read(a['path']); assert len(b)==a['byte_length']; assert hashlib.sha256(b).hexdigest()==a['sha256']
  with zipfile.ZipFile(io.BytesIO(b)) as state:
   assert state.testzip() is None
   actual=json.loads(state.read(a['receipt_path'])); expected=json.loads((ref/row['state_code']/'wasm-reply.json').read_text(encoding='utf-8')); assert actual==expected
   assert state.read('plan.rplan').decode()==expected['practitioner_export']['document_json']
   assert state.read('context.rctx').decode()==expected['practitioner_export']['context_json']
print('Independent Python ZIP/CRC, all 50 exact receipts and accepted plan/context bytes pass')`,root,reference],{encoding:'utf8',windowsHide:true,timeout:300000,maxBuffer:1024*1024});await fs.writeFile(path.join(root,'validation.log'),check.stdout+check.stderr);assert.equal(check.status,0,check.stderr);
 await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({schema_version:'bisect-national-zip-evidence-v1',status:'passed',states:50,bytes:zip.length,sha256:createHash('sha256').update(zip).digest('hex'),independent_python_zip_crc_and_exact_receipts:true,hostile_paths_and_duplicates_refused:true,reference,scope:'Output-only ZIP correction on actual native-checked WASM receipts. No browser scheduler/map/recovery or import support claim.'},null,2));console.log(JSON.stringify({root,bytes:zip.length,status:'passed'}));
}
