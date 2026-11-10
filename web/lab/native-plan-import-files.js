import {validateSourceFile,encodeBytes,validatePackagePath} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const NATIVE_PLAN_IMPORT_DEFAULTS={source:null,settings:{label:'native-import',state:'RI',year:'2020',format:'geojson',created_at:'',as_civic_counter_proposal:false,submitted_by:null,submitted_at:null}};
export function validateNativePlanImportInputs(v,{complete=false}={}){
 const s=v?.settings;
 if(!v||Object.keys(v).sort().join()!=='settings,source'||!s||Object.keys(s).sort().join()!=='as_civic_counter_proposal,created_at,format,label,state,submitted_at,submitted_by,year'||typeof s.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(s.label)||typeof s.state!=='string'||!/^[A-Za-z]{2}$/.test(s.state)||typeof s.year!=='string'||!/^\d{4}$/.test(s.year)||typeof s.format!=='string'||new TextEncoder().encode(s.format).length>50||typeof s.created_at!=='string'||new TextEncoder().encode(s.created_at).length>200||complete&&!s.created_at.trim()||typeof s.as_civic_counter_proposal!=='boolean'||[s.submitted_by,s.submitted_at].some(text=>text!==null&&(typeof text!=='string'||new TextEncoder().encode(text).length>200)))throw new Error('Select native import metadata and a recorded timestamp.');
 if(v.source!==null){const {name,base64}=v.source;if(typeof name!=='string'||!name||new TextEncoder().encode(name).length>200||/[\/\\\u0000-\u001f\u007f]/.test(name))throw new Error('Select a source basename up to 200 UTF-8 bytes.');if(base64!=='')validateSourceFile(v.source,{maxBytes:3*1024*1024});else if(Object.keys(v.source).sort().join()!=='base64,name')throw new Error('Invalid native import source.');}
 if(complete&&!v.source)throw new Error('Select an external native import source.');return v;
}
export async function readNativePlanImportSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>3*1024*1024)throw new Error('Native plan import is limited to 3 MiB.');
 if(/\.(shp|shx|dbf|prj|zip)$/.test(file.name))throw new Error('Native plan import does not directly support shapefile-family or ZIP input. Convert the plan to GeoJSON first.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Native import source changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};validateNativePlanImportInputs({source,settings:NATIVE_PLAN_IMPORT_DEFAULTS.settings});return source;
}
export function nativePlanImportRequest(v){validateNativePlanImportInputs(v,{complete:true});return {operation:'import-native-plan',input:{source_base64:v.source.base64,settings:{...v.settings,source_name:v.source.name}}};}
export async function nativePlanImportZip(result){
 if(result?.schema_version!=='bisect-native-plan-import-v1'||!['completed','partial'].includes(result.status)||!result.files||Object.keys(result.files).length!==(result.status==='completed'?3:2)||Object.keys(result.file_sha256||{}).length!==Object.keys(result.files).length)throw new Error('Run a native import first.');
 const expected=[result.label+'.rplan','final_assignments.json',...(result.status==='completed'?['manifest.json']:[])].sort();if(Object.keys(result.files).sort().join()!==expected.join())throw new Error('Native import file set changed.');
 const entries=[];for(const [path,text]of Object.entries(result.files)){validatePackagePath(path);if(typeof text!=='string')throw new Error('Native import file is not text.');const bytes=new TextEncoder().encode(text);if(bytes.length>8*1024*1024)throw new Error('Native import output exceeds 8 MiB.');const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==result.file_sha256[path])throw new Error('Native import output hash mismatch.');entries.push([path,bytes]);}
 return storedZipBytes(entries,{maxSourceBytes:24*1024*1024});
}
