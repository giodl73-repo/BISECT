import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readFletchCacheManifest} from '../../web/lab/fletch-cache-files.js';
import {selectFletchObjects,fletchVerifyRequest,verifyFletchCacheFiles,fletchVerifyReportBytes} from '../../web/lab/fletch-verify-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/fletch-verify-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const content=new TextEncoder().encode('selected object bytes'),digest='sha256:'+createHash('sha256').update(content).digest('hex');
const base={dataset_id:'dataset',source_url:'https://example.test/inert',cache_key:'opaque',relative_path:'objects/object',sha256:digest,bytes:content.length,fetched_at_ms:1,verified:false};
for(const [name,entries,available]of [
 ['verified',[base],true],['saved-flag-true',[{...base,verified:true}],true],['missing',[base],false],
 ['bad-hash',[{...base,sha256:'sha256:'+'a'.repeat(64)}],true],['bad-size',[{...base,bytes:content.length+1}],true],
 ['duplicate',[base,{...base,dataset_id:'duplicate'}],true],['backslash',[{...base,relative_path:'objects\\object'}],true],
 ['exact-u64',[{...base,bytes:1}],true],['empty',[],true],['streamed',[base],true]
]){
 const cacheRoot=path.resolve(root,name).replaceAll('\\','/');await fs.mkdir(cacheRoot+'/objects',{recursive:true});
 if(available)await fs.writeFile(cacheRoot+'/objects/object',content);
 let text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:cacheRoot,entries});
 if(name==='exact-u64')text=text.replace('"bytes":1','"bytes":9007199254740993');
 await fs.writeFile(root+'/manifest.json',text);
 const nativeText=execFileSync('target/debug/examples/fletch_verify_reference.exe',[root+'/manifest.json'],{encoding:'utf8'}),native=JSON.parse(nativeText);
 const bytes=new TextEncoder().encode(text),source=await readFletchCacheManifest({name:name+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});
 const selected=available?[{path:'objects/object',file:new Blob([content])}]:[];
 // Unreferenced selected objects must never be read.
 selected.push({path:'ignored',file:{size:1,slice(){throw new Error('Unreferenced file was read');}}});
 const progress=[];const result=await verifyFletchCacheFiles(engine,fletchVerifyRequest(source,selected),{onProgress:p=>progress.push(p)});
 const report=JSON.parse(result.report_json);for(const status of native.statuses)status.absolute_path=status.absolute_path.replaceAll('\\','/');
 assert.deepEqual(report,native,name+' native filesystem verification');
 const counts=text=>[...text.matchAll(/"(?:expected_bytes|actual_bytes)"\s*:\s*(\d+)/g)].map(m=>m[1]).sort();
 assert.deepEqual(counts(result.report_json),counts(nativeText));
 if(name==='exact-u64')assert.equal(result.expected_bytes_text,'9007199254740993');
 assert.equal(result.digest_source,'local-files-streamed-in-worker');assert.ok(result.ignored_selection_count>=1);
 if(name==='verified')assert.equal(result.verified_count,1);
 if(name==='duplicate')assert.equal(result.verified_count,2);
 if(name==='streamed'){
  const large=new Uint8Array(5*1024*1024+17).fill(31),sha='sha256:'+createHash('sha256').update(large).digest('hex');let largest=0;
  const file={size:large.length,slice(a,b){largest=Math.max(largest,b-a);return {arrayBuffer:async()=>large.slice(a,b).buffer};}};
  const request={operation:'verify-fletch-cache-files',cache_manifest_json:JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'inert',entries:[{...base,sha256:sha,bytes:large.length}]}),files:[{path:'objects/object',file}]};
  const streamed=await verifyFletchCacheFiles(engine,request);assert.equal(streamed.verified_count,1);assert.ok(largest<=1024*1024);
 }
 assert.equal(new TextDecoder().decode(fletchVerifyReportBytes(result)),result.report_json+'\n');
 const project=createProject({name:'Cache verification '+name,files:{fletchCacheManifest:source},operation:'verify-fletch-cache-files',constraints:[],result,lastOperation:'verify-fletch-cache-files'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
for(const relative_path of ['../object','/object','a//b','a/./b','C:/object']){
 const text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:path.resolve(root).replaceAll('\\','/'),entries:[{...base,relative_path}]});await fs.writeFile(root+'/manifest.json',text);
 assert.throws(()=>execFileSync('target/debug/examples/fletch_verify_reference.exe',[root+'/manifest.json'],{stdio:'pipe'}));
 assert.throws(()=>engine.execute({operation:'plan-fletch-cache-verification',cache_manifest_json:text}));
}
const source=await readFletchCacheManifest({name:'manifest.json',size:2,arrayBuffer:async()=>new TextEncoder().encode('{}').buffer});
await assert.rejects(()=>verifyFletchCacheFiles(engine,fletchVerifyRequest(source,[])));
assert.throws(()=>selectFletchObjects([{name:'../x',size:1}]));assert.throws(()=>selectFletchObjects([{name:'a',size:1},{name:'a',size:1}]));
// A failed or shortened local read must not produce a verification receipt,
// and the same WASM instance must remain usable for a subsequent explicit run.
const recoveryManifest=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'inert',entries:[base]});
for(const file of [
 {size:content.length,slice(){return {arrayBuffer:async()=>{throw new Error('Local read failed');}};}},
 {size:content.length,slice(){return {arrayBuffer:async()=>new Uint8Array(1).buffer};}}
]){
 const progress=[];
 await assert.rejects(()=>verifyFletchCacheFiles(engine,{operation:'verify-fletch-cache-files',cache_manifest_json:recoveryManifest,files:[{path:'objects/object',file}]},{onProgress:p=>progress.push(p)}),/Local read failed|File size changed/);
 assert.equal(progress.length,0);
 const recovered=await verifyFletchCacheFiles(engine,{operation:'verify-fletch-cache-files',cache_manifest_json:recoveryManifest,files:[{path:'objects/object',file:new Blob([content])}]});
 assert.equal(recovered.verified_count,1);
}
console.log('FLETCH selected-file read failures and shortened reads reject without receipts; subsequent verification recovers on the same WASM instance.');
console.log('FLETCH cache verification: ten actual WASM-streamed/native filesystem comparisons, saved-flag independence, hash/size/missing/duplicate/backslash/empty cases, exact u64, bounded streaming, unreferenced-file isolation, five native path rejections and project/report byte preservation passed.');
