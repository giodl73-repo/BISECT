import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {runStateBatch,canResumeStateBatch,canVerifySavedStateBatch,stateRunBatchZip} from '../../web/lab/state-run-batch.js';
import {executeStateSourceJob} from '../../web/lab/state-source-job.js';

const root='target/state-run-batch-recovery';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/state-run-batch/browser.bisect','utf8')),inputs=fixture.files.stateRunBatchInputs,selection=fixture.files.stateBatchInputs;
let alive=0;const executions=[];
class OwnedWorker extends EventTarget{
 constructor(artifact=engine.artifactIdentity){super();this.artifact=artifact;alive++;this.closed=false;}
 postMessage(m){setTimeout(()=>{if(this.closed)return;try{if(m.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:this.artifact}}));return;}if(['run-tui-configured','run-census-state-job'].includes(m.request.operation))executions.push(m.request.input.form.location);const result=m.request.operation==='run-census-state-job'?executeStateSourceJob(engine,m.request.input):engine.execute(m.request);this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:m.id,result}}));}catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:error.message,source_preparation:error.source_preparation}}));}},2);}
 terminate(){if(!this.closed){alive--;this.closed=true;}}
}
const run=(options={},v=inputs,s=selection)=>runStateBatch(v,s,{createWorker:()=>new OwnedWorker(),wasmUrl:'owned',hardwareConcurrency:2,...options});
const partial=()=>({...structuredClone(fixture.result),status:'cancelled',states:structuredClone(fixture.result.states.slice(0,1)),summary:{total:3,completed:1,succeeded:1,failed:0}});
const saved=partial();assert(canVerifySavedStateBatch(saved));assert(!canResumeStateBatch(saved));let before=executions.length;
const verified=await run({verifySaved:saved});assert.deepEqual(executions.slice(before),['IA']);assert.equal(verified.summary.completed,1);assert.equal(verified.status,'interrupted');assert(canResumeStateBatch(verified));assert(!canVerifySavedStateBatch(verified));assert.deepEqual(JSON.parse(JSON.stringify(verified.states)),saved.states);assert.equal(alive,0);
before=executions.length;const resumed=await run({resume:verified});assert.deepEqual(executions.slice(before),['RI','VT']);assert.deepEqual(JSON.parse(JSON.stringify(resumed.states)),fixture.result.states);assert.equal(alive,0);await fs.writeFile(root+'/resumed.zip',await stateRunBatchZip(resumed));
await fs.writeFile(root+'/partial.bisect',JSON.stringify({...fixture,result:saved}));

const rejectBeforeExecution=async(candidate,options={},v=inputs,s=selection,pattern=/checkpoint|changed|artifact|Choose|accessor/)=>{const count=executions.length;await assert.rejects(()=>run({verifySaved:candidate,...options},v,s),pattern);assert.equal(executions.length,count);assert.equal(alive,0);};
for(const change of [v=>v.extra=true,v=>v.workers=null,v=>v.workers.effective=99,v=>v.execution_performed=false,v=>v.states.push(structuredClone(v.states[0])),v=>v.summary.succeeded=2,v=>v.scheduled_states.push('ZZ'),v=>v.states[0].state_code='ZZ',v=>v.runtime_artifact.extra=true,v=>v.request_sha256='0'.repeat(64),v=>v.jobs_source_sha256='0'.repeat(64),v=>v.jobs_source_name='other.json',v=>v.selection.states[0].districts++]){const c=partial();change(c);await rejectBeforeExecution(c);}
await rejectBeforeExecution(partial(),{}, {...inputs,workers:1});await rejectBeforeExecution(partial(),{resume:verified});
await rejectBeforeExecution(partial(),{createWorker:()=>new OwnedWorker({...engine.artifactIdentity,sha256:'0'.repeat(64)})});
const accessor=partial();let accesses=0;Object.defineProperty(accessor.states[0].result,'hostile',{enumerable:true,get(){accesses++;return 'must not execute';}});await rejectBeforeExecution(accessor);assert.equal(accesses,0);

for(const change of [v=>v.states[0].result.form.seed='0',v=>v.states[0].result.practitioner_export.document_json+=' ',v=>v.states[0].result.report_manifest.manifest_json+=' ']){const candidate=partial();change(candidate);let latest;await assert.rejects(()=>run({verifySaved:candidate,onCheckpoint:c=>latest=c}),/does not match fresh native/);assert.equal(latest.summary.completed,0);assert.equal(alive,0);}
// Recomputed public hashes still cannot authorize a forged original receipt.
const changedJobs=JSON.parse(Buffer.from(inputs.jobs.base64,'base64'));changedJobs.find(job=>job.form.location==='IA').form.label='changed_IA';const changedInputs={...inputs,jobs:{name:inputs.jobs.name,base64:Buffer.from(JSON.stringify(changedJobs)).toString('base64')}},forged=partial();const digest=async text=>Buffer.from(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))).toString('hex');forged.request_sha256=await digest(JSON.stringify({inputs:changedInputs,selection}));forged.jobs_source_sha256=await digest(Buffer.from(changedInputs.jobs.base64,'base64').toString());await assert.rejects(()=>run({verifySaved:forged},changedInputs),/does not match fresh native/);assert.equal(alive,0);

// Earlier verified states survive a later mismatch or explicit cancellation.
const badLater=structuredClone(fixture.result);badLater.states[1].result.form.seed='0';let retained;await assert.rejects(()=>run({verifySaved:badLater,onCheckpoint:c=>retained=c}),/does not match fresh native/);assert.equal(retained.summary.succeeded,1);retained.status='interrupted';assert(canResumeStateBatch(retained));assert.deepEqual(JSON.parse(JSON.stringify(retained.states)),fixture.result.states.slice(0,1));
let cancelled;const controller=new AbortController();await assert.rejects(()=>run({verifySaved:fixture.result,signal:controller.signal,onCheckpoint:c=>{cancelled=c;if(c.summary.completed===1)controller.abort(new Error('Verification cancelled'));}}),/Verification cancelled/);cancelled.status='cancelled';assert(canResumeStateBatch(cancelled));assert.equal(alive,0);
const complete=await run({verifySaved:fixture.result});assert.equal(complete.status,'completed');assert(!canResumeStateBatch(complete));assert.deepEqual(JSON.parse(JSON.stringify(complete.states)),fixture.result.states);

// Verification also reproduces raw-source preparation and every selected native
// analysis/map/report output, rather than trusting the imported prepared files.
const rawFixture=JSON.parse(await fs.readFile('target/state-source-batch/browser.bisect','utf8'));
const raw=await run({verifySaved:rawFixture.result},rawFixture.files.stateRunBatchInputs,rawFixture.files.stateBatchInputs);assert.deepEqual(JSON.parse(JSON.stringify(raw.states)),rawFixture.result.states);assert.equal(alive,0);await fs.writeFile(root+'/source-verified.zip',await stateRunBatchZip(raw));
const changedRaw=structuredClone(rawFixture.result);changedRaw.states[0].source_preparation.geometry_json+=' ';await assert.rejects(()=>run({verifySaved:changedRaw},rawFixture.files.stateRunBatchInputs,rawFixture.files.stateBatchInputs),/does not match fresh native/);assert.equal(alive,0);
if(process.argv.includes('--real')){const original=JSON.parse(await fs.readFile('target/state-source-batch/real-ri.bisect','utf8')),real=await run({verifySaved:original.result},original.files.stateRunBatchInputs,original.files.stateBatchInputs);assert.equal(real.states[0].source_preparation.units,792);assert.deepEqual(JSON.parse(JSON.stringify(real.states)),original.result.states);assert.equal(alive,0);await fs.writeFile(root+'/original-ri-verified.zip',await stateRunBatchZip(real));console.log('Original RI recovery: fresh 792-unit source preparation and complete native Run/analysis/map/report bytes match the saved original-source result.');}
await fs.writeFile(root+'/evidence.json',JSON.stringify({saved_partial_verified_then_resumed:true,verification_only_executes_saved_successes:true,full_native_verified_receipt_equality:true,identity_schema_artifact_and_full_output_guards:true,raw_Census_preparation_and_followup_replayed:true,original_RI_792_unit_replay:process.argv.includes('--real'),cancellation_and_later_mismatch_retain_verified_states:true,owned_worker_cleanup:true},null,2));
console.log('Saved State batch recovery: full native replay before reuse, only saved successes verified, explicit pending Resume, exact receipts/ZIPs, source preparation/workflow replay, identity/mutation/schema/accessor guards and cancellation retention passed.');
