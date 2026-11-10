import {validateContextEngineSource} from './context-engine-files.js';
import {validatePreparedEngineInputs} from './prepared-engine-files.js';
import {nativeEngineGraphDownload} from './native-adjacency-files.js';

export const PREPARED_GRAPH_DEFAULTS={graph:null,template:null,districts:1,settings:{label:'Prepared native graph run',chamber:'congressional',created_at:'',inspect_only:true}};
export function validatePreparedGraphInputs(value){
 if(!value||Object.keys(value).filter(k=>k!=='context_source').sort().join()!=='districts,graph,settings,template'||!Number.isSafeInteger(value.districts)||value.districts<1||value.districts>500)throw new Error('Select a graph, full request template and 1–500 districts.');
 for(const key of ['graph','template'])validatePreparedEngineInputs({source:value[key],settings:value.settings});
 if(Object.hasOwn(value,'context_source'))validateContextEngineSource(value.context_source);return value;
}
export function preparedGraphRequest(value){
 validatePreparedGraphInputs(value);if(!value.graph||!value.template)throw new Error('Select both prepared graph JSON and a request template.');
 return {operation:'bind-prepared-engine-graph',input:{graph_json:value.graph.text,template_json:value.template.text,districts:value.districts}};
}
export async function preparedGraphFromNative(result,previous=PREPARED_GRAPH_DEFAULTS){
 previous=structuredClone(validatePreparedGraphInputs(previous));delete previous.context_source;const file=await nativeEngineGraphDownload(result);
 return validatePreparedGraphInputs({...previous,graph:{name:file.name,text:new TextDecoder('utf-8',{fatal:true}).decode(file.bytes)},districts:result.suggested_districts||1});
}
const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
export async function preparedEngineFromGraphBinding(inputs,result){
 inputs=structuredClone(validatePreparedGraphInputs(inputs));preparedGraphRequest(inputs);
 if(result?.schema_version!=='bisect-prepared-engine-binding-v1'||result.execution_performed!==false||result.csv_loading_performed!==false||result.districts!==inputs.districts||typeof result.bound_request_json!=='string'||result.review?.schema_version!=='bisect-prepared-engine-request-v1'||result.review?.csv_loading_performed!==false||result.review?.units!==result.units||result.review?.unit_kind!==result.unit_kind||result.review?.execution_performed!==false||result.review?.state!==result.state||result.review?.year!==result.year||result.review?.engine_request?.options?.districts!==inputs.districts)throw new Error('Bind the selected graph and template before selecting an engine request.');
 const next=validatePreparedEngineInputs({source:{name:'bound-engine-request.json',text:result.bound_request_json},settings:{...inputs.settings,inspect_only:true},...(inputs.context_source?{context_source:inputs.context_source}:{})});
 const [graph,template,bound]=await Promise.all([digest(inputs.graph.text),digest(inputs.template.text),digest(result.bound_request_json)]);
 if(graph!==result.graph_sha256||template!==result.template_sha256||bound!==result.bound_request_sha256||bound!==result.review.request_sha256)throw new Error('Graph, template or bound request changed. Bind again.');
 // Handoff selects a review. It never inherits an imported execution decision.
 return next;
}
