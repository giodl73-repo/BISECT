import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readFletchCacheManifest,fletchCacheRequest} from '../../web/lab/fletch-cache-files.js';
import {FETCH_PLAN_DEFAULTS} from '../../web/lab/fetch-plan-files.js';
const root='target/fletch-cache-wasm'; await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const entry=(dataset_id,verified=true)=>({dataset_id,source_url:'https://example.test',cache_key:'opaque',relative_path:'../evidence-only',sha256:'sha256:'+'g'.repeat(64),bytes:1,fetched_at_ms:1,verified});
// The pinned native validator checks prefix and length, not hex digits.
for(const [name,states,entries,changes={}] of [
 ['missing',['RI'],[]],['verified',['RI'],[entry('bisect.2020.ri.tiger')]],
 ['duplicates',['RI'],[entry('bisect.2020.ri.tiger',false),entry('bisect.2020.ri.tiger')]],
 ['unexpected',['RI'],[entry('other',false)]],['empty-expected',['XX'],[entry('other')]],
 ['exact-u64',['RI'],[entry('bisect.2020.ri.tiger')]],
 ['all-vintages',['RI'],[entry('bisect.2000.ri.tiger',false),entry('bisect.2010.ri.tiger'),entry('bisect.2020.ri.tiger')],{year:'all'}],
 ['all-states-vintages',[],[],{year:'all'}]]){
 const selection={...FETCH_PLAN_DEFAULTS,states,...changes};
 let text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'C:/not-opened',entries});
 if(name==='exact-u64')text=text.replace('"bytes":1','"bytes":9007199254740993');
 const bytes=new TextEncoder().encode(text);
 const source=await readFletchCacheManifest({name:name+'.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});
 const selectionBytes=new TextEncoder().encode(JSON.stringify(selection));
 const {encodeBytes}=await import('../../web/lab/package-files.js');
 const request=fletchCacheRequest({manifest:source,selection:{source:{name:'selection.json',base64:encodeBytes(selectionBytes)}}});
 assert.equal(request.input.cache_manifest_json,text);
 const result=engine.execute(request);
 if(changes.year==='all'){
  const nativeSelection={...selection,manifest_json:await fs.readFile('data/manifest.json','utf8')};
  await fs.writeFile(root+'/native-source-selection.json',JSON.stringify(nativeSelection));
  const nativeItems=JSON.parse(execFileSync('target/debug/examples/fetch_plan_reference.exe',[root+'/native-source-selection.json'],{encoding:'utf8'}));
  const rowKey=r=>JSON.stringify([r.state_code,r.year,r.kind,r.url,r.local_path,r.done_marker]);
  assert.deepEqual(result.source_plan.items.map(rowKey).sort(),nativeItems.map(rowKey).sort());
  assert.deepEqual([...new Set(result.source_plan.items.map(r=>r.year))].sort(),['2000','2010','2020']);
  if(name==='all-states-vintages')assert.equal(new Set(result.source_plan.items.map(r=>r.state_code)).size,50);
 }
 const expected_ids=result.source_plan.items.filter(r=>r.url!==null).map(r=>`bisect.${r.year}.${r.state_code.toLowerCase()}.${r.kind}`);
 await fs.writeFile(root+'/oracle.json',JSON.stringify({manifest_json:text,expected_ids}));
 const native=execFileSync('target/debug/examples/fletch_cache_reference.exe',[root+'/oracle.json'],{encoding:'utf8'}).trim();
 assert.equal(result.report_json,native,name);
 assert.equal(result.execution_performed,false);
 if(name==='exact-u64')assert.equal(result.byte_count_text,'9007199254740993');
 if(name==='duplicates'||name==='unexpected'||name==='all-vintages')assert.equal(result.gate_passed,false);
 if(name==='missing'||name==='empty-expected')assert.equal(result.gate_passed,true);
 await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
for(const patch of [{schema_version:'wrong'},{entries:[{...entry('a'),sha256:'bad'}]},{entries:[{...entry('a'),bytes:-1}]},{entries:[{...entry('a'),fetch_attempts:4294967296}]}]){
 const text=JSON.stringify({schema_version:'fletch.cache-manifest.v1',generated_by:'fixture',cache_root:'cache',entries:[],...patch});
 await fs.writeFile(root+'/oracle.json',JSON.stringify({manifest_json:text,expected_ids:[]}));
 assert.throws(()=>execFileSync('target/debug/examples/fletch_cache_reference.exe',[root+'/oracle.json'],{stdio:'pipe'}));
 assert.throws(()=>engine.execute({operation:'inspect-fletch-cache',input:{selection:FETCH_PLAN_DEFAULTS,cache_manifest_json:text}}));
}
console.log('FLETCH cache: eight actual WASM/native report comparisons, all-vintage/all-state independent native source planner comparisons, exact u64, duplicate gate evidence, missing/empty expectations, inert paths and four native validation rejection comparisons passed.');

let touched=false; await assert.rejects(()=>readFletchCacheManifest({name:'huge',size:8*1024*1024+1,arrayBuffer(){touched=true;}})); assert.equal(touched,false);
const malformed=new Uint8Array([0xff]); await assert.rejects(()=>readFletchCacheManifest({name:'bad.json',size:1,arrayBuffer:async()=>malformed.buffer}));

assert.throws(()=>fletchCacheRequest({manifest:null,selection:{source:null},settings:{year:'../all'}}));
