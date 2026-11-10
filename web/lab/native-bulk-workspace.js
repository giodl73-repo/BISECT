import {nativeWorkspaceChildren,hasNativeWorkspaceCheckpoint} from './native-bulk-checkpoint-verification.js';
import {stateRunJobs} from './state-run-batch.js';
import {stateBatchRequest} from './state-batch-files.js';
import {stateCommandArguments,stateCommandRequest} from './state-command-files.js';
import {nativeRunStateFilters} from './native-run-policy.js';
import {nativeStatePreparationScope} from './native-state-sources.js';
import {nativeTargetHash} from './native-state-target-authority.js';
import {nativeOutputComponent} from './native-state-output.js';
import {encodeBytes} from './package-files.js';
import {observeNativeOutputTarget,reviewNativeStateTargetActions,performNativeStateTargetActions} from './native-state-install.js';

const reviews=new WeakMap(),sessions=new WeakMap(),continuations=new WeakMap();
const key=scope=>scope.state+'/'+scope.year;
const commandHash=(inputs,selection)=>nativeTargetHash({inputs,selection});
function current(session){const state=sessions.get(session);if(!state||state.closed)throw new Error('Select and explicitly review a current native bulk workspace. Saved records grant no folder authority.');return state;}
const abort=signal=>signal?.throwIfAborted();
async function unchanged(state,target,{signal}={}){
 abort(signal);const canonical=await observeNativeOutputTarget(state.root,target.intent.target_namespace,{signal});
 const historical=target.historical_namespace===target.intent.target_namespace?canonical:await observeNativeOutputTarget(state.root,target.historical_namespace,{signal});
 if(canonical.sha256!==target.canonical_sha256||historical.sha256!==target.historical_sha256)throw new Error('Native bulk workspace changed after review for '+key(target.scope)+'. Review again before continuing.');
 return canonical;
}
export async function reviewNativeBulkWorkspace(execute,artifact,root,inputs,selection,{signal,close=()=>{},continuation=null}={}){
 if(typeof execute!=='function'||root?.kind!=='directory'||!artifact||!/^[a-f0-9]{64}$/.test(artifact.sha256))throw new Error('Choose a live native output root and loaded engine.');
 const recovery=continuation&&continuations.get(continuation);if(continuation&&!recovery)throw new Error('Verify this saved checkpoint through the current worker first.');
 const original=structuredClone({inputs,selection});inputs=original.inputs;selection=original.selection;
 const policy=inputs.run_command??inputs.bulk_command,run=!!inputs.run_command;
 if(!policy||selection.manifest)throw new Error('A reviewed native workspace needs an applied Run/States queue policy and embedded allocation.');
 const jobs=stateRunJobs(inputs.jobs,inputs.all_years?153:51),review=await execute(stateCommandRequest(policy,{run,bulk:!run}));
 if(review.status!=='parsed'||review.print_only||review.schema_version!==(run?'bisect-native-run-command-v1':'bisect-native-states-command-v1')||!run&&(!review.profile_filter?.allowed||!review.multiscale_levels?.allowed))throw new Error('Review executable native Run/States flags first.');
 if(!nativeOutputComponent(review.version))throw new Error('Choose a portable native output version.');
 for(const job of jobs){
  if(!job.native_state?.source_inputs||job.native_state.command_kind!==(run?'run':'states')||nativeStatePreparationScope(job.native_state.source_inputs.preparation).resolution!=='tract'||JSON.stringify(stateCommandArguments(job.native_state.command))!==JSON.stringify(stateCommandArguments(policy))||await nativeTargetHash(job.native_state)!==job.source_sha256)throw new Error('Apply the exact native bulk policy to unchanged original tract sources before reviewing outputs.');
 }
 const years=run?review.years:[review.year];if(!inputs.all_years&&years.length!==1||inputs.all_years&&!run)throw new Error('Match the applied native year policy to this workspace queue.');
 const state={root,execute,artifact:structuredClone(artifact),request_sha256:await commandHash(inputs,selection),review,years:new Map(),targets:new Map(),records:[],close,closed:false,recovery};
 for(const year of years){
  abort(signal);const base={...selection,year,states:run?nativeRunStateFilters(review):review.states,manifest:null};delete base.completion;
  const all=await execute(stateBatchRequest(base)),paths=[],targets=[];
  if(all.schema_version!=='bisect-state-batch-selection-v1'||all.year!==year||all.execution_performed!==false)throw new Error('Invalid native workspace selection.');
  for(const row of all.states){
   abort(signal);const job=jobs.find(j=>j.scope.state===row.state_code&&j.scope.year===year),scope={state:row.state_code,year,districts:row.districts,chamber:'congressional'};
   if(job?.scope.districts!==null&&job?.scope.districts!==undefined&&job.scope.districts!==row.districts)throw new Error('Queued source district count differs from native allocation.');
   const target=await execute({operation:'inspect-native-state-run-target',version:review.version,year,state:row.state_code,input:{source_name:'native-bulk-empty-target.json',folder_json:JSON.stringify({version:review.version,paths:[],manifests:{}}),force:false}});
   const namespace=target.target_namespace?.slice(8);if(target.schema_version!=='bisect-native-run-target-v1'||!target.target_namespace?.startsWith('outputs/')||!nativeOutputComponent(target.state_slug))throw new Error('Invalid native bulk target namespace.');
   const source_arguments_json=policy.arguments_json,binding={command_kind:run?'run':'states',command:policy,scope,source_sha256:job?.source_sha256??null};
   const intent={schema_version:'bisect-native-state-target-intent-v1',version:review.version,year,state:row.state_code,label:null,layout:'state',state_slug:target.state_slug,target_namespace:namespace,reset:run&&review.effective_configuration.reset===true,force:false,source_sha256:job?.source_sha256??null,request_sha256:await nativeTargetHash({binding,workspace_request_sha256:state.request_sha256}),binding_sha256:await nativeTargetHash(binding),runtime_artifact:artifact,source_arguments_json,engine_arguments_json:source_arguments_json,output_dir_intent:review.output_dir??null};
   const canonical=await observeNativeOutputTarget(root,namespace,{signal}),historical_namespace=review.version+'/'+year+'/states/'+row.state_code.toLowerCase(),historical=historical_namespace===namespace?canonical:await observeNativeOutputTarget(root,historical_namespace,{signal});
   for(const [name,snapshot]of [[namespace,canonical],[historical_namespace,historical]])for(const entry of snapshot.entries){paths.push(name.slice(review.version.length+1)+'/'+entry.path);if(paths.length>10000)throw new Error('Native workspace completion exceeds 10,000 observed paths in one year.');}
   const item={scope,intent,source_sha256:job?.source_sha256??null,historical_namespace,canonical_sha256:canonical.sha256,historical_sha256:historical.sha256,canonical,has_job:!!job};targets.push(item);state.targets.set(key(scope),item);
  }
  const completion={source:{name:'live-native-bulk-completion-'+year+'.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify({paths:[...new Set(paths)].sort()})))},reprocess:review.effective_configuration.reprocess===true};
  const selectedInputs={...base,completion},selected=await execute(stateBatchRequest(selectedInputs)),scheduled=new Set(selected.states.map(row=>row.state_code));
  const savedChild=recovery&&nativeWorkspaceChildren(recovery.checkpoint).find(row=>row.year===year)?.checkpoint;
  for(const target of targets){
   target.scheduled=scheduled.has(target.scope.state);target.review=null;
   if(savedChild){
    const code=target.scope.state,wasScheduled=savedChild.scheduled_states.includes(code),retained=savedChild.states.find(row=>row.state_code===code&&row.status==='ok');
    if(!wasScheduled&&target.scheduled)throw new Error('Previously skipped completion changed for '+code+'/'+year+'. Review a fresh Run.');
    if(retained){
     const verified=recovery.verification.completed.find(row=>row.year===year&&row.state===code);
     if(!verified||verified.row_sha256!==await nativeTargetHash(retained))throw new Error('Retained result differs from fresh workspace verification.');
     target.retained=structuredClone(retained);
     if(!target.scheduled){
      const historical=target.historical_namespace===target.intent.target_namespace?target.canonical:await observeNativeOutputTarget(root,target.historical_namespace,{signal});
      const snapshot=target.canonical.entries.some(e=>/^data\/final_assignments\.(json|pkl)$/.test(e.path))?target.canonical:historical;
      const actual=new Map(snapshot.entries.filter(e=>e.kind==='file').map(e=>[e.path,e.sha256]));
      if(Object.entries(verified.output.file_sha256).some(([path,hash])=>actual.get(path)!==hash))throw new Error('Live completed output differs from the verified retained package for '+code+'/'+year+'.');
      target.installed_bytes_verified=true;
     }
    }else if(wasScheduled&&!target.scheduled)target.review_error='A previously unfinished target now has completion markers but no verified retained result. Review its output separately before continuing.';
    target.scheduled=wasScheduled;
    const prior=[...(savedChild.native_workspace.previous_actions??[]),...savedChild.native_workspace.actions].some(action=>action?.intent?.target_namespace===target.intent.target_namespace||action?.receipt?.target_namespace===target.intent.target_namespace);
    if(prior&&!retained&&target.intent.reset){
     if(target.canonical.entries.some(e=>e.path==='data'))target.review_error='A prior action was recorded and data/ is present. A separate fresh reset/recovery review is required; continuation will not repeat removal.';
     else{target.intent.reset=false;target.reset_disposition='current-data-absent-no-repeat';}
    }
   }
   if(target.scheduled&&target.has_job&&!target.retained&&!target.review_error)try{target.review=await reviewNativeStateTargetActions(execute,root,target.intent,{signal});}catch(error){abort(signal);target.review_error=String(error.message??error);}
  }
  state.years.set(year,{selectionInputs:selectedInputs,selection:savedChild?structuredClone(savedChild.selection):selected,targets});
 }
 const display={schema_version:'bisect-native-bulk-workspace-review-v1',continuation:!!recovery,verified_retained_results:recovery?.verification.completed.length??0,directory_name:root.name??'selected outputs folder',version:review.version,reset:run&&review.effective_configuration.reset===true,reprocess:review.effective_configuration.reprocess===true,targets:[...state.targets.values()].map(t=>({scope:t.scope,target_namespace:t.intent.target_namespace,status:t.retained?'retained-verified-result':!t.scheduled?'skipped-complete':!t.has_job?'missing-source':t.review_error?'blocked-target':'scheduled',...(t.review_error?{error:t.review_error}:{}),entries_to_remove:t.review?.display.entries_to_remove??[]}))};
 const result={display:structuredClone(display)};reviews.set(result,state);return result;
}
export async function reviewNativeBulkWorkspaceRecovery(execute,artifact,root,inputs,selection,checkpoint,options={}){
 if(!hasNativeWorkspaceCheckpoint(checkpoint))throw new Error('Open a native workspace checkpoint first.');
 const original=structuredClone(checkpoint),digest=await nativeTargetHash(original);
 const verification=await execute({operation:'verify-native-bulk-workspace-checkpoint',input:{inputs:structuredClone(inputs),selection:structuredClone(selection),checkpoint:original}});
 options.signal?.throwIfAborted();
 if(verification.schema_version!=='bisect-native-workspace-checkpoint-verification-v1'||verification.status!=='verified'||verification.checkpoint_sha256!==digest||verification.request_sha256!==await commandHash(inputs,selection)||JSON.stringify(verification.runtime_artifact)!==JSON.stringify(artifact)||verification.filesystem_actions_performed!==false)throw new Error('Current worker did not verify this exact saved workspace checkpoint.');
 const token=Object.freeze({});continuations.set(token,{checkpoint:original,verification,verification_sha256:await nativeTargetHash(verification)});
 try{return await reviewNativeBulkWorkspace(execute,artifact,root,inputs,selection,{...options,continuation:token});}finally{continuations.delete(token);}
}
export async function nativeBulkWorkspaceRetained(session,year){const state=current(session);return structuredClone((state.years.get(year)?.targets??[]).filter(t=>t.retained).map(t=>t.retained));}
export async function nativeBulkWorkspaceVerifiedSelection(session,year){const state=current(session);return state.recovery?structuredClone(state.years.get(year)?.selection??null):null;}
export async function activateNativeBulkWorkspace(review,inputs,selection){
 const state=reviews.get(review);if(!state||state.request_sha256!==await commandHash(inputs,selection))throw new Error('Native bulk inputs changed. Review the selected workspace again.');
 reviews.delete(review);const session=Object.freeze({});sessions.set(session,state);return session;
}
export async function requireNativeBulkWorkspace(session,inputs,selection){const state=current(session);if(state.request_sha256!==await commandHash(inputs,selection))throw new Error('Native bulk workspace belongs to different inputs or selection.');}
export async function nativeBulkWorkspaceSelection(session,year,artifact,{signal}={}){
 const state=current(session),part=state.years.get(year);if(!part||JSON.stringify(state.artifact)!==JSON.stringify(artifact))throw new Error('Native workspace year or engine changed.');
 for(const target of part.targets)await unchanged(state,target,{signal});
 return structuredClone(part.selectionInputs);
}
export async function performNativeBulkWorkspaceTarget(session,job,{signal}={}){
 const state=current(session),target=state.targets.get(key(job.scope));
 if(target?.review_error)throw new Error(target.review_error);
 if(!target?.scheduled||!target.has_job||target.used||!target.review||job.scope.chamber!==target.scope.chamber||job.scope.districts!==target.scope.districts||target.source_sha256!==job.source_sha256||await nativeTargetHash(job.native_state)!==job.source_sha256)throw new Error('This exact source has no unused reviewed native workspace target.');
 await unchanged(state,target,{signal});target.used=true;
 try{
  const receipt=await performNativeStateTargetActions(state.execute,target.review,{signal,assertCurrent:()=>current(session)});
  const after=await observeNativeOutputTarget(state.root,target.intent.target_namespace,{signal});target.canonical_sha256=after.sha256;if(target.historical_namespace===target.intent.target_namespace)target.historical_sha256=after.sha256;
  const record={intent:structuredClone(target.intent),receipt,...(target.reset_disposition?{reset_disposition:target.reset_disposition}:{})};state.records.push(record);return record;
 }catch(error){if(error.native_data_reset)state.records.push({intent:structuredClone(target.intent),receipt:error.native_data_reset});throw error;}
}
export function nativeBulkWorkspaceRecord(session,year=null){const state=current(session);return {schema_version:'bisect-native-bulk-workspace-record-v1',request_sha256:state.request_sha256,directory_name:state.root.name??'selected outputs folder',...(year?{year}:{}),review:structuredClone(state.review),targets:[...state.targets.values()].filter(t=>!year||t.scope.year===year).map(t=>({scope:structuredClone(t.scope),target_namespace:t.intent.target_namespace,scheduled:t.scheduled,has_job:t.has_job})),actions:structuredClone(state.records.filter(r=>!year||r.intent.year===year)),...(state.recovery?{continuation:{schema_version:'bisect-native-workspace-continuation-v1',checkpoint_sha256:state.recovery.verification.checkpoint_sha256,verification_sha256:state.recovery.verification_sha256,completed_replay_performed:true,historical_actions_authenticated:false},previous_actions:structuredClone(nativeWorkspaceChildren(state.recovery.checkpoint).filter(row=>!year||row.year===year).flatMap(row=>[...(row.checkpoint.native_workspace.previous_actions??[]),...row.checkpoint.native_workspace.actions])),retained_results:[...state.targets.values()].filter(t=>t.retained&&(!year||t.scope.year===year)).map(t=>({state:t.scope.state,year:t.scope.year,installed_bytes_verified:t.installed_bytes_verified===true}))}:{}),installation_performed:false};}
export async function finishNativeBulkWorkspaceYear(session,year,{signal}={}){const state=current(session);for(const target of state.years.get(year)?.targets??[])await unchanged(state,target,{signal});}
export function closeNativeBulkWorkspace(session){const state=sessions.get(session);if(state&&!state.closed){state.closed=true;state.close();}}
