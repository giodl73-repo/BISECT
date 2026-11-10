import {parsePreparedGraphJson} from './project.js';
const bytes=s=>new TextEncoder().encode(s).length;
const exact=(v,keys)=>v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===keys.slice().sort().join();
const label=(s,empty=false)=>typeof s==='string'&&(empty||s.length>0)&&bytes(s)<=200&&!/[\u0000-\u001f\u007f-\u009f]/.test(s);
export function zoneDocument(input,graph=null){
 if(!exact(input,['source_name','assignments_json','alpha'])||!label(input.source_name)||typeof input.assignments_json!=='string'||!input.assignments_json.length||bytes(input.assignments_json)>8*1024*1024||!Number.isFinite(input.alpha)||input.alpha<0||input.alpha>100||Object.is(input.alpha,-0))throw new Error('Select a bounded zone assignment source and finite alpha in [0,100].');
 const d=parsePreparedGraphJson(input.assignments_json);
 if(!exact(d,['schema_version','state','year','unit_ids','zone_types','assignments'])||d.schema_version!==1||typeof d.state!=='string'||!/^[A-Z]{2}$/.test(d.state)||typeof d.year!=='string'||!/^\d{4}$/.test(d.year)||!Array.isArray(d.unit_ids)||!d.unit_ids.length||d.unit_ids.length>1000000||d.unit_ids.some(id=>!label(id))||new Set(d.unit_ids).size!==d.unit_ids.length||!Array.isArray(d.zone_types)||d.zone_types.length>32||d.zone_types.some(s=>!label(s))||new Set(d.zone_types).size!==d.zone_types.length||!Array.isArray(d.assignments)||d.assignments.length!==d.unit_ids.length||d.assignments.some(row=>!Array.isArray(row)||row.length>d.zone_types.length||row.some(v=>v!==null&&!label(v,true))))throw new Error('Zone assignments require exact ordered units, unique zone types and bounded rows; use null for unknown observations.');
 if(graph&&(d.state!==graph.state||d.year!==graph.year||JSON.stringify(d.unit_ids)!==JSON.stringify(graph.geoids)))throw new Error('Zone jurisdiction, year or exact unit order differs from the graph.');return d;
}
export function zoneRequest(config,state,graph=null){const input=config.weights==='zone-membership'?config.zones?.[state]:null;if(!input)return {};zoneDocument(input,graph);return {zone_membership:input};}
export async function zoneIdentity(input){zoneDocument(input);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(input.assignments_json)))].map(b=>b.toString(16).padStart(2,'0')).join('');}
export async function readZoneInput(file,alpha=1){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select zone assignment JSON up to 8 MiB.');
 const data=new Uint8Array(await file.arrayBuffer());if(data.length!==file.size)throw new Error('Zone source size changed while reading.');
 const input={source_name:file.name,assignments_json:new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(data),alpha};return {input,document:zoneDocument(input)};
}
export function zoneTemplate(graph){return {schema_version:1,state:graph.state,year:graph.year,unit_ids:[...graph.geoids],zone_types:['school-district','utility-service'],assignments:graph.geoids.map(()=>[null,null])};}
export function zoneWeights(graph,input){
 const d=zoneDocument(input,graph),weights=new Map();let jointly=0,shared=0,changed=0,total=0;
 for(const [u,v,w]of graph.edges){let available=0,same=0;const a=d.assignments[u],b=d.assignments[v];for(let i=0;i<Math.min(a.length,b.length);i++)if(a[i]!==null&&b[i]!==null){available++;if(a[i]===b[i])same++;}
  const value=w*(1+input.alpha*(available?same/available:0));total+=value;if(!Number.isFinite(value)||!Number.isFinite(total))throw new Error('Zone adjusted weights exceed the native finite range.');weights.set(u+':'+v,value);if(available)jointly++;if(same)shared++;if(value!==w)changed++;
 }
 return {weight:(u,v)=>weights.get(Math.min(u,v)+':'+Math.max(u,v)),evidence:(sha,scope)=>({method:'native-zone-membership-v1',effective_weights:'zone-membership',zone_data_applied:true,alpha:input.alpha,zone_types:d.zone_types,units:graph.geoids.length,source_name:input.source_name,source_sha256:sha,jointly_observed_edges:jointly,shared_edges:shared,changed_edges:changed,formula:'boundary-times-one-plus-alpha-times-shared-over-both-available',scope})};
}
export function verifyZoneWeighting(graph,input,value,scope='edge-weighting'){
 if(!input){verifyZoneFallback(value,scope);return null;}
 const computed=zoneWeights(graph,input);if(!/^[0-9a-f]{64}$/.test(value?.source_sha256||''))throw new Error('Missing zone source identity.');
 const expected=computed.evidence(value.source_sha256,scope);if(!exact(value,Object.keys(expected))||Object.entries(expected).some(([key,v])=>JSON.stringify(value[key])!==JSON.stringify(v)))throw new Error('Supplied zone weighting evidence disagrees with source assignments.');return computed;
}
export async function verifyZoneSource(graph,input,value,scope='edge-weighting'){const computed=verifyZoneWeighting(graph,input,value,scope);if(input&&await zoneIdentity(input)!==value.source_sha256)throw new Error('Zone source hash differs from retained bytes.');return computed;}
// With no selected source, mirror the native State command's warning/fallback.
export function zoneFallbackEvidence(scope='edge-weighting'){
 return {method:'native-zone-membership-fallback-v1',native_status:'spatial-join-not-implemented',effective_weights:'geographic',zone_data_applied:false,scope};
}
export function verifyZoneFallback(value,scope='edge-weighting'){
 const expected=zoneFallbackEvidence(scope);
 if(!value||Object.keys(value).sort().join()!==Object.keys(expected).sort().join()||Object.entries(expected).some(([key,v])=>value[key]!==v))throw new Error('Zone membership must record the native geographic fallback without applied zone data.');
}
