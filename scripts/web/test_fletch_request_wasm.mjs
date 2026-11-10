import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/fletch-request-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const bytes=Buffer.from('cached native source'),digest='sha256:'+createHash('sha256').update(bytes).digest('hex');await fs.writeFile(root+'/source.bin',bytes);
const identity={dataset_id:'fixture',version:'2020',source_url:'https://example.test/inert'};
const base={...identity,force:false,offline:true,object_exists:true,observation:{sha256:digest,bytes:String(bytes.length),modified_at_ms:'0'},trusted_entries_json:'[]',expected_sha256:null,fetched_at_ms:'9007199254740993'};
const key=engine.execute({operation:'resolve-fletch-cache-request',input:{...base,object_exists:false,offline:false}}).cache_key;
const entry={...identity,cache_key:key,relative_path:'objects/sha256/'+key.slice(7),sha256:digest,bytes:bytes.length,fetched_at_ms:7,verified:true,fetch_attempts:4,retry_count:3,last_retryable_error:'historical retry'};
const exact=JSON.stringify([entry]).replace('"fetched_at_ms":7','"fetched_at_ms":9007199254740993');
const cases=[['always-offline',{...base,freshness:'always-check'}],['age-fresh',{...base,freshness:'max-age-days',max_age_days:1,now_ms:String(Date.now()),object_modified_at_ms:String(Date.now()-1000)}],['age-stale',{...base,freshness:'max-age-days',max_age_days:1,now_ms:String(Date.now()),object_modified_at_ms:'0'}],['age-future',{...base,freshness:'max-age-days',max_age_days:0,now_ms:String(Date.now()),object_modified_at_ms:String(Date.now()+86400000)}],['untrusted',base],['modified-time',{...base,fetched_at_ms:null}],['expected',{...base,expected_sha256:digest}],['trusted',{...base,trusted_entries_json:exact}],['trusted-unverified',{...base,trusted_entries_json:exact.replace('"verified":true','"verified":false')}],['nonmatching-trust',{...base,trusted_entries_json:JSON.stringify([{...entry,dataset_id:'other'}])}],['online-hit',{...base,offline:false,trusted_entries_json:exact}],['trusted-hash-error',{...base,trusted_entries_json:JSON.stringify([{...entry,sha256:'sha256:'+'b'.repeat(64)}])}],['trusted-size-error',{...base,trusted_entries_json:JSON.stringify([{...entry,bytes:bytes.length+1}])}],['expected-error',{...base,expected_sha256:'sha256:'+'b'.repeat(64)}],['force-offline',{...base,force:true}],['miss-offline',{...base,object_exists:false,observation:null}]];
for(const [name,input]of cases){
 await fs.writeFile(root+'/input.json',JSON.stringify(input));await fs.mkdir(root+'/'+name,{recursive:true});
 const native=JSON.parse(execFileSync('target/debug/examples/fletch_request_reference.exe',[root+'/input.json',root+'/'+name,input.object_exists?root+'/source.bin':'none'],{encoding:'utf8'}));
 const observed={...input,observation:input.object_exists?{...input.observation,modified_at_ms:native.modified_at_ms}:null};
 if(native.error)assert.throws(()=>engine.execute({operation:'resolve-fletch-cache-request',input:observed}),e=>e.message===native.error,name);
 else{const result=engine.execute({operation:'resolve-fletch-cache-request',input:observed});assert.equal(result.entries_json,native.entries_json,name);assert.deepEqual(result.attempt_status,native.attempt_status);assert.equal(result.attempt_status.attempts,0);await fs.writeFile(root+'/'+name+'/result.json',JSON.stringify(result));}
}
for(const input of [{...base,offline:false,object_exists:false,observation:null},{...base,offline:false,force:true}])assert.equal(engine.execute({operation:'resolve-fletch-cache-request',input}).decision,'fetch');
assert.throws(()=>engine.execute({operation:'resolve-fletch-cache-request',input:{...base,observation:null}}),/observed/);
assert.throws(()=>engine.execute({operation:'resolve-fletch-cache-request',input:{...base,fetched_at_ms:'01'}}),/canonical/);
console.log('Immutable native/WASM requests: twelve actual upstream cache-hit/offline outcomes/errors match exact entry JSON and attempt status; trust/expected digest/retry history/exact u64/mtime/force semantics and fetch decisions passed.');

const age={...base,offline:false,freshness:'max-age-days',max_age_days:1,object_modified_at_ms:'1000',now_ms:'86401000'};
assert.equal(engine.execute({operation:'resolve-fletch-cache-request',input:age}).decision,'cache-hit');
assert.equal(engine.execute({operation:'resolve-fletch-cache-request',input:{...age,now_ms:'86401001'}}).decision,'fetch');
assert.equal(engine.execute({operation:'resolve-fletch-cache-request',input:{...base,offline:false,freshness:'always-check'}}).decision,'fetch');
for(const change of [{freshness:'unknown'},{freshness:'max-age-days',max_age_days:-1},{freshness:'max-age-days',max_age_days:4294967296},{freshness:'max-age-days',max_age_days:1},{freshness:'immutable',max_age_days:1}])assert.throws(()=>engine.execute({operation:'resolve-fletch-cache-request',input:{...base,...change}}));
console.log('Native freshness: actual upstream always-check, fresh/stale maximum-age and future-mtime offline outcomes match; millisecond inclusive boundary, online refresh decisions and policy/time/u32 guards pass.');
