import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {runInNewContext} from 'node:vm';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {isLabelBuildCheckpoint} from './fixtures/label-build-progress.mjs';
import {labelBuildRequest,startLabelBuildCheckpoint,retainLabelBuildCheckpoint} from '../../web/lab/label-build-files.js';
const archive=JSON.parse(await fs.readFile('target/label-build-wasm/browser-archive.bisect-build','utf8'));
const sources=archive.files.labelBuildInputs;
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const messages=[];let calls=0;
const scope={self:{postMessage:m=>{if(m.type==='build-started')assert.equal(calls,0);messages.push(m);}},fetch:async()=>({ok:true,arrayBuffer:async()=>new ArrayBuffer(0)}),instantiateEngine:async()=>({execute:(r,o)=>{calls++;assert.deepEqual(r,labelBuildRequest(sources));return engine.execute(r,o);},memoryBytes:()=>engine.memoryBytes()}),labelBuildRequest,startLabelBuildCheckpoint};
runInNewContext((await fs.readFile('web/lab/wasm-worker.js','utf8')).replace(/^import .*;\r?\n/gm,'').replaceAll('import.meta.url',JSON.stringify(new URL('../../web/lab/wasm-worker.js',import.meta.url).href)),scope);
await scope.self.onmessage({data:{type:'initialize',url:'local'}});
await scope.self.onmessage({data:{type:'run',id:72,request:{operation:'run-label-build-sources',sources}}});
assert.equal(calls,1);assert.deepEqual(messages.slice(0,3).map(m=>m.type),['ready','build-started','started']);
let retained=messages[1].checkpoint,nativeEvents=0;for(const m of messages.filter(m=>m.type==='progress')){assert.equal(m.id,72);if(!isLabelBuildCheckpoint(m.progress)){nativeEvents++;continue;}retained=retainLabelBuildCheckpoint(retained,m.progress);}
const result=messages.at(-1);assert.equal(result.type,'result');assert.equal(result.id,72);assert.deepEqual(retained.files,result.result.files);
let rejects=0;
for(const request of [{operation:'run-label-build-sources',sources,extra:true},{operation:'run-label-build-sources',sources:{...sources,jobs:null}},{operation:'run-label-build-sources',sources:{...sources,settings:{...sources.settings,years:['202']}}},{operation:'run-label-build-sources',sources:{...sources,jobs:{name:'bad.json',base64:Buffer.from('[{"extra":1}]').toString('base64')}}}]){await scope.self.onmessage({data:{type:'run',id:73,request}});assert.equal(messages.at(-1).type,'error');assert.equal(calls,1);rejects++;}
await fs.writeFile('target/label-build-wasm/start-evidence.json',JSON.stringify({worker_source_preflight:true,checkpoint_before_execution:true,native_round_events:nativeEvents,exact_final_files:true,rejects,engine_calls:calls},null,2));
console.log('Build worker start: exact source preflight, checkpoint before execution, real WASM final files and four pre-engine rejects passed.');
