import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {policyRequest,readPolicySource,validatePolicyInputs} from '../../web/lab/policy-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/policy-wasm';await fs.mkdir(root,{recursive:true});
const raw=await fs.readFile('data/location_policy.json','utf8'),database=JSON.parse(raw);
const request=query=>({operation:'inspect-location-policy',query,source_name:null,policy_json:null});
let parity=0,cli=0,rejections=0;const evidence=[];
async function native(r){const p=root+'/request.json';await fs.writeFile(p,JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[p],{encoding:'utf8',maxBuffer:8*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r){const out=engine.execute(r);assert.deepEqual(out,await native(r));parity++;return out;}
async function reject(r){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
function command(query,format,path){return execFileSync('target/debug/examples/policy_reference.exe',[query,format,...(path?[path]:[])],{encoding:'utf8',maxBuffer:8*1024*1024,stdio:['ignore','pipe','pipe']});}
for(const [code,entry]of Object.entries(database).filter(([,v])=>v&&typeof v==='object'&&v.name)){
 const out=await exact(request(code.toLowerCase()));assert.equal(out.resolved_code,code);assert.equal(out.fuzzy_match,false);assert.deepEqual(JSON.parse(out.policy_json),entry);
 assert.equal(out.source_sha256,'sha256:'+createHash('sha256').update(raw).digest('hex'));
 assert.deepEqual(JSON.parse(command(code.toLowerCase(),'json')),entry);assert.equal(command(code.toLowerCase(),'table'),out.table_text);cli+=2;evidence.push({code,source_sha256:out.source_sha256});
}
for(const query of ['Rhode Island','Ireland','louisiana','_test_el']){
 const out=await exact(request(query));assert.deepEqual(JSON.parse(command(query,'json')),JSON.parse(out.policy_json));assert.equal(command(query,'table'),out.table_text);cli+=2;
}
for(const query of ['Carolina','ZZ','Eldoria']){await reject(request(query));assert.throws(()=>command(query,'json'));cli++;}
const override='{ "XX": { "name": "Custom jurisdiction", "house_districts": 9007199254740993, "senate_districts": 2, "nesting_ratio": "2:1", "notes":"<script>inert text</script>", "extra":{"signed":-9007199254740993} } }\n';
const overridePath=resolve(root+'/override.json');await fs.writeFile(overridePath,override);
const bytes=new TextEncoder().encode(override);const source=await readPolicySource({name:'override.json',size:bytes.length,arrayBuffer:async()=>bytes.buffer});const inputs={query:'Custom',source};
const overrideOut=await exact(policyRequest(inputs));assert.equal(overrideOut.source_kind,'local-override');assert.equal(overrideOut.policy.house_districts,'9007199254740993');assert.equal(overrideOut.policy.extra.signed,'-9007199254740993');
assert.equal(command('Custom','json',overridePath).trimEnd(),overrideOut.policy_json);assert.equal(command('Custom','table',overridePath),overrideOut.table_text);cli+=2;
assert.match(overrideOut.policy_json,/9007199254740993/);assert.equal(countResultTables(overrideOut)[0].rows.find(r=>r[0]==='House districts')[1],'9007199254740993');
const project=createProject({name:'Local policy inspection',files:{policyInputs:inputs},operation:'inspect-location-policy',constraints:[],result:overrideOut,lastOperation:'inspect-location-policy'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(policyRequest(parseProject(JSON.stringify(project)).files.policyInputs).policy_json,override);
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/policy.json',overrideOut.policy_json);await fs.writeFile(root+'/policy.txt',overrideOut.table_text);
for(const change of [{query:''},{query:'x'.repeat(201)},{query:'é'.repeat(101)},{source_name:'override.json'},{policy_json:'{}'},{source_name:'',policy_json:'{}'},{source_name:'x',policy_json:'{'},{source_name:'x',policy_json:'[]'},{source_name:'x',policy_json:' '.repeat(512*1024+1)},{unexpected:true},{source_name:'x',policy_json:'{"XX":{"name":"X","constructor":{}}}'},{source_name:'x',policy_json:JSON.stringify(Object.fromEntries(Array.from({length:1001},(_,i)=>['X'+i,{name:'X'}])))}])await reject({...request('XX'),...change});
let nested={name:'Deep'};for(let i=0;i<65;i++)nested={child:nested};
await reject({...request('XX'),source_name:'deep.json',policy_json:JSON.stringify({XX:nested})});
await reject({...request('XX'),source_name:'many.json',policy_json:JSON.stringify({XX:{name:'Many',values:Array(50_001).fill(0)}})});
const boundaryText=override+' '.repeat(512*1024-new TextEncoder().encode(override).length);
const boundaryOutput=await exact({...policyRequest(inputs),policy_json:boundaryText});assert.equal(boundaryOutput.policy_json,overrideOut.policy_json);assert.notEqual(boundaryOutput.source_sha256,overrideOut.source_sha256);
assert.throws(()=>validatePolicyInputs({query:'RI',source:null,url:'https://attacker.invalid'}));
let read=false;await assert.rejects(()=>readPolicySource({name:'oversized.json',size:512*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readPolicySource({name:'invalid.json',size:2,arrayBuffer:async()=>Uint8Array.from([0xc3,0x28]).buffer}));
await assert.rejects(()=>readPolicySource({name:'changed.json',size:2,arrayBuffer:async()=>bytes.buffer}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,cli,rejections,entries:evidence,scope:'All embedded named locations, native CLI text/JSON, explicit override, exact large integers and local project boundaries.'},null,2));
console.log(`Policy WASM: ${parity} native/WASM comparisons, ${cli} actual CLI checks, ${rejections} matching rejections; exact JSON/text, source identity, override and inert project roundtrips passed.`);
