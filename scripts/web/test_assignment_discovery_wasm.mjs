import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest=JSON.parse(await fs.readFile('target/manifest-certificate-wasm/input.json','utf8')).manifest_json;
const parsed={...JSON.parse(manifest),year:'2020',label:'fallback'};
const manifest_json=JSON.stringify(parsed), b64=s=>Buffer.from(s).toString('base64');
const input=()=>({manifest_json,explicit_label:'chosen',files_base64:{}});
const put=(i,version,label,text,context=true)=>{const dir=`${version}/2020/plans/${label}`;i.files_base64[dir+'/data/final_assignments.json']=b64(text);if(context)i.files_base64[dir+'/manifest.json']=b64(manifest_json);};
const run=i=>engine.execute({operation:'discover-original-assignments',input:i});
let cases=0;
for(const version of ['v1','verify','international']){const i=input();put(i,version,'chosen',' {"unit":2} \n');const o=run(i);assert.equal(o.status,'found');assert.equal(o.source_path,`${version}/2020/plans/chosen/data/final_assignments.json`);assert.equal(o.source_json,' {"unit":2} \n');cases++;}
const i=input();put(i,'v1','chosen','{"wrong":3}',false);put(i,'verify','chosen','{"selected":2}');put(i,'international','chosen','{"later":1}');assert.equal(run(i).source_json,'{"selected":2}');cases++;
for(const [text,status] of [['broken','failed'],['{}','empty']]){const i=input();put(i,'v1','chosen',text);put(i,'verify','chosen','{"later":1}');assert.equal(run(i).status,status);cases++;}
const fallback=input();put(fallback,'v1','fallback','{"unit":1}',false);assert.equal(run(fallback).status,'found');cases++;
assert.equal(run(input()).status,'not-found');cases++;
assert.throws(()=>run({...input(),explicit_label:'../escape'}));
assert.throws(()=>run({...input(),files_base64:{'a.json':b64('{}'),'A.json':b64('{}')}}));
console.log(`Original assignment discovery: ${cases} WASM cases passed; traversal and case collisions rejected.`);
