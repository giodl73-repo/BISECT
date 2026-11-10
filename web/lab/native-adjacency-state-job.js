import {stateCommandArguments} from './state-command-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from './prepared-state-job.js';
import {encodeBytes,validateSourceFile} from './package-files.js';
import {executeNativeStateFollowup} from './native-state-followup.js';
import {emitNativeReportSourceWarning,applyNativeStateReportSources} from './native-state-report-pipeline.js';
const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');

// Called only after the ordinary immutable recipe validation in native-state-job.
export async function executeNativeAdjacencyStateJob(engine,job,{onProgress,partitionTiming=false}={}){
 const recipe=job.native_state,source=recipe.source_inputs,settings=source.settings,binding=recipe.workflow_binding;
 // The shared workflow recheck already binds these original bytes and intent.
 // Restore adjacency only for the verified-source Rust entry point; output
 // choices stay in the browser workflow, and no argument path is opened.
 const args=[...stateCommandArguments(recipe.command),...(binding?['--adjacency='+binding.source_override.adjacency_path_intent]:[])];
 onProgress?.({stage:'native-state-compilation',message:'Compiling State flags with selected native adjacency bytes'});
 const compiled=engine.execute({operation:'prepare-native-adjacency-state-run',input:{arguments:args,source:source.preparation.input,...(source.auxiliary?{auxiliary_json:source.auxiliary.text,auxiliary_source_name:source.auxiliary.name}:{}),label:settings.label,chamber:settings.chamber,created_at:settings.created_at}},{onProgress});
 const review=compiled.prepared_review,scope={state:review?.state,year:review?.year,chamber:settings.chamber,districts:review?.engine_request?.options?.districts},preparation=compiled.native_adjacency_source;
 if(compiled.schema_version!=='bisect-native-state-run-input-v1'||compiled.execution_performed!==false||JSON.stringify(compiled.native_command?.arguments)!==JSON.stringify(args)||compiled.adjacency_override?.path_accessed!==false||!preparation||scope.state!==job.scope.state||scope.year!==job.scope.year||job.scope.districts!==null&&scope.districts!==job.scope.districts||review?.request_sha256!==compiled.prepared_request_sha256||await hash(compiled.prepared_request_json)!==compiled.prepared_request_sha256)throw new Error('Selected adjacency State compilation differs from its original source scope or request identity. No partition executed.');
 const inputs=validatePreparedEngineInputs({source:{name:'native-adjacency-state-compiled.json',text:compiled.prepared_request_json},settings:{...settings,inspect_only:true}});
 let display=job.geometry;
 if(!display&&typeof preparation.geometry_json==='string'){
  if(await hash(preparation.geometry_json)!==preparation.geometry_sha256)throw new Error('Selected adjacency display geometry identity changed.');
  display={name:scope.state+'-'+scope.year+'-geometry.geojson',base64:encodeBytes(new TextEncoder().encode(preparation.geometry_json))};validateSourceFile(display,{maxBytes:64*1024*1024});
 }
 emitNativeReportSourceWarning(binding,onProgress);
 const prepared=await preparedStateJobFromResult(inputs,review,display),result=executePreparedStateJob(engine,prepared,{onProgress,partitionTiming});
 const receipt=Object.fromEntries(['schema_version','native_command','original_template_sha256','prepared_request_sha256','district_count','embedded_policy_sha256','state_graph_source','adjacency_override'].filter(k=>Object.hasOwn(compiled,k)).map(k=>[k,compiled[k]]));
 const retained=Object.fromEntries(Object.entries(preparation).filter(([k])=>!['geometry_json','graph_json'].includes(k)));
 result.native_state={recipe,source_sha256:job.source_sha256,request_sha256:compiled.prepared_request_sha256,compilation:{...receipt,native_adjacency_source:retained}};
 await applyNativeStateReportSources(engine,result,binding,{onProgress});
 if(recipe.followup&&result.native_report_sources?.status!=='failed')await executeNativeStateFollowup(engine,result,recipe.followup,settings.created_at,{onProgress});
 return {schema_version:'bisect-native-source-flags-state-run-v1',source_preparation:structuredClone(retained),result};
}
