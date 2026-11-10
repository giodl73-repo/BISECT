import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {fletchMergeRequest,fletchMergedManifestBytes,fletchMergedManifestSource} from '../../web/lab/fletch-merge-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/fletch-merge-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const entry=(id,key=id)=>({dataset_id:id,version:null,source_url:'https://example.test/inert',cache_key:key,relative_path:'objects/'+id,sha256:'sha256:'+'a'.repeat(64),bytes:1,fetched_at_ms:3,verified:false});
const base=entries=>({schema_version:'fletch.cache-manifest.v1',generated_by:'preserved-input',cache_root:'inert/根',entries});
const cases=[
 ['empty',[],[]],['no-updates',[entry('a')],[]],['append',[entry('a')],[entry('b')]],
 ['replace',[entry('a')],[entry('replacement','a')]],
 ['duplicate-base',[entry('first','a'),entry('second','a')],[entry('new','a')]],
 ['duplicate-updates',[entry('a')],[entry('first','a'),entry('last','a')]],
 ['dataset-not-key',[entry('a','first')],[entry('a','second')]],
 ['order',[entry('a'),entry('b'),entry('c')],[entry('replacement','b'),entry('d')]],
 ['flags',[entry('a')],[{...entry('a'),verified:true}]],
 ['retry-metadata',[],[{...entry('a'),fetch_attempts:4,retry_count:3,last_retryable_error:'quoted "\nerror'}]],
 ['native-defaults',[],[{...entry('a'),extra:'ignored'}]],
 ['native-sha-policy',[],[{...entry('a'),sha256:'sha256:'+'z'.repeat(64)}]],
 ['inert-paths',[],[{...entry('a'),relative_path:'../inert',source_url:'file:///inert'}]],
 ['exact-u64',[],[entry('a')]],['aggregate-over-u64',[entry('a')],[entry('b')]]
];
for(const [name,initial,updates] of cases){
 let baseText=JSON.stringify(base(initial)),entriesText=JSON.stringify(updates);
 if(name==='exact-u64')entriesText=entriesText.replace('"bytes":1','"bytes":18446744073709551615').replace('"fetched_at_ms":3','"fetched_at_ms":9007199254740993');
 if(name==='aggregate-over-u64'){baseText=baseText.replace('"bytes":1','"bytes":18446744073709551615');entriesText=entriesText.replace('"bytes":1','"bytes":18446744073709551615');}
 await fs.writeFile(root+'/base.json',baseText);await fs.writeFile(root+'/entries.json',entriesText);
 const native=execFileSync('target/debug/examples/fletch_merge_reference.exe',[root+'/base.json',root+'/entries.json'],{encoding:'utf8'});
 const result=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:baseText,entries_json:entriesText});
 assert.equal(result.manifest_json,native,name+' exact native bytes');
 assert.equal(result.base_sha256,createHash('sha256').update(baseText).digest('hex'));
 assert.equal(result.updates_sha256,createHash('sha256').update(entriesText).digest('hex'));
 const rows=JSON.parse(native).entries;
 if(name==='duplicate-base')assert.deepEqual(rows.map(r=>r.dataset_id),['new','second']);
 if(name==='dataset-not-key')assert.equal(rows.length,2);
 if(name==='duplicate-updates')assert.equal(rows[0].dataset_id,'last');
 await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
 const sources={fletchCacheManifest:{name:'base.json',base64:Buffer.from(baseText).toString('base64')},fletchCacheUpdates:{name:'updates.json',base64:Buffer.from(entriesText).toString('base64')}};
 assert.deepEqual(fletchMergeRequest(sources.fletchCacheManifest,sources.fletchCacheUpdates),{operation:'merge-fletch-cache-manifest',cache_manifest_json:baseText,entries_json:entriesText});
 assert.equal(Buffer.from(fletchMergedManifestBytes(result)).toString('utf8'),native);
 assert.equal(Buffer.from(fletchMergedManifestSource(result).base64,'base64').toString('utf8'),native);
 const project=createProject({name:'Cache merge '+name,files:sources,operation:'merge-fletch-cache-manifest',constraints:[],result,lastOperation:'merge-fletch-cache-manifest'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));
 const archive=parseWorkbenchArchive(createWorkbenchArchive(project));assert.deepEqual(archive.files,project.files);assert.deepEqual(archive.result,project.result);assert.equal(archive.operation,'merge-fletch-cache-manifest');
}
for(const [baseText,entriesText] of [[JSON.stringify({...base([]),schema_version:'wrong'}),'[]'],[JSON.stringify(base([{...entry('a'),sha256:'bad'}])),'[]'],[JSON.stringify(base([])),JSON.stringify([{...entry('a'),sha256:'bad'},entry('a')])],[JSON.stringify(base([])),'{}']]){
 await fs.writeFile(root+'/base.json',baseText);await fs.writeFile(root+'/entries.json',entriesText);
 assert.throws(()=>execFileSync('target/debug/examples/fletch_merge_reference.exe',[root+'/base.json',root+'/entries.json'],{stdio:'pipe'}));
 assert.throws(()=>engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:baseText,entries_json:entriesText}));
}
for(const [baseText,entriesText] of [[JSON.stringify(base(Array.from({length:2001},()=>entry('a')))),'[]'],[JSON.stringify(base([])),JSON.stringify(Array.from({length:2001},()=>entry('a')))],[' '.repeat(16*1024*1024+1),'[]'],[JSON.stringify(base(Array.from({length:1001},(_,i)=>entry('base'+i)))),JSON.stringify(Array.from({length:1000},(_,i)=>entry('new'+i)))]])assert.throws(()=>engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:baseText,entries_json:entriesText}),/exceeds|up to/);
console.log('FLETCH merge: 15 exact native/WASM manifest-byte comparisons, first-key replacement/order/duplicates/defaults/retry metadata/inert strings/exact u64, four native rejection pairs and browser size/entry bounds passed.');

// A native output larger than the original 8 MiB input boundary must remain
// mergeable; otherwise portable backup restoration fails on a valid manifest.
const largeBase=JSON.stringify(base([{...entry('large'),dataset_id:'x'.repeat(9*1024*1024)}]));
await fs.writeFile(root+'/base.json',largeBase);await fs.writeFile(root+'/entries.json','[]');
const largeNative=execFileSync('target/debug/examples/fletch_merge_reference.exe',[root+'/base.json',root+'/entries.json'],{encoding:'utf8',maxBuffer:32*1024*1024});
const largeResult=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:largeBase,entries_json:'[]'});assert.equal(largeResult.manifest_json,largeNative);
assert.equal(engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:largeResult.manifest_json,entries_json:'[]'}).manifest_json,largeNative);
console.log('Native/WASM 9 MiB manifest merge and output re-entry pass within the 16 MiB ABI boundary.');
