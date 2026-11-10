import {stateRunJobs,validateStateRunBatchInputs,validateSavedWorkspaceBatch} from './state-run-batch.js';
import {savedYears} from './state-run-years.js';
import {stateBatchRequest} from './state-batch-files.js';
import {stateCommandRequest} from './state-command-files.js';
import {nativeRunStateFilters} from './native-run-policy.js';
import {nativeTargetHash} from './native-state-target-authority.js';
import {executeNativeStateJob} from './native-state-job.js';
import {executeNativeStateOutput,nativeStateOutputSource} from './native-state-output.js';
import {executeNativeOutputVerification,nativeOutputVerificationRequest} from './native-output-verification.js';
import {encodeBytes} from './package-files.js';

const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
const sourceText=s=>new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0)));
const textHash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export function hasNativeWorkspaceCheckpoint(value){return value?.schema_version==='bisect-state-run-batch-v1'?!!value.native_workspace:value?.schema_version==='bisect-state-run-years-v1'&&Array.isArray(value.years)&&value.years.some(row=>row?.result?.native_workspace);}
export function nativeWorkspaceChildren(value){return value.schema_version==='bisect-state-run-years-v1'?value.years.filter(row=>row.result).map(row=>({year:row.year,checkpoint:row.result})):[{year:value.selection.year,checkpoint:value}];}

// Pure worker verification. No folder handles, target actions or installation
// enter this boundary. Completed runs are explicitly recomputed once, then the
// composed no-partition verifier checks their source/manifest/graph bindings.
export async function verifyNativeBulkCheckpoint(engine,input,{signal,onProgress=()=>{}}={}){
 if(!input||Object.keys(input).sort().join()!=='checkpoint,inputs,selection')throw new Error('Select exact native workspace inputs and checkpoint.');
 const {inputs,selection}=structuredClone(input);validateStateRunBatchInputs(inputs);
 if(!inputs.run_command&&!inputs.bulk_command||selection.manifest)throw new Error('Native workspace continuation requires its original bulk policy and embedded allocation.');
 const checkpoint=inputs.all_years?savedYears(input.checkpoint,validateSavedWorkspaceBatch):validateSavedWorkspaceBatch(input.checkpoint);
 const request_sha256=await nativeTargetHash({inputs,selection});
 if(checkpoint.request_sha256!==request_sha256||!same(checkpoint.runtime_artifact,engine.artifactIdentity))throw new Error('Workspace inputs or loaded WASM changed. Select the original inputs and engine.');
 const jobs=stateRunJobs(inputs.jobs,inputs.all_years?153:51),run=!!inputs.run_command,policy=inputs.run_command??inputs.bulk_command;
 const review=engine.execute(stateCommandRequest(policy,{run,bulk:!run}));
 if(review.status!=='parsed'||review.print_only)throw new Error('Review executable native bulk flags before continuation.');
 if(inputs.all_years&&!same(checkpoint.native_run_policy,inputs.run_command))throw new Error('Saved native Run policy differs.');
 const completed=[];
 for(const {year,checkpoint:child} of nativeWorkspaceChildren(checkpoint)){
  signal?.throwIfAborted();
  const childInputs=inputs.all_years?{workers:inputs.workers,run_command:structuredClone(inputs.run_command),jobs:{name:'prepared-'+year+'-jobs.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(jobs.filter(job=>job.scope.year===year))))}}:inputs;
  const childSelection={...selection,...(inputs.all_years?{year}:{})},expectedRequest=await nativeTargetHash({inputs:childInputs,selection:childSelection,...(child.missing_job_policy?{missing_job_policy:child.missing_job_policy}:{})});
  if(child.request_sha256!==expectedRequest||child.jobs_source_name!==childInputs.jobs.name||child.jobs_source_sha256!==await textHash(sourceText(childInputs.jobs))||!same(child.runtime_artifact,engine.artifactIdentity)||!same(child.native_run_command??child.native_states_command,review))throw new Error('Saved year source, request or native policy changed: '+year);
  const workspace=child.native_workspace;
  if(workspace.schema_version!=='bisect-native-bulk-workspace-record-v1'||workspace.request_sha256!==request_sha256||workspace.year!==year||!same(workspace.review,review)||workspace.installation_performed!==false||!Array.isArray(workspace.targets)||!Array.isArray(workspace.actions)||workspace.actions.length>153||workspace.previous_actions!==undefined&&(!Array.isArray(workspace.previous_actions)||workspace.previous_actions.length>1000))throw new Error('Invalid passive native workspace record.');
  const base={...selection,year,states:run?nativeRunStateFilters(review):review.states,manifest:null};delete base.completion;
  const all=engine.execute(stateBatchRequest(base)),selected=child.selection,scheduled=child.scheduled_states;
  if(!same(selected.requested_states,all.requested_states)||selected.manifest_sha256!==all.manifest_sha256||selected.manifest_source!==all.manifest_source||selected.year!==year||workspace.targets.length!==all.states.length)throw new Error('Saved native allocation differs.');
  const exact=scheduled.length?engine.execute(stateBatchRequest({...base,states:scheduled})):{states:[],selection_json:'[]',selection_sha256:await textHash('[]')};
  if(!same(selected.states,exact.states)||!same(selected.states.map(row=>row.state_code),scheduled)||selected.selection_json!==exact.selection_json||selected.selection_sha256!==exact.selection_sha256)throw new Error('Saved scheduled state allocation differs.');
  for(const [index,target]of workspace.targets.entries()){
   const row=all.states[index];if(!same(target.scope,{state:row.state_code,year,districts:row.districts,chamber:'congressional'})||target.target_namespace!==review.version+'/'+year+'/states/'+row.state_slug||target.scheduled!==scheduled.includes(row.state_code)||target.has_job!==jobs.some(job=>job.scope.state===row.state_code&&job.scope.year===year))throw new Error('Saved workspace target scope differs.');
  }
  for(const row of child.states.filter(row=>row.status==='ok')){
   signal?.throwIfAborted();const selectedRow=selected.states.find(value=>value.state_code===row.state_code),job=structuredClone(jobs.find(value=>value.scope.state===row.state_code&&value.scope.year===year));
   if(!job?.native_state?.source_inputs||!selectedRow)throw new Error('Missing original source for retained state.');
   if(job.scope.districts===null)job.scope.districts=selectedRow.districts;
   onProgress({stage:'workspace-checkpoint-replay',state:row.state_code,year,message:'Explicitly replaying retained result without target actions'});
   const replay=await executeNativeStateJob(engine,job,{onProgress:progress=>onProgress({...progress,state:row.state_code,year})}),actual={state_code:row.state_code,status:'ok',result:replay.result,source_preparation:replay.source_preparation},expected=structuredClone(row);
   // This sole field is a passive filesystem record, never replayed authority.
   const native=expected.result?.native_run??expected.result?.native_states;if(native)delete native.target_actions;
   if(!same(actual,expected))throw new Error('Saved result differs from original-source native replay: '+row.state_code+'/'+year);
   const output=await executeNativeStateOutput(engine,{source:nativeStateOutputSource(row.result),version:review.version,layout:'state'}),packageInputs={manifest:{name:'manifest.json',base64:output.files_base64['manifest.json']},files:Object.fromEntries(Object.entries(output.files_base64).filter(([name])=>name!=='manifest.json').map(([name,base64])=>[name,{name:name.split('/').at(-1),base64}]))};
   const inspection=await executeNativeOutputVerification(engine,nativeOutputVerificationRequest({packageVerificationInputs:packageInputs}).input);
   if(inspection.status!=='inspection-passed'||inspection.execution_performed||inspection.stages.original_sources.status!=='verified')throw new Error('Retained source/graph/manifest inspection is incomplete: '+row.state_code+'/'+year);
   signal?.throwIfAborted();completed.push({year,state:row.state_code,row_sha256:await nativeTargetHash(row),output,inspection});
  }
 }
 signal?.throwIfAborted();
 return {schema_version:'bisect-native-workspace-checkpoint-verification-v1',status:'verified',checkpoint_sha256:await nativeTargetHash(checkpoint),request_sha256,runtime_artifact:engine.artifactIdentity,completed,filesystem_actions_performed:false,network_requests_performed:false,source_authenticity_verified:false,scope:'Explicit original-source completed-state replay and composed source/manifest inspection. Historical target-action records are passive. Live folder review and continuation remain separate; no reset or installation authority is serialized.'};
}
