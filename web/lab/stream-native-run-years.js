import {nativeRunStateFilters} from './native-run-policy.js';
import {RUN_COMMAND_DEFAULTS,stateCommandRequest} from './state-command-files.js';
import {stateSourceAcquisitionChoices,validateStateSourceExecution} from './state-source-acquisition.js';
import {streamStateSourceBatch} from './stream-state-source-batch.js';

const YEARS=['2020','2010','2000'];
export function nativeRunYearsPlan(plan,files){return plan?.year==='all'&&files?.nativeStateSourceSettings?.enabled===true&&files.nativeStateSourceSettings.command_kind==='run';}

// The parent retains compact receipts only. Original archives, recipes and
// accepted output bundles are closed/read back by the existing per-state sink.
export async function streamNativeRunYears(plan,selectionInputs,files,runSettings,formSettings,workflowSettings,sink,{createWorker=()=>new Worker(new URL('./wasm-worker.js',import.meta.url),{type:'module'}),wasmUrl=new URL('./bisect_wasm.wasm',import.meta.url).href,signal,onCheckpoint=()=>{},onProgress=()=>{},...options}={}){
 const snapshot=structuredClone({plan,selectionInputs,files,runSettings,formSettings,workflowSettings});
 if(!nativeRunYearsPlan(snapshot.plan,snapshot.files))throw new Error('Select an all-vintage Fetch plan and native Run source flags.');
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);
 if(!sink||typeof sink.year!=='function'||typeof sink.checkpoint!=='function')throw new Error('Select a fresh multi-year local output folder.');
 let worker,serial=0,artifact,review;const years=[];
 const command=message=>new Promise((resolve,reject)=>{
  if(signal?.aborted){reject(signal.reason||new Error('Run cancelled.'));return;}
  const id=++serial,cleanup=()=>{worker.removeEventListener('message',receive);worker.removeEventListener('error',fail);signal?.removeEventListener('abort',abort);};
  const abort=()=>{cleanup();reject(signal.reason||new Error('Run cancelled.'));};
  const fail=e=>{cleanup();reject(new Error(e.message||'Native Run inspection worker failed.'));};
  const receive=({data})=>{if(message.type==='initialize'&&data.type==='ready'){cleanup();resolve(data.runtimeArtifact);}else if(data.id===id&&['result','error'].includes(data.type)){cleanup();data.type==='result'?resolve(data.result):reject(new Error(data.error));}};
  worker.addEventListener('message',receive);worker.addEventListener('error',fail);signal?.addEventListener('abort',abort,{once:true});try{worker.postMessage({...message,id});}catch(e){cleanup();reject(e);}
 });
 const receipt=status=>({schema_version:'bisect-stream-native-run-years-v1',status,requested_years:[...YEARS],output_folder:sink.name,fetch_manifest_sha256:snapshot.plan.manifest_sha256,runtime_artifact:structuredClone(artifact),native_run_command:structuredClone(review),years:structuredClone(years),summary:{total:3,completed:years.filter(r=>['completed','failed'].includes(r.status)).length,succeeded:years.filter(r=>r.status==='completed').length,failed:years.filter(r=>r.status==='failed').length},state_summary:{succeeded:years.reduce((n,r)=>n+(r.result?.summary.succeeded||0),0),failed:years.reduce((n,r)=>n+(r.result?.summary.failed||0),0)},native_output_summary:{completed:years.reduce((n,r)=>n+(r.result?.native_output_summary?.completed||0),0),failed:years.reduce((n,r)=>n+(r.result?.native_output_summary?.failed||0),0)},execution_performed:years.some(r=>r.result?.execution_performed),exit_code:status==='completed'?(years.some(r=>r.status==='failed')?1:0):null,scope:'Explicit native Run all-vintage source streaming in 2020, 2010, 2000 order. One WASM artifact; native flags, allocation and worker policy checked before acquisition. Each year has a fresh output folder, original sources and accepted state bundles. Failed years continue; cancellation stops scheduling. No automatic Resume, atomic promotion, implicit disk access on Open or native output-dir/reset filesystem semantics.'});
 const publish=async status=>{const value=receipt(status);await sink.checkpoint(value);await onCheckpoint(value);return value;};
 try{
  worker=createWorker();artifact=await command({type:'initialize',url:wasmUrl});
  if(!artifact||!/^[a-f0-9]{64}$/.test(artifact.sha256)||!Number.isSafeInteger(artifact.byte_length)||artifact.byte_length<1)throw new Error('Invalid native Run WASM artifact.');
  review=await command({type:'run',request:stateCommandRequest(snapshot.files.runCommandInputs??RUN_COMMAND_DEFAULTS,{run:true})});
 }finally{worker?.terminate();}
 if(review?.schema_version!=='bisect-native-run-command-v1'||review.status!=='parsed'||review.year!=='all'||review.print_only)throw new Error('Inspect native Run --year all flags before streaming. Print-only never acquires or partitions.');
 if(snapshot.selectionInputs.manifest)throw new Error('Native Run streaming requires embedded congressional allocation.');
 if(sink.nativeOutputSettings?.enabled&&sink.nativeOutputSettings.version!==review.version)throw new Error('Native output version must match native Run --version.');
 const scopes=stateSourceAcquisitionChoices(snapshot.plan),filters=nativeRunStateFilters(review);
 if(filters.some(state=>!scopes.some(r=>r.state===state)))throw new Error('Native Run filters have no supported tract pairs in the Fetch plan. No acquisition performed.');
 await publish('running');
 for(const year of YEARS){
  if(signal?.aborted)return publish('cancelled');
  const row={year,status:'running',result:null};years.push(row);await publish('running');
  try{
   const available=scopes.filter(r=>r.year===year);
   if(!available.length||filters.some(state=>!available.some(r=>r.state===state)))throw new Error('Fetch plan lacks requested supported tract pairs for '+year+'. No acquisition performed for this year. Supply original sources through the prepared queue or configure supported source acquisition.');
   const childSink=await sink.year(year);
   const child=await streamStateSourceBatch(snapshot.plan,year,snapshot.selectionInputs,snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings,childSink,{...options,createWorker,wasmUrl,signal,expectedArtifact:artifact,onProgress:p=>onProgress({...p,year}),onCheckpoint:async value=>{row.result=value;await publish('running');}});
   row.result=child;row.status=child.status==='completed'?(child.summary.failed||child.native_output_summary?.failed?'failed':'completed'):child.status;
   if(child.status==='cancelled')return publish('cancelled');
   if(child.status==='interrupted')return publish('interrupted');
  }catch(error){row.status=signal?.aborted?'cancelled':'failed';row.error=String(error.message).slice(0,2000);if(signal?.aborted)return publish('cancelled');if(error.workerFailure||/WASM.*artifact/i.test(error.message))return publish('interrupted');}
  await publish('running');
 }
 return publish('completed');
}
