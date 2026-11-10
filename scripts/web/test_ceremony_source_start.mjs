import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {startCeremonyFromSources,ceremonyStartSourceRequest,readCeremonyRuleProfile,CEREMONY_SOURCE_LIMIT} from '../../web/lab/ceremony-start-sources.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {ceremonyRequest} from '../../web/lab/package-files.js';
const option=name=>process.argv.find(arg=>arg.startsWith('--'+name+'='))?.slice(name.length+3);
const base=option('contract')??JSON.parse(await fs.readFile('target/cffi-legacy-gates/c-sidecar/ceremony-contract-roots.json','utf8')).node.root;
const contract=JSON.parse(await fs.readFile(path.join(base,'evidence.json'),'utf8'));assert.equal(contract.status,'passed-actual-CLI-ceremony-verification');
await fs.mkdir('target/ceremony-source-start',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/ceremony-source-start/run-'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.copyFile(path.join(base,'native-reference.exe'),path.join(root,'native-reference.exe'));
await fs.copyFile(option('wasm')??process.env.BISECT_TEST_WASM??path.join(base,'engine.wasm'),path.join(root,'engine.wasm'));
const tree=await fs.readFile(path.join(base,'tree.json'),'utf8');await fs.writeFile(path.join(root,'tree.json'),tree);
const engine=await instantiateEngine(await fs.readFile(path.join(root,'engine.wasm'))),records=[];
const calls=[],facade={execute(request){calls.push(request.operation);return engine.execute(request);}};
const cases=[['utf8',Buffer.from('Rule α\r\n'), '1000','10',2,['A','B']],['binary',Buffer.from([0,255,239,187,191,0,13,10]),'9007199254740993','10',2,['A','B']],['empty-profile',Buffer.alloc(0),'9223372036854775807','9223372036854775807',1,['one']],['negative-genesis',Buffer.from('negative'),'-9223372036854775808','1',2,[' A ','B']],['maximum-u64-review',Buffer.from('long review'),'1000','18446744073709551615',2,['A','B']],['zero-review',Buffer.from('invalid'),'1000','0',2,['A','B']],['trimmed-duplicate-witness',Buffer.from('duplicate'),'1000','10',2,[' A','A ']]];
for(const [name,profile,genesis,review,minimum,witnesses]of cases){
 const dir=path.join(root,name);await fs.mkdir(dir);await fs.writeFile(path.join(dir,'profile'),profile);
 const args=['ceremony','start','--tree',path.join(root,'tree.json'),'--rule-profile',path.join(dir,'profile'),'--genesis-unix-seconds='+genesis,'--review-seconds='+review,'--minimum-witnesses='+minimum,...witnesses.flatMap(w=>['--witness',w]),'--out',path.join(dir,'transcript.json')];
 const native=spawnSync(path.join(root,'native-reference.exe'),args,{encoding:'utf8',windowsHide:true});if(native.error)throw native.error;
 calls.length=0;let actual;try{actual={ok:true,result:await startCeremonyFromSources(facade,{tree_json:tree,rule_profile:{name:name+'.rules',base64:profile.toString('base64')},settings:{genesis_unix_seconds:genesis,review_seconds:review,minimum_witnesses:minimum,witness_receipts:witnesses}})};}catch(error){actual={ok:false,error:error.message};}
 await fs.writeFile(path.join(root,'progress.json'),JSON.stringify({status:'running',case:name,native:{exit_code:native.status,stdout:native.stdout,stderr:native.stderr},actual},null,2)+'\n');
 if(native.status===0){
  assert(actual.ok,actual.error);const nativeText=await fs.readFile(path.join(dir,'transcript.json'),'utf8');
  assert.deepEqual(JSON.parse(actual.result.transcript_json),JSON.parse(nativeText),name+' exact native JSON values');
  assert.equal(actual.result.source_preparation.rule_profile_sha256,hash(profile));
  assert(actual.result.source_preparation.config_json.includes('"genesis_unix_seconds":'+genesis));
  assert(actual.result.transcript_json.includes('"genesis_unix_seconds":'+genesis),'Exact large timestamp digits');
  assert(actual.result.transcript_json.includes('"minimum_review_seconds":'+review),'Exact u64 review digits');
 }else assert.equal(actual.ok,false,name+' must match native rejection');
 assert.deepEqual(calls,['start-ceremony'],'Use the actual native Start boundary without an extra tree reenumeration');
 records.push({name,native:{exit_code:native.status,stdout:native.stdout,stderr:native.stderr,arguments:args},actual});
}
const settings={genesis_unix_seconds:'1000',review_seconds:'10',minimum_witnesses:2,witness_receipts:['A','B']};
const maximum=records.find(record=>record.name==='maximum-u64-review').actual.result.transcript_json,maximumDir=path.join(root,'maximum-u64-review');
const lifecycle=[];
for(const [action,extra]of [['verify',['--tree',path.join(root,'tree.json')]],['halt',['--tree',path.join(root,'tree.json'),'--reason','Review interval cannot be reached','--out',path.join(maximumDir,'halted.json')]],['announce',['--tree',path.join(root,'tree.json'),'--published-unix-seconds=9223372036854775807','--witness','A','--witness','B','--out',path.join(maximumDir,'unreachable.json')]]]){
 const args=['ceremony',action,'--transcript',path.join(maximumDir,'transcript.json'),...extra],native=spawnSync(path.join(root,'native-reference.exe'),args,{encoding:'utf8',windowsHide:true});if(native.error)throw native.error;
 if(action==='verify'){assert.equal(native.status,0,native.stderr);assert.equal(engine.execute({operation:'verify-ceremony',tree_json:tree,transcript_json:maximum}).verified,true);}
 if(action==='halt'){assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(engine.execute({operation:'halt-ceremony',tree_json:tree,transcript_json:maximum,reason:'Review interval cannot be reached'}).transcript_json),JSON.parse(await fs.readFile(path.join(maximumDir,'halted.json'),'utf8')));}
 if(action==='announce'){assert.equal(native.status,1);assert.throws(()=>engine.execute({operation:'announce-ceremony',tree_json:tree,transcript_json:maximum,published_unix_seconds:'9223372036854775807',witness_receipts:['A','B']}));}
 lifecycle.push({action,arguments:args,exit_code:native.status,stdout:native.stdout,stderr:native.stderr});
}
for(const bad of ['01','+1','-0','9223372036854775808'])await assert.rejects(startCeremonyFromSources(engine,{tree_json:tree,rule_profile:{name:'empty',base64:''},settings:{...settings,genesis_unix_seconds:bad}}));
const mutation=Uint8Array.from([0,255,13,10]),expected=hash(mutation),input={tree_json:tree,rule_profile:{name:'mutation',base64:Buffer.from(mutation).toString('base64')},settings},pending=startCeremonyFromSources(engine,input);input.rule_profile.base64=Buffer.from([7,7,7,7]).toString('base64');settings.witness_receipts[0]='changed';const retained=await pending;
assert.equal(retained.source_preparation.rule_profile_sha256,expected);assert.deepEqual(retained.source_preparation.witness_receipts,['A','B']);
const corrupt=JSON.parse(tree);corrupt.nodes[0].certificate.instance_hash='sha256:'+'0'.repeat(64);
await assert.rejects(startCeremonyFromSources(engine,{tree_json:JSON.stringify(corrupt),rule_profile:{name:'empty',base64:''},settings:{...settings,witness_receipts:['A','B']}}),'Candidate JS metadata cannot bypass native proof/root binding');
const sourceInputs={rule_profile:{name:'empty.rules',base64:''},settings:{genesis_unix_seconds:'9007199254740993',review_seconds:'18446744073709551615',minimum_witnesses:2,witness_receipts:[' A ','B','']}};
const treeSource={name:'tree.json',base64:Buffer.from(tree).toString('base64')},request=ceremonyStartSourceRequest(treeSource,sourceInputs);
assert.equal(request.input.tree_json,tree);assert.equal(request.input.rule_profile.base64,'');assert.deepEqual(request.input.settings.witness_receipts,[' A ','B','']);
const bomTree=Buffer.concat([Buffer.from([239,187,191]),Buffer.from(tree)]),bomFile=path.join(root,'bom-tree.json');await fs.writeFile(bomFile,bomTree);
const bomRequest=ceremonyStartSourceRequest({name:'bom-tree.json',base64:bomTree.toString('base64')},sourceInputs);assert.deepEqual(Buffer.from(bomRequest.input.tree_json,'utf8'),bomTree,'Tree bytes including BOM must remain unchanged');
const bomNative=spawnSync(path.join(root,'native-reference.exe'),['ceremony','start','--tree',bomFile,'--rule-profile',path.join(root,'utf8','profile'),'--genesis-unix-seconds=1000','--review-seconds=10','--minimum-witnesses=2','--witness','A','--witness','B','--out',path.join(root,'bom-output.json')],{encoding:'utf8',windowsHide:true});assert.equal(bomNative.status,1);await assert.rejects(startCeremonyFromSources(engine,bomRequest.input));
const legacyBom=ceremonyRequest('start-ceremony',{ceremonyTree:{name:'bom-tree.json',base64:bomTree.toString('base64')},ceremonyConfig:{name:'config.json',base64:Buffer.from(records[0].actual.result.source_preparation.config_json).toString('base64')}},{published_unix_seconds:'',witness_receipts:[],reason:'',include_tree:false});assert.equal(legacyBom.tree_json.charCodeAt(0),0xfeff);assert.throws(()=>engine.execute(legacyBom),'Prepared-config ceremony also preserves/refuses the native-invalid BOM');
const saved=createProject({name:'Original profile',operation:'start-ceremony-from-sources',files:{ceremonyTree:treeSource,ceremonyStartInputs:sourceInputs},constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(saved)),saved);
for(const change of [v=>v.rule_profile.base64='Zh==',v=>v.rule_profile.javascript='execute',v=>v.settings.genesis_unix_seconds='01',v=>v.settings.review_seconds='18446744073709551616',v=>v.settings.genesis_unix_seconds=9007199254740992,v=>v.settings.extra='ignored']){
 const changed=structuredClone(saved);change(changed.files.ceremonyStartInputs);assert.throws(()=>parseProject(JSON.stringify(changed)),'Unsafe/ambiguous original-profile input must refuse Open');
}
let read=false;await assert.rejects(readCeremonyRuleProfile({name:'large',size:CEREMONY_SOURCE_LIMIT+1,arrayBuffer:async()=>{read=true;return new ArrayBuffer(0);}}));assert.equal(read,false);
await assert.rejects(readCeremonyRuleProfile({name:'changed',size:1,arrayBuffer:async()=>new ArrayBuffer(0)}),/size changed/);
assert.deepEqual(await readCeremonyRuleProfile({name:'empty.rules',size:0,arrayBuffer:async()=>new ArrayBuffer(0)}),{name:'empty.rules',base64:''});
await fs.copyFile('web/lab/ceremony-start-sources.js',path.join(root,'module-source.js'));await fs.copyFile('scripts/web/test_ceremony_source_start.mjs',path.join(root,'test-source.mjs'));
const sources=await Promise.all([path.join(root,'module-source.js'),path.join(root,'test-source.mjs'),path.join(root,'native-reference.exe'),path.join(root,'engine.wasm'),path.join(root,'tree.json'),'crates/bisect-cli/src/ceremony_cmd.rs','crates/bisect-ilp/src/ceremony.rs','crates/bisect-wasm/src/ceremony.rs','web/lab/package-files.js'].map(async file=>({path:file,sha256:hash(await fs.readFile(file))})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-ceremony-source-start',sources,records,maximum_u64_review_lifecycle:lifecycle,bom_tree:{native_exit:bomNative.status,native_stdout:bomNative.stdout,native_stderr:bomNative.stderr,bytes_preserved:true,browser_refused:true},exact_integer_rejections:4,snapshot_mutation_guard:true,native_root_binding_refusal:true,single_start_boundary:true,project_roundtrip_and_six_refusals:true,empty_profile_and_pre_read_guards:true,product_integrated:true,scope:'Production source adapter over existing native/WASM kernels. Five native start successes/two matching failures, matching BOM-tree refusal, raw profile SHA, exact i64/u64, maximum-review lifecycle and source/witness guards. This Node record does not establish actual UI/worker behavior.'},null,2)+'\n');console.log(JSON.stringify({status:'passed-ceremony-source-start',root}));
