import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {inspectStateCommandWithIdentity} from '../../web/lab/native-state-identity-preview.js';
import {statePreparedFlagRefusals} from '../../web/lab/state-command-files.js';
const wasmFile=process.env.BISECT_TEST_WASM;
assert(wasmFile,'Set BISECT_TEST_WASM');
const reference=path.resolve(process.argv[2]),root=path.resolve('target/state-identity/targets-'+Date.now());await fs.mkdir(root,{recursive:true});
const hash=b=>createHash('sha256').update(b).digest('hex'),raw=await fs.readFile(wasmFile),engine=await instantiateEngine(raw),calls=[];
const facade={execute(request){calls.push(request.operation);return engine.execute(request);}};
const native=JSON.parse(await fs.readFile(path.join(reference,'evidence.json'),'utf8'));assert.equal(native.status,'passed');
assert.equal(hash(await fs.readFile(native.native.path)),native.native.sha256);
const outcomes=[];
for(const row of native.outcomes){
 assert.equal(row.exit_code,0);assert.equal(row.arguments[0],'state');
 for(const pin of row.files){const bytes=await fs.readFile(pin.path);assert.equal(bytes.length,pin.byte_length);assert.equal(hash(bytes),pin.sha256);}
 const args=row.arguments.slice(1),manifestPin=row.files.find(p=>p.path.endsWith('/manifest.json'));assert(manifestPin);
 const manifest=JSON.parse(await fs.readFile(manifestPin.path,'utf8')),caseRoot=path.join(reference,row.name),relative=path.relative(caseRoot,path.dirname(path.resolve(manifestPin.path))).split(path.sep).join('/');
 const result=await inspectStateCommandWithIdentity(facade,args),preview=result.browser_identity_preview;
 assert.equal(preview.error,null,row.name);assert.equal(preview.relative_target,relative);
 assert.equal(preview.identity.effective_label,manifest.label);assert.equal(preview.identity.native_source_name_intents.adjacency_manifest,manifest.adjacency_file);
 assert.equal(preview.folder_presence_observed,false);assert.equal(preview.execution_authorized,false);
 if(args.includes('--state-name'))assert(statePreparedFlagRefusals(result).includes('--state-name'));
 const paths=row.files.map(p=>path.relative(caseRoot,path.resolve(p.path)).split(path.sep).join('/'));
 const manifestRelative=relative+'/manifest.json',input={source_name:'actual-native-folder.json',folder_json:JSON.stringify({version:'v1',paths,manifests:{[manifestRelative]:JSON.stringify(manifest)}}),force:false};
 const request={operation:'inspect-native-named-state-run-target',arguments:args,version:'v1',input};
 if(preview.identity.layout==='labeled')assert.throws(()=>facade.execute(request),/exists|collision|already/i);
 else assert.equal(facade.execute(request).target_namespace,'outputs/v1/'+relative);
 const forced=facade.execute({...request,input:{...input,force:true}});assert.equal(forced.target_namespace,'outputs/v1/'+relative);
 assert.throws(()=>facade.execute({...request,input:{...input,force:true,folder_json:JSON.stringify({version:'v1',paths:[relative+'/manifest.tmp'],manifests:{}})}}),/incomplete|manifest.tmp/i);
 outcomes.push({name:row.name,arguments:args,preview,native_manifest:manifestPin});
}
for(const alias of ['../outside','CON','trailing ']){
 const p=(await inspectStateCommandWithIdentity(facade,['--state','RI','--adjacency','unopened.adj.bin','--state-name',alias])).browser_identity_preview;
 assert.equal(p.identity.effective_state_name,alias);assert.equal(p.relative_target,null);assert.match(p.error,/portable/);assert.equal(p.execution_authorized,false);
}
const ignored=await inspectStateCommandWithIdentity(facade,['--state','RI','--state-name','Ignored']);assert.equal(ignored.browser_identity_preview.identity.override_ignored,true);assert.equal(ignored.browser_identity_preview.relative_target,'2020/states/rhode_island');
const badYear=await inspectStateCommandWithIdentity(facade,['--state','RI','--year','2024','--adjacency','unopened.adj.bin']);assert.equal(badYear.browser_identity_preview.relative_target,null);assert.equal(badYear.browser_identity_preview.identity.native_year_valid,false);
assert(calls.every(c=>c.startsWith('inspect-')));await fs.writeFile(path.join(root,'outcomes.json'),JSON.stringify(outcomes,null,2));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed',wasm:{path:wasmFile,sha256:hash(raw),byte_length:raw.length},native_reference:{path:path.join(reference,'evidence.json'),sha256:hash(await fs.readFile(path.join(reference,'evidence.json')))},actual_native_namespace_label_and_manifest_matches:outcomes.length,incomplete_never_bypassed:true,labeled_collision_requires_force:true,raw_names_preserved:true,unsafe_names_not_sanitized:true,native_year_preflight:true,read_only_operations:calls,execution_guard_retained:true,scope:'Native-equal naming and selected target inspection only; no workflow execution, live folder grant, installation, C profile or complete State flag acceptance.'},null,2));console.log(JSON.stringify({root,status:'passed',native_cases:outcomes.length}));
