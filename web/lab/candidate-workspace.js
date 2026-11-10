import {candidateRequest} from './candidate-files.js';

const reviews=new WeakMap();
const text=new TextEncoder();
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const binary=base64=>Uint8Array.from(atob(base64),c=>c.charCodeAt(0));
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
const limits={entries:20000,file:16*1024*1024};
const same=(a,b)=>a===b||a&&b&&typeof a.isSameEntry==='function'&&a.isSameEntry(b);
function safePart(name){
 if(typeof name!=='string'||!/^[A-Za-z0-9_. -]{1,200}$/.test(name)||name==='.'||name==='..'||/[. ]$/.test(name)||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(name)||['__proto__','constructor','prototype'].includes(name.toLowerCase()))throw new Error('Use portable workspace names without reserved or trailing-dot names.');
 return name;
}
async function permission(root,o){
 check(o);if(root?.kind!=='directory'||typeof root.entries!=='function')throw new Error('Select a live workspace output folder.');
 if(typeof root.queryPermission==='function'&&await root.queryPermission({mode:'readwrite'})!=='granted')throw new Error('Folder write permission is no longer granted.');check(o);
}
async function entries(dir,wanted,o){
 const actual=new Map(),match=new Map(wanted.map(n=>[n.toLowerCase(),n]));let count=0;
 for await(const [name,h] of dir.entries()){
  check(o);if(++count>limits.entries)throw new Error('Workspace directory exceeds 20,000 entries.');
  const key=match.get(name.toLowerCase());if(!key)continue;
  if(name!==key||actual.has(key))throw new Error('Case-colliding workspace entry: '+name);
  if(h.name!==name||!['file','directory'].includes(h.kind))throw new Error('Unsupported workspace entry: '+name);
  actual.set(key,h);
 }
 return actual;
}
async function identity(handle,o){
 if(!handle)return null;
 if(handle.kind!=='file')throw new Error('Package destination is a directory: '+handle.name);
 check(o);const file=await handle.getFile();check(o);
 if(!Number.isSafeInteger(file.size)||file.size<0||file.size>limits.file)throw new Error('Workspace file exceeds 16 MiB review limit: '+handle.name);
 const bytes=new Uint8Array(await file.arrayBuffer());check(o);if(bytes.length!==file.size)throw new Error('Workspace file changed while reading: '+handle.name);
 const sha256=await hash(bytes);check(o);return {handle,sha256,byte_length:bytes.length};
}
async function observe(root,parts,names,o){
 await permission(root,o);const dirs=[];let dir=root;
 for(const n of parts){
  const h=dir?(await entries(dir,[n],o)).get(n):null;
  if(h&&h.kind!=='directory')throw new Error('Package directory path is a file: '+n);
  dirs.push(h??null);dir=h;
 }
 const found=dir?await entries(dir,names,o):new Map(),files=new Map();
 for(const n of names)files.set(n,await identity(found.get(n),o));
 return {dirs,files};
}
async function unchanged(want,got){
 for(let i=0;i<want.dirs.length;i++)if(!await same(want.dirs[i],got.dirs[i]))throw new Error('Package folder changed after review.');
 for(const [name,x] of want.files){
  const y=got.files.get(name);
  if(x===null&&y===null)continue;
  if(!x||!y||x.sha256!==y.sha256||x.byte_length!==y.byte_length||!await same(x.handle,y.handle))throw new Error('Package destination changed after review: '+name);
 }
}
function assertFresh(claimed,fresh,request){
 const names=['annotations.json','manifest.json','provenance.json'];
 for(const r of [claimed,fresh]){
  if(r?.schema_version!=='bisect-candidate-ingest-v1'||r.status!=='packaged'||r.staged_path!==request.staged_path||r.manifest?.label!==request.settings.label||Object.keys(r.files??{}).sort().join()!==names.join()||names.some(n=>typeof r.files[n]!=='string'))throw new Error('Run the candidate package with these sources first.');
 }
 if(names.some(n=>claimed.files[n]!==fresh.files[n]))throw new Error('Candidate package differs from a fresh native recomputation.');
 return names;
}
export async function reviewCandidateWorkspace(engine,result,inputs,root,{version='v1',replace=false,...o}={}){
 if(typeof replace!=='boolean'||!/^[A-Za-z0-9_-]{1,100}$/.test(version))throw new Error('Use a portable output version.');
 const request=candidateRequest(structuredClone(inputs));
 const parts=[safePart(version),'civic_inputs',safePart(request.settings.label)];
 const doc=safePart(request.staged_path.split('/').at(-1));
 const nativeFiles=['candidate_race.csv',doc,'annotations.json','manifest.json','provenance.json'];
 if(new Set(nativeFiles.map(n=>n.toLowerCase())).size!==nativeFiles.length)throw new Error('Attestation name collides with a generated package file.');
 await permission(root,o);
 const fresh=await engine.execute(request);check(o);const jsonFiles=assertFresh(result,fresh,request);
 const raw=new Map([['candidate_race.csv',binary(request.csv_base64)],[doc,binary(request.documents[request.staged_path])],...jsonFiles.map(n=>[n,text.encode(fresh.files[n])])]);
 const paths=Object.create(null);
 for(const n of nativeFiles){const data=raw.get(n);if(data.length>limits.file)throw new Error('Generated package file exceeds 16 MiB.');paths[n]=Object.freeze({sha256:await hash(data),byte_length:data.length});check(o);}
 const manifest=JSON.parse(fresh.files['manifest.json']);
 for(const [name,key] of [['candidate_race.csv','original_csv'],[doc,'attestation_doc'],['annotations.json','annotations'],['provenance.json','provenance']])if(paths[name].sha256!==manifest[key+'_sha256'])throw new Error('Generated package manifest does not bind: '+name);
 if(manifest.attestation_doc!==doc||manifest.original_csv!=='candidate_race.csv'||manifest.label!==request.settings.label)throw new Error('Generated candidate package paths differ from the request.');
 const observed=await observe(root,parts,nativeFiles,o);
 const actions=nativeFiles.map(name=>Object.freeze({name,...paths[name],action:observed.files.get(name)?'replace':'create',previous_sha256:observed.files.get(name)?.sha256??null}));
 if(!replace&&actions.some(row=>row.action==='replace'))throw new Error('Package files already exist. Enable replacement and review again.');
 const review=Object.freeze({schema_version:'bisect-candidate-workspace-review-v1',root_name:root.name,path:parts.join('/'),actions:Object.freeze(actions),replace});
 reviews.set(review,{root,parts,nativeFiles,observed,raw});return review;
}
export function retireCandidateWorkspaceReview(review){if(review)reviews.delete(review);}
export async function applyCandidateWorkspace(review,{confirmed=false,onProgress,...o}={}){
 const grant=reviews.get(review);
 if(!grant)throw new Error('Review the selected output folder again.');
 if(confirmed!==true)throw new Error('Confirm the reviewed candidate package writes.');
 if(!globalThis.navigator?.locks?.request)throw new Error('This browser cannot coordinate workspace writers. Download the ZIP instead.');
 reviews.delete(review);
 const {root,parts,nativeFiles,observed,raw}=grant;
 const receipt={schema_version:'bisect-candidate-workspace-write-v1',status:'refused',root_name:review.root_name,path:review.path,
  effects_may_have_started:false,entries:review.actions.map(a=>({...a,status:'pending'}))};
 const run=async()=>{
  const snapshot=()=>observe(root,parts,nativeFiles,o);
  const want={dirs:[...observed.dirs],files:new Map(observed.files)};
  await unchanged(want,await snapshot());check(o);
  let dir=root;
  for(const [i,n] of parts.entries()){
   await unchanged(want,await snapshot());check(o);
   if(!want.dirs[i]){
    receipt.effects_may_have_started=true;const created=await dir.getDirectoryHandle(n,{create:true});check(o);
    want.dirs[i]=created;
   }
   dir=want.dirs[i];
  }
  for(const row of receipt.entries){
   await unchanged(want,await snapshot());check(o);
   receipt.effects_may_have_started=true;row.status='started';onProgress?.(structuredClone(receipt));check(o);
   const previous=want.files.get(row.name);
   const handle=previous?.handle??await dir.getFileHandle(row.name,{create:true});check(o);
   const writer=await handle.createWritable();
   try{check(o);await writer.write(raw.get(row.name));check(o);await writer.close();}catch(e){await writer.abort().catch(()=>{});throw e;}
   const back=await identity(handle,o);
   if(back.sha256!==row.sha256||back.byte_length!==row.byte_length)throw new Error('Package read-back differs: '+row.name);
   want.files.set(row.name,back);row.status='verified';onProgress?.(structuredClone(receipt));
  }
  await unchanged(want,await snapshot());check(o);receipt.status='completed';return receipt;
 };
 try{return await navigator.locks.request('bisect-candidate-workspace:'+review.path.toLowerCase(),o.signal?{signal:o.signal}:{},run);}
 catch(cause){
  const error=new Error(cause?.message??String(cause),{cause});
  receipt.status=o.signal?.aborted?'cancelled':receipt.effects_may_have_started?'interrupted':'refused';receipt.reason=error.message;error.candidateWorkspaceReceipt=receipt;throw error;
 }
}
