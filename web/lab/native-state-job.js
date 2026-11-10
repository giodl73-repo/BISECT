import {stateWorkflowExportLabel} from './native-state-name-effects.js';
import {stateDiagnosticObserver} from './native-state-diagnostics.js';
import {requireNativeStateTargetAuthority} from './native-state-target-authority.js';
import {validateNativeStateFollowup,executeNativeStateFollowup} from './native-state-followup.js';
import {validateNativeStateSourceInputs,nativeStatePreparationScope} from './native-state-sources.js';
import {stateGraphFromPreparedResult} from './state-graph-files.js';
import {validateStateSourceAcquisition} from './state-source-acquisition.js';
import {stateCommandArguments,statePreparedFlagRefusals,validateStateCommandInputs,stateCommandPreparationRequest,preparedEngineFromStateCommand} from './state-command-files.js';
import {validateStateGraphInputs,stateGraphRequest,preparedEngineFromStateGraph} from './state-graph-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from './prepared-state-job.js';
import {validateSourceFile} from './package-files.js';
import {characterStateFips} from './character-input.js';
import {validateStateWorkflowBinding,recheckStateWorkflowBinding} from './native-state-workflow.js';
import {executeNativeAdjacencyStateJob} from './native-adjacency-state-job.js';
import {emitNativeReportSourceWarning,applyNativeStateReportSources} from './native-state-report-pipeline.js';
const keys=(v,required,optional=[])=>v&&typeof v==='object'&&!Array.isArray(v)&&required.every(k=>Object.hasOwn(v,k))&&Object.keys(v).every(k=>required.includes(k)||optional.includes(k));
const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export function validateNativeStateJob(job){
 const source=Object.hasOwn(job?.native_state??{},'source_inputs');
 const custom=source&&job.native_state.source_scope_policy==='match-native-adjacency-flags';
 const selectedScope=source&&['match-native-flags','match-native-adjacency-flags'].includes(job.native_state.source_scope_policy);
 const count=job?.scope?.districts,validCount=source&&count===null||Number.isSafeInteger(count)&&count>=1&&count<=500;
 const validRequest=source?job?.request_sha256===null:/^[a-f0-9]{64}$/.test(job?.request_sha256??'');
 if(!keys(job,['native_state','scope','request_sha256','source_sha256'],source?['geometry','acquisition']:['geometry'])||!keys(job.scope,['state','year','districts','chamber'])||(custom?!/^[A-Z]{2}$/.test(job.scope.state)||!/^\d{4}$/.test(job.scope.year):!characterStateFips(job.scope.state)||!['2000','2010','2020'].includes(job.scope.year))||(selectedScope?!['congressional','house','senate'].includes(job.scope.chamber):job.scope.chamber!=='congressional')||!validCount||!validRequest||!/^[a-f0-9]{64}$/.test(job.source_sha256??''))throw new Error('Native State jobs require matching native state/year/chamber/district scope and reviewed source/request identities.');
 const recipe=job.native_state,graph=Object.hasOwn(recipe??{},'graph_inputs');
 if(!keys(recipe,['command',source?'source_inputs':graph?'graph_inputs':'prepared_inputs'],['followup','command_kind','workflow_binding','source_scope_policy']))throw new Error('Retain State flags and exactly one original graph or prepared-input recipe.');
 if(Object.hasOwn(recipe,'source_scope_policy')&&(!selectedScope||recipe.command_kind||job.acquisition))throw new Error('Matching State scope is for one selected source, not native Run/States or acquired national queue jobs.');
 if(Object.hasOwn(recipe,'command_kind')&&(!['states','run'].includes(recipe.command_kind)||!source))throw new Error('Native Run/States flags require original tract source inputs.');
 if(Object.hasOwn(recipe,'followup'))validateNativeStateFollowup(recipe.followup);
 validateStateCommandInputs(recipe.command);const inputs=source?validateNativeStateSourceInputs(recipe.source_inputs):graph?validateStateGraphInputs(recipe.graph_inputs):validatePreparedEngineInputs(recipe.prepared_inputs);
 if(custom&&(inputs.preparation.operation!=='prepare-native-engine-input'||!['precinct','imported'].includes(inputs.preparation.input.unit_kind)||recipe.workflow_binding&&!recipe.workflow_binding.source_override))throw new Error('Custom State sources require original native adjacency inputs; any workflow binding must retain their selected adjacency identity.');
 if(recipe.workflow_binding){const b=validateStateWorkflowBinding(recipe.workflow_binding);if(b.name_effects){const i=b.name_effects.preview.identity;if(i.state_code!==job.scope.state||i.effective_year!==job.scope.year||i.chamber!==job.scope.chamber)throw new Error('Native name identity belongs to a different selected state, year or chamber. Review each source scope explicitly.');}if(b.source_override&&(!source||inputs.preparation.operation!=='prepare-native-engine-input'||job.acquisition))throw new Error('An --adjacency binding requires its selected native binary source recipe, not a graph/template or acquired Census sources.');if(recipe.command_kind||JSON.stringify(stateCommandArguments(recipe.command))!==JSON.stringify(stateCommandArguments(b.engine_command))||stateWorkflowExportLabel(b)!==null&&inputs.settings.label!==stateWorkflowExportLabel(b)||recipe.followup&&recipe.followup.settings.dpi!==b.settings.dpi)throw new Error('State workflow flags, export label or follow-up DPI changed. Reapply or detach the binding before queueing.');}
 if((!source&&!(graph?inputs.graph:inputs.source))||inputs.settings.chamber!==job.scope.chamber||!inputs.settings.created_at.trim())throw new Error('Record matching chamber export metadata and source inputs before queuing State flags.');
 if(source){const selected=nativeStatePreparationScope(inputs.preparation);if(selected.state!==job.scope.state||selected.year!==job.scope.year)throw new Error('State source scope changed.');if(job.acquisition){validateStateSourceAcquisition(job.acquisition);if(job.acquisition.receipts[0].state!==job.scope.state||job.acquisition.receipts[0].year!==job.scope.year)throw new Error('Acquisition scope differs from State sources.');}}
 if(Object.hasOwn(job,'geometry'))validateSourceFile(job.geometry,{maxBytes:64*1024*1024});return job;
}
export async function nativeStateJobFromResult(command,inputs,result,{graph=false,geometry,followup,workflow_binding}={}){
 command=structuredClone(command);inputs=structuredClone(inputs);geometry=geometry?structuredClone(geometry):undefined;
 const selected=graph?await preparedEngineFromStateGraph(command,inputs,result):await preparedEngineFromStateCommand(command,inputs,result);
 const review=result.prepared_review,scope={state:review?.state,year:review?.year,districts:review?.engine_request?.options?.districts,chamber:selected.settings.chamber};
 const recipe={command,[graph?'graph_inputs':'prepared_inputs']:inputs,...(followup?{followup:structuredClone(validateNativeStateFollowup(followup))}:{}),...(workflow_binding?{workflow_binding:structuredClone(validateStateWorkflowBinding(workflow_binding))}:{})};
 return structuredClone(validateNativeStateJob({native_state:recipe,scope,request_sha256:result.prepared_request_sha256,source_sha256:await hash(JSON.stringify(recipe)),...(geometry?{geometry}:{})}));
}
export async function executeNativeStateJob(engine,input,{onProgress,nativeTargetAuthority}={}){
 const job=structuredClone(validateNativeStateJob(input)),recipe=job.native_state,run=recipe.command_kind==='run',bulk=run||recipe.command_kind==='states';
 if(await hash(JSON.stringify(recipe))!==job.source_sha256)throw new Error('Queued State flags or original sources changed. Compile and queue them again. No partition executed.');
 if(recipe.workflow_binding)await recheckStateWorkflowBinding(engine,recipe.workflow_binding,recipe.command,{forExecution:true,sourcePreparation:recipe.source_inputs?.preparation,sourceAuxiliary:(recipe.source_inputs??recipe.graph_inputs)?.auxiliary,nativeTargetAuthority});
 if(recipe.workflow_binding?.target_actions)await requireNativeStateTargetAuthority(nativeTargetAuthority,recipe.workflow_binding,{consume:true,sourceSha256:job.source_sha256});
 const observer=stateDiagnosticObserver(recipe.workflow_binding?.diagnostics,onProgress);onProgress=observer.onProgress;
 if(recipe.source_scope_policy==='match-native-adjacency-flags'||recipe.workflow_binding?.source_override){const value=await executeNativeAdjacencyStateJob(engine,job,{onProgress,partitionTiming:observer.partitionTiming});observer.finish(value.result);return value;}
 onProgress?.({stage:'native-state-compilation',message:'Compiling original native State flags and selected sources'});
 let graph=Object.hasOwn(recipe,'graph_inputs'),command=recipe.command,inputs=graph?recipe.graph_inputs:recipe.prepared_inputs,preparation=null,binding=null,display=job.geometry;
 try{
 if(recipe.source_inputs){
  if(!Number.isSafeInteger(job.scope.districts))throw new Error('Resolve native congressional allocation before source execution.');
  const source=recipe.source_inputs,scope=nativeStatePreparationScope(source.preparation);
  if(bulk){
   if(scope.resolution!=='tract')throw new Error('Native Run/States source jobs require tracts.');
   const review=engine.execute({operation:run?'inspect-native-run-command':'inspect-native-states-command',arguments:JSON.parse(command.arguments_json)});
   if(review.status!=='parsed'||review.print_only||(run?!review.years?.includes(scope.year):review.year!==scope.year||!review.profile_filter.allowed||!review.multiscale_levels.allowed)||review.states.length&&!review.states.some(state=>state.toUpperCase()===scope.state))throw new Error('Review native Run/States flags, year and execution intent before running source jobs. Print-only never acquires or partitions.');
  }else if(recipe.source_scope_policy==='match-native-flags'){
   const review=engine.execute({operation:'inspect-native-state-command',arguments:stateCommandArguments(command)}),field=id=>review.fields?.find(f=>f.id===id)?.values?.[0]??null;
   if(review.status!=='parsed'||review.profile_filter?.allowed!==true||statePreparedFlagRefusals(review).length||review.state?.toUpperCase()!==scope.state||review.year!==scope.year||review.resolution!==scope.resolution||field('chamber')!==job.scope.chamber||field('districts')!==null&&Number(field('districts'))!==job.scope.districts)throw new Error('Selected State scope or district count differs from the original native flags. No source prepared or partition executed.');
  }else {
  binding=engine.execute({operation:'scope-native-state-command',arguments:JSON.parse(command.arguments_json),state:scope.state,year:scope.year,resolution:scope.resolution,districts:job.scope.districts});
  if(binding.schema_version!=='bisect-native-state-scoped-command-v1'||binding.algorithm_unchanged!==true||binding.execution_performed!==false||binding.filesystem_reads_performed!==false||binding.scope?.state!==scope.state||binding.scope?.year!==scope.year||binding.scope?.resolution!==scope.resolution||binding.scope?.chamber!=='congressional'||binding.scope?.districts!==job.scope.districts||JSON.stringify(binding.source_arguments)!==JSON.stringify(JSON.parse(command.arguments_json)))throw new Error('Invalid native State source-scope binding.');
  command={arguments_json:JSON.stringify(binding.arguments)};
  const review=engine.execute({operation:'inspect-native-state-command',arguments:binding.arguments}),refused=statePreparedFlagRefusals(review);if(review.status!=='parsed'||review.profile_filter?.allowed!==true||refused.length)throw new Error('Review State flags before source preparation. Refused workflow flags: '+refused.join(', ')+'. No source prepared or partition executed.');
  }
  onProgress?.({stage:'native-state-source-preparation',message:'Preparing selected original source bytes'});
  preparation=engine.execute(source.preparation,{onProgress});
  if(preparation.state!==job.scope.state||preparation.year!==job.scope.year||preparation.execution_performed!==false)throw new Error('Prepared State source scope differs from native allocation. No partition executed.');
  const selected=await stateGraphFromPreparedResult(preparation);graph=true;inputs={...selected.inputs,auxiliary:source.auxiliary,settings:source.settings};display??=selected.geometry;
 }
 let compiled;
 compiled=engine.execute(graph?stateGraphRequest(command,inputs,{bulk:bulk&&!run,run}):stateCommandPreparationRequest(command,inputs,{bulk:bulk&&!run,run}));
 const scope=compiled.prepared_review;
 if(compiled.schema_version!=='bisect-native-state-run-input-v1'||compiled.execution_performed!==false||(job.request_sha256!==null&&compiled.prepared_request_sha256!==job.request_sha256)||scope?.state!==job.scope.state||scope?.year!==job.scope.year||scope?.engine_request?.options?.districts!==job.scope.districts)throw new Error('Native State compilation differs from the queued request or allocation. No partition executed.');
 const selected=graph?await preparedEngineFromStateGraph(command,inputs,compiled,{bulk:bulk&&!run,run}):await preparedEngineFromStateCommand(command,inputs,compiled,{bulk:bulk&&!run,run});
 emitNativeReportSourceWarning(recipe.workflow_binding,onProgress);
 const prepared=await preparedStateJobFromResult(selected,scope,display),result=executePreparedStateJob(engine,prepared,{onProgress,partitionTiming:observer.partitionTiming});
 // Retain the original source recipe once; the prepared Run already retains
 // compiled request bytes, native review, accepted replay and engine exports.
 const receipt=Object.fromEntries(['schema_version','native_command','original_template_sha256','prepared_request_sha256','district_count','embedded_policy_sha256','state_graph_source','scope'].filter(k=>Object.hasOwn(compiled,k)).map(k=>[k,compiled[k]]));
 result[run?'native_run':bulk?'native_states':'native_state']={...(bulk?{schema_version:run?'bisect-native-run-execution-v1':'bisect-native-states-execution-v1',execution_policy:{source_acquisition:'retained-source-bytes',followup_policy:'explicit-browser-workflow-controls',native_followup_flags_applied:false}}:{}),recipe,source_sha256:job.source_sha256,request_sha256:compiled.prepared_request_sha256,compilation:receipt,...(binding?{scope_binding:binding}:{}),...(job.acquisition?{acquisition:job.acquisition}:{})};
 await applyNativeStateReportSources(engine,result,recipe.workflow_binding,{onProgress});
 if(recipe.followup&&result.native_report_sources?.status!=='failed')await executeNativeStateFollowup(engine,result,recipe.followup,inputs.settings.created_at,{onProgress});
 observer.finish(result);
 return preparation?{schema_version:'bisect-native-source-flags-state-run-v1',source_preparation:preparation,result}:result;
 }catch(error){if(preparation)error.source_preparation=preparation;throw error;}
}
