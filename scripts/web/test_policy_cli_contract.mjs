import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {policyRequest,readPolicySource} from '../../web/lab/policy-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),hash=b=>createHash('sha256').update(b).digest('hex');
await fs.mkdir('target/policy-cli-contract',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/policy-cli-contract/run-'));
const pointer=JSON.parse(await fs.readFile('target/cffi-integration/fletch-cli-root.json','utf8'));
const retained=JSON.parse(await fs.readFile(path.join(pointer.node.root,'evidence.json'),'utf8')).native;
const nativeSource=option('native')||retained.path,native=path.join(root,'native.exe'),wasm=option('wasm')||process.env.BISECT_TEST_WASM||'target/cffi-legacy-gates/default.wasm';
const pin=async file=>({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256:hash(await fs.readFile(file))});
await fs.copyFile(nativeSource,native);const nativePin=await pin(native),wasmPin=await pin(wasm);if(!option('native'))assert.equal(nativePin.sha256,retained.sha256);
const engine=await instantiateEngine(await fs.readFile(wasm)),records=[],fallbacks=[];
function run(query,format,primary=null,alias=null){
 const env={...process.env};delete env.BISECT_LOCATION_POLICY;delete env.BISECT_STATE_POLICY;
 if(primary!==null)env.BISECT_LOCATION_POLICY=primary;if(alias!==null)env.BISECT_STATE_POLICY=alias;
 const args=['policy','--state='+query,'--format='+format],out=spawnSync(native,args,{encoding:'utf8',windowsHide:true,env,maxBuffer:8*1024*1024});if(out.error)throw out.error;
 return {arguments:args,exit_code:out.status,stdout:out.stdout,stderr:out.stderr};
}
async function test(name,query,text=null,{nativeFallback=false}={}){
 const folder=path.join(root,name);await fs.mkdir(folder);const bytes=text===null?null:Buffer.from(text),file=bytes?path.join(folder,'source.json'):null;
 if(file)await fs.writeFile(file,bytes);
 const source=bytes?await readPolicySource({name:'source.json',size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer}):null;
 const input={query,source},request=policyRequest(input),json=run(query,'json',file),table=run(query,'table',file);
 let result=null,error=null;try{result=engine.execute(request);}catch(e){error=e.message;}
 const row={name,input,request,native_json:json,native_table:table,result,error,native_fallback:nativeFallback};records.push(row);
 await fs.writeFile(path.join(root,'progress.json'),JSON.stringify({records,fallbacks},null,2)+'\n');
 if(bytes)assert.equal(request.policy_json,text,name+' preserves original UTF-8');
 if(nativeFallback){assert(error,'Explicit malformed override must refuse; browser never silently selects another database.');assert.match(json.stderr,/could not be parsed/);}
 else if(json.exit_code===0){assert.equal(error,null,error);assert.equal(result.policy_json+'\n',json.stdout,name+' native JSON bytes');assert.equal(result.table_text,table.stdout,name+' native table bytes');assert.equal(result.source_sha256,'sha256:'+hash(bytes??rawDatabase));}
 else {assert.equal(json.exit_code,1);assert(error);}
 const project=createProject({name,operation:'inspect-location-policy',files:{policyInputs:input},constraints:[],result,lastOperation:result?'inspect-location-policy':null});
 const reopened=parseProject(JSON.stringify(project));assert.deepEqual(reopened.files.policyInputs,input);
 await fs.writeFile(path.join(folder,'project.bisect'),JSON.stringify(project));return row;
}
const rawDatabase=await fs.readFile('data/location_policy.json'),database=JSON.parse(rawDatabase.toString('utf8'));
for(const [code,value]of Object.entries(database))if(value&&typeof value==='object'&&value.name)await test('embedded-'+code.toLowerCase(),code.toLowerCase());
for(const [name,query]of [['fuzzy','Rhode Island'],['ambiguous','Carolina'],['unknown','ZZ'],['test-code','_test_el'],['test-name-excluded','Eldoria']])await test(name,query);
const custom='{ "XX": { "name":"Custom policy fixture", "house_districts":9007199254740993, "senate_districts":2, "notes":"<script>inert</script>" } }';
await test('override','Custom',custom+'\n');await test('crlf','XX',custom+'\r\n');
await test('bom-unknown','XX','\ufeff'+custom+'\r\n',{nativeFallback:true});
await test('bom-embedded','RI','\ufeff'+custom,{nativeFallback:true});
await test('invalid-embedded','RI','{',{nativeFallback:true});
await test('empty-database','XX','{}');
// Actual CLI fallback precedence is retained independently from the explicit-file browser operation.
const primary=path.join(root,'primary.json'),alias=path.join(root,'alias.json'),bad=path.join(root,'invalid.json');
await fs.writeFile(primary,'{"XX":{"name":"Primary"}}');await fs.writeFile(alias,'{"XX":{"name":"Alias"}}');await fs.writeFile(bad,'{');
for(const [name,p,a,expected]of [['primary-wins',primary,alias,'Primary'],['invalid-primary-uses-alias',bad,alias,'Alias'],['missing-primary-uses-alias',path.join(root,'missing.json'),alias,'Alias'],['alias-only',null,alias,'Alias']]){
 const out=run('XX','json',p,a);assert.equal(out.exit_code,0);assert.equal(JSON.parse(out.stdout).name,expected);fallbacks.push({name,...out});
}
const embedded=run('RI','json',bad,path.join(root,'missing-alias.json'));assert.equal(embedded.exit_code,0);assert.equal(JSON.parse(embedded.stdout).name,'Rhode Island');fallbacks.push({name:'both-invalid-use-embedded',...embedded});
const oddFormat=run('RI','arbitrary');assert.equal(oddFormat.exit_code,0);assert.equal(oddFormat.stdout,run('RI','table').stdout);fallbacks.push({name:'non-json-format-is-table',...oddFormat});
const sources=await Promise.all(['data/location_policy.json','crates/bisect-cli/src/main.rs','crates/bisect-cli/src/args.rs','crates/bisect-cli/src/policy.rs','crates/bisect-analysis/src/location_policy.rs','crates/bisect-wasm/src/location_policy.rs','web/lab/policy-files.js','web/lab/package-files.js','scripts/web/test_policy_cli_contract.mjs'].map(pin));
const evidence={status:'passed-actual-policy-CLI-contract',sources,native:nativePin,wasm:wasmPin,records,fallbacks,scope:'Actual policy command JSON/table, embedded names/codes, raw selected override/hash and refusal; CLI environment fallback hierarchy separately observed. Browser explicit override refuses malformed sources without fallback. No filesystem environment discovery, policy application or legal accuracy claim.'};
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root,cases:records.length,fallback_cases:fallbacks.length}));
