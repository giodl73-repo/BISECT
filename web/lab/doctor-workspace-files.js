import {validateSourceFile,encodeBytes} from './package-files.js';
export const NATIVE_DOCTOR_DEFAULTS={enabled:false,output_base:'outputs',policy:{primary:null,alias:null,home:null},manifest:{primary:null,home:null},outputs:null,workspace:null};
const closed=(v,keys)=>!!v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join()===keys.slice().sort().join();
function source(v,max){if(v===null)return;if(!v||!['file','unavailable'].includes(v.status)||!closed(v,v.status==='file'?['status','name','base64']:['status','name'])||typeof v.name!=='string'||!v.name||new TextEncoder().encode(v.name).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(v.name))throw Error('Invalid native Doctor source.');if(v.status==='file')validateSourceFile({name:v.name,base64:v.base64},{maxBytes:max,allowEmpty:true});}
export function nativeDoctorPath(path){if(typeof path!=='string'||!path||path.length>1024||/[^\x20-\x7e]|[\\:*?"<>|]/.test(path)||path.startsWith('/')||path.split('/').some(p=>!p||p==='.'||p==='..'||/[. ]$/.test(p)))throw Error('Doctor snapshots require portable relative ASCII paths.');return path;}
export function validateNativeDoctorSnapshot(v){
 if(v===null)return;if(!closed(v,['entries'])||!Array.isArray(v.entries)||v.entries.length>20000)throw Error('Doctor snapshots support up to 20,000 entries.');const seen=new Map();let total=0;
 for(const e of v.entries){if(!closed(e,['path','kind'])||!['file','directory'].includes(e.kind))throw Error('Invalid Doctor snapshot entry.');nativeDoctorPath(e.path);total+=e.path.length;const key=e.path.toLowerCase();if(seen.has(key))throw Error('Duplicate/aliased Doctor snapshot path.');seen.set(key,e.kind);}
 if(total>8*1024*1024)throw Error('Doctor snapshot paths exceed 8 MiB.');for(const path of seen.keys()){const parts=path.split('/');parts.pop();while(parts.length){if(seen.get(parts.join('/'))!=='directory')throw Error('Doctor snapshots need directory ancestors.');parts.pop();}}return v;
}
export function validateNativeDoctorInputs(v){
 if(!closed(v,Object.keys(NATIVE_DOCTOR_DEFAULTS))||typeof v.enabled!=='boolean'||typeof v.output_base!=='string'||new TextEncoder().encode(v.output_base).length>4096||/[\u0000-\u001f\u007f-\u009f]/.test(v.output_base)||!closed(v.policy,['primary','alias','home'])||!closed(v.manifest,['primary','home']))throw Error('Invalid native Doctor workspace recipe.');
 for(const c of Object.values(v.policy))source(c,512*1024);for(const c of Object.values(v.manifest))source(c,2*1024*1024);validateNativeDoctorSnapshot(v.outputs);validateNativeDoctorSnapshot(v.workspace);return v;
}
export function nativeDoctorWorkspaceRequest(settings,value){validateNativeDoctorInputs(value);const {enabled,...input}=value;if(!enabled)throw Error('Enable native Doctor snapshot mode.');return {operation:'diagnose-native-location-workspace',input:{settings:structuredClone(settings),...structuredClone(input)}};}
export async function readNativeDoctorCandidate(file,max){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>max)throw Error('Source exceeds the '+max+' byte limit.');source({status:'unavailable',name:file.name},max);let bytes;try{bytes=new Uint8Array(await file.arrayBuffer());}catch{return {status:'unavailable',name:file.name};}if(bytes.length!==file.size)throw Error('Source changed while reading.');const result={status:'file',name:file.name,base64:encodeBytes(bytes)};source(result,max);return result;
}
export async function collectNativeDoctorSnapshot(root,{signal}={}){
 if(root?.kind!=='directory'||typeof root.entries!=='function')throw Error('Select a directory root.');const entries=[];
 async function visit(directory,prefix,depth){if(depth>32)throw Error('Doctor folder review exceeds 32 levels.');for await(const [name,handle]of directory.entries()){
  if(signal?.aborted)throw Error('Doctor folder review cancelled.');const path=prefix+name;nativeDoctorPath(path);if(!['file','directory'].includes(handle.kind))throw Error('Unsupported directory entry.');entries.push({path,kind:handle.kind});if(entries.length>20000)throw Error('Doctor folder review exceeds 20,000 entries.');if(handle.kind==='directory')await visit(handle,path+'/',depth+1);
 }}
 await visit(root,'',0);if(signal?.aborted)throw Error('Doctor folder review cancelled.');entries.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);return validateNativeDoctorSnapshot({entries});
}
