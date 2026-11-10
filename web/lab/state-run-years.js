import {validateStateCommandInputs} from './state-command-files.js';
import {validateJsonTree,MAX_WORKBENCH_ARCHIVE_BYTES} from './project.js';
import {stateJobScope} from './state-source-job.js';
import {encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const NATIVE_RUN_YEARS=['2020','2010','2000'];
const sha=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
const MAX_RETAINED=96*1024*1024;
const liveYears=new WeakMap();
const checkpointText=value=>JSON.stringify({...value,status:undefined});
export function canResumeStateYears(value){
 if(value?.years?.some(row=>row.result?.native_workspace))return false;
 const proof=value&&liveYears.get(value);
 return !!proof&&proof.session.closed&&['completed','cancelled','interrupted'].includes(value.status)&&
  (value.years.length<3||value.years.some(row=>!row.result||row.result.summary.succeeded<row.result.summary.total));
}

export function canVerifySavedStateYears(value){
 if(value?.years?.some(row=>row.result?.native_workspace))return false;
 return value?.schema_version==='bisect-state-run-years-v1'&&!liveYears.has(value)&&Array.isArray(value.years)&&value.years.some(row=>Array.isArray(row?.result?.states)&&row.result.states.some(state=>state?.status==='ok'));
}
export function savedYears(value,validateChild){
 validateJsonTree(value,4000000,MAX_WORKBENCH_ARCHIVE_BYTES);
 const keys='execution_performed,exit_code,request_sha256,requested_years,runtime_artifact,schema_version,scope,state_summary,status,summary,years';
 if(!value||Object.keys(value).filter(k=>k!=='native_run_policy').sort().join()!==keys||value.schema_version!=='bisect-state-run-years-v1'||!['running','completed','cancelled','interrupted'].includes(value.status)||JSON.stringify(value.requested_years)!==JSON.stringify(NATIVE_RUN_YEARS)||!Array.isArray(value.years)||value.years.length>3||typeof value.scope!=='string'||value.scope.length>4096||!/^[a-f0-9]{64}$/.test(value.request_sha256)||!value.runtime_artifact||Object.keys(value.runtime_artifact).sort().join()!=='byte_length,sha256'||!/^[a-f0-9]{64}$/.test(value.runtime_artifact.sha256)||!Number.isSafeInteger(value.runtime_artifact.byte_length)||value.runtime_artifact.byte_length<1)throw new Error('Invalid saved multi-year checkpoint.');
 if(Object.hasOwn(value,'native_run_policy'))validateStateCommandInputs(value.native_run_policy);
 for(const [index,row] of value.years.entries()){
  if(!row||Object.keys(row).filter(k=>k!=='error').sort().join()!=='result,status,year'||row.year!==NATIVE_RUN_YEARS[index]||!['running','completed','failed','cancelled','interrupted'].includes(row.status)||row.error!==undefined&&(typeof row.error!=='string'||row.error.length>10000)||row.result!==null&&(typeof row.result!=='object'||Array.isArray(row.result)))throw new Error('Invalid saved multi-year row.');
  if(row.result){validateChild(row.result);if(row.result.selection?.year!==row.year||JSON.stringify(row.result.runtime_artifact)!==JSON.stringify(value.runtime_artifact))throw new Error('Saved year scope or WASM artifact differs.');}
 }
 const summary={total:3,completed:value.years.filter(r=>['completed','failed'].includes(r.status)).length,succeeded:value.years.filter(r=>r.status==='completed').length,failed:value.years.filter(r=>r.status==='failed').length};
 const states={succeeded:value.years.reduce((n,r)=>n+(r.result?.summary.succeeded||0),0),failed:value.years.reduce((n,r)=>n+(r.result?.summary.failed||0),0)};
 const equal=(a,b)=>a&&Object.keys(a).sort().join()===Object.keys(b).sort().join()&&Object.keys(b).every(k=>a[k]===b[k]);
 if(!equal(value.summary,summary)||!equal(value.state_summary,states)||value.execution_performed!==value.years.some(r=>r.result?.execution_performed)||value.exit_code!==(value.status==='completed'?(value.years.some(r=>r.status==='failed')?1:0):null)||value.status==='completed'&&value.years.length!==3)throw new Error('Saved multi-year summary differs from its rows.');
 return structuredClone(value);
}

// Compose the existing fully validated single-year boundary; no new engine or
// allocation defaults are introduced. Children retain their independent receipts.
export async function runStateBatchYears(inputs,selection,{executeYear,jobs,onCheckpoint=()=>{},signal,expectedArtifact=null,resume=null,verifySaved=null,validateSavedChild,...options}){
 if(!inputs.jobs)throw new Error('Add or select prepared native Run jobs first.');
 if(resume!==null&&verifySaved!==null)throw new Error('Choose Resume or saved multi-year verification.');
 const saved=verifySaved===null?null:savedYears(verifySaved,validateSavedChild);
 const requestHash=await sha(JSON.stringify({inputs,selection}));
 if(saved&&(saved.request_sha256!==requestHash||JSON.stringify(saved.native_run_policy)!==JSON.stringify(inputs.run_command)))throw new Error('Saved multi-year inputs changed. Start a fresh Run.');
 const proof=resume&&liveYears.get(resume);
 if(resume&&(!canResumeStateYears(resume)||proof.requestHash!==requestHash||checkpointText(resume)!==proof.text))throw new Error('Multi-year Resume requires an unchanged checkpoint and inputs from this page session.');
 const years=resume?resume.years.map(row=>({...row})):[],session={closed:false};
 let artifact=resume?structuredClone(resume.runtime_artifact):saved?structuredClone(saved.runtime_artifact):expectedArtifact,status='running';
 if(expectedArtifact&&artifact&&JSON.stringify(expectedArtifact)!==JSON.stringify(artifact))throw new Error('Multi-year WASM artifact changed. Start a fresh Run.');
 const snapshot=()=>({
  schema_version:'bisect-state-run-years-v1',status,requested_years:[...NATIVE_RUN_YEARS],
  request_sha256:requestHash,runtime_artifact:artifact,...(inputs.run_command?{native_run_policy:structuredClone(inputs.run_command)}:{}),years:years.map(row=>({...row})),
  summary:{total:3,completed:years.filter(r=>['completed','failed'].includes(r.status)).length,
   succeeded:years.filter(r=>r.status==='completed').length,failed:years.filter(r=>r.status==='failed').length},
  state_summary:{succeeded:years.reduce((n,r)=>n+(r.result?.summary?.succeeded||0),0),failed:years.reduce((n,r)=>n+(r.result?.summary?.failed||0),0)},
  execution_performed:years.some(r=>r.result?.execution_performed),
  exit_code:status==='completed'?(years.some(r=>r.status==='failed')?1:0):null,
  scope:'Native Run year order 2020, 2010, 2000 with independently selected native congressional queues. Original per-state engine requests and retry modes remain unchanged. A failed year does not stop later years; cancellation stops scheduling and retains received state outputs. Every year loads the same WASM artifact. Full native filesystem installation and source acquisition are not performed. Explicit session Resume retains unchanged completed outputs after fresh native selection and same-artifact checks for each year; unfinished/failed states are retried. Saved multi-year results require explicit full native replay verification before Resume. Opening sources/results remains inert.'
 });
 const publish=()=>{
  const value=snapshot();
  if(new TextEncoder().encode(JSON.stringify(value)).length>MAX_RETAINED){const error=new Error('Multi-year retained receipts exceed 96 MiB; earlier checkpoints remain available.');error.multiYearCapacity=true;throw error;}
  liveYears.set(value,{session,requestHash,text:checkpointText(value)});onCheckpoint(value);return value;
 };
 try{
 publish();
 for(const year of (saved?saved.years.map(row=>row.year):NATIVE_RUN_YEARS)){
  if(signal?.aborted){status='cancelled';break;}
  const index=NATIVE_RUN_YEARS.indexOf(year),retained=resume?.years.find(row=>row.year===year)?.result,savedChild=saved?.years.find(row=>row.year===year)?.result;
  years[index]={year,status:'running',result:retained??null};publish();
  if(saved&&!savedChild){years[index]={year,status:'interrupted',result:null};publish();continue;}
  const selected=jobs.filter(job=>stateJobScope(job).year===year);
  const childInputs={workers:inputs.workers,...(inputs.run_command?{run_command:structuredClone(inputs.run_command)}:{}),jobs:{name:'prepared-'+year+'-jobs.json',base64:encodeBytes(new TextEncoder().encode(JSON.stringify(selected)))}};
  try{
   const child=await executeYear(childInputs,{...selection,year},{...options,signal,expectedArtifact:artifact,continueMissingJobs:true,...(inputs.run_command?{expectRunAllYears:true}:{}),...(retained?{resume:retained,allowCompletedResume:true}:{}),...(saved?{verifySaved:savedChild}:{}),onCheckpoint:checkpoint=>{
    if(artifact&&(artifact.sha256!==checkpoint.runtime_artifact?.sha256||artifact.byte_length!==checkpoint.runtime_artifact?.byte_length))throw new Error('All years must load the same WASM artifact.');
    artifact??=structuredClone(checkpoint.runtime_artifact);
    const previous=years[index];years[index]={year,status:'running',result:checkpoint};
    try{publish();}catch(error){years[index]=previous;throw error;}
   }});
   const previous=years[index];years[index]={year,status:saved?(child.status==='completed'?'completed':'interrupted'):child.summary.failed?'failed':'completed',result:child,...(child.summary.failed?{error:child.states.filter(row=>row.status==='failed').map(row=>row.error).join('; ').slice(0,4000)}:{})};
   try{publish();}catch(error){years[index]=previous;throw error;}
  }catch(error){
   const latest=years[index].result;
   years[index]={year,status:signal?.aborted?'cancelled':'failed',result:latest?Object.assign(latest,{status:signal?.aborted?'cancelled':'interrupted'}):null,error:error.message};
   if(signal?.aborted){status='cancelled';break;}
   if(saved){status='interrupted';break;}
   if(error.multiYearCapacity||error.workerFailure||/WASM.*(artifact|hash|checksum)/iu.test(error.message)){status='interrupted';break;}
   publish();
  }
 }
 if(status==='running')status=saved&&(years.length<3||years.some(row=>row.status!=='completed'))?'interrupted':'completed';
 return publish();
 }finally{session.closed=true;}
}

export async function stateRunYearsZip(result,childZip){
 if(result?.schema_version!=='bisect-state-run-years-v1'||!['completed','cancelled','interrupted'].includes(result.status)||!Array.isArray(result.years)||result.years.length>3||JSON.stringify(result.requested_years)!==JSON.stringify(NATIVE_RUN_YEARS))throw new Error('Finish or cancel a valid multi-year Run before export.');
 if(result.status==='completed'&&result.years.length!==3)throw new Error('A completed Run must record all three years.');
 const entries=[],seen=new Set();
 for(const row of result.years){
  if(row.year!==NATIVE_RUN_YEARS[seen.size]||seen.has(row.year))throw new Error('Invalid multi-year output scope.');seen.add(row.year);
  if(row.result){if(row.result.selection?.year!==row.year)throw new Error('Child batch differs from its recorded year.');entries.push([row.year+'/state-outputs.zip',await childZip(row.result)]);}
 }
 const receipt={...result,years:result.years.map(({result:child,...row})=>({...row,...(child?{summary:child.summary,output_archive:row.year+'/state-outputs.zip'}:{})}))};
 entries.push(['run-years-receipt.json',new TextEncoder().encode(JSON.stringify(receipt,null,2)+'\n')]);
 return storedZipBytes(entries,{maxSourceBytes:128*1024*1024});
}
