import {acquireStateSourceJob,stateSourceAcquisitionChoices,validateStateSourceAcquisition,validateStateSourceExecution,stateSourceJobFromAcquired} from './state-source-acquisition.js';
import {appendStateRunJob,stateRunJobs,validateStateRunBatchInputs} from './state-run-batch.js';
import {stateJobScope} from './state-source-job.js';
import {preflightStateSourceFlags} from './state-source-preflight.js';
import {nativeTargetHash} from './native-state-target-authority.js';
import {readCensusArchive} from './census-archive.js';
import {fetchPlanRequest} from './fetch-plan-files.js';
import {nativeToolSession} from './native-state-output-ui.js';
import {validateJsonTree,MAX_WORKBENCH_ARCHIVE_BYTES} from './project.js';

const live=new WeakMap(),id=scope=>scope.state+'/'+scope.year;
export const isStateSourceAcquisitionCheckpoint=v=>v?.schema_version==='bisect-state-source-batch-acquisition-v2';
export function canResumeStateSourceAcquisition(v){const proof=v&&live.get(v);return !!proof?.session.closed&&['completed','cancelled','interrupted'].includes(v.status)&&v.summary.queued<v.summary.total;}
export function canVerifyStateSourceAcquisition(v){return isStateSourceAcquisitionCheckpoint(v)&&!live.has(v)&&Array.isArray(v.states)&&v.states.some(row=>row?.status==='queued');}
function context(files,runSettings,formSettings,workflowSettings){const {stateRunBatchInputs,stateBatchInputs,...sources}=files;return {files:sources,runSettings,formSettings,workflowSettings};}
function policy(inputs){const {jobs,...settings}=inputs;return settings;}
const jobFor=(jobs,scope)=>jobs.find(job=>{const s=stateJobScope(job);return s.state===scope.state&&s.year===scope.year;});
function validate(value){
 validateJsonTree(value,4000000,MAX_WORKBENCH_ARCHIVE_BYTES);
 const keys=['schema_version','status','year','manifest_sha256','request_sha256','plan','inputs','states','summary','execution_performed'];
 if(!isStateSourceAcquisitionCheckpoint(value)||Object.keys(value).some(k=>![...keys,'source_verification'].includes(k))||keys.some(k=>!Object.hasOwn(value,k))||!['acquiring','completed','cancelled','interrupted'].includes(value.status)||value.execution_performed!==false||!Array.isArray(value.states)||value.states.length>50||!/^[a-f0-9]{64}$/.test(value.request_sha256??'')||value.plan?.manifest_sha256!==value.manifest_sha256)throw new Error('Invalid saved state-source acquisition checkpoint.');
 validateStateRunBatchInputs(value.inputs);const scopes=stateSourceAcquisitionChoices(value.plan).filter(s=>s.year===value.year),seen=new Set();
 for(const row of value.states){if(!row||Object.keys(row).some(k=>!['state','year','status','job_sha256','error'].includes(k))||!scopes.some(s=>id(s)===id(row))||seen.has(id(row))||!['queued','failed'].includes(row.status)||row.status==='queued'&&!/^[a-f0-9]{64}$/.test(row.job_sha256??'')||row.status==='failed'&&(typeof row.error!=='string'||row.error.length>4096))throw new Error('Invalid acquisition state row.');seen.add(id(row));}
 const summary={total:scopes.length,completed:value.states.length,queued:value.states.filter(r=>r.status==='queued').length,failed:value.states.filter(r=>r.status==='failed').length};
 if(!scopes.length||Object.keys(value.summary??{}).sort().join()!==Object.keys(summary).sort().join()||Object.keys(summary).some(k=>value.summary[k]!==summary[k]))throw new Error('Acquisition summary differs from its state rows.');return value;
}
async function requestHash(snapshot){return nativeTargetHash({plan:snapshot.plan,year:snapshot.year,context:context(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings),queue_policy:policy(snapshot.inputs)});}
async function remember(value,session){live.set(value,{session,digest:await nativeTargetHash(value),inputs:await nativeTargetHash(value.inputs)});return value;}

export async function acquireRecoverableStateSourceBatch(plan,year,inputs,files,runSettings,formSettings,workflowSettings,{resume=null,onCheckpoint=()=>{},onProgress=()=>{},signal,...options}={}){
 const snapshot=structuredClone({plan,year,inputs:validateStateRunBatchInputs(inputs),files,runSettings,formSettings,workflowSettings});
 const hash=await requestHash(snapshot),scopes=stateSourceAcquisitionChoices(snapshot.plan).filter(s=>s.year===year);if(!scopes.length)throw new Error('Choose supported state pairs from a fresh native Fetch plan.');
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);let rows=[],queued=snapshot.inputs;
 if(resume){const proof=live.get(resume);if(!proof||!canResumeStateSourceAcquisition(resume))throw new Error('Acquisition Resume needs a live checkpoint or explicit saved-source verification. Open never grants Resume authority.');validate(resume);if(resume.request_sha256!==hash||proof.digest!==await nativeTargetHash(resume)||proof.inputs!==await nativeTargetHash(inputs))throw new Error('Acquisition inputs, retained jobs or execution settings changed. Review sources again.');rows=structuredClone(resume.states.filter(r=>r.status==='queued'));}
 const inspectNativeCommand=await preflightStateSourceFlags(snapshot.files,{...options,signal}),session={closed:false};
 const publish=async status=>{
  const value={schema_version:'bisect-state-source-batch-acquisition-v2',status,year,manifest_sha256:plan.manifest_sha256,request_sha256:hash,plan:structuredClone(snapshot.plan),inputs:structuredClone(queued),states:structuredClone(rows),summary:{total:scopes.length,completed:rows.length,queued:rows.filter(r=>r.status==='queued').length,failed:rows.filter(r=>r.status==='failed').length},execution_performed:false};
  validate(value);await remember(value,session);onCheckpoint(value);return value;
 };
 try{
  await publish('acquiring');
  for(const scope of scopes){
   if(signal?.aborted)return await publish('cancelled');if(rows.some(r=>id(r)===id(scope)&&r.status==='queued'))continue;
   try{const job=await acquireStateSourceJob(snapshot.plan,scope,snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings,{...options,...(inspectNativeCommand?{inspectNativeCommand}:{}),signal,onProgress});if(signal?.aborted)return await publish('cancelled');if(!job.acquisition)throw new Error('Recoverable acquisition requires retained original archives.');const next=appendStateRunJob(queued,job),digest=await nativeTargetHash(job);signal?.throwIfAborted();queued=next;rows.push({...scope,status:'queued',job_sha256:digest});}
   catch(error){if(signal?.aborted)return await publish('cancelled');rows.push({...scope,status:'failed',error:String(error.message??error).slice(0,4096)});}
   await publish('acquiring');
  }
  return await publish('completed');
 }finally{session.closed=true;}
}

// Explicit source consistency verification, not historical execution or source
// authenticity. Decode retained archives and rebuild the exact queued jobs.
export async function verifySavedStateSourceAcquisition(value,inputs,files,runSettings,formSettings,workflowSettings,{inspectNativePlan,expectedArtifact,archiveReader=readCensusArchive,signal,onProgress=()=>{},...options}={}){
 const saved=structuredClone(validate(value)),snapshot=structuredClone({plan:saved.plan,year:saved.year,inputs,files,runSettings,formSettings,workflowSettings});
 if(saved.request_sha256!==await requestHash(snapshot)||await nativeTargetHash(inputs)!==await nativeTargetHash(saved.inputs))throw new Error('Saved acquisition inputs or execution settings changed.');
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);let tool;
 try{
  if(!inspectNativePlan){if(!expectedArtifact)throw new Error('Load the engine before verifying saved acquisition sources.');tool=nativeToolSession(expectedArtifact,signal);inspectNativePlan=r=>tool.execute(r);}
  await preflightStateSourceFlags(snapshot.files,{...options,expectedArtifact,signal});
  const fresh=await inspectNativePlan(fetchPlanRequest(snapshot.files.fetchPlanInputs??{source:null}));if(JSON.stringify(fresh)!==JSON.stringify(saved.plan))throw new Error('Saved acquisition plan differs from fresh native Fetch planning. No fetch performed.');
  const jobs=stateRunJobs(saved.inputs.jobs,saved.inputs.all_years?153:51),checked=[];
  for(const row of saved.states.filter(r=>r.status==='queued')){
   signal?.throwIfAborted();const job=jobFor(jobs,row);if(!job||await nativeTargetHash(job)!==row.job_sha256)throw new Error('Retained acquired job changed for '+id(row)+'.');
   const evidence=validateStateSourceAcquisition(job.acquisition);if(evidence.manifest_sha256!==saved.manifest_sha256||evidence.receipts[0].state!==row.state||evidence.receipts[0].year!==row.year)throw new Error('Retained acquisition scope differs from its checkpoint.');
   const acquired=[];
   for(let i=0;i<2;i++){
    const raw=evidence.archives[i],receipt=evidence.receipts[i],bytes=Uint8Array.from(atob(raw.source.base64),c=>c.charCodeAt(0)),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
    if(hash!==receipt.source_sha256)throw new Error('Retained archive byte hash differs for '+id(row)+'/'+raw.kind+'.');
    const planned=saved.plan.items.filter(item=>item.state_code===row.state&&item.year===row.year&&item.kind===raw.kind);if(planned.length!==1||planned[0].url!==raw.url)throw new Error('Retained archive differs from the native source plan.');
    onProgress({stage:'verify-retained-archive',state:row.state,year:row.year,kind:raw.kind});
    const archive=await archiveReader(new File([bytes],raw.source.name),{kind:raw.kind==='tiger'?'tiger':'pl94',year:Number(row.year),...(receipt.archive_profile?{archive_profile:receipt.archive_profile}:{})},{signal});signal?.throwIfAborted();if(archive.archive_sha256!==hash)throw new Error('Retained archive changed during extraction.');acquired.push({receipt,raw_archive_source:raw,inputs:{archive}});
   }
   const rebuilt=await stateSourceJobFromAcquired({...snapshot,scope:{state:row.state,year:row.year}},acquired);if(await nativeTargetHash(rebuilt)!==row.job_sha256)throw new Error('Retained job differs from its decoded original archives and current execution settings.');
   checked.push({state:row.state,year:row.year,job_sha256:row.job_sha256,archive_sha256:evidence.receipts.map(r=>r.source_sha256)});
  }
  signal?.throwIfAborted();const result={...saved,source_verification:{schema_version:'bisect-state-acquisition-source-consistency-v1',jobs:checked,network_requests_performed:false,partition_execution_performed:false,source_authenticity_verified:false,native_inspection_artifact:expectedArtifact??null}};await remember(result,{closed:true});if(signal?.aborted){live.delete(result);signal.throwIfAborted();}return result;
 }finally{tool?.close();}
}
