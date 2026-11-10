import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator,strictComparisonPairs=0;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);strictComparisonPairs++;}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {streamStateSourceBatch} from '../../web/lab/stream-state-source-batch.js';
import {createLocalStateOutput} from '../../web/lab/local-state-output.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/stream-native-states');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const {directory}=nativeOutputDirectoryHarness(root);
const files={nativeStateSourceSettings:{enabled:true,command_kind:'states'},statesCommandInputs:{arguments_json:JSON.stringify(['--year','2020','--version','vBulkStream','--output-dir','unopened path/intent','--states','RI','VT','--workers','4','--partition-mode','centroidal-voronoi','--cvd-iters','3','--cvd-metric','geographic','--balance-tolerance','50'])},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Bulk_stream',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}}};
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]}});
const selection={year:'2020',states:[],manifest:null};
let native=0,partitions=0,prepared=0,downloads=0,alive=0,peak=0;
const facade={artifactIdentity:engine.artifactIdentity,execute(request,options){
 if(request.operation==='run-prepared-engine'&&!request.input.inspect_only)partitions++;
 if(request.operation==='prepare-census-engine-input')prepared++;
 const result=engine.execute(request,options);
 // Compare every synchronous Rust reply (compile, sources, execution and exports).
 const file=path.join(root,'native-'+native+'.json');
 requireIdentity(request,result,file);return result;
}};
function requireIdentity(request,result,file){
 writeFileSync(file,JSON.stringify(request));
 const nativeReply=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:64*1024*1024}));
 compareNative(request,result,nativeReply);native++;
}
import {writeFileSync} from 'node:fs';
class Worker extends EventTarget{
 constructor(){super();alive++;peak=Math.max(peak,alive);}
 postMessage(message){setTimeout(async()=>{if(this.closed)return;try{if(message.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:engine.artifactIdentity}}));return;}
 const request=message.request,result=request.operation==='run-native-state-job'?await executeNativeStateJob(facade,request.input):facade.execute(request);
 this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
 }catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}},1);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
const fetcher=async url=>{downloads++;const item=plan.items.find(i=>i.url===url);return new Response(await fs.readFile(item.state_code==='RI'?'target/census-zip-acquisition/ri-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip':'target/state-source-acquisition/synthetic-states/'+item.state_code+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));};
const archiveReader=async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options);
async function run(source=files,{signal,onCheckpoint=()=>{},nativeOutputSettings,selected=selection,hardwareConcurrency=1}={}){
 const parent=await directory('run'),sink=await createLocalStateOutput(parent,{nativeOutputSettings});
 const receipt=await streamStateSourceBatch(plan,'2020',selected,source,null,null,null,sink,{createWorker:()=>new Worker(),wasmUrl:'local',fetcher,archiveReader,signal,onCheckpoint,hardwareConcurrency});
 assert.equal(alive,0);return {receipt,base:path.join(parent.dir,sink.name)};
}
const good=await run();assert.equal(good.receipt.summary.succeeded,2,JSON.stringify(good.receipt.states));assert(partitions>=2);const primaryExecutionRequests=partitions;assert.equal(prepared,2);assert.equal(downloads,4);assert.equal(peak,1);assert.equal(good.receipt.scheduling.requested_workers,'4');assert.equal(good.receipt.scheduling.effective_workers,1);assert.equal(good.receipt.scheduling.output_dir_accessed,false);
for(const row of good.receipt.states){
 const dir=path.join(good.base,row.state),jobs=JSON.parse(await fs.readFile(path.join(dir,'state-run-jobs.json'),'utf8')),job=jobs[0];
 assert.equal(job.native_state.command_kind,'states');assert.equal(job.native_state.command.arguments_json,files.statesCommandInputs.arguments_json);
 const project=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(path.join(dir,'source-project.bisect-workbench'))));
 assert.deepEqual(project.files.nativeStateSourceSettings,files.nativeStateSourceSettings);assert.deepEqual(project.files.statesCommandInputs,files.statesCommandInputs);assert.equal(project.result,null);
 assert.equal(parseProject(JSON.stringify(createProject({name:'Bulk settings',files:project.files,operation:'prepare-census-engine-input',constraints:[],result:null,lastOperation:null}))).files.nativeStateSourceSettings.command_kind,'states');
 await fs.copyFile(path.join(dir,'state-output.zip'),path.join(root,row.state+'-output.zip'));
 const ready=structuredClone(job);ready.scope.districts=row.state==='RI'?2:1;const result=await executeNativeStateJob(facade,ready);
 assert.equal(result.result.native_states.recipe.command_kind,'states');assert.equal(result.result.scope.districts,ready.scope.districts);assert.equal(result.result.source_review.engine_request.options.cvd_iters,3);assert.equal(result.result.source_review.engine_request.options.cvd_metric,'geographic');
 await fs.writeFile(path.join(root,row.state+'-execution.json'),JSON.stringify(result));
 const changed=structuredClone(ready);changed.native_state.command.arguments_json+=' ';const before=partitions;await assert.rejects(()=>executeNativeStateJob(facade,changed),/changed/);assert.equal(partitions,before);
}
const selected=structuredClone(files);selected.statesCommandInputs.arguments_json=JSON.stringify([...JSON.parse(files.statesCommandInputs.arguments_json).filter(x=>x!=='RI')]);
const one=await run(selected);assert.deepEqual(one.receipt.states.map(r=>r.state),['VT']);
for(const flags of [['--print-only'],['--year','2010'],['--states','CA']]){
 const bad=structuredClone(files);let args=JSON.parse(bad.statesCommandInputs.arguments_json);if(flags[0]==='--year')args=args.filter((_,i)=>i>1);if(flags[0]==='--states'){const start=args.indexOf('--states');args.splice(start,3);}bad.statesCommandInputs.arguments_json=JSON.stringify([...args,...flags]);const before={downloads,partitions,prepared};await assert.rejects(()=>run(bad));assert.deepEqual({downloads,partitions,prepared},before);assert.equal(alive,0);
}
const before={downloads,partitions};await assert.rejects(()=>run(files,{nativeOutputSettings:{enabled:true,version:'vWrong',layout:'state'}}),/version/);assert.deepEqual({downloads,partitions},before);
const controller=new AbortController(),partial=await run(files,{signal:controller.signal,onCheckpoint:r=>{if(r.summary.succeeded===1)controller.abort();}});assert.equal(partial.receipt.status,'cancelled');assert.equal(partial.receipt.summary.succeeded,1);
const browserProject=createProject({name:'Native States streamed source workflow',files:{...files,streamNativeOutputSettings:{enabled:true,version:'vBulkStream',layout:'state'},stateBatchInputs:selection,fetchPlanInputs:{source:{name:'two-state-selection.json',base64:Buffer.from(JSON.stringify({manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]})).toString('base64')}}},operation:'plan-fetch-sources',constraints:[],result:null,lastOperation:null});
await fs.writeFile(path.join(root,'browser.bisect'),JSON.stringify(browserProject));
await fs.writeFile(path.join(root,'receipt.json'),JSON.stringify(good.receipt));await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({strict_comparison_pairs:strictComparisonPairs,wasm:engine.artifactIdentity,native_replies:native,primary_runs:2,primary_execution_requests:primaryExecutionRequests,replay_runs:2,filtered_run:1,partial_run:1,embedded_district_counts:{RI:2,VT:1},source_preparation_and_bulk_options_preserved:true,filter_and_preflight_before_acquisition:true,one_cpu_slot_policy_verified:true,output_path_never_accessed:true,source_projects_inert:true,source_hash_tampering_refused:true,published:false},null,2));
console.log(`PASS native States source streaming: RI/VT native allocation, CVD settings, ${native} complete native/WASM replies, state filtering, cancellation, inert source projects and refusal before acquisition.`);
