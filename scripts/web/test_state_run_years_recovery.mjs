import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {runStateBatch,canResumeStateBatch,canVerifySavedStateBatch,stateRunBatchZip} from '../../web/lab/state-run-batch.js';
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
const saved=structuredClone(await cancelAfterFirst());assert(canVerifySavedStateBatch(saved));assert(!canResumeStateBatch(saved));
const before=runs.length,verified=await execute({verifySaved:saved});
assert.deepEqual(runs.slice(before),['2020']);assert.equal(verified.state_summary.succeeded,1);assert.equal(verified.status,'interrupted');assert(canResumeStateBatch(verified));assert(!canVerifySavedStateBatch(verified));
assert.deepEqual(verified.years[0].result.states,saved.years[0].result.states);
const beforeResume=runs.length,resumed=await execute({resume:verified});assert.deepEqual(runs.slice(beforeResume),['2010','2000']);assert.equal(resumed.state_summary.succeeded,3);
assert.equal(alive,0);
const complete=structuredClone(resumed),beforeComplete=runs.length,checked=await execute({verifySaved:complete});assert.equal(checked.status,'completed');assert.deepEqual(runs.slice(beforeComplete),['2020','2010','2000']);assert.equal(canResumeStateBatch(checked),false);
async function refused(value,options={},data=inputs){const count=runs.length;await assert.rejects(()=>execute({verifySaved:value,...options},data),/saved|Saved|Choose|accessor/);assert.equal(runs.length,count);assert.equal(alive,0);}
for(const change of [v=>v.extra=true,v=>v.years[0].result=false,v=>v.request_sha256='0'.repeat(64),v=>v.years.reverse(),v=>v.state_summary.succeeded++,v=>v.years[0].result.missing_job_policy='ignore',v=>v.years[0].result.runtime_artifact.sha256='0'.repeat(64)]){const value=structuredClone(complete);change(value);await refused(value);}
await refused(saved,{resume:verified});await refused(saved,{}, {...inputs,workers:inputs.workers===1?2:1});
const accessor=structuredClone(saved);let reads=0;Object.defineProperty(accessor.years[0],'hostile',{enumerable:true,get(){reads++;return true;}});await refused(accessor);assert.equal(reads,0);
const forged=structuredClone(complete);forged.years[1].result.states[0].result.prepared.result.metrics.max_deviation_percent+=1;
const beforeForged=runs.length,rejected=await execute({verifySaved:forged});assert.equal(rejected.status,'interrupted');assert.equal(rejected.state_summary.succeeded,1);assert.equal(rejected.years[1].result.states.length,0);assert.match(rejected.years[1].error,/does not match fresh native/);assert.deepEqual(runs.slice(beforeForged),['2020','2010']);assert(canResumeStateBatch(rejected));
const controller=new AbortController(),beforeCancelled=runs.length;
const interrupted=await execute({verifySaved:complete,signal:controller.signal,onCheckpoint:c=>{if(c.state_summary.succeeded===1)controller.abort(new Error('Cancel verification'));}});
assert.equal(interrupted.status,'cancelled');assert.equal(interrupted.state_summary.succeeded,1);assert.deepEqual(runs.slice(beforeCancelled),['2020']);assert(canResumeStateBatch(interrupted));assert.equal(alive,0);
wrongArtifact=true;const beforeArtifact=runs.length,badArtifact=await execute({verifySaved:complete});wrongArtifact=false;assert.equal(badArtifact.status,'interrupted');assert.equal(badArtifact.state_summary.succeeded,0);assert.equal(runs.length,beforeArtifact);assert.equal(alive,0);
const missingScope={...selection,states:['RI','VT']},missing=await execute({},inputs,missingScope);
assert.equal(missing.state_summary.failed,3);const beforeMissing=runs.length;
const checkedMissing=await execute({verifySaved:structuredClone(missing)},inputs,missingScope);
assert.equal(checkedMissing.state_summary.succeeded,3);assert.equal(checkedMissing.state_summary.failed,0);assert.deepEqual(runs.slice(beforeMissing),['2020','2010','2000']);assert(canResumeStateBatch(checkedMissing));
const beforeMissingResume=runs.length,missingRetried=await execute({resume:checkedMissing},inputs,missingScope);assert.equal(runs.length,beforeMissingResume);assert.equal(missingRetried.state_summary.failed,3);
const workflow=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8'));
const workflowChecked=await execute({verifySaved:workflow.result},workflow.files.stateRunBatchInputs,workflow.files.stateBatchInputs);
assert.equal(workflowChecked.state_summary.succeeded,3);
for(const [index,row] of workflowChecked.years.entries())assert.deepEqual(row.result.states,workflow.result.years[index].result.states);
await fs.mkdir('target/state-run-years-recovery',{recursive:true});await fs.writeFile('target/state-run-years-recovery/resumed.zip',await stateRunBatchZip(resumed));
await fs.writeFile('target/state-run-years-recovery/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,actual_wasm:true,scope:'Synthetic RI supplied graph jobs for 2020/2010/2000',verified_only_saved_successes:true,unfinished_waits_for_explicit_resume:true,complete_replay:true,forgery_stops_at_verified_prefix:true,cancellation_keeps_verified_prefix:true,schema_input_accessor_artifact_guards:true,workers_closed:true,three_configured_native_analysis_map_report_replays:true},null,2)+'\n');
console.log('PASS saved multi-year recovery: actual WASM replay, explicit Resume, complete output equality, edited results refused, verified-prefix retention, cancellation and closed workers');
