import {inspectJsonText} from './project.js';
export const MAX_COI_BYTES=3*1024*1024;
export const MULTISCALE_COI_SCOPE='tract-seed-only; fine/coarse sampling and percentile ranking remain unweighted; block-group IDs are not mapped to parent tracts';
const bytes=s=>new TextEncoder().encode(s);
export function validateCoiInput(input){
 if(!input||Array.isArray(input)||Object.keys(input).length!==2||!Object.hasOwn(input,'source_name')||!Object.hasOwn(input,'weights_json')||typeof input.source_name!=='string'||!input.source_name||bytes(input.source_name).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(input.source_name)||typeof input.weights_json!=='string'||bytes(input.weights_json).length>MAX_COI_BYTES)throw new Error('Select named UTF-8 COI JSON up to 3 MiB.');
 // Preserve original numeric text for Rust, including finite f64 factors beyond
 // the safe-integer range. Inspect duplicate/unsafe keys before parsing.
 inspectJsonText(input.weights_json,100001);
 const values=JSON.parse(input.weights_json);
 if(!values||Array.isArray(values)||typeof values!=='object'||Object.keys(values).length>100000||Object.entries(values).some(([id,w])=>!id||bytes(id).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(id)||typeof w!=='number'||!Number.isFinite(w)||w<0))throw new Error('COI source requires at most 100,000 safe unit IDs and finite nonnegative weights.');
 return input;
}
export async function readCoiInput(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_COI_BYTES)throw new Error('Select nonempty COI JSON up to 3 MiB.');
 const data=await file.arrayBuffer();if(data.byteLength!==file.size)throw new Error('COI source changed while reading.');
 let text;try{text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(data);}catch{throw new Error('COI source is not valid UTF-8.');}
 return validateCoiInput({source_name:file.name,weights_json:text});
}
export async function verifyMultiscaleCoi(graph,evidence,input,seedEdges=graph.edges,active=true){
 if(!input){if(Object.hasOwn(evidence,'coi_weighting'))throw new Error('Unexpected multiscale COI evidence.');return;}
 validateCoiInput(input);const values=JSON.parse(input.weights_json),e=evidence.coi_weighting;
 const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes(input.weights_json)))].map(v=>v.toString(16).padStart(2,'0')).join('');
 const matched=graph.geoids.filter(id=>Object.hasOwn(values,id)).length;
 let changed=0,total=0;
 for(const [u,v,weight]of seedEdges){const adjusted=weight*Math.sqrt((values[graph.geoids[u]]??1)*(values[graph.geoids[v]]??1));if(!Number.isFinite(adjusted)||!Number.isFinite(total+=adjusted))throw new Error('COI adjusted weights exceed the native numeric range.');if(adjusted!==weight)changed++;}
 const expected={schema_version:'bisect-coi-weighting-v1',source_name:input.source_name,source_sha256:hash,supplied_units:Object.keys(values).length,matched_units:matched,defaulted_units:graph.geoids.length-matched,ignored_units:Object.keys(values).length-matched,applied_edges:active?seedEdges.length:0,changed_edges:changed,status:!active?'no-effect-unweighted-native-empty-map':changed?'applied':'applied-no-edge-change',formula:'resolved_edge_weight * sqrt(unit_weight_u * unit_weight_v)',missing_unit_weight:1,algorithm_scope:MULTISCALE_COI_SCOPE};
 if(!e||Object.keys(e).length!==Object.keys(expected).length+1||typeof e.scope!=='string'||Object.entries(expected).some(([k,v])=>e[k]!==v))throw new Error('Multiscale COI evidence disagrees with selected source/tract weights.');
}
