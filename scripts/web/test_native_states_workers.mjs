import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {streamStateSourceBatch} from '../../web/lab/stream-state-source-batch.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {nativeStatesWorkerPolicy} from '../../web/lab/native-states-workers.js';
const root='target/native-states-workers';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const states=['DE','VT','WY'];
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states,year:'2020',data_types:[],selected_paths:[]}});
const files={nativeStateSourceSettings:{enabled:true,command_kind:'states'},statesCommandInputs:{arguments_json:'[]'},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Worker_fixture',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}}};
for(const [requested,available,count,expected] of [[4,2,3,2],[1,8,3,1],[0,2,3,2],[100,2,3,2],[4,0,3,1],[4,2,0,0]])assert.equal(nativeStatesWorkerPolicy({workers:String(requested)},count,available).effective_workers,expected);
for(const value of [-1,1.5,'bad',Number.MAX_SAFE_INTEGER+1])assert.throws(()=>nativeStatesWorkerPolicy({workers:String(value)},3,2));
let checks=0;
async function run({workers=4,hardwareConcurrency=2,cancel=false,ioFailure=false,workerFailure=false,artifactFailure=false,checkpointFailure=false}={}){
 const controller=new AbortController(),selected=structuredClone(files);selected.statesCommandInputs.arguments_json=JSON.stringify(['--year','2020','--version','vWorkers','--output-dir','unopened','--workers',String(workers),'--states',...states,'--partition-mode','centroidal-voronoi','--cvd-iters','3','--cvd-metric','geographic','--balance-tolerance','50']);
 let live=0,peak=0,active=0,peakRuns=0,started=[],finished=[],saves=0,peakSaves=0,checkpoints=0,peakCheckpoints=0,created=0,partitionRuns=0;const receipts=[],observed=[];
 const facade={artifactIdentity:engine.artifactIdentity,execute(request,options){if(request.operation==='run-prepared-engine'&&!request.input.inspect_only)partitionRuns++;return engine.execute(request,options);}};
 class Worker extends EventTarget{
  constructor(){super();live++;created++;this.number=created;peak=Math.max(peak,live);}
  postMessage(message){setTimeout(async()=>{if(this.closed)return;try{
   if(message.type==='initialize'){const artifact=artifactFailure&&this.number===3?{...engine.artifactIdentity,sha256:'0'.repeat(64)}:engine.artifactIdentity;this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:artifact}}));return;}
   const request=message.request;let result;
   if(request.operation==='run-native-state-job'){
    const state=request.input.scope.state;started.push(state);active++;peakRuns=Math.max(peakRuns,active);
    // Deliberately finish out of selection order while real WASM performs each Run.
    await new Promise(resolve=>setTimeout(resolve,state==='DE'?100:state==='VT'?15:1));
    if(this.closed)return;
    if(workerFailure&&state==='VT'){this.dispatchEvent(Object.assign(new Event('error'),{message:'Test worker crashed'}));return;}
    result=await executeNativeStateJob(facade,request.input);active--;finished.push(state);
   }else result=facade.execute(request);
   if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
  }catch(error){if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}},1);}
  terminate(){if(!this.closed){this.closed=true;live--;}}
 }
 const sink={name:'test-local-output',async archive(){},async checkpoint(receipt){checkpoints++;peakCheckpoints=Math.max(peakCheckpoints,checkpoints);await new Promise(resolve=>setTimeout(resolve,5));checkpoints--;if(checkpointFailure&&receipt.summary.completed)throw Error('Test checkpoint disk failure');receipts.push(receipt);},async state(job,output){saves++;peakSaves=Math.max(peakSaves,saves);try{await new Promise(resolve=>setTimeout(resolve,15));if(ioFailure&&job.scope.state==='VT')throw Error('Test disk full');assert.equal(output.result.scope.state,job.scope.state);return {directory:job.scope.state,artifacts:[]};}finally{saves--;}}};
 const fetcher=async url=>{const item=plan.items.find(i=>i.url===url);return new Response(await fs.readFile('target/state-source-acquisition/synthetic-states/'+item.state_code+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));};
 const promise=streamStateSourceBatch(plan,'2020',{year:'2020',states,manifest:null},selected,null,null,null,sink,{hardwareConcurrency,createWorker:()=>new Worker(),fetcher,archiveReader:async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options),signal:controller.signal,onCheckpoint:r=>{observed.push(r);if(cancel&&r.summary.succeeded>=1)controller.abort(new Error('Test user cancellation'));}});
 let receipt,error;try{receipt=await promise;}catch(e){error=e;}
 assert.equal(live,0);assert(peakSaves<=1);assert(peakCheckpoints<=1);assert(peak<=Math.min(workers||hardwareConcurrency,hardwareConcurrency,3));
 if(workerFailure||artifactFailure||checkpointFailure){assert(error);assert.equal(observed.at(-1)?.status,'interrupted');if(!checkpointFailure)assert.equal(receipts.at(-1)?.status,'interrupted');else assert.equal(receipts.at(-1)?.status,'running');}
 else {assert.ifError(error);assert.equal(receipt.scheduling.effective_workers,Math.min(workers||hardwareConcurrency,hardwareConcurrency,3));assert.equal(receipt.status,cancel?'cancelled':'completed');const order=receipt.states.map(r=>r.state);assert.deepEqual(order,states.filter(s=>order.includes(s)));if(cancel){assert(receipt.summary.succeeded>=1);assert(receipt.summary.succeeded<3);}else if(ioFailure){assert.equal(receipt.summary.failed,1);assert.equal(receipt.summary.succeeded,2);}else{assert.equal(receipt.summary.succeeded,3);assert.equal(partitionRuns,3);if(workers!==1){assert(peakRuns>=2);assert.notDeepEqual(finished,states);}}}
 checks++;return {workers,hardwareConcurrency,peak_workers:peak,peak_runs:peakRuns,peak_saves:peakSaves,peak_checkpoint_writes:peakCheckpoints,started,finished,summary:receipt?.summary,status:receipt?.status,error:error?.message,retained_states:receipts.at(-1)?.states.map(r=>r.state),partition_runs:partitionRuns};
}
const cases=[];for(const options of [{},{workers:1},{workers:0},{workers:100},{cancel:true},{ioFailure:true},{workerFailure:true},{artifactFailure:true},{checkpointFailure:true}])cases.push(await run(options));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,cases,worker_policy_cases:6,checked_runs:checks,real_wasm_execution:true,synthetic_one_seat_states:states,selection_order_preserved:true,install_and_checkpoint_writes_serialized:true,all_workers_terminated:true,published:false},null,2));
console.log(`PASS native States worker scheduling: ${checks} real-WASM scenarios, CPU/request/state caps, zero auto pool, out-of-order completion, serialized writes, cancellation, disk/worker/artifact failures and cleanup.`);
