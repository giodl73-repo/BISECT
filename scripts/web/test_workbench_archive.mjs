import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createWorkbenchArchive,parseWorkbenchArchive,readWorkbenchArchiveText,readWorkbenchArchiveFile,serializeWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {parseProject,MAX_WORKBENCH_ARCHIVE_BYTES} from '../../web/lab/project.js';
const root='target/workbench-archive-tests';await fs.mkdir(root,{recursive:true});
const source=JSON.parse(await fs.readFile('target/tract-preparation-wasm/'+(process.argv.includes('--local-ri')?'ri2020-run':'2020')+'.bisect','utf8'));
const text=createWorkbenchArchive({...source,result:{schema_version:'unverified-source-payload',text:'x'.repeat(26*1024*1024),retained_run:source.result}});
assert.ok(Buffer.byteLength(text)>25*1024*1024);const restored=parseWorkbenchArchive(text);assert.deepEqual(restored.files,source.files);assert.deepEqual(restored.result.retained_run,source.result);assert.equal(restored.result.text.length,26*1024*1024);assert.throws(()=>parseProject(text),/25 MB/);
await fs.writeFile(root+'/large.bisect-workbench',text);
const small=createWorkbenchArchive(source);await fs.writeFile(root+'/ri-run.bisect-workbench',small);assert.throws(()=>parseProject(small),/Unsupported project format/);
for(const change of [{schema_version:'bisect-project-v1'},{operation:'execute-code'},{files:{bad:{}}},{constraints:['unknown']},{lastOperation:'execute-code'}])assert.throws(()=>parseWorkbenchArchive(JSON.stringify({...restored,...change})));
for(const payload of ['{"schema_version":1,"schema_version":2}','{"__proto__":{}}','{"n":9007199254740993}','{"n":1e999}','['.repeat(65)+'0'+']'.repeat(65)])assert.throws(()=>parseWorkbenchArchive(payload));
assert.throws(()=>parseWorkbenchArchive(' '.repeat(MAX_WORKBENCH_ARCHIVE_BYTES+1)),/128 MiB/);
let read=false;await assert.rejects(()=>readWorkbenchArchiveText({size:MAX_WORKBENCH_ARCHIVE_BYTES+1,arrayBuffer(){read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readWorkbenchArchiveText({size:2,arrayBuffer:async()=>Uint8Array.of(0xff,0xff).buffer}));await assert.rejects(()=>readWorkbenchArchiveText({size:2,arrayBuffer:async()=>Uint8Array.of(65).buffer}),/changed/);
let terminated=0;const original=globalThis.Worker;globalThis.Worker=class{postMessage(){}terminate(){terminated++;}};
try{for(const action of [signal=>readWorkbenchArchiveFile({size:22},{signal}),signal=>serializeWorkbenchArchive(source,{signal})]){const c=new AbortController(),p=action(c.signal);c.abort();await assert.rejects(()=>p,/cancelled/);}assert.equal(terminated,2);}finally{globalThis.Worker=original;}
if(process.argv.includes('--local-ri')){
 const files=structuredClone(source.files);for(const [kind,name] of [['geography','rigeo2020.pl'],['population','ri000012020.pl']])files.pl94Inputs[kind]={name,base64:(await fs.readFile('data/2020/redistricting/rhode_island/'+name)).toString('base64')};
 const raw=createWorkbenchArchive({...source,files});assert.ok(Buffer.byteLength(raw)>25*1024*1024);assert.deepEqual(parseWorkbenchArchive(raw).files,files);await fs.writeFile(root+'/ri-uncompressed.bisect-workbench',raw);console.log('Real RI archive preserves original uncompressed PL94 bytes above 25 MiB with its graph, YAML seed and retained run.');
}
console.log('Workbench archive: above-25-MiB exact source/result roundtrip, distinct schema, unchanged ordinary cap, field/operation/prototype/duplicate/depth/exact-integer/128-MiB/UTF8/changed-read guards and import/save worker cancellation passed.');
