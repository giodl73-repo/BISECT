import {validateTuiRunWorkflowSettings,acceptedStateWorkflowRequest,stateWorkflowEntries} from './state-workflow-files.js';
import {validateSourceFile,exactSourceText} from './package-files.js';
import {validateStatisticsInputs} from './statistics-files.js';
import {validateElectionAnalysisSettings} from './election-analysis-settings.js';
import {validateElectionSourceFormat} from './election-source-format.js';
import {preparedEnginePlanSources} from './prepared-engine-results.js';
import {stateCommandArguments} from './state-command-files.js';
const sourceKeys=['statisticsInputs','splitsGeography','electionAnalysisSource','electionAnalysisSettings','electionCsvFormat'];
export function validateNativeStateFollowup(value){
 if(!value||Object.keys(value).sort().join()!=='settings,sources'||!value.sources||Array.isArray(value.sources)||Object.keys(value.sources).some(k=>!sourceKeys.includes(k)))throw new Error('Invalid native State follow-up recipe.');
 const s=validateTuiRunWorkflowSettings(value.settings);if(!s.analysis&&!s.map&&!s.report)throw new Error('Select at least one State follow-up stage.');
 for(const [key,source]of Object.entries(value.sources)){
  if(key==='statisticsInputs')validateStatisticsInputs(source);
  else if(key==='electionAnalysisSettings')validateElectionAnalysisSettings(source);
  else if(key==='electionCsvFormat')validateElectionSourceFormat(source);
  else {validateSourceFile(source,{maxBytes:3*1024*1024});exactSourceText(source);}
 }return value;
}
export function nativeStateFollowup(files){
 const settings=files.tuiRunWorkflowSettings;if(!settings)return undefined;validateTuiRunWorkflowSettings(settings);if(!settings.analysis&&!settings.map&&!settings.report)return undefined;
 return structuredClone(validateNativeStateFollowup({settings,sources:Object.fromEntries(sourceKeys.filter(k=>files[k]!==undefined).map(k=>[k,files[k]]))}));
}
export async function executeNativeStateFollowup(engine,result,recipe,createdAt,{onProgress}={}){
 // The accepted Run is already complete: optional follow-up failures must not
 // discard its request, assignment, seed, source receipt or physical exports.
 try{
  validateNativeStateFollowup(recipe);const exports=preparedEnginePlanSources(result.prepared);
  onProgress?.({stage:'native-state-followup',message:'Analyzing and rendering the accepted State plan'});
  const request=acceptedStateWorkflowRequest({...recipe.sources,...exports,...(result.geometry?{planGeometry:result.geometry}:{})},recipe.settings,createdAt),binding=result.native_state?.recipe.workflow_binding,adjacency=binding?.name_effects?result.native_state.recipe.source_inputs?.preparation?.input?.sources?.adjacency:null;
  if(adjacency){const bytes=Uint8Array.from(atob(adjacency.base64),c=>c.charCodeAt(0)),digest=await crypto.subtle.digest('SHA-256',bytes);request.input.native_named_source={arguments:stateCommandArguments(binding.source_command),adjacency_source_name:adjacency.name,adjacency_sha256:Array.from(new Uint8Array(digest),b=>b.toString(16).padStart(2,'0')).join('')};}
  const reply=engine.execute(request,{onProgress});
  if(reply.schema_version!=='bisect-accepted-state-workflow-v1'||reply.partition_execution_performed!==false)throw new Error('Invalid accepted State follow-up response.');
  await stateWorkflowEntries(reply.workflow,{document_json:request.input.document_json,context_json:request.input.context_json});result.workflow=reply.workflow;
 }catch(error){result.workflow_error=String(error.message).slice(0,2000);}
 return result;
}

export function nativeStateFollowupStatus(result){
 if(!(result?.native_state??result?.native_states??result?.native_run)?.recipe?.followup)return undefined;
 if(result.workflow_error)return {status:'failed',error:result.workflow_error};
 if(!result.workflow)return {status:'failed',error:'Selected follow-up stages returned no receipt.'};
 return {status:result.workflow.status,stages:result.workflow.stages.map(({name,status})=>({name,status}))};
}

export async function nativeStateFollowupSources(result){
 if(!result.workflow)return {manifest:null,analysis:{}};
 preparedEnginePlanSources(result.prepared);await stateWorkflowEntries(result.workflow,result.prepared.export);
 const files=result.workflow.files_base64,manifest=files['manifest.json'];if(!manifest)throw new Error('Selected State follow-up has no native manifest.');
 const analysis=Object.fromEntries(Object.entries(files).filter(([path])=>path.startsWith('analysis/')&&path.endsWith('.json')).map(([path,base64])=>[path.slice(9),{name:path.slice(9),base64}]));
 return {manifest:{name:'manifest.json',base64:manifest},analysis};
}
