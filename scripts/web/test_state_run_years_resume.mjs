import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {runStateBatch,canResumeStateBatch,stateRunBatchZip} from '../../web/lab/state-run-batch.js';
const fixture=JSON.parse(await fs.readFile('target/state-run-years/browser.bisect','utf8'));
const inputs=fixture.files.stateRunBatchInputs,selection=fixture.files.stateBatchInputs;
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let alive=0,wrongArtifact=false,failYear=null;const runs=[],selections=[];
class WorkerAdapter extends EventTarget{
 constructor(){super();this.closed=false;alive++;}
 postMessage(message){setTimeout(()=>{
  if(this.closed)return;
  try{
   if(message.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:wrongArtifact?{...engine.artifactIdentity,sha256:'0'.repeat(64)}:engine.artifactIdentity}}));return;}
   const request=message.request;
   if(request.operation==='inspect-state-batch-selection')selections.push(request.input.year);
   if(request.operation==='run-prepared-state-job'){runs.push(request.input.scope.year);if(request.input.scope.year===failYear)throw new Error('Injected transient state failure');}
   const result=request.operation==='run-prepared-state-job'?executePreparedStateJob(engine,request.input):engine.execute(request);
   this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
  }catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}
 },0);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
const execute=(options={},data=inputs,scope=selection)=>runStateBatch(data,scope,{createWorker:()=>new WorkerAdapter(),hardwareConcurrency:2,wasmUrl:'local',...options});
async function cancelAfterFirst(){const controller=new AbortController();return execute({signal:controller.signal,onCheckpoint:c=>{if(c.state_summary.succeeded===1)controller.abort(new Error('Test cancellation'));}});}
const first=await cancelAfterFirst();assert.equal(first.status,'cancelled');assert(canResumeStateBatch(first));assert.equal(alive,0);
const retained=JSON.stringify(first.years[0].result.states),before=runs.length,selectionBefore=selections.length;
const resumed=await execute({resume:first});
assert.equal(resumed.status,'completed');assert.equal(resumed.state_summary.succeeded,3);assert.equal(canResumeStateBatch(resumed),false);
assert.deepEqual(runs.slice(before),['2010','2000'],'Completed 2020 must not execute again');
assert.deepEqual(selections.slice(selectionBefore),['2020','2010','2000'],'Every retained year must freshly validate its native queue');
assert.equal(JSON.stringify(resumed.years[0].result.states),retained);assert.equal(alive,0);
await assert.rejects(()=>execute({resume:structuredClone(first)}),/unchanged checkpoint/);
await assert.rejects(()=>execute({resume:first},{...inputs,workers:inputs.workers===1?2:1}),/unchanged checkpoint/);
await assert.rejects(()=>execute({resume:first},inputs,{...selection,states:['VT']}),/unchanged checkpoint/);
const altered=await cancelAfterFirst();altered.years[0].result.states[0].result.prepared.result.metrics.within_requested_tolerance=false;
await assert.rejects(()=>execute({resume:altered}),/unchanged checkpoint/);
const artifactCase=await cancelAfterFirst(),beforeArtifact=runs.length;wrongArtifact=true;
const rejected=await execute({resume:artifactCase});wrongArtifact=false;
assert.equal(rejected.status,'interrupted');assert.equal(runs.length,beforeArtifact);assert.equal(alive,0);
assert.equal(JSON.stringify(rejected.years[0].result.states),retained);
// Cancelling a second Resume must preserve still-retained later year outputs too.
const secondController=new AbortController();secondController.abort(new Error('Already cancelled'));
const twice=await execute({resume:first,signal:secondController.signal});assert.equal(twice.status,'cancelled');assert.equal(JSON.stringify(twice.years[0].result.states),retained);
assert(canResumeStateBatch(twice));
const restored=await execute({resume:twice});assert.equal(restored.state_summary.succeeded,3);
failYear='2010';const failed=await execute();failYear=null;
assert.equal(failed.exit_code,1);assert(canResumeStateBatch(failed));assert.deepEqual(failed.years.map(r=>r.status),['completed','failed','completed']);
const beforeRetry=runs.length,retried=await execute({resume:failed});
assert.equal(retried.state_summary.succeeded,3);assert.deepEqual(runs.slice(beforeRetry),['2010']);
for(const index of [0,2])assert.deepEqual(retried.years[index].result.states,failed.years[index].result.states);
await fs.mkdir('target/state-run-years-resume',{recursive:true});
await fs.writeFile('target/state-run-years-resume/resumed.zip',await stateRunBatchZip(restored));
await fs.writeFile('target/state-run-years-resume/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,real_wasm:true,scope:'Retained synthetic RI jobs for three vintages; wrapper workers run actual WASM',resume_only_unfinished:true,retained_receipts_unchanged:true,fresh_native_selection_all_years:true,import_mutation_inputs_and_artifact_guards:true,repeated_cancellation:true,workers_closed:alive===0},null,2)+'\n');
assert.equal(alive,0);console.log('PASS multi-year session Resume: actual WASM, unfinished-only execution, unchanged outputs, fresh selections, repeat cancellation, import/tamper/input/artifact refusals, ZIP and closed workers');
