import {STATE_COMMAND_DEFAULTS,stateCommandRequest,statePreparedFlagRefusals} from './state-command-files.js';
import {recheckStateWorkflowBinding} from './native-state-workflow.js';

// Inspection uses the actual Rust parser. It never acquires sources or runs a
// partition. Cached replies are private to one immutable acquisition snapshot.
export async function preflightStateSourceFlags(files,{inspectNativeCommand,signal,expectedArtifact=null,createInspectionWorker=()=>new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'}),inspectionWasmUrl=new URL('./bisect_wasm.wasm',import.meta.url).href}={}){
 if(!files.nativeStateSourceSettings?.enabled||['run','states'].includes(files.nativeStateSourceSettings.command_kind))return undefined;
 signal?.throwIfAborted();
 if(files.stateReportSourceSnapshot!=null)throw new Error('Apply selected report sources before source acquisition.');
 const snapshot=structuredClone({stateCommandInputs:files.stateCommandInputs,stateWorkflowBinding:files.stateWorkflowBinding,stateGraphInputs:{auxiliary:files.stateGraphInputs?.auxiliary}}),replies=new Map();let worker,serial=0;
 const command=message=>new Promise((resolve,reject)=>{
  if(signal?.aborted){reject(signal.reason||new Error('Source preflight cancelled.'));return;}
  const id=++serial,cleanup=()=>{worker.removeEventListener('message',receive);worker.removeEventListener('error',fail);signal?.removeEventListener('abort',abort);};
  const fail=event=>{cleanup();reject(new Error(event.message||'Native source inspection worker failed.'));};
  const abort=()=>{cleanup();reject(signal.reason||new Error('Source preflight cancelled.'));};
  const receive=({data})=>{if(message.type==='initialize'&&data.type==='ready'){cleanup();resolve(data.runtimeArtifact);return;}if(data.id!==id)return;if(data.type==='result'){cleanup();resolve(data.result);}else if(data.type==='error'){cleanup();reject(new Error(data.error));}};
  worker.addEventListener('message',receive);worker.addEventListener('error',fail);signal?.addEventListener('abort',abort,{once:true});
  try{worker.postMessage({...message,id});}catch(error){cleanup();reject(error);}
 });
 try{
  if(!inspectNativeCommand){
   worker=createInspectionWorker();
   const artifact=await command({type:'initialize',url:inspectionWasmUrl,...(expectedArtifact?{sha256:expectedArtifact.sha256}:{})});
   if(!artifact||!/^[a-f0-9]{64}$/.test(artifact.sha256)||!Number.isSafeInteger(artifact.byte_length)||artifact.byte_length<1||expectedArtifact&&(artifact.sha256!==expectedArtifact.sha256||artifact.byte_length!==expectedArtifact.byte_length))throw new Error('Source preflight must inspect with the selected WASM artifact.');
   inspectNativeCommand=request=>command({type:'run',request});
  }
  const inspect=async request=>{signal?.throwIfAborted();const key=JSON.stringify(request);if(!replies.has(key))replies.set(key,structuredClone(await inspectNativeCommand(request)));return structuredClone(replies.get(key));};
  if(snapshot.stateWorkflowBinding)await recheckStateWorkflowBinding({execute:inspect},snapshot.stateWorkflowBinding,snapshot.stateCommandInputs??STATE_COMMAND_DEFAULTS,{forExecution:true,sourceAuxiliary:snapshot.stateGraphInputs?.auxiliary});
  else{
   const review=await inspect(stateCommandRequest(snapshot.stateCommandInputs??STATE_COMMAND_DEFAULTS)),refused=statePreparedFlagRefusals(review);
   if(review.status!=='parsed'||review.profile_filter?.allowed!==true||refused.length)throw new Error('Review State flags before source acquisition: '+refused.join(', ')+'. No acquisition or partition performed.');
  }
  signal?.throwIfAborted();
  return request=>{signal?.throwIfAborted();const key=JSON.stringify(request);if(!replies.has(key))throw new Error('State flags changed after source preflight. Inspect again before acquisition.');return structuredClone(replies.get(key));};
 }finally{worker?.terminate();}
}
