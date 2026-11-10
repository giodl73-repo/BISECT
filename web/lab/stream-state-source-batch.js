import {nativeRunStateFilters} from './native-run-policy.js';
import {preflightStateSourceFlags} from './state-source-preflight.js';
import {nativeStatesWorkerPolicy,serializeStateWrites} from './native-states-workers.js';
import {RUN_COMMAND_DEFAULTS,STATES_COMMAND_DEFAULTS,stateCommandRequest} from './state-command-files.js';
import {nativeStateOutputSource,validateStreamNativeOutputSettings} from './native-state-output.js';
import {nativeStateFollowupStatus} from './native-state-followup.js';
import {acquireStateSourceJob,stateSourceAcquisitionChoices,validateStateSourceExecution} from './state-source-acquisition.js';
import {stateBatchRequest} from './state-batch-files.js';

// Each state gets a separate WASM instance. Only compact disk references survive
// a completed state, so national staging does not accumulate all source recipes.
export async function streamStateSourceBatch(plan,year,selectionInputs,files,runSettings,formSettings,workflowSettings,sink,{createWorker=()=>new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'}),wasmUrl=new URL('./bisect_wasm.wasm',import.meta.url).href,signal,expectedArtifact=null,hardwareConcurrency=globalThis.navigator?.hardwareConcurrency??1,onCheckpoint=()=>{},onProgress=()=>{},...acquisitionOptions}={}){
 const snapshot=structuredClone({plan,year,selectionInputs,files,runSettings,formSettings,workflowSettings});
 const scopes=stateSourceAcquisitionChoices(snapshot.plan).filter(scope=>scope.year===year);
 if(!scopes.length)throw new Error('Select a vintage with supported pairs in a fresh Fetch plan.');
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);
 let selectionRequest=stateBatchRequest({...snapshot.selectionInputs,year,states:scopes.map(scope=>scope.state)});
 if(!sink||typeof sink.state!=='function'||typeof sink.checkpoint!=='function'||typeof sink.archive!=='function')throw new Error('Select a local output folder first.');
 let nativeSettings=sink.nativeOutputSettings?structuredClone(validateStreamNativeOutputSettings(sink.nativeOutputSettings)):null;
 const externalSignal=signal,controller=new AbortController(),relay=()=>controller.abort(externalSignal.reason);if(externalSignal?.aborted)relay();else externalSignal?.addEventListener('abort',relay,{once:true});signal=controller.signal;
 const sharedWrite=serializeStateWrites();
 const owned=new Set();let serial=0,artifact=expectedArtifact?structuredClone(expectedArtifact):undefined,selection,bulkReview,workerPolicy,acceptedExecution=false;const rows=[];
 const command=(worker,message)=>new Promise((resolve,reject)=>{
  signal?.throwIfAborted();const id=++serial;
  const cleanup=()=>{worker.removeEventListener('message',receive);worker.removeEventListener('error',fail);signal?.removeEventListener('abort',abort);};
  const abort=()=>{cleanup();reject(signal.reason||new Error('State run cancelled.'));};
  const fail=e=>{cleanup();const error=new Error(e.message||'State worker failed.');error.workerFailure=true;reject(error);};
  const receive=({data})=>{if(message.type==='initialize'&&data.type==='ready'){cleanup();resolve(data.runtimeArtifact);return;}if(data.id!==id)return;if(data.type==='progress'){onProgress({stage:'run',progress:data.progress});return;}if(data.type==='result'){cleanup();resolve(data.result);}else if(data.type==='error'){cleanup();reject(new Error(data.error));}};
  worker.addEventListener('message',receive);worker.addEventListener('error',fail);signal?.addEventListener('abort',abort,{once:true});try{worker.postMessage({...message,id});}catch(error){cleanup();reject(error);}
 });
 const open=async()=>{signal?.throwIfAborted();const worker=createWorker();owned.add(worker);const actual=await command(worker,{type:'initialize',url:wasmUrl,...(artifact?{sha256:artifact.sha256}:{})});if(!actual||!/^[a-f0-9]{64}$/.test(actual.sha256)||!Number.isSafeInteger(actual.byte_length)||actual.byte_length<1||artifact&&(actual.sha256!==artifact.sha256||actual.byte_length!==artifact.byte_length)){const error=new Error('State workers must load the same WASM artifact.');error.workerFailure=true;throw error;}artifact=structuredClone(actual);return worker;};
 const close=worker=>{worker.terminate();owned.delete(worker);};
 const checkpoint=status=>({schema_version:'bisect-stream-state-batch-v1',status,year,output_folder:sink.name,fetch_manifest_sha256:snapshot.plan.manifest_sha256,runtime_artifact:structuredClone(artifact),selection:structuredClone(selection),...(bulkReview?{[snapshot.files.nativeStateSourceSettings?.command_kind==='run'?'native_run_command':'native_states_command']:structuredClone(bulkReview),scheduling:{...workerPolicy,output_dir_intent:bulkReview.output_dir,output_dir_accessed:false,followup_policy:'explicit-browser-workflow-controls',native_followup_flags_applied:false}}:{}),states:structuredClone(rows.filter(Boolean)),summary:{total:selection.states.length,completed:rows.filter(Boolean).length,succeeded:rows.filter(row=>row.status==='ok').length,failed:rows.filter(row=>row.status==='failed').length},...(nativeSettings?.enabled?{native_output_settings:nativeSettings,native_output_summary:{completed:rows.filter(r=>r.native_output?.status==='completed').length,failed:rows.filter(r=>r.native_output?.status==='failed').length}}:{}),execution_performed:acceptedExecution||rows.some(row=>row.execution_performed),scope:'Pinned tract acquisition and independent native-allocated State Runs. State recipes and complete output ZIPs are saved in a fresh local folder and verified after close. Compact references remain in this receipt. Optional native-layout files install into the fresh run folder after accepted source/output bundles are saved, without another partition. Native output failures are disclosed separately from accepted Runs. Completion markers are written last; no atomic folder promotion or automatic recovery is claimed. Open never reads local paths or executes.'});
 const publish=status=>sharedWrite(async()=>{const receipt=checkpoint(status);await sink.checkpoint(receipt);await onCheckpoint(receipt);return receipt;});
 try{
  let worker=await open();
  const inspectNativeCommand=await preflightStateSourceFlags(snapshot.files,{inspectNativeCommand:request=>command(worker,{type:'run',request}),signal});
  if(snapshot.files.nativeStateSourceSettings?.enabled&&['states','run'].includes(snapshot.files.nativeStateSourceSettings.command_kind)){
   const runPolicy=snapshot.files.nativeStateSourceSettings.command_kind==='run';
   bulkReview=await command(worker,{type:'run',request:stateCommandRequest(runPolicy?(snapshot.files.runCommandInputs??RUN_COMMAND_DEFAULTS):(snapshot.files.statesCommandInputs??STATES_COMMAND_DEFAULTS),{bulk:!runPolicy,run:runPolicy})});
   if(bulkReview?.schema_version!==(runPolicy?'bisect-native-run-command-v1':'bisect-native-states-command-v1')||bulkReview.status!=='parsed'||bulkReview.print_only||(runPolicy?!bulkReview.years?.includes(year):bulkReview.year!==year||!bulkReview.profile_filter?.allowed||!bulkReview.multiscale_levels?.allowed))throw new Error('Review native States flags and vintage before streaming. Print-only does not acquire or execute states.');
   if(snapshot.selectionInputs.manifest)throw new Error('Native States streaming uses the embedded congressional allocation. Clear the custom manifest or use State flags.');
   const requested=runPolicy?nativeRunStateFilters(bulkReview):bulkReview.states.map(state=>state.toUpperCase());
   if(requested.some(state=>!scopes.some(scope=>scope.state===state)))throw new Error('A requested native States filter has no source pair in this Fetch plan.');
   selectionRequest=stateBatchRequest({...snapshot.selectionInputs,year,states:requested.length?requested:scopes.map(scope=>scope.state),...(snapshot.selectionInputs.completion?{completion:{...snapshot.selectionInputs.completion,reprocess:bulkReview.effective_configuration.reprocess}}:{})});
   if(nativeSettings?.enabled&&nativeSettings.version!==bulkReview.version)throw new Error('Native output version must match --version in States flags. Review the output settings before streaming.');
  }
  selection=await command(worker,{type:'run',request:selectionRequest});close(worker);worker=null;
  if(!['run','states'].includes(snapshot.files.nativeStateSourceSettings?.command_kind)&&snapshot.files.stateWorkflowBinding?.settings.label!==null&&snapshot.files.stateWorkflowBinding&&nativeSettings?.enabled&&nativeSettings.layout==='labeled'&&selection.states?.length>1)throw new Error('One explicit State label cannot install multiple states into the same native plan namespace. Select one state or queue distinct per-state labels. No acquisition performed.');
  if(selection?.schema_version!=='bisect-state-batch-selection-v1'||selection.year!==year||!Array.isArray(selection.states)||selection.states.length>51||new Set(selection.states.map(row=>row.state_code)).size!==selection.states.length||selection.states.some(row=>!scopes.some(scope=>scope.state===row.state_code)||!Number.isSafeInteger(row.districts)||row.districts<1||row.districts>500))throw new Error('Native state selection differs from the Fetch plan.');
  workerPolicy=bulkReview?nativeStatesWorkerPolicy(bulkReview,selection.states.length,hardwareConcurrency):{effective_workers:Math.min(1,selection.states.length)};
  await publish('running');
  const executeRow=async(row,index)=>{
   if(signal?.aborted)return;let executed=false,worker=null;
   try{
    onProgress({stage:'acquisition',state:row.state_code,year});
    const job=await acquireStateSourceJob(snapshot.plan,{state:row.state_code,year},snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings,{...acquisitionOptions,...(inspectNativeCommand?{inspectNativeCommand}:{}),archiveProfile:'large-state',retainRawArchive:false,onArchive:archive=>sink.archive(archive,{signal}),signal,onProgress});
    const input=structuredClone(job);if(input.native_state||input.prepared_engine_template)input.scope.districts=row.districts;else input.configuration.settings.districts=row.districts;
    worker=await open();executed=true;const output=await command(worker,{type:'run',request:{operation:input.native_state?'run-native-state-job':input.prepared_engine_template?'run-census-prepared-state-job':'run-census-state-job',input}});acceptedExecution=true;
    let nativeOutput,nativeOutputError;
    if(nativeSettings?.enabled){
     try{onProgress({stage:'native-output-preparation',state:row.state_code,year});nativeOutput=await command(worker,{type:'run',request:{operation:'prepare-native-state-output',input:{source:nativeStateOutputSource(output.result),version:nativeSettings.version,layout:nativeSettings.layout}}});
      if(nativeSettings.layout==='state'&&nativeOutput.state_slug!==row.state_slug)throw new Error('Selected manifest state folder differs from the embedded native output catalog. Use labeled output or the built-in state names.');
     }catch(error){signal?.throwIfAborted();nativeOutput=undefined;nativeOutputError=String(error.message).slice(0,2000);}
    }
    onProgress({stage:'saving',state:row.state_code,year});const saved=await sharedWrite(()=>sink.state(job,output,{signal,...(nativeSettings?.enabled?{nativeOutput,nativeOutputError,execute:request=>command(worker,{type:'run',request})}:{})}));close(worker);worker=null;
    rows[index]={state:row.state_code,status:'ok',execution_performed:true,...(nativeStateFollowupStatus(output.result)?{followup:nativeStateFollowupStatus(output.result)}:{}),seed:(job.native_state||job.prepared_engine_template)?output.result.source_review.engine_request.options.seed:job.form.seed,effective_seed:(job.native_state||job.prepared_engine_template)?(output.result.prepared.executed_request.request??output.result.prepared.executed_request).options.seed:output.result.configured_run.result.options.seed,...(saved.native_output?{native_output:saved.native_output}:{}),output:saved};
   }catch(error){if(worker){close(worker);worker=null;}if(signal?.aborted)return;if(error.workerFailure)throw error;rows[index]={state:row.state_code,status:'failed',execution_performed:executed,error:String(error.message).slice(0,2000)};}
   await publish('running');
  };
  let next=0,fatal;
  const slots=Array.from({length:workerPolicy.effective_workers},async()=>{
   try{while(!signal.aborted&&next<selection.states.length){const index=next++;await executeRow(selection.states[index],index);}}
   catch(error){if(!fatal)fatal=error;controller.abort(error);}
  });
  await Promise.allSettled(slots);
  if(fatal)throw fatal;
  return await publish(signal.aborted?'cancelled':'completed');
 }catch(error){if(selection?.states){const receipt=checkpoint(externalSignal?.aborted?'cancelled':'interrupted');await onCheckpoint(receipt);try{await sink.checkpoint(receipt);}catch{/* Earlier verified state markers remain on disk. */}}throw error;}finally{controller.abort(new Error('State source workers closed.'));externalSignal?.removeEventListener('abort',relay);for(const worker of owned)worker.terminate();}
}
