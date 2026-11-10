import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/fletch-prune-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const entry=(relative_path,bytes=4)=>({dataset_id:'dataset',version:null,source_url:'https://example.test/inert',cache_key:'opaque',relative_path,sha256:'sha256:'+'a'.repeat(64),bytes,fetched_at_ms:1,verified:false});
const cases=[['empty',[],{}],['kept',[entry('objects/kept')],{'objects/kept':'same'}],['orphan',[],{'objects/a':'one'}],['mixed',[entry('objects/kept')],{'objects/kept':'same','objects/z':'last','objects/a/nested':'nested','unrelated.txt':'outside'}],['duplicate',[entry('objects/kept'),entry('objects/kept',9)],{'objects/kept':'same','objects/unused':''}],['backslash',[entry('objects\\kept')],{'objects/kept':'same','objects/a':'a'}],['missing-reference',[entry('objects/missing')],{'objects/a':'one'}],['exact-u64',[entry('objects/missing')],{'objects/a':'one'}]];
for(const [name,entries,available]of cases){
 const cacheRoot=path.resolve(root,name).replaceAll('\\','/');await fs.mkdir(cacheRoot,{recursive:true});
 for(const [relative,body]of Object.entries(available)){await fs.mkdir(path.dirname(cacheRoot+'/'+relative),{recursive:true});await fs.writeFile(cacheRoot+'/'+relative,body);}
 let text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:cacheRoot,entries});if(name==='exact-u64')text=text.replace('"bytes":4','"bytes":9007199254740993');
 const manifestFile=cacheRoot+'/manifest.json';await fs.writeFile(manifestFile,text);
 const native=execFileSync('target/debug/examples/fletch_prune_reference.exe',[manifestFile],{encoding:'utf8'});
 const files=Object.fromEntries(Object.entries(available).map(([relative,body])=>[relative,String(Buffer.byteLength(body))]));
 const result=engine.execute({operation:'plan-fletch-cache-prune',cache_manifest_json:text,files});assert.equal(result.plan_json,native,name);assert.equal(result.manifest_sha256,createHash('sha256').update(text).digest('hex'));
 assert.equal(result.prune_count,JSON.parse(native).prune_count);if(name==='exact-u64')assert.equal(result.keep_bytes_text,'9007199254740993');
 await fs.writeFile(cacheRoot+'/result.json',JSON.stringify(result));
}
const base=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'cache',entries:[]});
for(const files of [{'../escape':'1'},{'objects/a':'01'},{'objects/a':'18446744073709551616'},{'objects/a':'1','objects\\a':'1'},{'objects/a':1}])assert.throws(()=>engine.execute({operation:'plan-fletch-cache-prune',cache_manifest_json:base,files}));
assert.throws(()=>engine.execute({operation:'plan-fletch-cache-prune',cache_manifest_json:' '.repeat(16*1024*1024+1),files:{}}),/bounds/);
assert.throws(()=>engine.execute({operation:'plan-fletch-cache-prune',cache_manifest_json:base,files:Object.fromEntries(Array.from({length:10001},(_,i)=>['objects/'+i,'1']))}),/bounds/);
console.log('Native/WASM cache prune: eight actual upstream filesystem-plan byte comparisons, sorted orphan selection, retained duplicate/missing references, backslash normalization, exact u64 and seven input/bound guards passed.');

for(const [cacheRoot,absolute]of [['','objects/a'],['/','/objects/a']]){
 const text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:cacheRoot,entries:[]});
 const result=engine.execute({operation:'plan-fletch-cache-prune',cache_manifest_json:text,files:{'objects/a':'1'}});
 assert.equal(JSON.parse(result.plan_json).candidates[0].absolute_path,absolute);
}
console.log('Portable prune paths preserve empty and absolute slash-only roots.');
