import {validateSourceFile,encodeBytes} from './package-files.js';
import {inspectJsonText} from './project.js';
export const NATIVE_ADJACENCY_DEFAULTS={adjacency:null,geoids:null,centroids:null,geometry:null};
export const NATIVE_ADJACENCY_SETTINGS={state:'RI',year:'2020'};
export function validateNativeAdjacencySettings(v){
 const custom=['precinct','imported'].includes(v?.unit_kind);
 if(!v||!['state,year','state,unit_kind,year'].includes(Object.keys(v).filter(k=>k!=='require_geometry').sort().join())||Object.hasOwn(v,'require_geometry')&&typeof v.require_geometry!=='boolean'||Object.hasOwn(v,'unit_kind')&&!['county','tract','block-group','block','precinct','imported'].includes(v.unit_kind)||typeof v.year!=='string'||typeof v.state!=='string'||!/^[A-Z]{2}$/.test(v.state)||(custom?!/^\d{4}$/.test(v.year):!['2000','2010','2020'].includes(v.year)))throw new Error('Select a state and Census vintage, or an explicit four-digit precinct/imported year.');return v;
}
export function nativeEngineInputRequest(sources,settings){validateNativeAdjacencySettings(settings);return {operation:'prepare-native-engine-input',input:{sources:validateNativeAdjacencyInputs(sources,{requireComplete:true}),...settings}};}
export async function nativeEngineGraphDownload(result){
 if(result?.schema_version!=='bisect-native-engine-input-v1'||result.status!=='prepared'||result.execution_performed!==false||typeof result.graph_json!=='string'||new TextEncoder().encode(result.graph_json).length>8*1024*1024)throw new Error('Prepare fresh native engine inputs first.');
 validateNativeAdjacencySettings({state:result.state,year:result.year,...(result.unit_kind?{unit_kind:result.unit_kind}:{})});
 const bytes=new TextEncoder().encode(result.graph_json),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==result.graph_sha256)throw new Error('Native engine graph bytes changed.');return {name:result.state+'-'+result.year+'-graph.json',bytes,type:'application/json'};
}
export async function preparedNativeRunSources(result){const file=await nativeEngineGraphDownload(result),graph={name:file.name,base64:encodeBytes(file.bytes)};validateSourceFile(graph);return {graph,settings:{chamber:'congressional',districts:result.suggested_districts||1}};}
export function validateNativeAdjacencyInputs(v,{requireComplete=false}={}){
 if(!v||!['adjacency,centroids,geoids','adjacency,centroids,geoids,geometry'].includes(Object.keys(v).filter(k=>k!=='county_ids').sort().join()))throw new Error('Select native adjacency and optional companions.');
 for(const kind of ['adjacency','geoids','centroids','geometry','county_ids'])if(v[kind]!==null&&!(['geometry','county_ids'].includes(kind)&&!Object.hasOwn(v,kind))){const source=v[kind];validateSourceFile(source,{maxBytes:(kind==='adjacency'?64:16)*1024*1024});if(!source.name||new TextEncoder().encode(source.name).length>200)throw new Error('Select a source name up to 200 UTF-8 bytes.');if(kind!=='adjacency'){const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));inspectJsonText(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes),4000000);}}
 if(requireComplete&&!v.adjacency)throw new Error('Select a native .adj.bin file.');return v;
}
export async function readNativeAdjacencySource(file,kind){
 if(!['adjacency','geoids','centroids','geometry','county_ids'].includes(kind)||!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>(kind==='adjacency'?64:16)*1024*1024)throw new Error('Select a bounded native adjacency source.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Native source changed during selection.');const source={name:file.name,base64:encodeBytes(bytes)};validateNativeAdjacencyInputs({...NATIVE_ADJACENCY_DEFAULTS,[kind]:source});return source;
}
export function nativeAdjacencyRequest(value){return {operation:'inspect-native-adjacency',input:validateNativeAdjacencyInputs(value,{requireComplete:true})};}
export async function nativeAdjacencyDownload(result){
 if(result?.schema_version!=='bisect-native-adjacency-inspection-v1'||result.status!=='inspected'||result.execution_performed!==false||typeof result.loaded_graph_json!=='string'||new TextEncoder().encode(result.loaded_graph_json).length>64*1024*1024)throw new Error('Inspect fresh native adjacency first.');
 const bytes=new TextEncoder().encode(result.loaded_graph_json),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==result.loaded_graph_sha256)throw new Error('Native adjacency output bytes changed.');return {name:'native-adjacency.json',bytes,type:'application/json'};
}
