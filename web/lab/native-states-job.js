import {validateStateCommandInputs,stateCommandRequest,stateCommandPreparationRequest,preparedEngineFromStateCommand} from './state-command-files.js';
import {validateStateGraphInputs,stateGraphRequest,preparedEngineFromStateGraph} from './state-graph-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {validateSourceFile} from './package-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from './prepared-state-job.js';
const hash=async value=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(value)))),b=>b.toString(16).padStart(2,'0')).join('');
export function nativeStatesJobRequest(command,inputs,{graph=false,geometry}={}){
 validateStateCommandInputs(command);if(graph)validateStateGraphInputs(inputs);else validatePreparedEngineInputs(inputs);
 if(geometry)validateSourceFile(geometry,{maxBytes:64*1024*1024});
 return {operation:'run-native-states-job',input:structuredClone({command,[graph?'graph_inputs':'prepared_inputs']:inputs,...(geometry?{geometry}:{})})};
}
export async function executeNativeStatesJob(engine,value,{onProgress}={}){
 const graph=Object.hasOwn(value??{},'graph_inputs');
 if(!value||Object.keys(value).some(k=>!['command',graph?'graph_inputs':'prepared_inputs','geometry'].includes(k)))throw new Error('Select native States flags and exactly one original graph or request template.');
 const input=nativeStatesJobRequest(value.command,graph?value.graph_inputs:value.prepared_inputs,{graph,geometry:value.geometry}).input;
 const source=graph?input.graph_inputs:input.prepared_inputs;
 const inspection=stateCommandRequest(input.command,{bulk:true});
 if(inspection.arguments.includes('--print-only')){
  const review=engine.execute(inspection);
  if(review?.schema_version!=='bisect-native-states-command-v1'||review.status!=='parsed'||review.print_only!==true)throw new Error(review?.error||'Review valid native States --print-only flags. No source prepared or partition executed.');
  return {schema_version:'bisect-native-state-run-input-v1',native_command:review,execution_performed:false,native_states_print_only:true,native_states_inspect_only:!graph&&source.settings.inspect_only,native_states_execution_policy:'native-print-only',scope:'Workbench print-only inspects original States flags without reading or preparing a graph or template.'};
 }
 onProgress?.({stage:'native-states-compilation',message:'Checking actual native bulk States defaults and selected sources'});
 const request=graph?stateGraphRequest(input.command,source,{bulk:true}):stateCommandPreparationRequest(input.command,source,{bulk:true});
 const compiled=engine.execute(request);
 const selected=graph?await preparedEngineFromStateGraph(input.command,source,compiled,{bulk:true}):await preparedEngineFromStateCommand(input.command,source,compiled,{bulk:true});
 if(compiled.native_command?.schema_version!=='bisect-native-states-command-v1'||compiled.native_command.status!=='parsed')throw new Error('Native States compilation returned an invalid command boundary.');
 if(compiled.native_command.print_only||!graph&&source.settings.inspect_only)return {...compiled,native_states_print_only:compiled.native_command.print_only,native_states_inspect_only:!graph&&source.settings.inspect_only,native_states_execution_policy:compiled.native_command.print_only?'native-print-only':'prepared-template-inspection'};
 selected.settings.inspect_only=false;
 const job=await preparedStateJobFromResult(selected,compiled.prepared_review,input.geometry);
 const result=executePreparedStateJob(engine,job,{onProgress});
 const {prepared_request_json,prepared_review,...compilation}=compiled;
 result.native_states={schema_version:'bisect-native-states-execution-v1',recipe:input,source_sha256:await hash(input),request_sha256:compiled.prepared_request_sha256,compilation,scope:'One selected native States graph/template executed through the actual shared bulk configuration and native population retry/export machinery. Original flags and selected sources are retained. This is not native filesystem discovery, multi-state scheduling or authentication of historical execution.'};
 return result;
}
