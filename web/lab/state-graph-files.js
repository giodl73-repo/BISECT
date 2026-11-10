import {encodeBytes,validateSourceFile} from './package-files.js';
import {inspectJsonText} from './project.js';
import {validatePreparedEngineInputs,readPreparedEngineSource} from './prepared-engine-files.js';
import {validateContextEngineSource} from './context-engine-files.js';
import {stateCommandArguments} from './state-command-files.js';
const size=text=>new TextEncoder().encode(text).length;
const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export const STATE_GRAPH_DEFAULTS={graph:null,auxiliary:null,settings:{label:'Native State graph run',chamber:'congressional',created_at:'',inspect_only:true}};
export function validateStateGraphInputs(value){
 if(!value||Object.keys(value).filter(k=>k!=='context_source').sort().join()!=='auxiliary,graph,settings'||value.settings?.inspect_only!==true)throw new Error('Select native State graph sources and review metadata.');
 for(const key of ['graph','auxiliary'])validatePreparedEngineInputs({source:value[key],settings:value.settings});
 if(size(value.graph?.text??'')+size(value.auxiliary?.text??'')>25*1024*1024)throw new Error('Graph and auxiliary JSON together exceed 25 MiB.');
 if(Object.hasOwn(value,'context_source'))validateContextEngineSource(value.context_source);return value;
}
export async function readStateGraphSource(file){return readPreparedEngineSource(file);}
export function stateGraphRequest(command,value,{bulk=false,run=false}={}){
 validateStateGraphInputs(value);if(!value.graph)throw new Error('Select prepared graph JSON first.');
 inspectJsonText(value.graph.text,4000000);if(value.auxiliary)inspectJsonText(value.auxiliary.text,4000000);
 return {operation:run?'prepare-native-run-graph-run':bulk?'prepare-native-states-graph-run':'prepare-native-state-graph-run',input:{arguments:stateCommandArguments(command),graph_json:value.graph.text,graph_source_name:value.graph.name,...(value.auxiliary?{auxiliary_json:value.auxiliary.text,auxiliary_source_name:value.auxiliary.name}:{}),...(value.context_source?{context_source:value.context_source}:{}),label:value.settings.label,chamber:value.settings.chamber,created_at:value.settings.created_at}};
}
export function stateGraphDefaultCommand(value,{bulk=false,run=false}={}){
 validateStateGraphInputs(value);if(!value.graph)throw new Error('Select a graph before choosing its State defaults.');
 inspectJsonText(value.graph.text,4000000);const graph=JSON.parse(value.graph.text),kind=graph.unit_kind??'tract';
 if(!/^[A-Z]{2}$/.test(graph.state)||!['2000','2010','2020'].includes(graph.year)||!['tract','block-group','block'].includes(kind))throw new Error('Native State flags require a scoped Census tract, block-group or block graph. Other unit kinds use the full prepared engine operation.');
 if(run){if(kind!=='tract'||value.settings.chamber!=='congressional')throw new Error('Native Run uses congressional tract graphs.');return {arguments_json:JSON.stringify(['--year',graph.year,'--states',graph.state])};}
 if(bulk){if(kind!=='tract'||value.settings.chamber!=='congressional')throw new Error('Native States uses congressional tract graphs.');return {arguments_json:JSON.stringify(['--year',graph.year,'--version','v1','--output-dir','outputs/v1','--states',graph.state])};}
 return {arguments_json:JSON.stringify(['--state',graph.state,'--year',graph.year,'--resolution',kind==='block-group'?'block_group':kind,'--metis-engine','metis-core','--chamber',value.settings.chamber])};
}
export async function stateGraphFromPreparedResult(result,{context_source}={}){
 if(!['bisect-native-engine-input-v1','bisect-census-input-preparation-v1','bisect-tract-input-preparation-v1'].includes(result?.schema_version)||result.status!=='prepared'||result.execution_performed!==false||typeof result.graph_json!=='string'||size(result.graph_json)>25*1024*1024||!result.graph_json.length||await hash(result.graph_json)!==result.graph_sha256)throw new Error('Prepare a fresh native/Census graph before selecting State flags.');
 if(result.source_context&&!context_source)throw new Error('Retain the original RCTX source with this graph.');
 const inputs=validateStateGraphInputs({...structuredClone(STATE_GRAPH_DEFAULTS),graph:{name:'native-state-graph.json',text:result.graph_json},...(context_source?{context_source}:{}),settings:{...STATE_GRAPH_DEFAULTS.settings,label:result.state+' native State run'}});
 const command=stateGraphDefaultCommand(inputs);let geometry;
 if(typeof result.geometry_json==='string'){if(size(result.geometry_json)>64*1024*1024||await hash(result.geometry_json)!==result.geometry_sha256)throw new Error('Prepared display geometry bytes changed.');geometry={name:result.state+'-'+result.year+'-geometry.geojson',base64:encodeBytes(new TextEncoder().encode(result.geometry_json))};validateSourceFile(geometry,{maxBytes:64*1024*1024});}
 return {inputs,command,geometry};
}
function sameContextSource(a,b){
 if(!a||!b)return a==null&&b==null;
 return ['context_json','source_name','physical_edge_weights_metres'].every(k=>a[k]===b[k])&&['source_name','attributes_json'].every(k=>(a.attributes_source?.[k]??null)===(b.attributes_source?.[k]??null));
}
export async function preparedEngineFromStateGraph(command,inputs,result,{bulk=false,run=false}={}){
 const request=stateGraphRequest(command,inputs,{bulk,run}),source=result?.state_graph_source;
 if(result?.schema_version!=='bisect-native-state-run-input-v1'||result.execution_performed!==false||source?.schema_version!=='bisect-native-state-graph-source-v1'||JSON.stringify(result.native_command?.arguments)!==JSON.stringify(request.input.arguments)||typeof result.prepared_request_json!=='string')throw new Error('Prepare a fresh native State graph request first.');
 if(!sameContextSource(inputs.context_source,result.prepared_review?.context_engine_source?.source))throw new Error('Original RCTX source changed. Prepare again.');
 if(source.graph_source_name!==inputs.graph.name||source.auxiliary_source_name!==(inputs.auxiliary?.name??null)||source.graph_sha256!==await hash(inputs.graph.text)||source.auxiliary_sha256!==(inputs.auxiliary?await hash(inputs.auxiliary.text):null)||result.prepared_request_sha256!==await hash(result.prepared_request_json)||result.prepared_review?.request_sha256!==result.prepared_request_sha256)throw new Error('State graph, auxiliary source or compiled request changed. Prepare again.');
 return validatePreparedEngineInputs({source:{name:'native-state-compiled.json',text:result.prepared_request_json},settings:{...inputs.settings,inspect_only:true},...(inputs.context_source?{context_source:inputs.context_source}:{})});
}
export const stateGraphAuxiliaryTemplate=()=>JSON.stringify(Object.fromEntries(['demographics','partisan','elections','character','native_character_source','coi'].map(k=>[k,null])),null,2)+'\n';
