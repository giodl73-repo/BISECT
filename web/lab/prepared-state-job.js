import {validatePartitionTimingReport} from './partition-timing.js';
import {nativeStateDataEntries,nativeStateResultEntries} from './native-state-files.js';
import {validateContextEngineOrigin,contextEngineSourcesEqual} from './context-engine-files.js';
import {validatePreparedEngineInputs,preparedEngineRequest} from './prepared-engine-files.js';
import {preparedEngineMapDocument,preparedEnginePlanSources} from './prepared-engine-results.js';
import {validateSourceFile} from './package-files.js';
import {parseGeometryJson} from './project.js';
import {planMapModel} from './plan-map.js';
import {storedZipBytes} from './reproducibility-files.js';
import {characterStateFips} from './character-input.js';

const keys=(value,required,optional=[])=>value&&typeof value==='object'&&!Array.isArray(value)&&required.every(k=>Object.hasOwn(value,k))&&Object.keys(value).every(k=>[...required,...optional].includes(k));
export function validatePreparedStateJob(job){
 let customScope=false;
 if(job?.scope&&(!characterStateFips(job.scope.state)||!['2000','2010','2020'].includes(job.scope.year))){
  validatePreparedEngineInputs(job.prepared_engine);const value=JSON.parse(job.prepared_engine.source?.text??'null'),graph=value?.operation==='run-engine-build'?value.request?.graph:value?.graph;
  customScope=['precinct','imported'].includes(graph?.unit_kind)&&/^[A-Z]{2}$/.test(job.scope.state)&&/^\d{4}$/.test(job.scope.year)&&graph.state===job.scope.state&&graph.year===job.scope.year;
 }
 if(!keys(job,['prepared_engine','scope','request_sha256'],['geometry'])||!keys(job.scope,['state','year','districts','chamber'])||!customScope&&(!characterStateFips(job.scope.state)||!['2000','2010','2020'].includes(job.scope.year))||!['congressional','house','senate'].includes(job.scope.chamber)||!Number.isSafeInteger(job.scope.districts)||job.scope.districts<1||job.scope.districts>500||!/^[a-f0-9]{64}$/.test(job.request_sha256??''))throw new Error('Prepared state jobs require matching native state/year/chamber/district scope and reviewed request identity.');
 validatePreparedEngineInputs(job.prepared_engine);if(job.prepared_engine.settings.inspect_only!==false||job.prepared_engine.settings.chamber!==job.scope.chamber||!job.prepared_engine.settings.created_at.trim())throw new Error('Queued prepared jobs need explicit execution mode and recorded export metadata.');preparedEngineRequest(job.prepared_engine);
 if(Object.hasOwn(job,'geometry'))validateSourceFile(job.geometry,{maxBytes:64*1024*1024});return job;
}
function matchingContextSource(inputs,result){const selected=inputs.context_source,recorded=result?.context_engine_source;if(selected){if(!recorded||Object.keys(recorded).sort().join()!=='origin,source'||!contextEngineSourcesEqual(selected,recorded.source))throw new Error('Original RCTX source changed since prepared review or run. Review again before queuing.');validateContextEngineOrigin(recorded.origin);if(recorded.origin.source_name!==selected.source_name||Boolean(selected.attributes_source)!==Object.hasOwn(recorded.origin,'attributes_source')||selected.attributes_source&&(recorded.origin.attributes_source.source_name!==selected.attributes_source.source_name||recorded.origin.attributes_source.bytes!==new TextEncoder().encode(selected.attributes_source.attributes_json).length))throw new Error('Original attribute source identity changed.');}else if(recorded!==undefined)throw new Error('Prepared result has an unrequested original RCTX attachment.');}
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function preparedStateJobFromResult(inputs,result,geometry){
 inputs=structuredClone(validatePreparedEngineInputs(inputs));geometry=geometry?structuredClone(geometry):undefined;const reviewedHash=result?.request_sha256;if(!['bisect-prepared-engine-request-v1','bisect-prepared-engine-v1'].includes(result?.schema_version)||!inputs.source)throw new Error('Review or run the selected prepared request before queuing it.');
 const review=result.schema_version==='bisect-prepared-engine-request-v1',request=result.run_kind==='multiscale'?result.executed_request?.request:result.executed_request;
 const scope={state:review?result.state:request?.graph?.state,year:review?result.year:request?.graph?.year,districts:review?result.engine_request?.options?.districts:request?.options?.districts,chamber:inputs.settings.chamber};
 matchingContextSource(inputs,result);if(inputs.context_source&&await digest(inputs.context_source.context_json)!==result.context_engine_source.origin.source_sha256)throw new Error('Original RCTX bytes disagree with the reviewed source hash.');if(inputs.context_source?.attributes_source&&await digest(inputs.context_source.attributes_source.attributes_json)!==result.context_engine_source.origin.attributes_source.source_sha256)throw new Error('Original attribute bytes disagree with the reviewed source hash.');
 if(await digest(inputs.source.text)!==reviewedHash)throw new Error('Prepared request changed since review. Review it again before queuing.');
 return structuredClone(validatePreparedStateJob({prepared_engine:{...inputs,settings:{...inputs.settings,inspect_only:false}},scope,request_sha256:reviewedHash,...(geometry?{geometry}:{})}));
}
export function executePreparedStateJob(engine,job,{onProgress,partitionTiming=false}={}){
 validatePreparedStateJob(job);onProgress?.({stage:'prepared-state-review',message:'Checking exact prepared state scope before execution'});
 const review=engine.execute(preparedEngineRequest({...job.prepared_engine,settings:{...job.prepared_engine.settings,inspect_only:true}}));
 if(review.schema_version!=='bisect-prepared-engine-request-v1'||review.execution_performed!==false||review.state!==job.scope.state||review.year!==job.scope.year||review.engine_request?.options?.districts!==job.scope.districts||review.request_sha256!==job.request_sha256)throw new Error('Native prepared request scope or byte identity differs from its queued state. No partition executed.');
 matchingContextSource(job.prepared_engine,review);
 onProgress?.({stage:'prepared-state-run',message:'Executing original prepared request'});const request=preparedEngineRequest(job.prepared_engine);if(partitionTiming)request.operation='run-prepared-engine-timed';const prepared=engine.execute(request,{onProgress});
 if(prepared.schema_version!=='bisect-prepared-engine-v1'||prepared.request_sha256!==review.request_sha256)throw new Error('Prepared state worker returned a mismatched execution receipt.');
 if(partitionTiming){const timing=validatePartitionTimingReport(prepared.partition_timing),attempts=prepared.result?.build_report?.attempts;const count=attempts?.length??1;if(timing.events.length+timing.omitted_events!==count||timing.events.some((e,i)=>e.state!==job.scope.state||e.year!==job.scope.year||e.districts!==job.scope.districts||e.units!==review.units||e.attempt!==i||attempts&&e.seed!==attempts[i].seed))throw new Error('Partition timing differs from the executed scope or retry schedule.');}
 matchingContextSource(job.prepared_engine,prepared);
 const output={schema_version:'bisect-prepared-state-run-v1',scope:structuredClone(job.scope),request_source:structuredClone(job.prepared_engine),source_review:review,prepared};
 if(job.geometry){output.geometry=structuredClone(job.geometry);try{preparedStateRunMap(output);}catch(error){output.geometry_error=error.message;}}
 return output;
}
export function preparedStateRunMap(result){
 if(result?.schema_version!=='bisect-prepared-state-run-v1'||!result.geometry)throw new Error('This prepared state has no supplied display geometry.');
 validateSourceFile(result.geometry,{maxBytes:64*1024*1024});const document=preparedEngineMapDocument(result.prepared),geometry=parseGeometryJson(new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(result.geometry.base64),c=>c.charCodeAt(0))));
 if(document.plan.units.state!==result.scope?.state||document.plan.units.year!==result.scope?.year||document.plan.k!==result.scope?.districts)throw new Error('Prepared state result differs from its declared scope.');
 return planMapModel(document,geometry);
}
export function preparedStateRunZip(result){
 if(result?.schema_version!=='bisect-prepared-state-run-v1')throw new Error('Select a completed prepared state result.');preparedEngineMapDocument(result.prepared);
 const entries=[['request.json',result.request_source.source.text],['prepared-run-receipt.json',JSON.stringify(result,null,2)+'\n'],['final_assignments.json',JSON.stringify(result.prepared.result.assignments,null,2)+'\n'],['accepted-replay.json',result.prepared.replay_artifact_json]];
 if(result.prepared.export){const files=preparedEnginePlanSources(result.prepared);for(const [key,name]of [['auditPlanText','plan.rplan'],['auditContextText','context.rctx']])entries.push([name,new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(files[key].base64),c=>c.charCodeAt(0)))]);}
 if(result.native_state){const n=result.native_state;if(n.scope_binding)entries.push(['state-flags/scope-binding.json',JSON.stringify(n.scope_binding,null,2)+'\n']);if(n.acquisition)entries.push(['state-flags/acquisition.json',JSON.stringify(n.acquisition,null,2)+'\n']);entries.push(['state-flags/recipe.json',JSON.stringify(n.recipe,null,2)+'\n'],['state-flags/arguments.json',n.recipe.command.arguments_json],['state-flags/compilation.json',JSON.stringify(n.compilation,null,2)+'\n'],['state-flags/identities.json',JSON.stringify({source_sha256:n.source_sha256,request_sha256:n.request_sha256},null,2)+'\n']);}
 if(result.native_state?.recipe?.workflow_binding){const b=result.native_state.recipe.workflow_binding;entries.push(['state-flags/workflow-binding.json',JSON.stringify(b,null,2)+'\n'],['state-flags/original-workflow-arguments.json',b.source_command.arguments_json]);}
 if(result.native_states){const n=result.native_states;entries.push(['states-flags/recipe.json',JSON.stringify(n.recipe,null,2)+'\n'],['states-flags/arguments.json',n.recipe.command.arguments_json],['states-flags/compilation.json',JSON.stringify(n.compilation,null,2)+'\n'],['states-flags/identities.json',JSON.stringify({source_sha256:n.source_sha256,request_sha256:n.request_sha256},null,2)+'\n']);}
 if(result.native_run){const n=result.native_run;entries.push(['run-flags/recipe.json',JSON.stringify(n.recipe,null,2)+'\n'],['run-flags/arguments.json',n.recipe.command.arguments_json],['run-flags/compilation.json',JSON.stringify(n.compilation,null,2)+'\n'],['run-flags/identities.json',JSON.stringify({source_sha256:n.source_sha256,request_sha256:n.request_sha256},null,2)+'\n']);}
 if(result.template_source)entries.push(['request-template.json',result.template_source.source.text],['graph-binding.json',JSON.stringify(result.graph_binding,null,2)+'\n']);
 if(result.prepared.context_engine_source){const context=result.prepared.context_engine_source;entries.push(['context-inputs/original.rctx',context.source.context_json],['context-inputs/origin.json',JSON.stringify(context.origin,null,2)+'\n']);if(context.source.attributes_source)entries.push(['context-inputs/attributes.json',context.source.attributes_source.attributes_json]);}
 if(result.native_input){entries.push(['native-inputs/recipe.json',JSON.stringify(result.native_input,null,2)+'\n']);for(const [key,name]of [['adjacency','adjacency.bin'],['geoids','geoids.json'],['centroids','centroids.json'],['geometry','physical-geometry.json'],['county_ids','county-ids.json']])if(result.native_input.sources[key])entries.push(['native-inputs/'+name,Uint8Array.from(atob(result.native_input.sources[key].base64),c=>c.charCodeAt(0))]);}
 if(result.geometry)entries.push(['display-geometry.geojson',new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(result.geometry.base64),c=>c.charCodeAt(0)))]);
 const run=result.prepared;if(result.native_state||result.native_states||result.native_run)entries.push(...nativeStateResultEntries(run));else if(run.result.metrics?.native_state_vra)entries.push(...nativeStateDataEntries(run.run_kind==='ordinary'?run.executed_request:run.executed_request.request,run.result));
 return storedZipBytes(entries.map(([name,text])=>{if(text instanceof Uint8Array)return [name,text];if(typeof text!=='string')throw new Error('Prepared state result lacks an accepted artifact.');return [name,new TextEncoder().encode(text)];}),{maxSourceBytes:128*1024*1024});
}
