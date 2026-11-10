import {validateSourceFile,readElectionSource} from './package-files.js';
import {parseSafeJson,parsePreparedGraphJson} from './project.js';
import {configValidationRequest} from './config-files.js';
export const CONFIG_RUN_DEFAULTS={chamber:'congressional',districts:2};
export function configRunSourceText(source){
 validateSourceFile(source);const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
}
export function validateConfigRunSettings(value){
 if(!value||!['chamber','districts'].every(k=>Object.hasOwn(value,k))||Object.keys(value).some(k=>!['chamber','districts','native_build','capture_rounds','vra_absent_file_fallback','seats_per_district','total_seats','compact_geometry'].includes(k))||Object.hasOwn(value,'native_build')&&typeof value.native_build!=='boolean'||Object.hasOwn(value,'vra_absent_file_fallback')&&value.vra_absent_file_fallback!==true||Object.hasOwn(value,'capture_rounds')&&typeof value.capture_rounds!=='boolean'||(typeof value.chamber!=='string'||value.chamber.length===0||new TextEncoder().encode(value.chamber).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(value.chamber))||!Number.isSafeInteger(value.districts)||value.districts<1||value.districts>500)throw new Error('Choose a chamber and 1–500 districts, with an optional native retry switch.');if(Object.hasOwn(value,'compact_geometry')&&!['prepared','native-state-circular'].includes(value.compact_geometry))throw new Error('Choose prepared or native State circular CompactBisect geometry.');if(Object.hasOwn(value,'seats_per_district')&&(!Number.isSafeInteger(value.seats_per_district)||value.seats_per_district<0||value.seats_per_district>4294967295)||Object.hasOwn(value,'total_seats')&&(!Number.isSafeInteger(value.total_seats)||value.total_seats<0||value.total_seats>4294967295))throw new Error('Choose 0–4294967295 seats per district and an optional total in the same range.');return value;
}
export function validateConfigRunInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(k=>!['graph','demographics','coi'].includes(k)))throw new Error('Invalid configuration run sources.');
 for(const [key,source] of Object.entries(value))validateSourceFile(source,key==='coi'?{maxBytes:3*1024*1024}:{});
 return value;
}
export async function readConfigRunSource(file){
 const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('Run source size changed while reading.');
 parsePreparedGraphJson(configRunSourceText(source));return source;
}
export function configRunRequest(source,profile,inputs,settings){
 validateConfigRunInputs(inputs);validateConfigRunSettings(settings);
 if(!inputs.graph)throw new Error('Select the prepared state graph JSON.');
 const config=configValidationRequest(source,profile),graph=parsePreparedGraphJson(configRunSourceText(inputs.graph));
 if(graph.schema_version!==1||!/^[A-Z]{2}$/.test(graph.state)||!/^\d{4}$/.test(graph.year)||!Array.isArray(graph.geoids))throw new Error('Select a scoped prepared state graph, not map GeoJSON.');
 return {operation:settings.native_build===true?'run-algorithm-build':'run-algorithm-config',input:{yaml_text:config.yaml_text,source_name:config.source_name,profile:config.profile,graph,chamber:settings.chamber,districts:settings.districts,...(Object.hasOwn(settings,'seats_per_district')||Object.hasOwn(settings,'total_seats')?{electoral:{seats_per_district:settings.seats_per_district??1,...(Object.hasOwn(settings,'total_seats')?{total_seats:settings.total_seats}:{})}}:{}),...(settings.vra_absent_file_fallback===true?{vra_absent_file_fallback:true}:{}),...(settings.capture_rounds===true?{capture_rounds:true}:{}),...(Object.hasOwn(settings,'compact_geometry')?{compact_geometry:settings.compact_geometry}:{}),...(inputs.demographics?{demographics:parseSafeJson(configRunSourceText(inputs.demographics))}:{}),...(inputs.coi?{coi:{source_name:inputs.coi.name,weights_json:configRunSourceText(inputs.coi)}}:{})}};
}
// Display adapter only: native engine labels are one-based; RPLAN map labels are zero-based.
export function configuredRunMapDocument(result){
 const graph=result?.engine_request?.graph,options=result?.engine_request?.options,assignments=result?.result?.assignments;
 if(result?.schema_version!=='bisect-configured-run-v1'||!graph||!options||!assignments||Object.keys(assignments).length!==graph.geoids?.length)throw new Error('Configured result has incomplete assignments.');
 return {plan:{k:options.districts,units:{unit_ids:graph.geoids,state:graph.state,year:graph.year},assignment:graph.geoids.map(id=>{const d=assignments[id];if(!Number.isInteger(d)||d<1||d>options.districts)throw new Error('Invalid engine district label.');return d-1;})}};
}
export function configuredRunExportRequest(result,{label,created_at}){
 configuredRunMapDocument(result);
 const c=result.configuration,r=result.engine_request;
 if(!c||typeof c.yaml_text!=='string')throw new Error('Missing original run configuration.');
 for(const text of [label,created_at])if(typeof text!=='string'||!text.trim()||new TextEncoder().encode(text).length>200)throw new Error('Export label and timestamp must be nonempty and at most 200 UTF-8 bytes.');
 return {operation:'export-configured-plan',input:{yaml_text:c.yaml_text,source_name:c.source_name,profile:c.profile,graph:r.graph,chamber:c.execution_chamber,districts:r.options.districts,...(r.options.vra_absent_file_fallback===true?{vra_absent_file_fallback:true}:{}),...(r.options.capture_rounds===true?{capture_rounds:true}:{}),...(Object.hasOwn(r.options,'compact_geometry')?{compact_geometry:r.options.compact_geometry}:{}),...(r.demographics?{demographics:r.demographics}:{}),...(r.coi?{coi:r.coi}:{}),...(r.electoral?{electoral:r.electoral}:{})},assignments:result.result.assignments,label,created_at,...(result.build_report?{attempt:result.build_report.accepted_attempt}:{})};
}

// Selected COI JSON stays exact UTF-8 text; native Rust validates numeric factors.
export async function readConfigCoiSource(file){if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>3*1024*1024)throw new Error('Select nonempty COI JSON up to 3 MiB.');const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('COI source changed while reading.');configRunSourceText(source);return source;}

// Source scope only; Rust validates physical geometry and graph consistency on Run.
const graphScopeCache=new WeakMap();
export function preparedGraphScope(source){
 if(!source)throw new Error('Select a prepared engine graph first.');
 const cached=graphScopeCache.get(source);if(cached?.base64===source.base64)return cached.scope;
 const graph=parsePreparedGraphJson(configRunSourceText(source)),kind=graph.unit_kind??'tract';
 if(graph.schema_version!==1||!/^[A-Z]{2}$/.test(graph.state)||!/^\d{4}$/.test(graph.year)||!['county','tract','block-group','block','precinct','imported'].includes(kind)||!Array.isArray(graph.geoids)||!graph.geoids.length)throw new Error('Select a prepared engine graph with a supported unit type, state and year. Audit contexts and map GeoJSON are separate inputs.');
 const scope=Object.freeze({state:graph.state,year:graph.year,unit_kind:kind,resolution:kind==='block-group'?'block_group':kind,units:graph.geoids.length,county_identity:['precinct','imported'].includes(kind)?'explicit county IDs':'Census GEOID prefix'});
 graphScopeCache.set(source,{base64:source.base64,scope});return scope;
}
