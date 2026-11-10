import {createNativeStateSourceJob} from './native-state-sources.js';
import {RUN_COMMAND_DEFAULTS,STATES_COMMAND_DEFAULTS,STATE_COMMAND_DEFAULTS,validateStateCommandInputs} from './state-command-files.js';
import {STATE_GRAPH_DEFAULTS,validateStateGraphInputs} from './state-graph-files.js';
import {validatePreparedCensusTemplateInputs} from './prepared-census-job.js';
import {acquireFetchSource,publicFetchUrl,validateFetchAcquisition,validateFetchArchiveSource} from './fetch-acquisition.js';
import {createStateSourceJob} from './state-source-job.js';
import {censusPreparationRequest} from './tract-input-files.js';
import {configValidationRequest} from './config-files.js';
import {validateConfigRunSettings} from './config-run.js';
import {validateTuiRunSettings} from './tui-run-files.js';
import {validateTuiRunWorkflowSettings} from './state-workflow-files.js';
import {appendStateRunJob,validateStateRunBatchInputs} from './state-run-batch.js';
import {selectedStateWorkflowBinding} from './native-state-workflow.js';
import {preflightStateSourceFlags} from './state-source-preflight.js';

export function validateStateSourceExecution(files,runSettings,formSettings,workflowSettings){
 if(files.nativeStateSourceSettings?.enabled&&!['run','states'].includes(files.nativeStateSourceSettings.command_kind)&&files.stateWorkflowBinding?.source_override)throw new Error('The --adjacency handoff selects local native binary sources. Queue those selected sources; it cannot acquire or substitute Census sources.');
 if(files.nativeStateSourceSettings?.enabled&&!['run','states'].includes(files.nativeStateSourceSettings.command_kind)&&files.stateWorkflowBinding){const b=selectedStateWorkflowBinding(files,files.stateCommandInputs??STATE_COMMAND_DEFAULTS),metadata=(files.stateGraphInputs??STATE_GRAPH_DEFAULTS).settings;if(b.settings.label!==null&&metadata.label!==b.settings.label)throw new Error('State workflow export label changed. Reapply or detach the binding before acquiring sources.');}
 if(files.nativeStateSourceSettings?.enabled){validateStateCommandInputs(files.nativeStateSourceSettings.command_kind==='run'?(files.runCommandInputs??RUN_COMMAND_DEFAULTS):files.nativeStateSourceSettings.command_kind==='states'?(files.statesCommandInputs??STATES_COMMAND_DEFAULTS):(files.stateCommandInputs??STATE_COMMAND_DEFAULTS));const inputs=files.stateGraphInputs??STATE_GRAPH_DEFAULTS;validateStateGraphInputs(inputs);if(inputs.settings.chamber!=='congressional'||!inputs.settings.created_at.trim())throw new Error('Record congressional State-flag export metadata before acquiring sources.');return;}
 if(files.preparedCensusTemplateInputs?.enabled){const input=validatePreparedCensusTemplateInputs(files.preparedCensusTemplateInputs);if(!input.source||input.settings.chamber!=='congressional'||!input.settings.created_at.trim())throw new Error('Select an engine template and record congressional export metadata before acquiring sources.');return;}
 configValidationRequest(files.configSource,files.configSettings||{profile:'open'});validateConfigRunSettings({...runSettings,districts:1,chamber:'congressional'});validateTuiRunSettings(formSettings);validateTuiRunWorkflowSettings(workflowSettings);
}
export function stateSourceAcquisitionChoices(plan){
 if(plan?.schema_version!=='bisect-fetch-source-plan-v1'||plan.status!=='planned'||plan.acquisition_performed!==false)return [];
 const scopes=new Map();for(const item of plan.items||[])if(['tiger','pl94171'].includes(item.kind)){const key=item.state_code+'/'+item.year;if(!scopes.has(key))scopes.set(key,{state:item.state_code,year:item.year});}
 return [...scopes.values()].filter(scope=>{try{selectedPair(plan,scope);return true;}catch{return false;}});
}
function selectedPair(plan,{state,year}){
 if(plan?.schema_version!=='bisect-fetch-source-plan-v1'||plan.status!=='planned'||plan.acquisition_performed!==false||!/^[a-f0-9]{64}$/.test(plan.manifest_sha256))throw new Error('Acquire state sources from a fresh native Fetch plan.');
 const pair=['tiger','pl94171'].map(kind=>{const matching=(plan.items||[]).map((item,index)=>({item,index})).filter(({item})=>item.state_code===state&&item.year===year&&item.kind===kind);if(matching.length!==1)throw new Error('Select one matching TIGER/PL94 pair for the state and vintage.');publicFetchUrl(matching[0].item);return matching[0];});return pair;
}
export function validateStateSourceAcquisition(value){
 if(!value||Object.keys(value).sort().join()!=='archives,manifest_sha256,receipts,schema_version'||value.schema_version!=='bisect-state-source-acquisition-v1'||!/^[a-f0-9]{64}$/.test(value.manifest_sha256)||!Array.isArray(value.receipts)||value.receipts.length!==2||!Array.isArray(value.archives)||value.archives.length!==2)throw new Error('Invalid paired state acquisition evidence.');
 for(let i=0;i<2;i++){const receipt=validateFetchAcquisition(value.receipts[i]),archive=validateFetchArchiveSource(value.archives[i]);if(receipt.kind!==['tiger','pl94171'][i]||receipt.availability!=='acquired'||receipt.state!==archive.state||receipt.year!==archive.year||receipt.kind!==archive.kind||receipt.url!==archive.url||receipt.byte_length!==archive.source.base64.length/4*3-(archive.source.base64.endsWith('==')?2:archive.source.base64.endsWith('=')?1:0))throw new Error('State acquisition evidence differs from its retained archive.');}
 if(value.receipts[0].state!==value.receipts[1].state||value.receipts[0].year!==value.receipts[1].year)throw new Error('Paired state acquisition scope differs.');return value;
}

// The network path and explicit saved-source verification build precisely the
// same job from decoded archives. This helper performs no acquisition or Run.
export async function stateSourceJobFromAcquired(snapshot,acquired,{retainRawArchive=true}={}){
 const boundScope=snapshot.scope,tiger=acquired[0].inputs.archive,pl94=acquired[1].inputs.archive;
 if(tiger.kind!=='tiger'||pl94.kind!=='pl94'||tiger.year!==Number(boundScope.year)||pl94.year!==Number(boundScope.year))throw new Error('Acquired state archive scope differs.');
 const sources={...snapshot.files,tigerInputs:{kind:'tracts',year:Number(boundScope.year),shp:tiger.sources.shp,dbf:tiger.sources.dbf,shx:tiger.sources.shx||null},pl94Inputs:{year:Number(boundScope.year),geography:pl94.sources.geography,population:pl94.sources.population||null},tractPreparationSettings:{state:boundScope.state,projection:tiger.sources.prj||null,...(snapshot.files.tractPreparationSettings?.geometry_profile?{geometry_profile:snapshot.files.tractPreparationSettings.geometry_profile}:{})}};
 censusPreparationRequest(sources);
 const job=snapshot.files.nativeStateSourceSettings?.enabled?await createNativeStateSourceJob(sources,'census'):createStateSourceJob(sources,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);
 if(retainRawArchive)job.acquisition=validateStateSourceAcquisition({schema_version:'bisect-state-source-acquisition-v1',manifest_sha256:snapshot.plan.manifest_sha256,receipts:acquired.map(a=>a.receipt),archives:acquired.map(a=>a.raw_archive_source)});
 return job;
}

// Stage both sources together. No project mutation, engine execution or cache
// activation occurs here; the caller commits the returned job after validation.
export async function acquireStateSourceJob(plan,scope,files,runSettings,formSettings,workflowSettings,{signal,onProgress=()=>{},...options}={}){
 const snapshot=structuredClone({plan,scope,files,runSettings,formSettings,workflowSettings}),selected=selectedPair(snapshot.plan,snapshot.scope);
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);
 await preflightStateSourceFlags(snapshot.files,{...options,signal});
 signal?.throwIfAborted();const controller=new AbortController(),abort=()=>controller.abort(signal.reason);signal?.addEventListener('abort',abort,{once:true});
 try{
  const acquired=await Promise.all(selected.map(async({item,index})=>{onProgress({stage:'acquisition',state:item.state_code,year:item.year,kind:item.kind});const output=await acquireFetchSource(snapshot.plan,index,{...options,signal:controller.signal});if(output.receipt.availability!=='acquired'||!output.inputs?.archive||options.retainRawArchive!==false&&!output.raw_archive_source)throw new Error('Required '+item.kind+' source is unavailable for '+item.state_code+'/'+item.year+'. No state queued.');return output;}));
  signal?.throwIfAborted();const job=await stateSourceJobFromAcquired(snapshot,acquired,{retainRawArchive:options.retainRawArchive!==false});signal?.throwIfAborted();return job;
 }catch(error){controller.abort();throw error;}finally{signal?.removeEventListener('abort',abort);}
}

// Serial acquisition bounds transient ZIP/decoded-source memory. A checkpoint
// contains only fully validated jobs; one state's failure cannot erase others.
export async function acquireStateSourceBatch(plan,year,inputs,files,runSettings,formSettings,workflowSettings,{signal,onCheckpoint=()=>{},onProgress=()=>{},...options}={}){
 const snapshot=structuredClone({plan,year,inputs:validateStateRunBatchInputs(inputs),files,runSettings,formSettings,workflowSettings});
 const scopes=stateSourceAcquisitionChoices(snapshot.plan).filter(scope=>scope.year===snapshot.year);
 if(!scopes.length)throw new Error('Select a vintage with supported state pairs in a fresh Fetch plan.');
 validateStateSourceExecution(snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings);
 const inspectNativeCommand=await preflightStateSourceFlags(snapshot.files,{...options,signal});
 let queued=snapshot.inputs;const rows=[];
 const checkpoint=status=>({schema_version:'bisect-state-source-batch-acquisition-v1',status,year:snapshot.year,manifest_sha256:snapshot.plan.manifest_sha256,inputs:structuredClone(queued),states:structuredClone(rows),summary:{total:scopes.length,completed:rows.length,queued:rows.filter(row=>row.status==='queued').length,failed:rows.filter(row=>row.status==='failed').length},execution_performed:false});
 onCheckpoint(checkpoint('acquiring'));
 for(const scope of scopes){
  if(signal?.aborted)return checkpoint('cancelled');
  try{
   const job=await acquireStateSourceJob(snapshot.plan,scope,snapshot.files,snapshot.runSettings,snapshot.formSettings,snapshot.workflowSettings,{...options,...(inspectNativeCommand?{inspectNativeCommand}:{}),signal,onProgress});
   if(signal?.aborted)return checkpoint('cancelled');
   queued=appendStateRunJob(queued,job);rows.push({state:scope.state,year:scope.year,status:'queued'});
  }catch(error){if(signal?.aborted)return checkpoint('cancelled');rows.push({state:scope.state,year:scope.year,status:'failed',error:scope.state+': '+error.message});}
  onCheckpoint(checkpoint('acquiring'));
 }
 return checkpoint('completed');
}
