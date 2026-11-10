import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {runInNewContext} from 'node:vm';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createBuildArchive,parseBuildArchive,readBuildArchiveText,MAX_BUILD_ARCHIVE_BYTES} from '../../web/lab/label-build-archive.js';
import {labelBuildRequest,labelBuildZip} from '../../web/lab/label-build-files.js';
const request=JSON.parse(await fs.readFile('target/label-build-wasm/case-0.json','utf8'));
const i=request.input;const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={config:source(i.source_name,i.yaml_text),jobs:source('jobs.json',JSON.stringify(i.jobs)),registry:null,settings:{label:i.label,profile:i.profile,years:['2020','2010'],force:i.force,created_at:i.created_at}};
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const result=engine.execute(labelBuildRequest(inputs));
const text=createBuildArchive({name:'Archive fixture',inputs,result,lastOperation:'run-label-build'}),archive=parseBuildArchive(text);
assert.deepEqual(archive.result,result);assert.deepEqual(archive.files.labelBuildInputs,inputs);assert.deepEqual(labelBuildRequest(archive.files.labelBuildInputs),labelBuildRequest(inputs));assert.deepEqual(labelBuildZip(archive.result),labelBuildZip(result));
let rejects=0;
for(const mutate of [a=>a.extra=true,a=>a.operation='audit-plan',a=>a.constraints=['population'],a=>a.result.files['../escape']='x',a=>a.files.other={},a=>a.result.status='verified',a=>a.result.extra='unknown',a=>a.schema_version='bisect-project-v1']){const a=structuredClone(archive);mutate(a);assert.throws(()=>parseBuildArchive(JSON.stringify(a)));rejects++;}
for(const invalid of ['{"a":1,"a":2}','{"__proto__":{}}','{"seed":18446744073709551615}']){assert.throws(()=>parseBuildArchive(invalid));rejects++;}
let read=false;await assert.rejects(()=>readBuildArchiveText({size:MAX_BUILD_ARCHIVE_BYTES+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readBuildArchiveText({size:2,arrayBuffer:async()=>new Uint8Array([0xff,0xff]).buffer}));
await assert.rejects(()=>readBuildArchiveText({size:2,arrayBuffer:async()=>new Uint8Array([32]).buffer}));
const messages=[];const scope={self:{postMessage:v=>messages.push(v)},createBuildArchive,parseBuildArchive,readBuildArchiveText};runInNewContext((await fs.readFile('web/lab/label-build-archive-worker.js','utf8')).replace(/^import .*;\r?\n/gm,''),scope);
await scope.self.onmessage({data:new Blob([text])});assert.equal(messages.at(-1).ok,true);assert.deepEqual(messages.at(-1).project.result,result);
await scope.self.onmessage({data:new Blob(['{"a":1,"a":2}'])});assert.equal(messages.at(-1).ok,false);
await scope.self.onmessage({data:{type:'save-build-archive',data:{name:'Worker save',inputs,result,lastOperation:'run-label-build'}}});assert.equal(messages.at(-1).ok,true);assert.deepEqual(parseBuildArchive(messages.at(-1).text).result,result);
await fs.writeFile('target/label-build-wasm/browser-archive.bisect-build',text);await fs.writeFile('target/label-build-wasm/archive-evidence.json',JSON.stringify({exact_result:true,exact_sources:true,exact_zip:true,rejects,pre_read_cap:true,strict_utf8:true,dishonest_size:true,worker:true},null,2));
console.log(`Build archive: exact real-engine sources/results/request/ZIP, ${rejects} rejects, pre-read/UTF-8/size checks and worker parsing passed.`);
