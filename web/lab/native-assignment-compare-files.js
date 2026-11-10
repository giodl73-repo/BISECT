import {encodeBytes,validateSourceFile} from './package-files.js';

export const DIRECT_ASSIGNMENT_DEFAULTS=Object.freeze({a:null,b:null});
const maxBytes=16*1024*1024;
const decoder=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});

function checkName(name){
  if(typeof name!=='string'||!name||name.length>200||/[\p{Cc}\/\\]/u.test(name))throw new Error('Use a plain assignment file name.');
}

function sourceText(source){
  validateSourceFile(source,{maxBytes});
  checkName(source.name);
  // Avoid the temporary iterator array made by Uint8Array.from on large strings.
  const binary=atob(source.base64),bytes=new Uint8Array(binary.length);
  for(let i=0;i<bytes.length;i++)bytes[i]=binary.charCodeAt(i);
  return decoder.decode(bytes);
}

function pair(value){
  if(!value||typeof value!=='object'||Object.keys(value).sort().join()!=='a,b')throw new Error('Invalid direct assignment sources.');
  return value;
}

export function validateDirectAssignmentInputs(value){
  pair(value);
  for(const key of ['a','b'])if(value[key]!==null)sourceText(value[key]);
  return value;
}

export function directAssignmentRequest(value){
  pair(value);
  if(!value.a||!value.b)throw new Error('Select both original assignment files.');
  const role=source=>({label:'./'+source.name,text:sourceText(source)});
  return {operation:'compare-native-assignment-files',input:{plan_a:role(value.a),plan_b:role(value.b)}};
}

export async function readDirectAssignmentFile(file){
  if(!file)return null;
  if(!Number.isSafeInteger(file.size)||file.size<1||file.size>maxBytes)throw new Error('Select a nonempty assignment file of at most 16 MiB.');
  checkName(file.name);
  const bytes=new Uint8Array(await file.arrayBuffer());
  if(bytes.length!==file.size)throw new Error('Assignment file changed while reading.');
  decoder.decode(bytes);
  return {name:file.name,base64:encodeBytes(bytes)};
}
