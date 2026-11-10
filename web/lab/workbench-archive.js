import {WORKBENCH_ARCHIVE_SCHEMA,MAX_WORKBENCH_ARCHIVE_BYTES,MAX_WORKBENCH_EXPANDED_BYTES,MAX_WORKBENCH_VALUES,validateJsonTree,validateExpandedWorkbenchArchive,validateWorkbenchArchive,parseBuildArchiveJson,parseCompressedWorkbenchJson,readValidatedFile} from './project.js?v=workbench-archive-v1';
export function workbenchArchiveSnapshot({name,files,operation,constraints=[],result=null,lastOperation=null}){
 return validateExpandedWorkbenchArchive({schema_version:WORKBENCH_ARCHIVE_SCHEMA,name:name||'Untitled project',saved_at_utc:new Date().toISOString(),files,operation,constraints,result,lastOperation});
}
export const SHARED_WORKBENCH_ARCHIVE_SCHEMA='bisect-workbench-archive-v2';
const PROJECT_FIELDS=['schema_version','name','saved_at_utc','files','operation','constraints','result','lastOperation'];
const SHARED_FIELDS=[...PROJECT_FIELDS,'shared_strings','shared_string_refs'];
const MIN_SHARED_STRING_CHARS=64*1024,MAX_SHARED_STRINGS=2048,MAX_SHARED_REFERENCES=10000;
const PREFER_COMPRESSED_ARCHIVE_BYTES=32*1024*1024;
const encoder=new TextEncoder();
const byteLength=text=>encoder.encode(text).byteLength;

// Count the actual JSON representation before building a large output string.
// Only validated plain JSON reaches this walk. Memoize large repeated strings.
function jsonByteLength(root,limit=MAX_WORKBENCH_ARCHIVE_BYTES){
 const pending=[root],largeSizes=new Map();let total=0;
 const stringSize=text=>{if(text.length<MIN_SHARED_STRING_CHARS)return byteLength(JSON.stringify(text));let size=largeSizes.get(text);if(size===undefined){size=byteLength(JSON.stringify(text));largeSizes.set(text,size);}return size;};
 while(pending.length){const value=pending.pop();
  if(typeof value==='string')total+=stringSize(value);
  else if(Array.isArray(value)){total+=2+Math.max(0,value.length-1);for(const item of value)pending.push(item);}
  else if(value!==null&&typeof value==='object'){const keys=Object.keys(value);total+=2+Math.max(0,keys.length-1);for(const key of keys){total+=stringSize(key)+1;pending.push(value[key]);}}
  else {const encoded=JSON.stringify(value);if(encoded===undefined)throw new Error('Archive contains a non-JSON value.');total+=encoded.length;}
  if(total>limit)return total;
 }return total;
}
function sharedRepresentation(project){
 const strings=[],references=[],indices=new Map();
 const copy=(value,path)=>{
  if(typeof value==='string'&&value.length>=MIN_SHARED_STRING_CHARS){
   let index=indices.get(value);if(index===undefined){index=strings.length;if(index>=MAX_SHARED_STRINGS)throw new Error('Archive has too many shared strings.');strings.push(value);indices.set(value,index);}
   if(references.length>=MAX_SHARED_REFERENCES)throw new Error('Archive has too many shared references.');
   references.push({path,string_index:index});return null;
  }
  if(Array.isArray(value))return value.map((item,index)=>copy(item,[...path,index]));
  if(value!==null&&typeof value==='object')return Object.fromEntries(Object.entries(value).map(([key,item])=>[key,copy(item,[...path,key])]));
  return value;
 };
 const body=copy(project,[]);body.schema_version=SHARED_WORKBENCH_ARCHIVE_SCHEMA;
 return {...body,shared_strings:strings,shared_string_refs:references};
}
function createTextArchive(project,size){
 // Keep existing v1 files and their interpretation unchanged when they fit.
 if(size<=MAX_WORKBENCH_ARCHIVE_BYTES)return JSON.stringify(project);
 const shared=sharedRepresentation(project);
 if(jsonByteLength(shared)>MAX_WORKBENCH_ARCHIVE_BYTES)throw new Error('Workbench archive exceeds 128 MiB after sharing repeated content.');
 validateJsonTree(shared,MAX_WORKBENCH_VALUES,MAX_WORKBENCH_ARCHIVE_BYTES);return JSON.stringify(shared);
}
export function createWorkbenchArchive(data){
 const project=workbenchArchiveSnapshot(data);
 return createTextArchive(project,jsonByteLength(project));
}
function restoreSharedArchive(archive){
 if(Object.keys(archive).length!==SHARED_FIELDS.length||SHARED_FIELDS.some(key=>!Object.hasOwn(archive,key)))throw new Error('Invalid shared archive fields.');
 const strings=archive.shared_strings,references=archive.shared_string_refs;
 if(!Array.isArray(strings)||!strings.length||strings.length>MAX_SHARED_STRINGS||strings.some(text=>typeof text!=='string')||!Array.isArray(references)||!references.length||references.length>MAX_SHARED_REFERENCES)throw new Error('Invalid shared archive table.');
 const project=Object.fromEntries(PROJECT_FIELDS.map(key=>[key,archive[key]]));project.schema_version=WORKBENCH_ARCHIVE_SCHEMA;
 const sizes=strings.map(byteLength),used=new Set(),paths=new Set(),targets=[];
 // Reserve the full expanded string budget before installing even one ref.
 let expandedBytes=0;const stack=[project];while(stack.length){const value=stack.pop();if(typeof value==='string')expandedBytes+=byteLength(value);else if(value!==null&&typeof value==='object')for(const [key,item]of Object.entries(value)){expandedBytes+=byteLength(key);stack.push(item);}}
 for(const ref of references){
  if(!ref||typeof ref!=='object'||Array.isArray(ref)||Object.keys(ref).sort().join()!=='path,string_index'||!Array.isArray(ref.path)||!ref.path.length||ref.path.length>64||!Number.isSafeInteger(ref.string_index)||Object.is(ref.string_index,-0)||ref.string_index<0||ref.string_index>=strings.length)throw new Error('Invalid shared archive reference.');
  const identity=JSON.stringify(ref.path);if(paths.has(identity))throw new Error('Duplicate shared archive reference.');paths.add(identity);
  let parent=project;for(let i=0;i<ref.path.length;i++){
   const key=ref.path[i];
   if(!parent||typeof parent!=='object'||(Array.isArray(parent)?!Number.isSafeInteger(key)||Object.is(key,-0)||key<0||key>=parent.length:typeof key!=='string')||['__proto__','constructor','prototype'].includes(key)||!Object.hasOwn(parent,key))throw new Error('Invalid shared archive reference path.');
   if(i===ref.path.length-1){if(parent[key]!==null)throw new Error('Shared archive reference must target a null placeholder.');targets.push([parent,key,ref.string_index]);}else parent=parent[key];
  }
  expandedBytes+=sizes[ref.string_index];if(expandedBytes>MAX_WORKBENCH_EXPANDED_BYTES)throw new Error('Shared archive expanded strings exceed 256 MiB.');used.add(ref.string_index);
 }
 if(used.size!==strings.length)throw new Error('Shared archive contains unused content.');
 for(const [parent,key,index]of targets)parent[key]=strings[index];
 return validateExpandedWorkbenchArchive(project);
}
export function parseWorkbenchArchive(text){
 const archive=parseBuildArchiveJson(text);
 return archive.schema_version===SHARED_WORKBENCH_ARCHIVE_SCHEMA?restoreSharedArchive(archive):validateWorkbenchArchive(archive);
}
async function boundedStream(stream,cap,message){
 const reader=stream.getReader(),chunks=[];let total=0;
 try{for(;;){const {done,value}=await reader.read();if(done)break;total+=value.byteLength;if(total>cap)throw new Error(message);chunks.push(value);}}
 catch(error){await reader.cancel().catch(()=>{});throw error;}finally{reader.releaseLock();}
 const bytes=new Uint8Array(total);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.byteLength;}return bytes;
}
async function compressProject(project){
 if(typeof CompressionStream!=='function')throw new Error('This browser cannot compress a larger workbench archive.');
 const shared=sharedRepresentation(project);
 if(!shared.shared_strings.length)throw new Error('Larger compressed archive requires shared content.');
 if(jsonByteLength(shared,MAX_WORKBENCH_EXPANDED_BYTES)>MAX_WORKBENCH_EXPANDED_BYTES)throw new Error('Compressed archive decoded JSON exceeds 256 MiB.');
 validateJsonTree(shared,MAX_WORKBENCH_VALUES,MAX_WORKBENCH_EXPANDED_BYTES);
 const bytes=encoder.encode(JSON.stringify(shared));
 return boundedStream(new Blob([bytes]).stream().pipeThrough(new CompressionStream('gzip')),MAX_WORKBENCH_ARCHIVE_BYTES,'Compressed workbench archive exceeds 128 MiB.');
}
export async function createCompressedWorkbenchArchive(data){
 return compressProject(workbenchArchiveSnapshot(data));
}
export async function createStoredWorkbenchArchive(data){
 const project=workbenchArchiveSnapshot(data),size=jsonByteLength(project);
 // Keep portable small files and the existing validated fallback. Prefer the
 // already-supported gzip format when it materially reduces large downloads.
 if(size>=PREFER_COMPRESSED_ARCHIVE_BYTES&&size<=MAX_WORKBENCH_ARCHIVE_BYTES&&
    typeof CompressionStream==='function'&&typeof DecompressionStream==='function'){
  try{const packed=await compressProject(project);if(packed.byteLength<size)return packed;}
  catch{/* The validated plain archive still fits. */}
 }
 try{return createTextArchive(project,size);}
 catch(error){if(error.message!=='Workbench archive exceeds 128 MiB after sharing repeated content.')throw error;return compressProject(project);}
}
export async function parseWorkbenchArchiveBytes(bytes){
 if(!(bytes instanceof Uint8Array)||!bytes.length||bytes.byteLength>MAX_WORKBENCH_ARCHIVE_BYTES)throw new Error('Select a workbench archive within 128 MiB.');
 if(bytes[0]!==31||bytes[1]!==139)return parseWorkbenchArchive(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
 if(typeof DecompressionStream!=='function')throw new Error('This browser cannot open compressed workbench archives.');
 const decoded=await boundedStream(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip')),MAX_WORKBENCH_EXPANDED_BYTES,'Compressed archive decoded JSON exceeds 256 MiB.');
 const archive=parseCompressedWorkbenchJson(new TextDecoder('utf-8',{fatal:true}).decode(decoded));
 if(archive.schema_version!==SHARED_WORKBENCH_ARCHIVE_SCHEMA)throw new Error('Compressed archive requires the shared workbench schema.');
 return restoreSharedArchive(archive);
}
export async function readWorkbenchArchiveText(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_WORKBENCH_ARCHIVE_BYTES)throw new Error('Select a workbench archive within 128 MiB.');
 const bytes=await file.arrayBuffer();if(bytes.byteLength!==file.size)throw new Error('Archive changed while reading.');return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}
export function readWorkbenchArchiveFile(file,{signal}={}){return readValidatedFile(file,new URL('./workbench-archive-worker.js',import.meta.url),{signal,maxBytes:MAX_WORKBENCH_ARCHIVE_BYTES,timeoutMs:60000});}
export function serializeWorkbenchArchive(data,{signal}={}){
 return new Promise((resolve,reject)=>{if(signal?.aborted){reject(new Error('Archive save cancelled.'));return;}const worker=new Worker(new URL('./workbench-archive-worker.js',import.meta.url),{type:'module'});let settled=false;
  const finish=(error,text)=>{if(settled)return;settled=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(text);},abort=()=>finish(new Error('Archive save cancelled.')),timer=setTimeout(()=>finish(new Error('Archive save timed out.')),60000);
  signal?.addEventListener('abort',abort,{once:true});worker.onmessage=({data:r})=>r?.ok&&(typeof r.text==='string'||r.bytes instanceof Uint8Array)?finish(null,r.text??r.bytes):finish(new Error(r?.error||'Archive save failed.'));worker.onerror=worker.onmessageerror=()=>finish(new Error('Archive save failed.'));
  try{worker.postMessage({type:'save-workbench-archive',data});}catch(error){finish(error);}
 });
}
