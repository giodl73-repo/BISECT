import {startCeremonyFromSources} from './ceremony-start-sources.js';
import {inspectStateCommandWithIdentity} from './native-state-identity-preview.js';
import {executeNativeSummaryBatch} from './native-label-summary.js';
import {verifyNativeBulkCheckpoint} from './native-bulk-checkpoint-verification.js';
import {executeFetchArchivePackage} from './fetch-archive-package.js';
import {executeNativeOutputVerification} from './native-output-verification.js';
import {executeReviewedNativeSelectedStateSource} from './native-state-target-run.js';
import {executeNativeRunJob} from './native-run-job.js';
import {executeNativeStatesJob} from './native-states-job.js';
import {executeNativeStateOutput} from './native-state-output.js';
import {executeNativeStateJob} from './native-state-job.js';
import {executeNativeSelectedStateSource} from './native-selected-state-source.js';
import {executeNationalMapDirectory} from './national-map-directory.js';
import {executeRoundMapDirectory} from './round-map-directory.js';
import {makeWorkerFailure} from './worker-failure.js';
import {executePreparedNativeJob} from './prepared-native-job.js';
import {executePreparedCensusJob} from './prepared-census-job.js';
import {executePreparedStateJob} from './prepared-state-job.js';
import {executeNationalMapBatch} from './national-map-batch.js';
import {executeStateSourceJob} from './state-source-job.js';
import {verifyFletchCacheFiles} from './fletch-verify-files.js';
import {labelBuildRequest,startLabelBuildCheckpoint} from './label-build-files.js?v=label-build-checkpoint-v1';
import {verifyTutorialFiles} from './tutorial-files.js';
import {instantiateEngine} from './wasm-engine.js?v=runtime-artifact-v1';
import {loadMetisSidecar} from './metis-c-sidecar.js';
import {startLabelReportYearsCheckpoint} from './label-report-files.js';
import {startLabelAnalysisYearsCheckpoint} from './label-analysis-files.js';
let engine,engineBytes,sidecarLoaded=false,activeRun=null,initialization=0,initializing=false;
function runtimeRecovery(){
  if(!engine?.cMetisRetired)return {};
  // Keep this request's outcome. Only a later explicit C Run loads new assets.
  sidecarLoaded=false;
  return {runtimeRecovery:'c-metis-reload-on-next-run'};
}
async function enableCMetis(){
  if(sidecarLoaded)return;
  if(!engine?.supportsCMetis)throw new Error('This engine build does not include C METIS support.');
  const selected=engine;
  const assetBase='./c-metis/';
  const read=async(name,limit)=>{
    const response=await fetch(new URL(assetBase+name,import.meta.url),{credentials:'omit',redirect:'error'});
    if(!response.ok)throw new Error('C METIS package asset unavailable: '+name);
    const reader=response.body?.getReader();if(!reader)throw new Error('C METIS asset stream unavailable.');
    const chunks=[];let length=0;
    try{for(;;){const {value,done}=await reader.read();if(done)break;length+=value.length;if(length>limit)throw new Error('C METIS asset exceeds its package limit.');chunks.push(value);}}
    finally{await reader.cancel();reader.releaseLock();}
    const bytes=new Uint8Array(length);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}return bytes;
  };
  const manifest=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(await read('metis-sidecar.json',128*1024)));
  const javascript=await read('metis-sidecar.mjs',256*1024),wasm=await read('metis-sidecar.wasm',4*1024*1024);
  const metisSidecar=await loadMetisSidecar({javascript,wasm,manifest});
  if(engine!==selected)throw new Error('Engine changed while C METIS was loading.');
  engine=await instantiateEngine(engineBytes,{metisSidecar});sidecarLoaded=true;
}
let pendingNativeTarget=null;
self.onmessage=async ({data})=>{
  if(data.type==='native-state-target-confirmed'){if(pendingNativeTarget&&data.id===pendingNativeTarget.id&&data.nonce===pendingNativeTarget.nonce){const pending=pendingNativeTarget;pendingNativeTarget=null;data.error?pending.reject(new Error(data.error)):pending.resolve(data.receipt);}else self.postMessage({type:'native-state-target-confirmation-ignored',id:data.id});return;}
  const token=Symbol('worker-run');
  let initToken=null;
  try {
    if (data.type==='initialize') {
      if(activeRun)throw new Error('Finish the current engine request first.');
      initToken=++initialization;initializing=true;
      engine=undefined;
      engineBytes=undefined;sidecarLoaded=false;
      const response=await fetch(data.url);
      if (!response.ok) throw new Error('Browser engine download failed.');
      const bytes=await response.arrayBuffer();
      if(data.sha256){const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==data.sha256)throw new Error('Browser engine failed its SHA-256 integrity check.');}
      const selectedEngine=await instantiateEngine(bytes);
      if(initToken!==initialization)return;
      engine=selectedEngine;
      engineBytes=bytes;
      self.postMessage({type:'ready',runtimeArtifact:engine.artifactIdentity,cMetisAvailable:engine.supportsCMetis});
    } else if (data.type==='run') {
      if (!engine||initializing) throw new Error('Browser engine is not initialized.');
      if(activeRun)throw new Error('Finish the current engine request first.');
      activeRun=token;
      if(data.cMetis===true)await enableCMetis();
      else if(engine.runtimeArtifacts.metis_sidecar!==null){
        // C authority belongs to this Run, not to a previous checkbox or
        // saved project. Drop the old instance rather than retain two heaps.
        engine=await instantiateEngine(engineBytes);sidecarLoaded=false;
      }
      let request=data.request;if(request?.operation==='run-label-build-sources'){if(Object.keys(request).sort().join()!=='operation,sources')throw new Error('Invalid build source request.');request=labelBuildRequest(request.sources);self.postMessage({type:'build-started',id:data.id,checkpoint:startLabelBuildCheckpoint(request)});}
      if(request?.operation==='render-native-national-map-batch'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid national map batch request.');
      if(request?.operation==='run-native-prepared-state-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native source job request.');
      if(request?.operation==='run-census-prepared-state-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid prepared Census job request.');
      if(request?.operation==='prepare-native-state-output'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native output preparation request.');
      if(request?.operation==='run-native-run-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native Run job request.');
      if(request?.operation==='run-native-states-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native bulk States job request.');
      if(request?.operation==='run-native-state-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native State flags job request.');
      if(request?.operation==='run-native-selected-state-source'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid selected State source Run request.');
      if(request?.operation==='run-prepared-state-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid prepared State job request.');
      if(request?.operation==='run-census-state-job'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid source State job request.');
      if(request?.operation==='run-native-label-summary-batch'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native summary batch request.');
      self.postMessage({type:'started',id:data.id});
      if(['run-label-report-years','resume-label-report-years'].includes(request?.operation)){const binding=await startLabelReportYearsCheckpoint({...request,operation:'run-label-report-years'});if(request.operation==='resume-label-report-years')binding.resume_prefix_count=request.checkpoint?.years?.length;self.postMessage({type:'report-years-started',id:data.id,binding});}
      if(['run-label-analysis-years','resume-label-analysis-years'].includes(request?.operation)){const binding=await startLabelAnalysisYearsCheckpoint({...request,operation:'run-label-analysis-years'});if(request.operation==='resume-label-analysis-years')binding.resume_prefix_count=request.checkpoint?.years?.length;self.postMessage({type:'analysis-years-started',id:data.id,binding});}
      if(['verify-run-manifest','verify-tui-manifest','verify-manifest-replay'].includes(request?.operation))request={...request,runtime_artifact:engine.artifactIdentity};
      if(request?.operation==='render-native-round-map-directory'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid round map directory request.');
      if(request?.operation==='verify-native-output-workspace'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native output verification request.');
      if(request?.operation==='verify-native-bulk-workspace-checkpoint'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid native workspace checkpoint verification request.');
      if(request?.operation==='prepare-fetch-archive-package'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid Fetch archive package request.');
      if(request?.operation==='start-ceremony-from-sources'&&Object.keys(request).sort().join()!=='input,operation')throw new Error('Invalid ceremony source Start request.');
      const result=request?.operation==='start-ceremony-from-sources'?await startCeremonyFromSources(engine,request.input):request?.operation==='inspect-native-state-command'?await inspectStateCommandWithIdentity(engine,request.arguments):request?.operation==='run-native-label-summary-batch'?await executeNativeSummaryBatch(engine,request.input,{onProgress:snapshot=>self.postMessage({type:'progress',id:data.id,progress:{schema_version:'bisect-native-label-summary-progress-v1',snapshot}})}):request?.operation==='prepare-fetch-archive-package'?await executeFetchArchivePackage(engine,request.input):request?.operation==='verify-native-bulk-workspace-checkpoint'?await verifyNativeBulkCheckpoint(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='verify-native-output-workspace'?await executeNativeOutputVerification(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-native-selected-state-source'?await executeReviewedNativeSelectedStateSource(engine,request.input,{performTargetAction:intent=>new Promise((resolve,reject)=>{const nonce=crypto.randomUUID();pendingNativeTarget={id:data.id,nonce,resolve,reject};self.postMessage({type:'native-state-target-required',id:data.id,nonce,intent});}),onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-native-run-job'?await executeNativeRunJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-native-states-job'?await executeNativeStatesJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='prepare-native-state-output'?await executeNativeStateOutput(engine,request.input):request?.operation==='run-native-state-job'?await executeNativeStateJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='render-native-national-directory'?await executeNationalMapDirectory(engine,request,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='render-native-round-map-directory'?await executeRoundMapDirectory(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-native-prepared-state-job'?executePreparedNativeJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-census-prepared-state-job'?executePreparedCensusJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-prepared-state-job'?executePreparedStateJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='render-native-national-map-batch'?await executeNationalMapBatch(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='run-census-state-job'?executeStateSourceJob(engine,request.input,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='verify-fletch-cache-files'?await verifyFletchCacheFiles(engine,request,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})}):request?.operation==='verify-tutorial-files'?await verifyTutorialFiles(engine,request):engine.execute(request,{onProgress:progress=>self.postMessage({type:'progress',id:data.id,progress})});
      self.postMessage({type:'result',id:data.id,result,memoryBytes:engine.memoryBytes(),runtimeArtifact:engine.artifactIdentity,runtimeArtifacts:engine.runtimeArtifacts,...runtimeRecovery()});
    } else throw new Error('Unknown worker request.');
  } catch(error) { if(initToken!==null&&initToken!==initialization)return;let failure;try{if(data.type==='run')failure=await makeWorkerFailure(data.request,engine?.artifactIdentity??null,error);}catch{}self.postMessage({type:'error',id:data.id,error:String(error?.message??error),...runtimeRecovery(),...(failure?{failure}:{}),...(error?.source_preparation?{source_preparation:error.source_preparation}:{}),...(error?.native_target_actions?{native_target_actions:error.native_target_actions}:{})}); }
  finally{if(activeRun===token)activeRun=null;if(initToken!==null&&initToken===initialization)initializing=false;}
};
