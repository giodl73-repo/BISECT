import {inspectJsonText,parseSafeJson} from './project.js';
import {encodeBytes,isBase64Encoding} from './package-files.js';
import {configuredRunMapDocument} from './config-run.js';
import {packageVerificationRequest} from './package-verification-files.js';
const limit=16*1024*1024,record=v=>v&&typeof v==='object'&&!Array.isArray(v);
function sourceText(source,key){
 const cap=key==='original'?limit/2:limit;
 if(!record(source)||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||!source.name||source.name.length>240||typeof source.base64!=='string'||source.base64.length>Math.ceil(cap/3)*4||!isBase64Encoding(source.base64))throw new Error('Invalid bounded replay source.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(!bytes.length||bytes.length>cap||encodeBytes(bytes)!==source.base64)throw new Error('Replay sources exceed their byte limit.');
 // Match saved-workbench complexity for real state graphs; byte/depth,
 // duplicate-key and unsafe-key checks still apply before parsing.
 const text=new TextDecoder('utf-8',{fatal:true}).decode(bytes);inspectJsonText(text,4000000);const value=JSON.parse(text);if(!record(value))throw new Error('Replay sources must contain JSON objects.');
 if(key==='run'){const kinds=['engine','engine-build','multiscale','multiscale-build','configured','configured-build'],payload=value.kind?.startsWith('engine')?'request':'input';if(!kinds.includes(value.kind)||Object.keys(value).sort().join()!==['kind',payload,...(value.kind==='configured'&&Object.hasOwn(value,'attempt')?['attempt']:[]),...(value.kind?.startsWith('multiscale')&&Object.hasOwn(value,'fine_edges')?['fine_edges']:[])].sort().join()||!record(value[payload]))throw new Error('Select a supported run artifact with kind and request/input.');}
 if(key==='run'&&Object.hasOwn(value,'attempt')&&(value.kind!=='configured'||!Number.isInteger(value.attempt)||value.attempt<0||value.attempt>50))throw new Error('Configured replay attempt must be an integer from 0 through 50.');
 // Parsing only inspects structure; return the original text so u64 seeds are never rounded.
 return text;
}
export function validateEngineReplayInputs(v){
 if(!record(v)||!['minSimilarity','original','run'].every(k=>Object.hasOwn(v,k))||Object.keys(v).some(k=>!['minSimilarity','original','run','skipBinaryCheck'].includes(k))||Object.hasOwn(v,'skipBinaryCheck')&&typeof v.skipBinaryCheck!=='boolean'||!Number.isFinite(v.minSimilarity))throw new Error('Select replay sources and a finite native similarity threshold.');
 let bytes=0;for(const key of ['run','original'])if(v[key]!==null){const text=sourceText(v[key],key);bytes+=new TextEncoder().encode(text).length;}if(bytes>limit)throw new Error('Replay sources must total at most 16 MiB.');return v;
}
export function engineReplayRequest(v){validateEngineReplayInputs(v);if(!v.run||!v.original)throw new Error('Select a run artifact and original assignment JSON.');return {operation:'replay-engine-run',input:{run_json:sourceText(v.run,'run'),original_assignments_json:sourceText(v.original,'original'),min_similarity:v.minSimilarity}};}
export async function readEngineReplaySource(file,key){
 if(!['run','original'].includes(key))throw new Error('Unknown replay source.');const cap=key==='original'?limit/2:limit;
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>cap)throw new Error('Select a bounded nonempty replay JSON file.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Replay source size changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};sourceText(source,key);return source;
}
export function engineReplayReceiptBytes(result){
 if(result?.schema_version==='bisect-manifest-replay-verification-v1'){
  if(!['match','different','unavailable','precheck-failed','binding-failed','replay-failed'].includes(result.status)||result.verification_exit_code!==(result.status==='match'?0:1))throw new Error('Run combined manifest/replay verification first.');
  const text=JSON.stringify(result,null,2)+'\n';parseSafeJson(text);return new TextEncoder().encode(text);
 }
 if(result?.schema_version!=='bisect-engine-replay-verification-v1'||!['match','different','unavailable','failed'].includes(result.status)||result.comparison_exit_code!==(result.status==='match'?0:1))throw new Error('Run engine replay first.');
 const text=JSON.stringify(result,null,2)+'\n';parseSafeJson(text);return new TextEncoder().encode(text);
}
export function manifestReplayRequest(packageInputs,replayInputs){
 const prechecks=packageVerificationRequest(packageInputs,'verify-manifest-prechecks').input,replay=engineReplayRequest(replayInputs).input;
 const bytes=new TextEncoder().encode(prechecks.manifest_json).length+Object.values(prechecks.files_base64).reduce((n,b)=>n+atob(b).length,0)+new TextEncoder().encode(replay.run_json).length+new TextEncoder().encode(replay.original_assignments_json).length;
 if(bytes>limit)throw new Error('Combined verification sources exceed 16 MiB.');
 return {operation:'verify-manifest-replay',input:{prechecks,replay},...(Object.hasOwn(replayInputs,'skipBinaryCheck')?{skip_binary_check:replayInputs.skipBinaryCheck}:{})};
}
export function engineReplayFromConfigured(result){
 configuredRunMapDocument(result);
 const source=(name,value)=>({name,base64:encodeBytes(new TextEncoder().encode(JSON.stringify(value)))});
 const inputs={run:source('accepted-engine-run.json',{kind:'engine',request:result.engine_request}),original:source('original-final-assignments.json',result.result.assignments),minSimilarity:1};
 validateEngineReplayInputs(inputs);return inputs;
}
