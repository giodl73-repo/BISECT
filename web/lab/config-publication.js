import {configTemplateRequest} from './config-template.js';
import {validatePackagePath} from './package-files.js';

export const CONFIG_PUBLICATION_DEFAULTS={path:'',force:false,dry_run:false};
const reviews=new WeakMap(),MAX=256*1024,MAX_EXISTING=8*1024*1024;
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
function safePath(path){
 validatePackagePath(path);
 const parts=path.split('/');
 if(parts.length>16||parts.some(p=>p.length>200||p.endsWith('.')||p.endsWith(' ')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)))throw Error('Choose a portable relative config path without reserved or trailing-dot names.');
 return parts;
}
export function validateConfigPublicationSettings(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!=='dry_run,force,path'||typeof value.path!=='string'||new TextEncoder().encode(value.path).length>4096||/[\u0000-\u001f\u007f-\u009f]/.test(value.path)||typeof value.force!=='boolean'||typeof value.dry_run!=='boolean')throw Error('Invalid config publication settings.');
 // Native dry-run never consumes --out. Retain bounded intent as inert text.
 if(!value.dry_run&&value.path!=='')safePath(value.path);
 return value;
}
async function permission(root,o){
 check(o);if(root?.kind!=='directory')throw Error('Select a live config destination folder.');
 if(typeof root.queryPermission==='function'&&await root.queryPermission({mode:'readwrite'})!=='granted')throw Error('Folder write permission is no longer granted. Select it again.');check(o);
}
async function entry(directory,name,o){
 check(o);let found=null,count=0;
 for await(const [actual,handle]of directory.entries()){
  check(o);if(++count>20000)throw Error('Config destination directory exceeds 20,000 entries.');
  if(actual.toLowerCase()===name.toLowerCase()){
   if(actual!==name||found)throw Error('Case-colliding config destination.');
   if(handle.name!==actual||!['file','directory'].includes(handle.kind))throw Error('Unsupported config destination entry.');found=handle;
  }
 }
 return found;
}
async function fileIdentity(handle,o){
 check(o);const file=await handle.getFile();check(o);
 if(!Number.isSafeInteger(file.size)||file.size<0||file.size>MAX_EXISTING)throw Error('Existing config destination exceeds the 8 MiB review limit.');
 const bytes=new Uint8Array(await file.arrayBuffer());check(o);if(bytes.length!==file.size)throw Error('Config destination changed while reading.');
 const sha256=await hash(bytes);check(o);return {kind:'file',sha256,byte_length:bytes.length};
}
async function observe(root,path,o,{readTarget=true}={}){
 await permission(root,o);const parts=safePath(path),parents=[];let dir=root,missing=false;
 for(let i=0;i<parts.length-1;i++){
  const prefix=parts.slice(0,i+1).join('/'),handle=missing?null:await entry(dir,parts[i],o);
  if(handle&&handle.kind!=='directory')throw Error('Config destination parent is a file: '+prefix);
  parents.push({path:prefix,kind:handle?'directory':'missing'});if(handle)dir=handle;else missing=true;
 }
 const target=missing?null:await entry(dir,parts.at(-1),o);check(o);
 return {parents,target:!target?{kind:'missing'}:target.kind==='directory'?{kind:'directory'}:readTarget?await fileIdentity(target,o):{kind:'file'}};
}
const equal=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
export function retireConfigPublicationReview(review){if(review)reviews.delete(review);}
export async function reviewConfigPublication(engine,result,template,settings,root,o={}){
 const recipe=structuredClone(validateConfigPublicationSettings(settings)),request=configTemplateRequest(structuredClone(template));
 const claimed={schema:result?.schema_version,yaml:result?.yaml_text,filename:result?.generated_filename,sha256:result?.source_sha256};
 const fresh=await engine.execute(request);check(o);
 if(claimed.schema!=='bisect-algorithm-config-template-v1'||fresh.schema_version!==claimed.schema||typeof fresh.yaml_text!=='string'||fresh.yaml_text!==claimed.yaml||fresh.generated_filename!==claimed.filename)throw Error('Generate this config again; the YAML differs from a fresh native recomputation.');
 const bytes=new TextEncoder().encode(fresh.yaml_text);if(bytes.length>MAX)throw Error('Generated YAML exceeds 256 KiB.');const sha256=await hash(bytes);check(o);
 if(sha256!==fresh.source_sha256||sha256!==claimed.sha256)throw Error('Generated config byte identity changed.');
 const path=recipe.path||'configs/'+fresh.generated_filename;
 let observed=null;
 if(!recipe.dry_run){
  safePath(path);observed=await observe(root,path,o,{readTarget:recipe.force});
  if(observed.target.kind!=='missing'&&!recipe.force){const error=Error('Config destination already exists. Enable force and review again to replace a file.');error.configPublicationReceipt={schema_version:'bisect-config-publication-outcome-v1',status:'refused',path,force:false,dry_run:false,root_name:root.name,yaml_sha256:sha256,byte_length:bytes.length,effects_may_have_started:false,write_started:false,readback_verified:false,reason:'existing-destination',authority:'Passive refusal record; no folder or write authority.'};throw error;}
  if(observed.target.kind==='directory')throw Error('Config destination is a directory; force cannot replace it with a file.');
 }
 check(o);const review=Object.freeze({schema_version:'bisect-config-publication-review-v1',path,force:recipe.force,dry_run:recipe.dry_run,root_name:recipe.dry_run?null:root.name,action:recipe.dry_run?'dry-run':observed.target.kind==='file'?'replace':'add',yaml_sha256:sha256,byte_length:bytes.length,previous_sha256:observed?.target.sha256??null,missing_parents:Object.freeze((observed?.parents??[]).filter(p=>p.kind==='missing').map(p=>p.path)),authority:'Private current-session review only. Native writer semantics adapted to a selected root; no OS-atomic transaction or external source authentication.'});
 reviews.set(review,{root:recipe.dry_run?null:root,observed,bytes,used:false});return review;
}
export async function applyConfigPublication(review,{confirmed=false,...o}={}){
 const grant=reviews.get(review);if(!grant||grant.used)throw Error('Review the current config destination again; imported or consumed reviews grant no authority.');
 if(confirmed!==true)throw Error('Confirm the concrete reviewed config destination.');grant.used=true;
 const receipt={schema_version:'bisect-config-publication-outcome-v1',status:'incomplete',path:review.path,force:review.force,dry_run:review.dry_run,root_name:review.root_name,yaml_sha256:review.yaml_sha256,byte_length:review.byte_length,previous_sha256:review.previous_sha256,created_parents:[],attempted_path:null,effects_may_have_started:false,file_creation_requested:false,writable_requested:false,write_started:false,close_completed:false,readback_verified:false,authority:'Historical byte/outcome record only; no filesystem grant or restored write authority. No OS-atomic/no-clobber transaction is claimed.'};
 const emit=()=>{try{o.onProgress?.(structuredClone(receipt));}catch{/* Observers cannot affect a write or grant authority. */}};
 try{
  check(o);if(review.dry_run){receipt.status='dry-run';emit();return receipt;}
  const current=await observe(grant.root,review.path,o);if(!equal(current,grant.observed))throw Error('Config destination changed after review.');
  const parts=safePath(review.path),expected=structuredClone(grant.observed);let dir=grant.root;
  for(let i=0;i<parts.length-1;i++){
   check(o);await permission(grant.root,o);let handle=await entry(dir,parts[i],o);const parent=expected.parents[i];
   if((handle?'directory':'missing')!==parent.kind||handle&&handle.kind!=='directory')throw Error('Config parent changed after review: '+parent.path);
   if(!handle){
    receipt.attempted_path=parent.path;receipt.effects_may_have_started=true;emit();check(o);handle=await dir.getDirectoryHandle(parts[i],{create:true});
    if(handle.name!==parts[i]||handle.kind!=='directory')throw Error('Config parent name changed during creation.');
    receipt.created_parents.push(parent.path);parent.kind='directory';receipt.attempted_path=null;emit();check(o);
   }
   dir=handle;
  }
  const beforeWrite=await observe(grant.root,review.path,o);if(!equal(beforeWrite,expected))throw Error('Config destination changed before writing.');
  await permission(grant.root,o);receipt.attempted_path=review.path;receipt.effects_may_have_started=true;receipt.file_creation_requested=expected.target.kind==='missing';emit();check(o);
  const handle=await dir.getFileHandle(parts.at(-1),{create:true});check(o);if(handle.name!==parts.at(-1)||handle.kind!=='file')throw Error('Config destination name changed during creation.');
  const opened=await fileIdentity(handle,o);
  if(expected.target.kind==='file'?!equal(opened,expected.target):opened.byte_length!==0)throw Error('Config destination changed while opening.');
  receipt.writable_requested=true;emit();check(o);const writer=await handle.createWritable();
  try{check(o);receipt.write_started=true;emit();check(o);await writer.write(grant.bytes);check(o);await writer.close();receipt.close_completed=true;emit();}
  catch(error){await writer.abort().catch(()=>{});throw error;}
  check(o);const after=await observe(grant.root,review.path,o);if(after.target.sha256!==review.yaml_sha256||after.target.byte_length!==review.byte_length||!equal(after.parents,expected.parents))throw Error('Config write read-back differs.');
  receipt.readback_verified=true;receipt.attempted_path=null;receipt.status='completed';emit();return receipt;
 }catch(error){const failure=error instanceof Error?error:Error(String(error));receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=String(failure.message).slice(0,2000);emit();failure.configPublicationReceipt=structuredClone(receipt);throw failure;}
 finally{reviews.delete(review);}
}
