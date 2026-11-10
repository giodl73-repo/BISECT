import {encodeBytes,isBase64Encoding} from './package-files.js';
import {packageVerificationRequest} from './package-verification-files.js';
const MiB=1024*1024;
const safe=name=>typeof name==='string'&&name.length>0&&name.length<=240&&/^[A-Za-z0-9_./+-]+$/.test(name)&&!name.split('/').some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p));
export function validateNativeReplaySources(input){
 if(!input||typeof input!=='object'||Array.isArray(input)||Object.keys(input).join()!=='sources')throw Error('Select a recorded source collection.');
 const sources=input.sources;
 if(!Array.isArray(sources)||sources.length>64)throw Error('Select at most 64 recorded source files.');
 const names=new Set();let total=0;
 for(const source of sources){
  if(!source||typeof source!=='object'||Array.isArray(source)||Object.keys(source).sort().join()!=='base64,name'||!safe(source.name)||names.has(source.name.toLowerCase()))throw Error('Invalid or case-colliding recorded source label.');
  names.add(source.name.toLowerCase());
  if(typeof source.base64!=='string'||source.base64.length>Math.ceil(64*MiB/3)*4||!isBase64Encoding(source.base64))throw Error('Invalid bounded recorded source encoding.');
  const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));total+=bytes.length;
  if(total>64*MiB||encodeBytes(bytes)!==source.base64)throw Error('Recorded sources must use canonical base64 and total at most 64 MiB.');
 }
 return input;
}
export async function readNativeReplaySources(files){
 const selected=Array.from(files);if(selected.length>64)throw Error('Select at most 64 recorded source files.');
 let total=0;for(const file of selected){if(!Number.isSafeInteger(file.size)||file.size<0||(total+=file.size)>64*MiB)throw Error('Select recorded source files totaling at most 64 MiB.');}
 const sources=[];for(const file of selected){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw Error('Selected source changed while reading.');sources.push({name:file.name,base64:encodeBytes(bytes)});}
 return validateNativeReplaySources({sources});
}
export function nativeReplaySourceRequest(packageInputs,sources){
 return {operation:'inspect-native-replay-sources',input:{prechecks:packageVerificationRequest(packageInputs,'inspect-native-replay-recipe').input,sources:validateNativeReplaySources(sources).sources}};
}
