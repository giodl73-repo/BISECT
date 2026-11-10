export const FAILURE_REQUEST_LIMIT=25*1024*1024;
export const FAILURE_SCOPE='Recorded failed worker request and measured loaded WASM identity. This is not a resolved configuration, an accepted plan, proof that no partial work occurred, authenticated provenance or a legal certificate. Worker-owned runtime bindings and adapters may override submitted fields. Imported receipts are unverified data and never execute or fetch anything.';
const utf8=s=>new TextEncoder().encode(s),sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export function validateWorkerFailure(v){
 const keys='error,error_truncated,recorded_at,request_bytes,request_json,request_retention,request_sha256,runtime_artifact,schema_version,scope,submitted_operation';
 if(!v||Object.keys(v).sort().join()!==keys||v.schema_version!=='bisect-worker-failure-v1'||v.scope!==FAILURE_SCOPE||typeof v.error!=='string'||utf8(v.error).length>65539||typeof v.error_truncated!=='boolean'||typeof v.recorded_at!=='string'||utf8(v.recorded_at).length>100||!Number.isFinite(Date.parse(v.recorded_at))||typeof v.submitted_operation!=='string'||utf8(v.submitted_operation).length>200||!['retained','over-limit','unserializable'].includes(v.request_retention))throw new Error('Invalid worker failure receipt.');
 if(v.runtime_artifact!==null&&(!v.runtime_artifact||Object.keys(v.runtime_artifact).sort().join()!=='byte_length,sha256'||!/^[a-f0-9]{64}$/.test(v.runtime_artifact.sha256)||!Number.isSafeInteger(v.runtime_artifact.byte_length)||v.runtime_artifact.byte_length<1))throw new Error('Invalid recorded failure runtime identity.');
 if(v.request_retention==='unserializable'){if(v.request_json!==null||v.request_bytes!==null||v.request_sha256!==null)throw new Error('Unserializable failure request must not claim a digest.');}
 else{if(!Number.isSafeInteger(v.request_bytes)||v.request_bytes<0||!/^[a-f0-9]{64}$/.test(v.request_sha256))throw new Error('Invalid failure request digest metadata.');if(v.request_retention==='retained'){if(typeof v.request_json!=='string'||utf8(v.request_json).length!==v.request_bytes||v.request_bytes>FAILURE_REQUEST_LIMIT)throw new Error('Failure request bytes exceed retention limits.');}else if(v.request_json!==null||v.request_bytes<=FAILURE_REQUEST_LIMIT)throw new Error('Oversized failure request retention metadata is inconsistent.');}
 return v;
}
export async function makeWorkerFailure(request,runtimeArtifact,error,{now=()=>new Date().toISOString()}={}){
 const message=String(error?.message??error),bytes=utf8(message),v={schema_version:'bisect-worker-failure-v1',recorded_at:now(),submitted_operation:typeof request?.operation==='string'?new TextDecoder().decode(utf8(request.operation).subarray(0,197)):'engine',runtime_artifact:runtimeArtifact?{sha256:runtimeArtifact.sha256,byte_length:runtimeArtifact.byte_length}:null,error:bytes.length>65536?new TextDecoder().decode(bytes.subarray(0,65536)):message,error_truncated:bytes.length>65536,request_json:null,request_sha256:null,request_bytes:null,request_retention:'unserializable',scope:FAILURE_SCOPE};
 try{
  // Host File/Blob objects, cycles and non-JSON values cannot be replayed from JSON.
  const pending=[{value:request,depth:0}],active=new Set();let visited=0;
  while(pending.length){const {value,depth,exit}=pending.pop();if(exit){active.delete(value);continue;}if(++visited>4000000||depth>64)throw new Error('Request exceeds capture structure limits');if(value===null||typeof value==='string'||typeof value==='boolean')continue;if(typeof value==='number'){if(!Number.isFinite(value))throw new Error('Nonfinite request value');continue;}if(typeof value!=='object'||(!Array.isArray(value)&&![Object.prototype,null].includes(Object.getPrototypeOf(value)))||active.has(value))throw new Error('Non-JSON or cyclic request object');active.add(value);pending.push({value,depth,exit:true});for(const child of Object.values(value))pending.push({value:child,depth:depth+1});}
  const text=JSON.stringify(request);if(typeof text!=='string')throw new Error('No serializable request');const data=utf8(text);v.request_sha256=await sha(data);v.request_bytes=data.length;v.request_retention=data.length>FAILURE_REQUEST_LIMIT?'over-limit':'retained';v.request_json=v.request_retention==='retained'?text:null;}catch{}
 return validateWorkerFailure(v);
}
export async function verifyWorkerFailure(v){validateWorkerFailure(v);if(v.request_json===null)throw new Error('This receipt does not retain request bytes; use the original project sources.');return await sha(utf8(v.request_json))===v.request_sha256;}
// Display declaration text only. Never resolve YAML, follow paths, or execute a request.
export function workerFailureSeeds(v){
 validateWorkerFailure(v);if(!v.request_json)return [];
 const displayNumbers=(_k,value,context)=>typeof value==='number'&&!Number.isSafeInteger(value)&&(Number.isInteger(value)||!Number.isFinite(value))?(context?.source??'[exact numeric token available in retained request]'):value;
 const out=[];let root;try{root=JSON.parse(v.request_json,displayNumbers);}catch{return [];}
 const inspect=(r,prefix)=>{
  for(const path of [['seed'],['options','seed'],['request','options','seed'],['input','request','options','seed'],['input','options','seed'],['input','seed'],['input','form','seed'],['settings','seed']]){let node=r;for(const key of path)node=node&&typeof node==='object'&&Object.hasOwn(node,key)?node[key]:undefined;if(node!==undefined&&node!==null&&['string','number'].includes(typeof node))out.push(prefix+path.join('.')+': '+String(node).slice(0,200));}
  for(const [label,text]of [['yaml_text',r?.yaml_text],['input.yaml_text',r?.input?.yaml_text],['input.run.yaml_text',r?.input?.run?.yaml_text]])if(typeof text==='string'){for(const line of text.split('\n'))if(/^\s*seed\s*:/.test(line))out.push(prefix+label+': '+line.trim().slice(0,200));}
 };
 inspect(root,'');const raw=root?.input?.request_json;if(typeof raw==='string')try{inspect(JSON.parse(raw,displayNumbers),'input.request_json → ');}catch{}
 return [...new Set(out)].slice(0,20);
}
