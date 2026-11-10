import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {policyRequest} from '../../web/lab/policy-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),hash=b=>createHash('sha256').update(b).digest('hex');
const wasm=option('wasm'),reference=option('reference');if(!wasm||!reference)throw Error('Pass --wasm=<fresh fallback engine> and --reference=<actual Policy CLI case root>.');
await fs.mkdir('target/policy-fallback',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/policy-fallback/run-'));
const priorFile=path.join(reference,'evidence.json'),prior=JSON.parse(await fs.readFile(priorFile,'utf8')),native=prior.native;assert.equal(prior.status,'passed-actual-policy-CLI-contract');assert.equal(hash(await fs.readFile(native.path)),native.sha256);
for(const ref of prior.sources.filter(r=>r.path.startsWith('crates/bisect-cli/')||r.path==='data/location_policy.json'||r.path==='crates/bisect-analysis/src/location_policy.rs'))assert.equal(hash(await fs.readFile(ref.path)),ref.sha256,'Actual policy CLI consumer changed; rebuild reference.');
const engine=await instantiateEngine(await fs.readFile(wasm)),raw=name=>Buffer.from('{"XX":{"name":"'+name+'","house_districts":9007199254740993}}\r\n'),records=[];
const cases=[
 ['embedded','RI',null,null,'embedded'],
 ['primary-wins','XX',raw('Primary'),raw('Alias'),'primary'],
 ['alias-only','XX',null,raw('Alias'),'alias'],
 ['parse-fallback','XX',Buffer.from('{'),raw('Alias'),'alias'],
 ['bom-fallback','XX',Buffer.concat([Buffer.from([239,187,191]),raw('Primary')]),raw('Alias'),'alias'],
 ['utf8-fallback','XX',Buffer.from([255]),raw('Alias'),'alias'],
 ['empty-fallback','XX',Buffer.alloc(0),raw('Alias'),'alias'],
 ['unavailable-fallback','XX','unavailable',raw('Alias'),'alias'],
 ['both-invalid','RI',Buffer.from('{'),Buffer.from('['),'embedded'],
 ['both-unavailable','RI','unavailable','unavailable','embedded'],
 ['empty-object-stops','XX',Buffer.from('{}'),raw('Alias'),'primary'],
 ['scalar-stops','XX',Buffer.from('42'),raw('Alias'),'primary'],
 ['array-stops','XX',Buffer.from('[]'),raw('Alias'),'primary'],
 ['alias-lookup-stops','RI',Buffer.from('{'),raw('Alias'),'alias'],
 ['embedded-lookup-refused','ZZ',Buffer.from('{'),Buffer.from('['),'embedded'],
];
for(const [name,query,primary,alias,expectedRole]of cases){
 const dir=path.join(root,name);await fs.mkdir(dir);const fallback={primary:null,alias:null},env={...process.env};delete env.BISECT_LOCATION_POLICY;delete env.BISECT_STATE_POLICY;
 for(const [role,value]of [['primary',primary],['alias',alias]]){
  if(value===null)continue;const file=path.join(dir,role+'.json');env[role==='primary'?'BISECT_LOCATION_POLICY':'BISECT_STATE_POLICY']=file;
  if(value==='unavailable')fallback[role]={status:'unavailable',name:role+'.json'};
  else {await fs.writeFile(file,value);fallback[role]={status:'file',name:role+'.json',base64:value.toString('base64')};}
 }
 const execute=format=>{const args=['policy','--state='+query,'--format='+format],out=spawnSync(native.path,args,{encoding:'utf8',windowsHide:true,env,maxBuffer:8*1024*1024});if(out.error)throw out.error;return {arguments:args,exit_code:out.status,stdout:out.stdout,stderr:out.stderr};};
 const json=execute('json'),table=execute('table'),input={query,source:null,fallback},request=policyRequest(input),result=engine.execute(request);
 assert.equal(result.selection.selected_role,expectedRole,name);assert.equal(result.selection.attempts.at(-1).status,'selected');
 if(expectedRole==='primary')assert.equal(result.selection.attempts.length,1,'No alias after valid primary parse');
 if(json.exit_code===0){assert.equal(result.schema_version,'bisect-location-policy-v1');assert.equal(result.policy_json+'\n',json.stdout,name+' JSON');assert.equal(result.table_text,table.stdout,name+' table');if(expectedRole!=='embedded'){assert.equal(result.source_sha256,'sha256:'+hash(expectedRole==='primary'?primary:alias));assert.equal(result.selection.selected_sha256,result.source_sha256);}else assert.equal(result.source_kind,'embedded');}
 else {assert.equal(json.exit_code,1);assert.equal(result.schema_version,'bisect-location-policy-fallback-v1');assert.equal(result.status,'lookup-refused');}
 const project=createProject({name,operation:'inspect-location-policy',files:{policyInputs:input},constraints:[],result,lastOperation:'inspect-location-policy'});assert.deepEqual(parseProject(JSON.stringify(project)).files.policyInputs,input);await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));
 records.push({name,input,request,result,native_json:json,native_table:table});await fs.writeFile(path.join(root,'progress.json'),JSON.stringify({records},null,2)+'\n');
}
const ignored=engine.execute({operation:'inspect-location-policy',query:'XX',source_name:null,policy_json:null,fallback:{primary:{status:'file',name:'primary.json',base64:raw('Primary').toString('base64')},alias:{status:'file',name:'alias.json',base64:'not base64'}}});assert.equal(ignored.policy.name,'Primary');assert.equal(ignored.selection.attempts.length,1);
assert.throws(()=>engine.execute({operation:'inspect-location-policy',query:'RI',source_name:'strict.json',policy_json:'{}',fallback:{primary:null,alias:null}}),/not both/);
let backward=0;for(const row of prior.records){if(row.error)assert.throws(()=>engine.execute(row.request));else assert.deepEqual(engine.execute(row.request),row.result);backward++;}assert.equal(backward,68);
const sources=[];for(const file of ['crates/bisect-wasm/src/location_policy.rs','crates/bisect-wasm/src/toolkit.rs','web/lab/policy-files.js','web/lab/policy-fallback.js','scripts/web/test_policy_fallback_wasm.mjs'])sources.push({path:file,sha256:hash(await fs.readFile(file))});
const evidence={status:'passed-native-policy-fallback',sources,native,wasm:{path:wasm,sha256:hash(await fs.readFile(wasm))},records,strict_cli_reference:{path:priorFile,sha256:hash(await fs.readFile(priorFile))},backward_strict_cli_cases:backward,ignored_alias_not_decoded:true,scope:'Actual CLI primary/alias/read/parse precedence and exact successful JSON/table versus fresh WASM; valid parse prevents further fallback even on lookup refusal. Explicit unavailable observations and browser bounds are input data, not authenticated live filesystem access. No plan policy application/legal claim.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({root,status:evidence.status,cases:records.length}));
