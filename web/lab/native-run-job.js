import {validateStateCommandInputs,stateCommandRequest,stateCommandPreparationRequest,preparedEngineFromStateCommand} from './state-command-files.js';
import {validateStateGraphInputs,stateGraphRequest,preparedEngineFromStateGraph} from './state-graph-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {validateSourceFile} from './package-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from './prepared-state-job.js';
const hash=async value=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(value)))),b=>b.toString(16).padStart(2,'0')).join('');
export function nativeRunJobRequest(command,inputs,{graph=false,geometry}={}){
 validateStateCommandInputs(command);if(graph)validateStateGraphInputs(inputs);else validatePreparedEngineInputs(inputs);
 if(geometry)validateSourceFile(geometry,{maxBytes:64*1024*1024});
 return {operation:'run-native-run-job',input:structuredClone({command,[graph?'graph_inputs':'prepared_inputs']:inputs,...(geometry?{geometry}:{})})};
}
export async function executeNativeRunJob(engine,value,{onProgress}={}){
 const graph=Object.hasOwn(value??{},'graph_inputs');
 if(!value||Object.keys(value).some(k=>!['command',graph?'graph_inputs':'prepared_inputs','geometry'].includes(k)))throw new Error('Select native Run flags and exactly one original graph or request template.');
 const input=nativeRunJobRequest(value.command,graph?value.graph_inputs:value.prepared_inputs,{graph,geometry:value.geometry}).input;
 const source=graph?input.graph_inputs:input.prepared_inputs;
 const inspection=stateCommandRequest(input.command,{run:true});
 if(inspection.arguments.includes('--print-only')){
  const review=engine.execute(inspection);
  if(review?.schema_version!=='bisect-native-run-command-v1'||review.status!=='parsed'||review.print_only!==true)throw new Error(review?.error||'Review valid native Run --print-only flags. No source prepared or partition executed.');
  return {schema_version:'bisect-native-state-run-input-v1',native_command:review,execution_performed:false,native_run_print_only:true,native_run_inspect_only:!graph&&source.settings.inspect_only,native_run_execution_policy:'native-print-only',scope:'Workbench print-only inspects original Run flags without reading or preparing a graph or template.'};
 }
 onProgress?.({stage:'native-run-compilation',message:'Checking actual native Run default-mode compositor and selected sources'});
 const request=graph?stateGraphRequest(input.command,source,{run:true}):stateCommandPreparationRequest(input.command,source,{run:true});
 const compiled=engine.execute(request);
 const selected=graph?await preparedEngineFromStateGraph(input.command,source,compiled,{run:true}):await preparedEngineFromStateCommand(input.command,source,compiled,{run:true});
 if(compiled.native_command?.schema_version!=='bisect-native-run-command-v1'||compiled.native_command.status!=='parsed')throw new Error('Native Run compilation returned an invalid command boundary.');
 if(compiled.native_command.print_only||!graph&&source.settings.inspect_only)return {...compiled,native_run_print_only:compiled.native_command.print_only,native_run_inspect_only:!graph&&source.settings.inspect_only,native_run_execution_policy:compiled.native_command.print_only?'native-print-only':'prepared-template-inspection'};
 selected.settings.inspect_only=false;
 const job=await preparedStateJobFromResult(selected,compiled.prepared_review,input.geometry);
 const result=executePreparedStateJob(engine,job,{onProgress});
 const {prepared_request_json,prepared_review,...compilation}=compiled;
 result.native_run={schema_version:'bisect-native-run-execution-v1',recipe:input,source_sha256:await hash(input),request_sha256:compiled.prepared_request_sha256,compilation,scope:'One selected native Run graph/template executed through the actual shared default-mode configuration and native population retry/export machinery. Original flags and selected sources are retained. This is not native filesystem discovery, multi-year/multi-state scheduling or authentication of historical execution.'};
 return result;
}
