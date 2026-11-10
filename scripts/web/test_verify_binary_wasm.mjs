import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import vm from 'node:vm';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {manifestReplayRequest,validateEngineReplayInputs} from '../../web/lab/engine-replay-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/verify-binary-wasm';await fs.mkdir(root,{recursive:true});
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes),identity=engine.artifactIdentity;
const base=JSON.parse(await fs.readFile('target/replay-bindings-wasm/bound.bisect','utf8'));
let matches=0,refusals=0;const evidence=[];
for(const [name,recorded,runtime,skip,status]of [
 ['match',identity.sha256,identity,false,'match'],['mismatch','0'.repeat(64),identity,false,'mismatch'],
 ['literal-uppercase',identity.sha256.toUpperCase(),identity,false,'mismatch'],['unavailable',identity.sha256,null,false,'unavailable'],
 ['skip',identity.sha256,identity,true,'skipped'],['missing','',identity,false,'not-recorded'],
 ['sentinel','(not available)',identity,true,'not-recorded'],['missing-skip','',null,true,'not-recorded']]){
 const files=structuredClone(base.files),text=Buffer.from(files.packageVerificationInputs.manifest.base64,'base64').toString(),field='\"binary_sha256\":'+JSON.stringify(recorded),updated=/\"binary_sha256\"\s*:\s*\"[^\"]*\"/.test(text)?text.replace(/\"binary_sha256\"\s*:\s*\"[^\"]*\"/,()=>field):text.replace('{','{'+field+',');files.packageVerificationInputs.manifest.base64=Buffer.from(updated).toString('base64');files.engineReplayInputs.skipBinaryCheck=skip;
 const request={...manifestReplayRequest(files.packageVerificationInputs,files.engineReplayInputs),runtime_artifact:runtime},result=engine.execute(request);
 const filename=root+'/'+name+'-request.json';await fs.writeFile(filename,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[filename],{encoding:'utf8',maxBuffer:32*1024*1024,windowsHide:true}));assert.deepEqual(result,native);matches++;
 assert.equal(result.binary_check.status,status);assert.equal(result.binary_check.advisory,true);assert.equal(result.status,'match');assert.equal(result.verification_exit_code,0);assert.equal(result.execution_completed,true);assert.equal(result.skip_binary_check,skip);const rows=countResultTables(result)[0].rows;assert.ok(rows.some(([key,value])=>key==='Binary advisory'&&value===status));assert.ok(rows.some(([key])=>key==='Loaded WASM SHA-256'));
 const project=createProject({name,operation:'verify-manifest-replay',files,constraints:[],result,lastOperation:'verify-manifest-replay'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result,null,2));evidence.push({name,binary:status,replay:result.status});
}
const good=JSON.parse(await fs.readFile(root+'/match-request.json','utf8'));
for(const runtime of [{...identity,extra:1},{sha256:'X'.repeat(64),byte_length:1},{...identity,byte_length:0},{...identity,byte_length:256*1024*1024+1}]){assert.throws(()=>engine.execute({...good,runtime_artifact:runtime}));refusals++;}
assert.throws(()=>engine.execute({...good,skip_binary_check:'yes'}));refusals++;
assert.throws(()=>validateEngineReplayInputs({...base.files.engineReplayInputs,skipBinaryCheck:'yes'}));refusals++;
const badProject=JSON.parse(await fs.readFile('target/manifest-replay-wasm/bad-digest.bisect','utf8')),bad={...manifestReplayRequest(badProject.files.packageVerificationInputs,badProject.files.engineReplayInputs),runtime_artifact:{sha256:'invalid',byte_length:0}};const blocked=engine.execute(bad);assert.equal(blocked.status,'precheck-failed');assert.equal(blocked.binary_check.status,'not-run');assert.equal(blocked.execution_performed,false);assert.equal(blocked.replay,null);await fs.writeFile(root+'/blocked-request.json',JSON.stringify(bad));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/blocked-request.json'],{encoding:'utf8',windowsHide:true})),blocked);matches++;
const messages=[],self={postMessage:m=>messages.push(m)},worker=await fs.readFile('web/lab/wasm-worker.js','utf8');const sandbox=vm.createContext({self,crypto,Uint8Array,ArrayBuffer,Number,Error,URL,fetch:async()=>({ok:true,arrayBuffer:async()=>Uint8Array.from(bytes).buffer}),instantiateEngine});vm.runInContext(worker.replace(/^import .*;\r?$/gm,'').replaceAll('import.meta.url',JSON.stringify(new URL('../../web/lab/wasm-worker.js',import.meta.url).href)),sandbox);await self.onmessage({data:{type:'initialize',url:'https://local.invalid/engine.wasm'}});
await self.onmessage({data:{type:'run',id:1,request:{...good,runtime_artifact:{sha256:'0'.repeat(64),byte_length:1}}}});assert.equal(messages.at(-1).type,'result');assert.deepEqual(messages.at(-1).result.binary_check.loaded_wasm,identity);assert.equal(messages.at(-1).result.binary_check.status,'match');
await fs.writeFile(root+'/evidence.json',JSON.stringify({matches,refusals,cases:evidence,identity,worker_overrides_claim:true,prechecks_stop_before_binary_and_execution:true},null,2));console.log(`Composed replay binary advisory: ${matches} whole native/WASM matches, ${refusals} refusals, worker-owned fingerprint, literal/sentinel/skip semantics and stopped precheck ordering passed.`);
