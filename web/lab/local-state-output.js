import {validateStreamNativeOutputSettings,nativeStateOutputSource,nativeOutputHash} from './native-state-output.js';
import {reviewNativeStateInstallation,installNativeStateOutput} from './native-state-install.js';
import {nativeStateFollowupStatus} from './native-state-followup.js';
import {validateNativeStateJob} from './native-state-job.js';
import {validatePreparedCensusJob} from './prepared-census-job.js';
import {validateFetchAcquisition} from './fetch-acquisition.js';
import {validateStateSourceJob,stateJobScope} from './state-source-job.js';
import {stateRunBatchZip} from './state-run-batch.js';
import {createWorkbenchArchive,createStoredWorkbenchArchive} from './workbench-archive.js';
import {encodeBytes} from './package-files.js';

const jsonBytes=value=>new TextEncoder().encode(JSON.stringify(value,null,2)+'\n');
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
function stateSourceWorkbenchData(job,output){
 if(job.native_state){validateNativeStateJob(job);const source=job.native_state.source_inputs,p=source.preparation.input,native=source.preparation.operation==='prepare-native-engine-input';return {name:job.scope.state+' '+job.scope.year+' original State flag sources',files:{...(['match-native-flags','match-native-adjacency-flags'].includes(job.native_state.source_scope_policy)?{nativeSelectedStateSourceSettings:{kind:native?'native':source.preparation.operation==='prepare-tract-engine-input'?'tract':'census'}}:{}),nativeStateSourceSettings:{enabled:true,...(job.native_state.command_kind?{command_kind:job.native_state.command_kind}:{})},[job.native_state.command_kind==='run'?'runCommandInputs':job.native_state.command_kind==='states'?'statesCommandInputs':'stateCommandInputs']:job.native_state.command,...(job.native_state.workflow_binding?{stateWorkflowBinding:job.native_state.workflow_binding,...(!job.native_state.followup?{tuiRunWorkflowSettings:{analysis:false,map:false,report:false,dpi:job.native_state.workflow_binding.settings.dpi}}:{}),nativeStateOutputInputs:{source:null,version:job.native_state.workflow_binding.settings.version,layout:job.native_state.workflow_binding.settings.layout},streamNativeOutputSettings:{enabled:job.native_state.workflow_binding.settings.manifest_requested,version:job.native_state.workflow_binding.settings.version,layout:job.native_state.workflow_binding.settings.layout}}:{}),...(job.native_state.followup?{...job.native_state.followup.sources,tuiRunWorkflowSettings:job.native_state.followup.settings}:{}),stateGraphInputs:{graph:null,auxiliary:source.auxiliary,settings:source.settings},...(native?{nativeAdjacencyInputs:p.sources,nativeAdjacencySettings:Object.fromEntries(Object.entries(p).filter(([k])=>k!=='sources'))}:{tigerInputs:p.tiger,pl94Inputs:p.pl94,tractPreparationSettings:{state:p.state,projection:p.projection,...(p.geometry_profile?{geometry_profile:p.geometry_profile}:{})}}),...(job.geometry?{planGeometry:job.geometry}:{})},operation:['match-native-flags','match-native-adjacency-flags'].includes(job.native_state.source_scope_policy)?'run-native-selected-state-source':source.preparation.operation,result:null,lastOperation:null};}

 if(job.prepared_engine_template){validatePreparedCensusJob(job);const p=job.source_preparation;return {name:job.scope.state+' '+job.scope.year+' retained template sources',files:{tigerInputs:p.tiger,pl94Inputs:p.pl94,tractPreparationSettings:{state:p.state,projection:p.projection,...(p.geometry_profile?{geometry_profile:p.geometry_profile}:{})},preparedCensusTemplateInputs:{enabled:true,use_display_geometry:!!job.geometry,...job.prepared_engine_template},...(job.geometry?{planGeometry:job.geometry}:{})},operation:'prepare-census-engine-input',result:null,lastOperation:null};}
 validateStateSourceJob(job);const preparation=job.source_preparation,c=job.configuration;
 const files={tigerInputs:preparation.tiger,pl94Inputs:preparation.pl94,tractPreparationSettings:{state:preparation.state,projection:preparation.projection,...(preparation.geometry_profile?{geometry_profile:preparation.geometry_profile}:{})},configSource:c.source,configSettings:c.profile,configRunInputs:c.inputs,configRunSettings:{...c.settings,districts:output.result.configured_run.result.options.districts},tuiRunSettings:{form:job.form,created_at:job.created_at,...(job.target?{target:{force:job.target.force,source:{name:job.target.source_name,base64:encodeBytes(new TextEncoder().encode(job.target.folder_json))}}}:{})},...(job.followup?{...job.followup.sources,tuiRunWorkflowSettings:job.followup.settings}:{})};
 return {name:job.form.location+' '+job.form.year+' retained sources',files,operation:'prepare-census-engine-input',result:null,lastOperation:null};
}
export function stateSourceWorkbench(job,output){return new TextEncoder().encode(createWorkbenchArchive(stateSourceWorkbenchData(job,output)));}
export async function stateSourceWorkbenchBytes(job,output,{nativeOutputSettings}={}){const data=stateSourceWorkbenchData(job,output);if(nativeOutputSettings)data.files.streamNativeOutputSettings=structuredClone(validateStreamNativeOutputSettings(nativeOutputSettings));const stored=await createStoredWorkbenchArchive(data);return typeof stored==='string'?new TextEncoder().encode(stored):stored;}
async function storedStateRecipe(job){
 const bytes=jsonBytes([job]);
 if(bytes.length<=128*1024*1024)return {name:bytes.length<=64*1024*1024?'state-run-jobs.json':'state-source-recipe.json',bytes};
 if(bytes.length>256*1024*1024)throw new Error('Individual state recipe decoded JSON exceeds 256 MiB.');
 if(typeof CompressionStream!=='function')throw new Error('This browser cannot compress a larger state recipe.');
 const reader=new Blob([bytes]).stream().pipeThrough(new CompressionStream('gzip')).getReader(),chunks=[];let total=0;
 try{for(;;){const {done,value}=await reader.read();if(done)break;total+=value.byteLength;if(total>128*1024*1024)throw new Error('Compressed state recipe exceeds 128 MiB.');chunks.push(value);}}
 catch(error){await reader.cancel().catch(()=>{});throw error;}finally{reader.releaseLock();}
 const compressed=new Uint8Array(total);let offset=0;for(const chunk of chunks){compressed.set(chunk,offset);offset+=chunk.byteLength;}
 return {name:'state-source-recipe.json.gz',bytes:compressed};
}
async function freshDirectory(parent,name){
 try{await parent.getDirectoryHandle(name);throw new Error('Output directory already exists: '+name);}catch(error){if(error.name!=='NotFoundError')throw error;}
 return parent.getDirectoryHandle(name,{create:true});
}
async function writeFile(directory,name,bytes,signal){
 signal?.throwIfAborted();const handle=await directory.getFileHandle(name,{create:true}),stream=await handle.createWritable();
 try{signal?.throwIfAborted();await stream.write(bytes);signal?.throwIfAborted();await stream.close();}catch(error){await stream.abort().catch(()=>{});throw error;}
 // Verify what the filesystem reports after close, before a completion marker.
 const file=await handle.getFile(),actual=new Uint8Array(await file.arrayBuffer()),expected=await hash(bytes);
 if(actual.length!==bytes.length||await hash(actual)!==expected)throw new Error('Saved output verification failed: '+name);
 return {name,byte_length:bytes.length,sha256:expected};
}

// A new application-owned folder avoids modifying selected existing outputs.
// File close and readback precede the marker; this is not an atomic folder rename.
export async function createLocalStateYearsOutput(parent,{name='bisect-run-'+crypto.randomUUID(),nativeOutputSettings}={}){
 if(!parent||parent.kind!=='directory'||!/^bisect-run-[a-zA-Z0-9-]{1,80}$/.test(name))throw new Error('Choose a local output directory and a fresh run folder name.');
 const settings=nativeOutputSettings?structuredClone(validateStreamNativeOutputSettings(nativeOutputSettings)):null;
 const directory=await freshDirectory(parent,name),years=new Set();
 return {name,...(settings?{nativeOutputSettings:settings}:{}),async year(year){
  if(!['2020','2010','2000'].includes(year)||years.has(year))throw new Error('Select each native Run vintage once.');
  years.add(year);return createLocalStateOutput(await freshDirectory(directory,year),{name:'bisect-run-states',nativeOutputSettings:settings});
 },async checkpoint(receipt){return writeFile(directory,'run-years-receipt.json',jsonBytes(receipt));}};
}
export async function createLocalStateOutput(parent,{name='bisect-run-'+crypto.randomUUID(),nativeOutputSettings}={}){
 if(!parent||parent.kind!=='directory'||!/^bisect-run-[a-zA-Z0-9-]{1,80}$/.test(name))throw new Error('Choose a local output directory and a fresh run folder name.');
 const nativeSettings=nativeOutputSettings?structuredClone(validateStreamNativeOutputSettings(nativeOutputSettings)):null;
 const directory=await freshDirectory(parent,name),states=new Map();
 const stateDirectory=state=>{if(!states.has(state))states.set(state,{directory:freshDirectory(directory,state),archives:new Map()});return states.get(state);};
 return {name,...(nativeSettings?{nativeOutputSettings:structuredClone(nativeSettings)}:{}),
  async archive(value,{signal}={}){const {state,year,kind,bytes,receipt}=value;validateFetchAcquisition(receipt);if(!/^[A-Z]{2}$/.test(state)||receipt.state!==state||receipt.year!==year||receipt.kind!==kind||!['tiger','pl94171'].includes(kind)||!(bytes instanceof Uint8Array)||bytes.length!==receipt.byte_length||await hash(bytes)!==receipt.source_sha256)throw new Error('Invalid original archive identity.');const owned=stateDirectory(state);if(owned.archives.has(kind))throw new Error('Archive already saved for this state.');owned.archives.set(kind,null);const artifact=await writeFile(await owned.directory,kind+'.zip',bytes,signal);owned.archives.set(kind,{artifact,receipt:structuredClone(receipt)});return artifact;},
  async checkpoint(receipt,{signal}={}){return writeFile(directory,'batch-receipt.json',jsonBytes(receipt),signal);},
  async state(job,output,{signal,nativeOutput,nativeOutputError,execute}={}){
   if(job.native_state)validateNativeStateJob(job);else if(job.prepared_engine_template)validatePreparedCensusJob(job);else validateStateSourceJob(job);const scope=stateJobScope(job),state=scope.state;
   if(!/^[A-Z]{2}$/.test(state)||((job.native_state||job.prepared_engine_template)?(output?.result?.schema_version!=='bisect-prepared-state-run-v1'||output.result.scope.state!==state||output.result.scope.year!==scope.year):(output?.result?.schema_version!=='bisect-tui-run-v1'||output.result.form.location!==state||output.result.form.year!==scope.year))||output.source_preparation?.state!==state)throw new Error('Saved state output scope differs from its recipe.');
   const recipe=await storedStateRecipe(job);
   signal?.throwIfAborted();const owned=stateDirectory(state),target=await owned.directory,retained=['tiger','pl94171'].map(kind=>owned.archives.get(kind));if(retained.some(value=>!value||value.receipt.state!==state||value.receipt.year!==scope.year))throw new Error('Both matching original archives must be saved before completing a state.');const artifacts=retained.map(value=>value.artifact);artifacts.push(await writeFile(target,'acquisition-receipts.json',jsonBytes(retained.map(value=>value.receipt)),signal));
   artifacts.push(await writeFile(target,recipe.name,recipe.bytes,signal));
   artifacts.push(await writeFile(target,'source-project.bisect-workbench',await stateSourceWorkbenchBytes(job,output,{nativeOutputSettings:nativeSettings}),signal));
   const bytes=await stateRunBatchZip({schema_version:'bisect-state-run-batch-v1',status:'completed',states:[{state_code:state,status:'ok',result:output.result,source_preparation:output.source_preparation}]});
   artifacts.push(await writeFile(target,'state-output.zip',bytes,signal));
   let nativeSummary;
   if(nativeSettings?.enabled){
    try{
     if(nativeOutputError)throw new Error(nativeOutputError);
     if(!nativeOutput||typeof execute!=='function'||nativeOutput.version!==nativeSettings.version||(nativeOutput.layout??'labeled')!==nativeSettings.layout||nativeOutput.state!==state||nativeOutput.year!==scope.year)throw new Error('Native output package differs from the selected stream scope/settings.');
     const acceptedSource=nativeStateOutputSource(output.result),sourceBytes=Uint8Array.from(atob(acceptedSource.base64),c=>c.charCodeAt(0));
     if(nativeOutput.source_name!==acceptedSource.name||nativeOutput.source_sha256!==await nativeOutputHash(sourceBytes))throw new Error('Native output source differs from the retained accepted Run.');
     const review=await reviewNativeStateInstallation(execute,directory,nativeOutput,{signal});
     const installed=await installNativeStateOutput(execute,review,{signal});
     nativeSummary={status:'completed',target_namespace:installed.target_namespace,package_sha256:installed.package_sha256,install_receipt_sha256:installed.install_receipt_sha256};
    }catch(error){signal?.throwIfAborted();nativeSummary={status:'failed',...(nativeOutput?.target_namespace?{target_namespace:nativeOutput.target_namespace}:{}),error:String(error.message).slice(0,2000),accepted_output_retained:true};artifacts.push(await writeFile(target,'native-output-error.json',jsonBytes(nativeSummary),signal));}
   }
   const receipt={schema_version:'bisect-local-state-output-v1',state,year:scope.year,...(nativeSummary?{native_output:nativeSummary}:{}),...(nativeStateFollowupStatus(output.result)?{followup:nativeStateFollowupStatus(output.result)}:{}),seed:(job.native_state||job.prepared_engine_template)?output.result.source_review.engine_request.options.seed:job.form.seed,effective_seed:(job.native_state||job.prepared_engine_template)?(output.result.prepared.executed_request.request??output.result.prepared.executed_request).options.seed:output.result.configured_run.result.options.seed,artifacts};
   artifacts.push(await writeFile(target,'completed.json',jsonBytes(receipt),signal));
   return {directory:state,artifacts,...(nativeSummary?{native_output:nativeSummary}:{})};
  }
 };
}
