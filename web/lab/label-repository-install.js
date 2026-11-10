import {labelPackageMergeRequest} from './label-package-merge-files.js';
const reviews=new WeakMap(),MAX=25*1024*1024;
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const encoded=text=>new TextEncoder().encode(text);
const check=options=>{options.signal?.throwIfAborted();options.assertCurrent?.();};
function pathParts(path){
 if(typeof path!=='string'||path.length>512)throw new Error('Unsafe repository path.');
 const parts=path.split('/');if(parts.some(part=>!part||part.length>200||!/^[-A-Za-z0-9_.]+$/.test(part)||['.','..','__proto__','constructor','prototype'].includes(part)||part.endsWith('.')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)))throw new Error('Unsafe repository path.');return parts;
}
async function permission(root,options){check(options);if(root?.kind!=='directory')throw new Error('Select a live repository directory.');if(typeof root.queryPermission==='function'&&await root.queryPermission({mode:'readwrite'})!=='granted')throw new Error('Repository write permission is no longer granted. Select it again.');check(options);}
async function child(directory,name,kind,create,options){
 check(options);let count=0;for await(const [actual]of directory.entries()){check(options);if(++count>20000)throw new Error('Repository directory exceeds 20,000 entries.');if(actual.toLowerCase()===name.toLowerCase()&&actual!==name)throw new Error('Case-colliding repository path.');}
 try{return kind==='directory'?await directory.getDirectoryHandle(name,{create}):await directory.getFileHandle(name,{create});}catch(error){if(!create&&error.name==='NotFoundError')return null;throw error;}
}
async function resolve(root,path,create,options){let directory=root;const parts=pathParts(path);for(const name of parts.slice(0,-1)){directory=await child(directory,name,'directory',create,options);if(!directory)return null;}return child(directory,parts.at(-1),'file',create,options);}
async function observation(root,path,options){
 check(options);const handle=await resolve(root,path,false,options);if(!handle)return null;
 const file=await handle.getFile();if(!Number.isSafeInteger(file.size)||file.size>MAX)throw new Error('Repository file exceeds 25 MiB.');const bytes=new Uint8Array(await file.arrayBuffer());check(options);if(bytes.length!==file.size)throw new Error('Repository file changed while reading.');return {sha256:await hash(bytes),byte_length:bytes.length};
}
async function observe(root,paths,options){await permission(root,options);let total=0;const observed={};for(const path of paths){const value=await observation(root,path,options);if((total+=value?.byte_length||0)>MAX)throw new Error('Selected repository snapshot exceeds 25 MiB.');observed[path]=value;}check(options);return observed;}
const identity=async value=>hash(encoded(JSON.stringify(value)));
function validateResult(result){
 if(result?.schema_version!=='bisect-label-package-merge-v1'||result.status!=='prepared'||!result.reconciliation||!result.files||!result.file_sha256||!result.current_file_sha256||!Array.isArray(result.changes)||result.changes.length>2000||Object.keys(result.files).length>2000)throw new Error('Prepare a fresh native repository merge first.');
 const paths=Object.keys(result.files),folded=new Set();let total=0;
 for(const path of paths){pathParts(path);if(folded.has(path.toLowerCase())||typeof result.files[path]!=='string'||(total+=encoded(result.files[path]).length)>MAX)throw new Error('Invalid repository file inventory.');folded.add(path.toLowerCase());}
 const changed=new Set();for(const change of result.changes){pathParts(change.path);if(changed.has(change.path)||!['add','replace','unchanged','reconcile'].includes(change.action)||!Object.hasOwn(result.files,change.path)||result.file_sha256[change.path]!==change.sha256)throw new Error('Invalid repository change inventory.');changed.add(change.path);}
 if(!changed.has('.bisect')||result.files['.bisect']!==result.reconciliation.registry_json)throw new Error('Missing reconciled registry.');return paths;
}
function expectedStates(result,observed,continuation){
 const changes=new Map(result.changes.map(change=>[change.path,change])),pending=[],retained=[];
 for(const [path,value]of Object.entries(observed)){
  const before=result.current_file_sha256[path]??null,after=result.file_sha256[path],actual=value?.sha256??null;
  if(continuation&&changes.has(path)&&actual===after){retained.push(path);continue;}
  if(actual!==before)throw new Error('Repository snapshot changed or has unknown partial bytes: '+path);
  if(changes.has(path)&&actual!==after)pending.push(path);
 }
 if(continuation&&retained.includes('.bisect')&&pending.length)throw new Error('Registry already records this stage but its artifact bytes are incomplete. Review/repair the repository separately.');
 const index=result.reconciliation.generated_index_path;
 pending.sort((a,b)=>a==='.bisect'?1:b==='.bisect'?-1:a===index?1:b===index?-1:a.localeCompare(b,'en'));
 return {pending,retained};
}
export async function reviewLabelRepositoryInstallation(engine,result,inputs,root,{continuation=false,...options}={}){
 if(typeof continuation!=='boolean')throw new Error('Choose fresh installation or explicit continuation.');
 const request=labelPackageMergeRequest(inputs),sourceBytes=Uint8Array.from(atob(inputs.source.base64),c=>c.charCodeAt(0)),sourceSha=await hash(sourceBytes),requestSha=await identity(request),sourceResult=await engine.execute(request);check(options);
 if(await identity(sourceResult)!==await identity(result))throw new Error('Repository merge differs from a fresh native recomputation.');
 const paths=validateResult(sourceResult);for(const path of paths)if(await hash(encoded(sourceResult.files[path]))!==sourceResult.file_sha256[path])throw new Error('Prepared repository file hash changed.');
 const selected=[...new Set([...Object.keys(sourceResult.current_file_sha256),...sourceResult.changes.map(row=>row.path)])].sort();
 const observed=await observe(root,selected,options),state=expectedStates(sourceResult,observed,continuation);
 const review=Object.freeze({schema_version:'bisect-label-repository-review-v1',root_name:root.name,mode:continuation?'continuation':'fresh',label:sourceResult.label,year:sourceResult.year,stage:sourceResult.stage,pending_paths:Object.freeze(state.pending.slice()),retained_paths:Object.freeze(state.retained.slice()),source_sha256:sourceSha,request_sha256:requestSha,prepared_sha256:await identity(sourceResult),registry_committed_last:true,write_authority:'current private session only; never serialized'});
 reviews.set(review,{root,result:structuredClone(sourceResult),observed,selected,used:false,requestIdentity:review.request_sha256});return review;
}
export function retireLabelRepositoryReview(review){if(review)reviews.delete(review);}
export async function applyLabelRepositoryInstallation(review,{confirmed=false,...options}={}){
 const grant=reviews.get(review);if(!grant||grant.used)throw new Error('Review this repository again; imported or consumed reviews grant no authority.');if(confirmed!==true)throw new Error('Confirm the concrete reviewed repository changes.');grant.used=true;
 const receipt={schema_version:'bisect-label-repository-installation-v1',status:'incomplete',label:review.label,year:review.year,stage:review.stage,mode:review.mode,source_sha256:review.source_sha256,request_sha256:review.request_sha256,prepared_sha256:review.prepared_sha256,root_name:review.root_name,completed_paths:[],retained_paths:[...review.retained_paths],attempted_path:null,registry_write_started:false,registry_readback_verified:false,installation_performed:false,effects_may_have_started:false,authority:'Historical record only. Open never restores a root, write grant or continuation authority.'};
 const expected=structuredClone(grant.observed);
 const emit=()=>{try{options.onProgress?.(structuredClone(receipt));}catch{/* Progress observers grant no authority and cannot change write outcomes. */}};
 async function recheckAll(){const current=await observe(grant.root,grant.selected,options);for(const path of grant.selected)if((current[path]?.sha256??null)!==(expected[path]?.sha256??null))throw new Error('Repository changed after review: '+path);}
 try{
  await recheckAll();
  const index=grant.result.reconciliation.generated_index_path;
  for(const path of review.pending_paths){
   check(options);await permission(grant.root,options);if(path==='.bisect'||path===index)await recheckAll();
   const current=await observation(grant.root,path,options);if((current?.sha256??null)!==(expected[path]?.sha256??null))throw new Error('Repository target changed before writing: '+path);
   receipt.attempted_path=path;receipt.effects_may_have_started=true;if(path==='.bisect')receipt.registry_write_started=true;emit();check(options);
   const bytes=encoded(grant.result.files[path]),handle=await resolve(grant.root,path,true,options);check(options);const writable=await handle.createWritable();
   try{check(options);await writable.write(bytes);check(options);await writable.close();}catch(error){await writable.abort().catch(()=>{});throw error;}
   const written=await observation(grant.root,path,options);if(written?.sha256!==grant.result.file_sha256[path]||written.byte_length!==bytes.length)throw new Error('Repository write read-back mismatch: '+path);
   expected[path]=written;receipt.completed_paths.push(path);receipt.installation_performed=true;if(path==='.bisect')receipt.registry_readback_verified=true;receipt.attempted_path=null;emit();
  }
  await recheckAll();
  // Every output, including unchanged current sources, must match the merge.
  for(const [path,sha256]of Object.entries(grant.result.file_sha256)){const actual=await observation(grant.root,path,options);if(actual?.sha256!==sha256)throw new Error('Final repository bytes differ: '+path);}
  receipt.registry_readback_verified=true;receipt.status='completed';emit();return receipt;
 }catch(error){receipt.status=options.signal?.aborted?'cancelled':'interrupted';receipt.error=error.message;emit();error.installationReceipt=structuredClone(receipt);throw error;}
 finally{reviews.delete(review);}
}
