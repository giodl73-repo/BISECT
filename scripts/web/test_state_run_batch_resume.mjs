import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {runStateBatch,canResumeStateBatch,stateRunBatchZip} from '../../web/lab/state-run-batch.js';
import {executeStateSourceJob} from '../../web/lab/state-source-job.js';

const root='target/state-run-batch-resume';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/state-run-batch/browser.bisect','utf8'));
const inputs=fixture.files.stateRunBatchInputs,selection=fixture.files.stateBatchInputs;
let alive=0;const executions=[],sourcePreparations=[];
class OwnedWorker extends EventTarget{
 constructor(artifact=engine.artifactIdentity){super();this.artifact=artifact;this.closed=false;alive++;}
 postMessage(message){setTimeout(()=>{if(this.closed)return;try{if(message.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:this.artifact}}));return;}if(['run-tui-configured','run-census-state-job'].includes(message.request.operation))executions.push(message.request.input.form.location);let result;if(message.request.operation==='run-census-state-job'){sourcePreparations.push(message.request.input.form.location);result=executeStateSourceJob(engine,message.request.input);}else result=engine.execute(message.request);this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));}catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message,source_preparation:error.source_preparation}}));}},2);}
 terminate(){if(!this.closed){alive--;this.closed=true;}}
}
const run=(options={},v=inputs,s=selection)=>runStateBatch(v,s,{createWorker:()=>new OwnedWorker(),wasmUrl:'owned',hardwareConcurrency:1,...options});
async function cancelAfterOne(v=inputs){let checkpoint;const controller=new AbortController();await assert.rejects(()=>run({signal:controller.signal,onCheckpoint:next=>{checkpoint=next;if(next.summary.completed===1)controller.abort(new Error('Intentional cancellation'));}},v),/Intentional cancellation/);assert.equal(alive,0);checkpoint.status='cancelled';assert(canResumeStateBatch(checkpoint));return checkpoint;}

const checkpoint=await cancelAfterOne(),retained=structuredClone(checkpoint.states[0]),before=executions.length;
const resumed=await run({resume:checkpoint});assert.equal(alive,0);assert.equal(resumed.summary.succeeded,3);assert.deepEqual(JSON.parse(JSON.stringify(resumed.states)),fixture.result.states);assert.deepEqual(resumed.states.find(row=>row.state_code===retained.state_code),retained);assert.equal(executions.length-before,2);assert(!executions.slice(before).includes(retained.state_code));assert.equal(resumed.workers.effective,1);assert(!canResumeStateBatch(resumed));
await fs.writeFile(root+'/resumed.zip',await stateRunBatchZip(resumed));

// Imported/copied receipts cannot authorize execution reuse, even with exact
// hashes. Each mutation is tested against a genuine live capability.
const rejected=async(checkpoint,options={},v=inputs,s=selection,pattern=/checkpoint|changed|Imported|fresh/i)=>{const before=executions.length;await assert.rejects(()=>run({resume:checkpoint,...options},v,s),pattern);assert.equal(executions.length,before);assert.equal(alive,0);};
assert(!canResumeStateBatch(structuredClone(checkpoint)));await rejected(structuredClone(checkpoint));
await rejected(checkpoint,{}, {...inputs,workers:2});await rejected(checkpoint,{},inputs,{...selection,states:['VT']});
const artifactChanged={...engine.artifactIdentity,sha256:'0'.repeat(64)};await rejected(checkpoint,{createWorker:()=>new OwnedWorker(artifactChanged)},inputs,selection,/artifact/i);
for(const change of [c=>c.states[0].result.form.seed='0',c=>{const a=c.states[0].result.configured_run.result.assignments;a[Object.keys(a)[0]]=99;},c=>c.states[0].state_code='VT',c=>c.runtime_artifact.sha256='0'.repeat(64),c=>c.selection.states[0].districts++,c=>c.scheduled_states.push('ZZ'),c=>c.jobs_source_name='changed.json']){
 const fresh=await cancelAfterOne();change(fresh);await rejected(fresh);
}

const accessorCheckpoint=await cancelAfterOne();let getterCalls=0;Object.defineProperty(accessorCheckpoint.states[0].result,'hostile',{enumerable:true,get(){getterCalls++;return 'must not execute';}});await rejected(accessorCheckpoint,{},inputs,selection,/accessor/);assert.equal(getterCalls,0);
const nonfiniteCheckpoint=await cancelAfterOne();nonfiniteCheckpoint.states[0].result.extra=NaN;await rejected(nonfiniteCheckpoint,{},inputs,selection,/number/);

// Checkpoint callbacks cannot mutate the scheduler's internal native receipts.
let initial;const isolated=await run({onCheckpoint:c=>{if(c.summary.completed===1){initial=c;c.status='cancelled';assert(!canResumeStateBatch(c));c.states[0].result.form.seed='not-a-seed';c.selection.states[0].districts=999;}}});assert.deepEqual(JSON.parse(JSON.stringify(isolated.states)),fixture.result.states);assert.notEqual(isolated.selection.states[0].districts,999);assert.equal(alive,0);

// A second cancellation can itself be resumed; only pending states execute.
const twice=await cancelAfterOne();let second;const controller=new AbortController();await assert.rejects(()=>run({resume:twice,signal:controller.signal,onCheckpoint:c=>{second=c;if(c.summary.completed===2)controller.abort(new Error('Second cancellation'));}}),/Second cancellation/);second.status='cancelled';const lastBefore=executions.length,finished=await run({resume:second});assert.equal(executions.length-lastBefore,1);assert.deepEqual(JSON.parse(JSON.stringify(finished.states)),fixture.result.states);assert.equal(alive,0);

// Completed batches with a failed state retry that state, preserving successes.
const badJobs=JSON.parse(Buffer.from(inputs.jobs.base64,'base64').toString());badJobs.find(j=>j.form.location==='RI').target={source_name:'incomplete.json',folder_json:JSON.stringify({version:'v1',paths:['2020/plans/batch_RI/manifest.tmp'],manifests:{}}),force:true};const badInputs={...inputs,jobs:{name:'failed-jobs.json',base64:Buffer.from(JSON.stringify(badJobs)).toString('base64')}};
const failed=await run({},badInputs);assert.equal(failed.summary.failed,1);assert(canResumeStateBatch(failed));const failedBefore=executions.length,retried=await run({resume:failed},badInputs);assert.deepEqual(executions.slice(failedBefore),['RI']);assert.deepEqual(retried.states,failed.states);assert.equal(alive,0);
// A completed raw-Census state keeps exact prepared files and native follow-up
// outputs; resume neither extracts nor prepares that state again.
const sourceFixture=JSON.parse(await fs.readFile('target/state-source-batch/browser.bisect','utf8')),sourceJob=JSON.parse(Buffer.from(sourceFixture.files.stateRunBatchInputs.jobs.base64,'base64'))[0],vt=JSON.parse(Buffer.from(inputs.jobs.base64,'base64')).find(j=>j.form.location==='VT'),sourceJobs=[sourceJob,vt],sourceInputs={jobs:{name:'mixed-census-prepared.json',base64:Buffer.from(JSON.stringify(sourceJobs)).toString('base64')},workers:1},sourceSelection={...sourceFixture.files.stateBatchInputs,states:['RI','VT']};
let sourceCheckpoint;const sourceController=new AbortController();await assert.rejects(()=>run({signal:sourceController.signal,onCheckpoint:c=>{sourceCheckpoint=c;if(c.summary.completed===1)sourceController.abort(new Error('Source state received'));}},sourceInputs,sourceSelection),/Source state received/);sourceCheckpoint.status='cancelled';assert.equal(sourceCheckpoint.states[0].state_code,'RI');assert.deepEqual(JSON.parse(JSON.stringify(sourceCheckpoint.states[0])),sourceFixture.result.states[0]);const preparationBefore=sourcePreparations.length,sourceRunBefore=executions.length,sourceResumed=await run({resume:sourceCheckpoint},sourceInputs,sourceSelection);assert.equal(sourcePreparations.length,preparationBefore);assert.deepEqual(executions.slice(sourceRunBefore),['VT']);assert.deepEqual(sourceResumed.states[0],sourceCheckpoint.states[0]);assert.equal(sourceResumed.summary.succeeded,2);assert.equal(alive,0);await fs.writeFile(root+'/source-resumed.zip',await stateRunBatchZip(sourceResumed));
const changedSource=structuredClone(sourceJobs);changedSource[0].source_preparation.projection.name='changed.prj';await rejected(sourceCheckpoint,{}, {...sourceInputs,jobs:{name:sourceInputs.jobs.name,base64:Buffer.from(JSON.stringify(changedSource)).toString('base64')}},sourceSelection);
sourceCheckpoint.states[0].source_preparation.geometry_json+=' ';await rejected(sourceCheckpoint,{},sourceInputs,sourceSelection);

await fs.writeFile(root+'/evidence.json',JSON.stringify({completed_native_receipts:3,resumed_without_reexecuting_completed_states:true,chained_cancellation:true,failed_state_retry:true,imported_checkpoint_rejected:true,input_artifact_selection_output_identity_guards:true,callback_isolation:true,exact_Census_preparation_and_followup_retained_without_reexecution:true,changed_Census_source_or_prepared_output_rejected:true,owned_worker_cleanup:true},null,2));
console.log('State batch Resume: exact native-verified receipts, completed-state reuse, chained cancellation, failure retry, imported/mutated/input/artifact rejection, callback isolation, exact raw-Census preparation/follow-up reuse and worker cleanup passed.');
