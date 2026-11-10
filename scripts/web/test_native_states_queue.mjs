import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {acquireStateSourceJob} from '../../web/lab/state-source-acquisition.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {appendStateRunJob,prepareNativeStatesQueue,runStateBatch,stateRunBatchZip,canResumeStateBatch} from '../../web/lab/state-run-batch.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/native-states-queue');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['DE','VT','WY'],year:'2020',data_types:[],selected_paths:[]}});
const files={nativeStateSourceSettings:{enabled:true},stateCommandInputs:{arguments_json:'["--state","RI","--metis-engine","metis-core","--seed","123"]'},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Queued_bulk',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}}};
let inputs={jobs:null,workers:1};
for(const state of ['DE','VT','WY']){
 const job=await acquireStateSourceJob(plan,{state,year:'2020'},files,null,null,null,{inspectNativeCommand:r=>engine.execute(r),fetcher:async url=>{const item=plan.items.find(i=>i.url===url);return new Response(await fs.readFile('target/state-source-acquisition/synthetic-states/'+state+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));},archiveReader:async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options)});
 inputs=appendStateRunJob(inputs,job);
}
const tampered=structuredClone(inputs),changedJobs=JSON.parse(Buffer.from(tampered.jobs.base64,'base64'));changedJobs[0].native_state.source_inputs.settings.label='Changed after queue';tampered.jobs.base64=Buffer.from(JSON.stringify(changedJobs)).toString('base64');
const original=structuredClone(inputs),selection={year:'2020',states:['DE'],manifest:null};
const command=(workers=100,states=['VT','WY'],extra=[])=>({arguments_json:JSON.stringify(['--year','2020','--version','vQueue','--output-dir','unopened paths','--workers',String(workers),'--states',...states,'--partition-mode','centroidal-voronoi','--cvd-iters','3','--cvd-metric','geographic','--balance-tolerance','50',...extra])});
const inspect=c=>engine.execute({operation:'inspect-native-states-command',arguments:JSON.parse(c.arguments_json)});
await assert.rejects(()=>prepareNativeStatesQueue(command(),tampered,selection,inspect(command())),/identity changed/);
const selected=await prepareNativeStatesQueue(command(),inputs,selection,inspect(command()));assert.deepEqual(inputs,original);assert.equal(selected.inputs.workers,1);assert.deepEqual(selected.selection.states,['VT','WY']);assert(selected.inputs.bulk_command);
let alive=0,peak=0,cuts=0,prepared=0,nativeReplies=0;
class Worker extends EventTarget{
 constructor(){super();alive++;peak=Math.max(peak,alive);}
 postMessage(message){setTimeout(async()=>{if(this.closed)return;try{if(message.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:engine.artifactIdentity}}));return;}
 const facade={artifactIdentity:engine.artifactIdentity,execute(r,o){if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)cuts++;if(r.operation==='prepare-census-engine-input')prepared++;const reply=engine.execute(r,o),file=path.join(root,'native-'+nativeReplies+'.json');writeFileSync(file,JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:64*1024*1024}));compareNative(r,reply,native);nativeReplies++;return reply;}};
 const result=message.request.operation==='run-native-state-job'?await executeNativeStateJob(facade,message.request.input):facade.execute(message.request);if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
 }catch(error){if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}},1);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
const run=(v=selected,options={})=>runStateBatch(v.inputs,v.selection,{createWorker:()=>new Worker(),hardwareConcurrency:2,wasmUrl:'local',...options});
const good=await run();assert.equal(good.summary.succeeded,2);assert.equal(cuts,2);assert.equal(alive,0);assert.equal(peak,2);assert.equal(good.workers.requested,100);assert.equal(good.workers.effective,2);assert.deepEqual(good.states.map(r=>r.state_code),['VT','WY']);assert.equal(good.native_states_command.workers,'100');
for(const row of good.states){assert.equal(row.result.native_states.recipe.command.arguments_json,command().arguments_json);assert.equal(row.result.source_review.engine_request.options.cvd_iters,3);assert.equal(row.result.source_review.engine_request.options.cvd_metric,'geographic');assert.equal(row.result.prepared.executed_request.options.cvd_iters,3);assert.equal(row.result.prepared.executed_request.options.cvd_metric,'geographic');}
await fs.writeFile(root+'/accepted.zip',await stateRunBatchZip(good));await fs.writeFile(root+'/accepted.json',JSON.stringify(good));
const restored=JSON.parse(JSON.stringify(good));const verified=await run(selected,{verifySaved:restored});assert.equal(verified.summary.succeeded,2);assert.equal(verified.workers.effective,1);
const forged=structuredClone(restored);forged.native_states_command.workers='1';const before=cuts;await assert.rejects(()=>run(selected,{verifySaved:forged}),/policy changed/);assert.equal(cuts,before);
const zero=await prepareNativeStatesQueue(command(0),inputs,selection,inspect(command(0)));assert.equal((await run(zero)).workers.effective,2);
const controller=new AbortController();let partial;await assert.rejects(()=>run(selected,{hardwareConcurrency:1,signal:controller.signal,onCheckpoint:r=>{partial=r;if(r.summary.succeeded>=1)controller.abort();}}));partial.status='cancelled';assert(canResumeStateBatch(partial));const resumed=await run(selected,{resume:partial});assert.equal(resumed.summary.succeeded,2);
for(const mutate of [v=>{v.inputs.bulk_command=command(1);},v=>{v.selection.year='2010';},v=>{v.inputs.bulk_command=command(1,['VT'],['--print-only']);},v=>{v.selection.manifest={name:'custom.json',base64:Buffer.from('{}').toString('base64')};}]){
 const invalid=structuredClone(selected);mutate(invalid);const before={cuts,prepared};await assert.rejects(()=>run(invalid));assert.deepEqual({cuts,prepared},before);assert.equal(alive,0);
}
const missing=await prepareNativeStatesQueue(command(2,['CA']),inputs,selection,inspect(command(2,['CA'])));const missingBefore=cuts;await assert.rejects(()=>run(missing),/Missing prepared/);assert.equal(cuts,missingBefore);
const project=createProject({name:'Inspect bulk flags and apply to source queue',files:{statesCommandInputs:command(),stateRunBatchInputs:inputs,stateBatchInputs:selection},operation:'inspect-native-states-command',constraints:[],result:null,lastOperation:null});assert.equal(parseProject(JSON.stringify(project)).operation,'inspect-native-states-command');await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,source_states:['DE','VT','WY'],selected_states:['VT','WY'],synthetic_tract_sources:true,primary_runs:2,whole_native_wasm_replies:nativeReplies,original_source_tampering_refused_before_rebinding:true,requested_workers:100,effective_workers:2,zero_auto_workers:2,actual_cvd_flags_retained:true,explicit_handoff_no_preparation:true,original_queue_unchanged:true,saved_full_replay_verified:true,tampered_policy_refused_before_partition:true,live_resume_verified:true,preflight_no_partition_or_preparation:true,published:false},null,2));
console.log('PASS queued native States policy: explicit source-preserving handoff, native filters/worker caps/zero auto, CVD Runs, saved replay, live Resume and preflight refusals.');
