import {validateSourceFile,encodeBytes} from './package-files.js';
export const MAX_POLICY_FALLBACK_BYTES=512*1024;
export function validatePolicyFallback(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!=='alias,primary')throw Error('Supply primary and alias policy sources.');
 for(const candidate of [value.primary,value.alias]){
  if(candidate===null)continue;
  if(!candidate||typeof candidate!=='object'||Array.isArray(candidate)||!['file','unavailable'].includes(candidate.status)||Object.keys(candidate).sort().join()!==(candidate.status==='file'?'base64,name,status':'name,status')||typeof candidate.name!=='string'||!candidate.name||new TextEncoder().encode(candidate.name).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(candidate.name))throw Error('Invalid policy fallback source.');
  if(candidate.status==='file'&&candidate.base64!=='')validateSourceFile({name:candidate.name,base64:candidate.base64},{maxBytes:MAX_POLICY_FALLBACK_BYTES});
 }
 return value;
}
export async function readPolicyFallbackSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>MAX_POLICY_FALLBACK_BYTES)throw Error('Select a policy source of at most 512 KiB.');
 validatePolicyFallback({primary:{status:'unavailable',name:file.name},alias:null});
 let bytes;try{bytes=new Uint8Array(await file.arrayBuffer());}catch{return {status:'unavailable',name:file.name};}
 if(bytes.length!==file.size)throw Error('Policy source changed while reading. Select it again.');
 const candidate={status:'file',name:file.name,base64:encodeBytes(bytes)};validatePolicyFallback({primary:candidate,alias:null});return candidate;
}
