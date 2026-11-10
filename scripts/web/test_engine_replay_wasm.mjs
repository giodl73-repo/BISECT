import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import * as helper from '../../web/lab/engine-replay-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/engine-replay-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/optional-seed-wasm/case-0.json','utf8'));
const multi=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input;
const evidence=[];
async function check(name,artifact,original,status='match',threshold=1,raw=null){
 const input={run_json:raw??JSON.stringify(artifact),original_assignments_json:JSON.stringify(original),min_similarity:threshold},request={operation:'replay-engine-run',input};
 const result=engine.execute(request),path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));
 const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(result,native);assert.equal(result.status,status);assert.equal(result.comparison_exit_code,status==='match'?0:1);
 const sources={run:{name:'run.json',base64:Buffer.from(input.run_json).toString('base64')},original:{name:'original.json',base64:Buffer.from(input.original_assignments_json).toString('base64')},minSimilarity:threshold};
 // Invalid run artifacts deliberately exercise Rust's inspectable failure receipt.
 if(artifact){assert.deepEqual(helper.engineReplayRequest(sources),request);const project=createProject({name:'Engine replay: '+name,files:{engineReplayInputs:sources},operation:'replay-engine-run',constraints:[],result,lastOperation:'replay-engine-run'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));}
 assert.deepEqual(JSON.parse(new TextDecoder().decode(helper.engineReplayReceiptBytes(result))),result);await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));evidence.push({name,status,kind:result.run_kind,execution_performed:result.execution_performed,execution_completed:result.execution_completed});return result;
}
for(const seed of [null,42,'18446744073709551615']){
 const request={graph:base.graph,options:{...base.options,seed,balance_tolerance:50}};
 for(const build of [false,true]){const direct=engine.execute(build?{operation:'run-engine-build',request}:request);const result=await check('engine-'+(build?'build-':'')+String(seed),{kind:build?'engine-build':'engine',request},direct.assignments);assert.deepEqual(result.replayed_result,direct);assert.equal(result.execution_completed,true);}
 if(seed!==null){const raw=JSON.stringify({kind:'engine',request}).replace('"seed":"18446744073709551615"','"seed":18446744073709551615');const result=await check('raw-seed-'+seed,{kind:'engine',request},engine.execute(request).assignments,'match',1,raw);assert.equal(String(result.replayed_seed),String(seed));}
}
for(const build of [false,true]){const input=structuredClone(multi);input.request.options.balance_tolerance=50;input.request.options.seed='18446744073709551615';input.total_steps=0;const direct=engine.execute({operation:build?'run-multiscale-build':'run-multiscale',input});const result=await check(build?'multiscale-build':'multiscale',{kind:build?'multiscale-build':'multiscale',input},direct.assignments);assert.deepEqual(result.replayed_result,direct);}
for(const build of [false,true]){const input={graph:base.graph,yaml_text:base.yaml_text.replace('balance_tolerance: 25','balance_tolerance: 50')+'  seed: 18446744073709551615\n',districts:2,chamber:'congressional',source_name:'run.yml',profile:'open'};const direct=engine.execute({operation:build?'run-algorithm-build':'run-algorithm-config',input});const result=await check(build?'configured-build':'configured',{kind:build?'configured-build':'configured',input},direct.result.assignments);assert.deepEqual(result.replayed_result,direct);
 if(!build){await fs.writeFile(root+'/yaml-oracle.json',JSON.stringify({graph:base.graph,yaml_text:input.yaml_text,options:{districts:2}}));const oracle=JSON.parse(execFileSync('target/debug/examples/config_seed_reference.exe',[root+'/yaml-oracle.json'],{encoding:'utf8'}));assert.deepEqual(result.replayed_result.result.assignments,oracle.assignments);}}
const artifact={kind:'engine',request:{graph:base.graph,options:{...base.options,seed:42}}},original=engine.execute(artifact.request).assignments;
const changed=Object.fromEntries(Object.entries(original).map(([id,d])=>[id,d===1?2:1]));assert.equal((await check('different',artifact,changed,'different')).agreement.similarity,0);
const unavailable=await check('empty-original',artifact,{},'unavailable',-1);assert.equal(unavailable.execution_performed,false);
const invalid=await check('invalid-artifact',null,original,'failed',1,'{bad');assert.equal(invalid.execution_performed,false);
const invalidGraph={kind:'engine',request:{...artifact.request,graph:{...base.graph,population:[]}}};const failed=await check('failed-execution',invalidGraph,original,'failed');assert.equal(failed.execution_performed,true);assert.equal(failed.execution_completed,false);
await check('above-one',artifact,original,'different',1.1);await check('negative-threshold',artifact,changed,'match',-1);
let rejected=0;for(const patch of [{run_json:''},{run_json:' '.repeat(16*1024*1024+1)},{original_assignments_json:'[]'},{original_assignments_json:'{"a":4294967296}'}]){assert.throws(()=>engine.execute({operation:'replay-engine-run',input:{run_json:JSON.stringify(artifact),original_assignments_json:JSON.stringify(original),min_similarity:1,...patch}}));rejected++;}
let reads=0;await assert.rejects(()=>helper.readEngineReplaySource({name:'too-large.json',size:16*1024*1024+1,arrayBuffer:()=>{reads++;}},'run'));assert.equal(reads,0);
await assert.rejects(()=>helper.readEngineReplaySource({name:'bad.json',size:2,arrayBuffer:async()=>Uint8Array.from([255,255]).buffer},'run'));
assert.throws(()=>helper.validateEngineReplayInputs({run:null,original:null,minSimilarity:Infinity}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence,boundary_rejections:rejected,native_yaml_oracle:true},null,2));console.log(`Engine replay: ${evidence.length} native/WASM parity cases; all six run kinds, full u64 raw seeds, actual YAML oracle, failure/execution receipts, empty original, thresholds, source bounds and project retention passed.`);

for(const name of ['configured','configured-build']){
 const configured=JSON.parse(await fs.readFile(root+'/'+name+'-result.json','utf8')).replayed_result;
 const inputs=helper.engineReplayFromConfigured(configured),request=helper.engineReplayRequest(inputs),artifact=JSON.parse(request.input.run_json);
 assert.equal(artifact.kind,'engine');assert.deepEqual(artifact.request,configured.engine_request);assert.equal(String(artifact.request.options.seed),String(configured.engine_request.options.seed));
 assert.deepEqual(JSON.parse(request.input.original_assignments_json),configured.result.assignments);assert.equal(engine.execute(request).status,'match');
 const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
 const files={configSource:source('run.yml',configured.configuration.yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(configured.engine_request.graph))},configRunSettings:{chamber:'congressional',districts:2,native_build:name==='configured-build'}};
 const project=createProject({name:'Configured handoff '+name,files,operation:'run-algorithm-config',constraints:[],result:configured,lastOperation:'run-algorithm-config'});
 await fs.writeFile(root+'/handoff-'+name+'.bisect',JSON.stringify(project));
}
assert.throws(()=>helper.engineReplayFromConfigured({schema_version:'bisect-configured-run-v1',engine_request:{},result:{assignments:{}}}),/incomplete/);
console.log('Configured replay handoff: exact accepted options, seed and assignments, explicit engine replay and malformed-result rejection passed.');

{
 const graph={...base.graph,population:base.graph.population.map((_,i)=>100+(i*37)%67)};
 const input={graph,yaml_text:'name: retry\nalgorithm:\n  structure: nway\n  engine: metis-core\n  search: single\n  seeds: 3\n  balance_tolerance: 2\n  seed: 18446744073709551615\n',source_name:'retry.yml',profile:'open',chamber:'congressional',districts:4};
 const built=engine.execute({operation:'run-algorithm-build',input});assert.equal(built.build_report.accepted_attempt,17);assert.equal(String(built.engine_request.options.seed),'16');
 const selected=helper.engineReplayRequest(helper.engineReplayFromConfigured(built));const replay=engine.execute(selected);assert.equal(replay.status,'match');assert.equal(String(replay.replayed_seed),'16');assert.equal(replay.build_report,null);assert.deepEqual(replay.replayed_result.assignments,built.result.assignments);
 await fs.writeFile(root+'/accepted-retry-request.json',JSON.stringify(selected));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/accepted-retry-request.json'],{encoding:'utf8'}));assert.deepEqual(replay,native);
 await fs.writeFile(root+'/accepted-retry-evidence.json',JSON.stringify({original_seed:'18446744073709551615',accepted_attempt:17,accepted_seed:'16',replayed_seed:String(replay.replayed_seed),agreement:replay.agreement},null,2));
 console.log('Accepted retry replay: native/WASM exact match after 17 retries and u64 wraparound; replays accepted seed 16 without rerunning retry policy.');
}
