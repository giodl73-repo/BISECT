import {validateLabelBuildApplicationBaseline} from './label-build-application.js';
import {parseBuildArchiveJson,validateJsonTree,readValidatedFile} from './project.js?v=label-build-archive-v1';
import {validateLabelBuildInputs} from './label-build-files.js?v=label-build-checkpoint-v1';
export const MAX_BUILD_ARCHIVE_BYTES=128*1024*1024;
const fields=['schema_version','name','saved_at_utc','files','operation','constraints','result','lastOperation'];
const safePath=p=>typeof p==='string'&&p.length>0&&p.length<=240&&/^[A-Za-z0-9_./ -]+$/.test(p)&&!p.split('/').some(x=>!x||['.','..','__proto__','constructor','prototype'].includes(x));
export function validateBuildArchive(p){
 validateJsonTree(p,500000,MAX_BUILD_ARCHIVE_BYTES);
 if(!p||Array.isArray(p)||Object.keys(p).length!==fields.length||fields.some(k=>!Object.hasOwn(p,k))||p.schema_version!=='bisect-build-project-v1'||typeof p.name!=='string'||p.name.length>200||typeof p.saved_at_utc!=='string'||p.saved_at_utc.length>40||!Number.isFinite(Date.parse(p.saved_at_utc))||p.operation!=='run-label-build'||!Array.isArray(p.constraints)||p.constraints.length||!p.files||!Object.hasOwn(p.files,'labelBuildInputs')||Object.keys(p.files).some(k=>!['labelBuildInputs','labelBuildApplicationBaseline'].includes(k))||![null,'run-label-build'].includes(p.lastOperation))throw new Error('Invalid build archive fields.');
 validateLabelBuildInputs(p.files.labelBuildInputs);
 if(Object.hasOwn(p.files,'labelBuildApplicationBaseline'))validateLabelBuildApplicationBaseline(p.files.labelBuildApplicationBaseline);
 const r=p.result;
 if(r!==null){
  const allowed=['schema_version','status','label','states','years','invalidated_downstream','files','output_hashes','scope','expected_jobs','checkpoint_sequence','checkpoint_request_sha256'];
  if(!r||Array.isArray(r)||Object.keys(r).some(k=>!allowed.includes(k))||r.schema_version!=='bisect-label-build-v1'||!['running','cancelled','interrupted','checkpoint-error','built','partial-failure','skipped'].includes(r.status)||typeof r.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(r.label)||!Array.isArray(r.states)||r.states.length>600||!Array.isArray(r.years)||r.years.length>10||!r.files||Array.isArray(r.files)||Object.keys(r.files).length>2000)throw new Error('Invalid retained build result.');
  let size=0;for(const [path,text]of Object.entries(r.files)){if(!safePath(path)||typeof text!=='string')throw new Error('Invalid retained build output path or text.');size+=new TextEncoder().encode(text).length;}
  if(size>64*1024*1024)throw new Error('Retained build output exceeds 64 MiB.');
 }
 return p;
}
export function createBuildArchive({name,inputs,result=null,lastOperation=null,baseline=null}){
 const p={schema_version:'bisect-build-project-v1',name:name||'Untitled build',saved_at_utc:new Date().toISOString(),files:{labelBuildInputs:inputs,...(baseline!==null?{labelBuildApplicationBaseline:baseline}:{})},operation:'run-label-build',constraints:[],result,lastOperation};
 const text=JSON.stringify(validateBuildArchive(p));
 if(new TextEncoder().encode(text).length>MAX_BUILD_ARCHIVE_BYTES)throw new Error('Build archive exceeds 128 MiB.');
 return text;
}
export function parseBuildArchive(text){return validateBuildArchive(parseBuildArchiveJson(text));}
export async function readBuildArchiveText(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_BUILD_ARCHIVE_BYTES)throw new Error('Select a nonempty build archive within 128 MiB.');
 const bytes=await file.arrayBuffer();if(bytes.byteLength!==file.size)throw new Error('Build archive size changed while reading.');
 return new TextDecoder('utf-8',{fatal:true}).decode(bytes);
}
export function readBuildArchiveFile(file,{signal}={}){return readValidatedFile(file,new URL('./label-build-archive-worker.js?v=label-build-archive-v1',import.meta.url),{signal,maxBytes:MAX_BUILD_ARCHIVE_BYTES,timeoutMs:60000});}
export function serializeBuildArchive(data){
 return new Promise((resolve,reject)=>{
  const worker=new Worker(new URL('./label-build-archive-worker.js?v=label-build-archive-v2',import.meta.url),{type:'module'});
  let settled=false;const timer=setTimeout(()=>finish(new Error('Build archive preparation timed out.')),60000);
  function finish(error,text){if(settled)return;settled=true;clearTimeout(timer);worker.terminate();error?reject(error):resolve(text);}
  worker.onmessage=({data:r})=>{if(r?.ok===true&&typeof r.text==='string')finish(null,r.text);else finish(new Error(r?.error||'Build archive preparation failed.'));};
  worker.onerror=worker.onmessageerror=()=>finish(new Error('Build archive preparation failed.'));
  try{worker.postMessage({type:'save-build-archive',data});}catch(e){finish(e);}
 });
}
