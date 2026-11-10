import {labelImportRequest,labelImportFiles} from './label-import-files.js';
import {encodeBytes} from './package-files.js';
const grants=new WeakMap(),MAX_BEFORE=25*1024*1024,MAX_MIXED=41*1024*1024;
const encoded=text=>new TextEncoder().encode(text);
const hash=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
const identity=async value=>hash(encoded(JSON.stringify(value)));
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
function pathParts(path){
 if(typeof path!=='string'||path.length>512)throw Error('Unsafe import output path.');const parts=path.split('/');
 if(parts.some(p=>!p||p.length>200||!/^[-A-Za-z0-9_.]+$/.test(p)||['.','..','__proto__','constructor','prototype'].includes(p)||p.endsWith('.')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)))throw Error('Unsafe import output path.');return parts;
}
const same=(a,b)=>a===null?b===null:b!==null&&a.sha256===b.sha256&&a.byte_length===b.byte_length;
export function validateLabelImportBaseline(value){
 if(!value||Object.keys(value).sort().join()!=='before,index_path,label,prepared_sha256,request_sha256,schema_version,year'||value.schema_version!=='bisect-label-import-baseline-v1'||typeof value.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(value.label)||typeof value.year!=='string'||!/^\d{4}$/.test(value.year)||!['request_sha256','prepared_sha256'].every(k=>typeof value[k]==='string'&&/^[a-f0-9]{64}$/.test(value[k]))||!value.before||Array.isArray(value.before)||Object.keys(value.before).length>53)throw Error('Invalid import installation baseline.');
 const root='runs/'+value.label+'/'+value.year+'/';if(value.index_path!==root+'index.json'||!Object.hasOwn(value.before,'.bisect')||!Object.hasOwn(value.before,value.index_path))throw Error('Import baseline has no expected index/registry.');
 let total=0;for(const [path,observed]of Object.entries(value.before)){pathParts(path);if(path!=='.bisect'&&path!==value.index_path&&(!path.startsWith(root)||!/^[a-z_]+\/assignments\.json$/.test(path.slice(root.length))))throw Error('Import baseline contains an unrelated path.');if(observed!==null){if(!observed||Object.keys(observed).sort().join()!=='byte_length,sha256'||typeof observed.sha256!=='string'||!/^[a-f0-9]{64}$/.test(observed.sha256)||!Number.isSafeInteger(observed.byte_length)||observed.byte_length<0||(total+=observed.byte_length)>MAX_BEFORE)throw Error('Import baseline contains invalid byte identities.');}}
 return value;
}
async function permission(root,o,write=false){check(o);if(root?.kind!=='directory')throw Error('Select a live import repository directory.');if(root.queryPermission&&await root.queryPermission({mode:write?'readwrite':'read'})!=='granted')throw Error('Import repository permission is no longer granted. Select it again.');check(o);}
async function child(parent,name,kind,create,o){check(o);let count=0;for await(const [actual,handle]of parent.entries()){check(o);if(++count>20000)throw Error('Import directory exceeds 20,000 entries.');if(actual.toLowerCase()===name.toLowerCase()){if(actual!==name)throw Error('Case-colliding import path.');if(handle.kind!==kind)throw Error('Import path has the wrong file/directory kind.');}}
 try{return kind==='directory'?await parent.getDirectoryHandle(name,{create}):await parent.getFileHandle(name,{create});}catch(e){if(!create&&e.name==='NotFoundError')return null;throw e;}}
async function resolve(root,path,create,o){let directory=root;const parts=pathParts(path);for(const name of parts.slice(0,-1)){directory=await child(directory,name,'directory',create,o);if(!directory)return null;}return child(directory,parts.at(-1),'file',create,o);}
async function fileBytes(handle,o,limit=MAX_BEFORE){check(o);const file=await handle.getFile();if(!Number.isSafeInteger(file.size)||file.size<0||file.size>limit)throw Error('Import target exceeds the selected byte limit.');const b=new Uint8Array(await file.arrayBuffer());check(o);if(b.length!==file.size)throw Error('Import target changed while reading.');return b;}
async function observation(root,path,o){const h=await resolve(root,path,false,o);if(!h)return null;const b=await fileBytes(h,o);return {sha256:await hash(b),byte_length:b.length};}
async function observe(root,paths,o,limit=MAX_MIXED){await permission(root,o,true);const out={};let total=0;for(const path of paths){const v=await observation(root,path,o);if((total+=v?.byte_length||0)>limit)throw Error('Selected import targets exceed snapshot byte limits.');out[path]=v;}check(o);return out;}
export async function readLabelImportRegistry(root,o={}){await permission(root,o);const handle=await resolve(root,'.bisect',false,o);if(!handle)return null;const b=await fileBytes(handle,o,1024*1024);return {name:'.bisect',base64:encodeBytes(b)};}
export async function reviewLabelImportApplication(engine,result,inputs,root,{continuation=false,baseline=null,...o}={}){
 if(typeof continuation!=='boolean')throw Error('Choose fresh import review or explicit continuation.');const request=structuredClone(labelImportRequest(inputs)),requestSha=await identity(request),fresh=await engine.execute(request);check(o);
 if(await identity(fresh)!==await identity(result))throw Error('Import differs from fresh native recomputation.');
 const entries=await labelImportFiles(fresh,inputs);check(o);const files=Object.fromEntries(entries),paths=Object.keys(files).sort(),after={};for(const path of paths){pathParts(path);after[path]={sha256:await hash(files[path]),byte_length:files[path].length};}
 const originalRegistry=request.input.registry_json===null?null:{sha256:await hash(encoded(request.input.registry_json)),byte_length:encoded(request.input.registry_json).length},preparedSha=await identity(fresh);
 const observed=await observe(root,paths,o,continuation?MAX_MIXED:MAX_BEFORE);let before;
 if(continuation){validateLabelImportBaseline(baseline);if(baseline.request_sha256!==requestSha||baseline.prepared_sha256!==preparedSha||baseline.label!==fresh.label||baseline.year!==fresh.year||baseline.index_path!==fresh.index_path||JSON.stringify(Object.keys(baseline.before).sort())!==JSON.stringify(paths)||!same(baseline.before['.bisect'],originalRegistry))throw Error('Import baseline does not match original sources, registry and prepared output.');before=structuredClone(baseline.before);}
 else{if(!same(observed['.bisect'],originalRegistry))throw Error('Live registry differs from the prepared snapshot. Read it and prepare the import again; no registry may be silently replaced.');before=structuredClone(observed);}
 const pending=[],retained=[],changes=[];
 for(const path of paths){const actual=observed[path];if(continuation&&!same(actual,before[path])&&!same(actual,after[path]))throw Error('Import target has unknown partial or concurrent bytes: '+path);if(same(actual,after[path]))retained.push(path);else pending.push(path);changes.push(Object.freeze({path,action:same(actual,after[path])?'unchanged':actual===null?'add':'replace',before_sha256:actual?.sha256??null,after_sha256:after[path].sha256,byte_length:after[path].byte_length}));}
 if(continuation&&!same(originalRegistry,after['.bisect'])&&same(observed['.bisect'],after['.bisect'])&&pending.some(p=>p!=='.bisect'))throw Error('Registry records this import but its artifact bytes are incomplete. Review/repair separately.');
 pending.sort((a,b)=>a==='.bisect'?1:b==='.bisect'?-1:a===fresh.index_path?1:b===fresh.index_path?-1:a.localeCompare(b,'en'));
 const baselineData=structuredClone({schema_version:'bisect-label-import-baseline-v1',label:fresh.label,year:fresh.year,index_path:fresh.index_path,request_sha256:requestSha,prepared_sha256:preparedSha,before});validateLabelImportBaseline(baselineData);
 const review=Object.freeze({schema_version:'bisect-label-import-review-v1',root_name:root.name,label:fresh.label,year:fresh.year,mode:continuation?'continuation':'fresh',format:fresh.format,source_sha256:fresh.source.sha256,request_sha256:requestSha,prepared_sha256:preparedSha,pending_paths:Object.freeze(pending),retained_paths:Object.freeze(retained),changes:Object.freeze(changes),registry_last:true,authority:'Private single-use current root review only. Serialized baselines and receipts grant no authority.'});
 grants.set(review,{root,files,paths,after,observed,baseline:baselineData,used:false});return {review,baseline:structuredClone(baselineData)};
}
export function retireLabelImportReview(review){if(review)grants.delete(review);}
export async function applyLabelImportApplication(review,{confirmed=false,...o}={}){
 const g=grants.get(review);if(!g||g.used)throw Error('Review import again; imported or consumed reviews grant no authority.');if(confirmed!==true)throw Error('Confirm the concrete reviewed import overwrites.');g.used=true;
 const receipt={schema_version:'bisect-label-import-application-v1',status:'incomplete',root_name:review.root_name,label:review.label,year:review.year,mode:review.mode,source_sha256:review.source_sha256,request_sha256:review.request_sha256,prepared_sha256:review.prepared_sha256,baseline:structuredClone(g.baseline),completed_paths:[],retained_paths:[...review.retained_paths],attempted_path:null,effects_may_have_started:false,registry_write_started:false,registry_readback_verified:false,installation_performed:false,authority:'Historical data only. Baseline hashes are declared prior observations, not authenticated history or a write grant. Open never selects or writes a root.'};
 const expected=structuredClone(g.observed),emit=()=>{try{o.onProgress?.(structuredClone(receipt));}catch{}};
 async function recheck(){const current=await observe(g.root,g.paths,o);for(const path of g.paths)if(!same(current[path],expected[path]))throw Error('Import repository changed after review: '+path);}
 try{
  await recheck();for(const path of review.pending_paths){await recheck();check(o);receipt.attempted_path=path;emit();check(o);receipt.effects_may_have_started=true;if(path==='.bisect')receipt.registry_write_started=true;
   const handle=await resolve(g.root,path,true,o);check(o);const stream=await handle.createWritable();try{check(o);await stream.write(g.files[path]);check(o);await stream.close();}catch(e){await stream.abort().catch(()=>{});throw e;}
   const written=await observation(g.root,path,o);if(!same(written,g.after[path]))throw Error('Import write read-back mismatch: '+path);expected[path]=written;receipt.completed_paths.push(path);receipt.installation_performed=true;if(path==='.bisect')receipt.registry_readback_verified=true;receipt.attempted_path=null;emit();
  }
  await recheck();for(const path of g.paths)if(!same(expected[path],g.after[path]))throw Error('Final import differs from intended bytes: '+path);
  receipt.registry_readback_verified=true;receipt.status='completed';emit();return receipt;
 }catch(e){receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=e.message;emit();e.importReceipt=structuredClone(receipt);throw e;}finally{grants.delete(review);}
}
