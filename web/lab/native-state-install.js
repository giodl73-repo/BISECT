import {stateCommandArguments} from './state-command-files.js';
import {validateNativeStateNameEffects} from './native-state-name-effects.js';
import {nativeStateOutputEntries,nativeOutputHash,nativeOutputComponent,nativeStateNameComponent} from './native-state-output.js';
const encode=value=>new TextEncoder().encode(JSON.stringify(value,null,2)+'\n');
// Recovery authority is session-local and comes only from a live folder review.
const recoveryReviews=new WeakMap();
const dataResetReviews=new WeakMap();
const targetActionReviews=new WeakMap();
const nameOK=name=>typeof name==='string'&&name.length<=150&&name.trim()===name&&/^[-A-Za-z0-9_. ]+$/.test(name)&&!['.','..','__proto__','constructor','prototype'].includes(name)&&!name.endsWith('.')&&!/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(name);
async function find(root,parts,{create=false,signal,assertCurrent}={}){let directory=root;for(const part of parts){signal?.throwIfAborted();assertCurrent?.();if(!nameOK(part))throw new Error('Unsafe native output component.');try{directory=await directory.getDirectoryHandle(part,{create});signal?.throwIfAborted();assertCurrent?.();}catch(error){if(!create&&error.name==='NotFoundError')return null;throw error;}}return directory;}
async function observe(root,namespace,{signal}={}){signal?.throwIfAborted();
 const directory=await find(root,namespace.split('/')),entries=[],manifests={};let total=0,marker_text=null;
 async function visit(dir,prefix=''){for await(const [name,handle]of dir.entries()){signal?.throwIfAborted();if(!nameOK(name)||entries.length>=1000)throw new Error('Target has unsupported paths or exceeds 1,000 observed entries.');const path=prefix+name;if(path.length>300)throw new Error('Target path exceeds 300 characters.');if(handle.kind==='directory'){entries.push({path,kind:'directory'});await visit(handle,path+'/');}else if(handle.kind==='file'){const file=await handle.getFile();if(!Number.isSafeInteger(file.size)||file.size<0||file.size>128*1024*1024||(total+=file.size)>128*1024*1024)throw new Error('Target review exceeds 128 MiB.');const data=new Uint8Array(await file.arrayBuffer());signal?.throwIfAborted();if(data.length!==file.size)throw new Error('Target file changed while reading.');entries.push({path,kind:'file',byte_length:data.length,sha256:await nativeOutputHash(data)});if(path==='manifest.tmp'){if(data.length>64*1024)throw new Error('Target incomplete marker exceeds 64 KiB.');try{marker_text=new TextDecoder('utf-8',{fatal:true}).decode(data);}catch{/* An invalid marker cannot authorize recovery. */}}if(path==='manifest.json'){if(data.length>1024*1024)throw new Error('Target manifest exceeds 1 MiB.');try{manifests[path]=new TextDecoder('utf-8',{fatal:true}).decode(data);}catch{/* Existence still blocks the native non-force guard. */}}}else throw new Error('Unsupported target entry.');}}
 if(directory)await visit(directory);entries.sort((a,b)=>a.path.localeCompare(b.path,'en'));const evidence={exists:!!directory,entries};return {...evidence,manifests,marker_text,sha256:await nativeOutputHash(encode(evidence))};
}
function guardRequest(output,snapshot,force){const relative=output.target_namespace.slice(output.version.length+1),paths=snapshot.entries.map(e=>relative+'/'+e.path);if(snapshot.exists)paths.push(relative);return {operation:output.state_name_command?'inspect-native-named-state-run-target':output.layout==='state'?'inspect-native-state-run-target':'inspect-native-run-target',version:output.version,...(output.state_name_command?{arguments:stateCommandArguments(output.state_name_command)}:{year:output.year,...(output.layout==='state'?{state:output.state}:{label:output.label})}),input:{source_name:'live-native-output-target.json',folder_json:JSON.stringify({version:output.version,paths,manifests:Object.fromEntries(Object.entries(snapshot.manifests).map(([name,text])=>[relative+'/'+name,text]))}),force}};}
async function nativeGuard(execute,output,snapshot,force){
 const result=await execute(guardRequest(output,snapshot,force));if(output.state_name_command&&JSON.stringify(result.identity)!==JSON.stringify(validateNativeStateNameEffects(output.native_name_effects).preview.identity))throw Error('Native naming or policy changed; prepare and review the named output again.');if(result.schema_version!=='bisect-native-run-target-v1'||result.status!=='ready'||result.target_namespace!=='outputs/'+output.target_namespace||result.force!==force)throw new Error('Invalid native target guard response.');
 return result;
}
async function guard(execute,output,snapshot,force){
 const result=await nativeGuard(execute,output,snapshot,force);
 if(snapshot.entries.some(e=>['.final_assignments.tmp.json','.vra_analysis.tmp.json','.provenance.tmp.json'].includes(e.path.split('/').at(-1))))throw new Error('Native temporary data files indicate an incomplete target. Recover it before installation.');
 if(!force&&snapshot.entries.length)throw new Error('The target contains existing files or directories. Review and select overwrite, or choose a different version/export label.');return result;
}
export async function reviewNativeStateInstallation(execute,root,output,{force=false,signal}={}){
 if(root?.kind!=='directory'||typeof force!=='boolean'||!nativeOutputComponent(output?.version))throw new Error('Choose the local outputs directory.');await nativeStateOutputEntries(output);
 const snapshot=await observe(root,output.target_namespace,{signal}),native_guard=await guard(execute,output,snapshot,force);
 return {schema_version:'bisect-native-state-install-review-v1',root,output:structuredClone(output),force,snapshot,native_guard,package_sha256:await nativeOutputHash(encode(output)),display:{directory_name:root.name??'selected outputs folder',target_namespace:output.target_namespace,files_to_write:Object.entries(output.file_sha256).map(([path,sha256])=>({path,sha256})),existing_entries:snapshot.entries,force}};
}
// Native State checks manifest.tmp and labeled collisions before --reset, then
// removes only plan_root/data. Its separate --force gate does not mean replacing
// the entire target. This action intentionally preserves the existing installer.
export async function reviewNativeStateDataReset(execute,root,output,{force=false,signal}={}){
 if(root?.kind!=='directory'||typeof force!=='boolean')throw new Error('Choose the local outputs directory.');await nativeStateOutputEntries(output);
 return reviewDataResetScope(execute,root,output,{force,signal});
}
async function reviewDataResetScope(execute,root,output,{force,signal}){
 const snapshot=await observe(root,output.target_namespace,{signal}),native_guard=await nativeGuard(execute,output,snapshot,force),data=snapshot.entries.find(e=>e.path==='data');
 if(data&&data.kind!=='directory')throw new Error('Native data reset requires data/ to be a directory; a file at that path is not removed.');
 signal?.throwIfAborted();const package_sha256=await nativeOutputHash(encode(output)),paths=snapshot.entries.filter(e=>e.path==='data'||e.path.startsWith('data/'));
 const review={schema_version:'bisect-native-state-data-reset-review-v1',root,output:structuredClone(output),force,snapshot,native_guard,package_sha256,display:{directory_name:root.name??'selected outputs folder',target_namespace:output.target_namespace,reset_path:output.target_namespace+'/data',data_exists:!!data,entries_to_remove:paths,preserved_entries:snapshot.entries.filter(e=>e.path!=='data'&&!e.path.startsWith('data/')),force}};
 dataResetReviews.set(review,{root,package_sha256,snapshot_sha256:snapshot.sha256,force});return review;
}
export async function observeNativeOutputTarget(root,namespace,options={}){
 if(root?.kind!=='directory'||typeof namespace!=='string'||!namespace||namespace.length>300||namespace.split('/').some(part=>!nameOK(part)))throw new Error('Choose a live folder and a bounded portable target namespace.');
 return observe(root,namespace,options);
}
function targetIntent(intent){
 if(intent?.schema_version!=='bisect-native-state-target-intent-v1'||typeof intent.reset!=='boolean'||typeof intent.force!=='boolean'||!nativeOutputComponent(intent.version)||!/^\d{4}$/.test(intent.year)||!/^[A-Z]{2}$/.test(intent.state)||!/^([a-f0-9]{64})$/.test(intent.request_sha256)||!/^([a-f0-9]{64})$/.test(intent.binding_sha256)||!['state','labeled'].includes(intent.layout)||!(intent.layout==='state'?nativeStateNameComponent(intent.state_slug):nativeOutputComponent(intent.label))||intent.target_namespace!==intent.version+'/'+intent.year+'/'+(intent.layout==='state'?'states/'+intent.state_slug:'plans/'+intent.label))throw new Error('Review a native-derived target intent for this exact Run.');
 return intent;
}
export async function reviewNativeStateTargetActions(execute,root,intent,{signal}={}){
 targetIntent(intent);if(root?.kind!=='directory')throw new Error('Choose the local outputs directory.');
 const reset_review=intent.reset?await reviewDataResetScope(execute,root,intent,{force:intent.force,signal}):null;
 const snapshot=reset_review?.snapshot??await observe(root,intent.target_namespace,{signal}),native_guard=reset_review?.native_guard??await nativeGuard(execute,intent,snapshot,intent.force);
 const review={schema_version:'bisect-native-state-target-action-review-v1',root,intent:structuredClone(intent),snapshot,native_guard,reset_review,display:reset_review?.display??{directory_name:root.name??'selected outputs folder',target_namespace:intent.target_namespace,entries_to_remove:[],preserved_entries:snapshot.entries,data_exists:snapshot.entries.some(e=>e.path==='data'),force:intent.force}};
 targetActionReviews.set(review,{root,intent_sha256:await nativeOutputHash(encode(intent)),snapshot_sha256:snapshot.sha256,reset_review});return review;
}
export async function performNativeStateTargetActions(execute,review,{signal,assertCurrent}={}){
 const authority=targetActionReviews.get(review);if(!authority||review.root!==authority.root||await nativeOutputHash(encode(review.intent))!==authority.intent_sha256||review.snapshot?.sha256!==authority.snapshot_sha256||review.reset_review!==authority.reset_review)throw new Error('Review this native target action in the currently selected folder first.');
 targetActionReviews.delete(review);signal?.throwIfAborted();assertCurrent?.();
 const now=await observe(review.root,review.intent.target_namespace,{signal});if(now.sha256!==authority.snapshot_sha256)throw new Error('Native target changed after review. No action completed.');
 let reset_receipt=null;if(review.intent.reset)reset_receipt=await resetNativeStateData(execute,review.reset_review,{signal,assertCurrent});else{await nativeGuard(execute,review.intent,now,review.intent.force);signal?.throwIfAborted();assertCurrent?.();}
 return {schema_version:'bisect-native-state-target-actions-v1',status:'completed',request_sha256:review.intent.request_sha256,target_namespace:review.intent.target_namespace,reset:review.intent.reset,force:review.intent.force,previous_target_sha256:authority.snapshot_sha256,reset_receipt,partition_execution_performed:false};
}
export async function resetNativeStateData(execute,review,{signal,assertCurrent}={}){
 const authority=dataResetReviews.get(review);
 if(!authority||review?.schema_version!=='bisect-native-state-data-reset-review-v1'||review.root!==authority.root||review.force!==authority.force||review.snapshot?.sha256!==authority.snapshot_sha256||await nativeOutputHash(encode(review.output))!==authority.package_sha256)throw new Error('Review this exact native data reset in the currently selected folder first.');
 dataResetReviews.delete(review);signal?.throwIfAborted();assertCurrent?.();
 const before=await observe(review.root,review.output.target_namespace,{signal});
 if(before.sha256!==authority.snapshot_sha256)throw new Error('Native reset target changed after review. No data removed; review it again.');
 await nativeGuard(execute,review.output,before,review.force);signal?.throwIfAborted();assertCurrent?.();
 const target=await find(review.root,review.output.target_namespace.split('/'),{signal,assertCurrent}),present=before.entries.some(e=>e.path==='data');
 const receipt={schema_version:'bisect-browser-native-state-data-reset-v1',target_namespace:review.output.target_namespace,reset_path:review.output.target_namespace+'/data',package_sha256:authority.package_sha256,previous_target_sha256:before.sha256,force:review.force,removed_entries:present?before.entries.filter(e=>e.path==='data'||e.path.startsWith('data/')):[],data_removal_attempted:false,partition_execution_performed:false,scope:'Explicit reviewed native State data reset. Shared native incomplete/labeled-collision guards run first. Only data/ is removed; other target entries are preserved. No Run, installation, completion-marker rewrite or restoration occurs automatically. Filesystem operations do not lock out concurrent programs.'};
 try{
  // Re-observe after asynchronous handle resolution as well. No new directory
  // is created when the target/data does not exist.
  const ready=await observe(review.root,review.output.target_namespace,{signal});if(ready.sha256!==before.sha256)throw new Error('Native reset target changed before removal. No data removed.');signal?.throwIfAborted();assertCurrent?.();
  if(present){receipt.data_removal_attempted=true;await target.removeEntry('data',{recursive:true});}
  signal?.throwIfAborted();assertCurrent?.();const after=await observe(review.root,review.output.target_namespace,{signal});
  const expected=before.entries.filter(e=>e.path!=='data'&&!e.path.startsWith('data/'));
  if(JSON.stringify(after.entries)!==JSON.stringify(expected)||after.exists!==before.exists)throw new Error('Target changed during native data reset; remaining entries could not be verified. Review the folder before another action.');
  return {...receipt,status:'completed',remaining_target_sha256:after.sha256};
 }catch(error){error.native_data_reset={...receipt,status:'interrupted',error:String(error.message??error)};throw error;}
}
async function recoveryGuard(execute,output,snapshot,packageSha256){
 let marker;try{marker=JSON.parse(snapshot.marker_text);}catch{throw new Error('No valid browser interruption marker. This target cannot be resumed.');}
 if(!marker||Array.isArray(marker)||Object.keys(marker).sort().join(',')!=='package_sha256,schema_version,target_namespace'||marker.schema_version!=='bisect-browser-native-install-incomplete-v1'||marker.target_namespace!==output.target_namespace||marker.package_sha256!==packageSha256)throw new Error('Interruption marker does not match this exact output package. Keep the target and select the original package.');
 const files=await nativeStateOutputEntries(output),allowed=new Set([...files.map(([name])=>name),'manifest.tmp','browser/install-receipt.json']);
 for(const entry of snapshot.entries){const owned=entry.kind==='file'?allowed.has(entry.path):[...allowed].some(name=>name.startsWith(entry.path+'/'));if(!owned)throw new Error('Interrupted target contains an unrelated path; recovery will not replace or delete it: '+entry.path);}
 // The actual native guard still checks layout/collisions. Only this validated
 // browser marker is omitted from its virtual observation; disk is untouched.
 const native_guard=await guard(execute,output,{...snapshot,entries:snapshot.entries.filter(e=>e.path!=='manifest.tmp')},true);
 return {native_guard,marker_sha256:snapshot.entries.find(e=>e.path==='manifest.tmp')?.sha256};
}
export async function reviewNativeStateRecovery(execute,root,output,{signal}={}){
 if(root?.kind!=='directory'||!nativeOutputComponent(output?.version))throw new Error('Choose the local outputs directory.');await nativeStateOutputEntries(output);
 const snapshot=await observe(root,output.target_namespace,{signal}),package_sha256=await nativeOutputHash(encode(output)),recovery=await recoveryGuard(execute,output,snapshot,package_sha256);
 signal?.throwIfAborted();const review={schema_version:'bisect-native-state-install-review-v1',root,output:structuredClone(output),force:false,recovery:true,snapshot,native_guard:recovery.native_guard,package_sha256,display:{directory_name:root.name??'selected outputs folder',target_namespace:output.target_namespace,files_to_write:Object.entries(output.file_sha256).map(([path,sha256])=>({path,sha256})),existing_entries:snapshot.entries,force:false,recovery:true}};
 recoveryReviews.set(review,{root,package_sha256,snapshot_sha256:snapshot.sha256});return review;
}
// Classify a batch member from live bytes, never from a saved completion receipt.
export async function reviewNativeStateRecoveryTarget(execute,root,output,{signal}={}){
 if(root?.kind!=='directory')throw new Error('Choose the local outputs directory.');const files=await nativeStateOutputEntries(output),snapshot=await observe(root,output.target_namespace,{signal});
 if(snapshot.entries.some(e=>e.path==='manifest.tmp'))return {action:'resumed',review:await reviewNativeStateRecovery(execute,root,output,{signal})};
 if(!snapshot.entries.length)return {action:'installed',review:await reviewNativeStateInstallation(execute,root,output,{signal})};
 const actual=new Map(snapshot.entries.filter(e=>e.kind==='file').map(e=>[e.path,e]));
 for(const [path,data]of files)if(actual.get(path)?.byte_length!==data.length||actual.get(path)?.sha256!==output.file_sha256[path])throw new Error('Existing target is neither an exact completed package nor a recoverable browser interruption: '+output.target_namespace+'/'+path);
 const native_guard=await guard(execute,output,snapshot,true);signal?.throwIfAborted();
 return {action:'retained',review:{schema_version:'bisect-native-state-retained-target-v1',root,output:structuredClone(output),force:false,snapshot,native_guard,package_sha256:await nativeOutputHash(encode(output)),display:{directory_name:root.name??'selected outputs folder',target_namespace:output.target_namespace,files_to_write:[],existing_entries:snapshot.entries,force:false}}};
}
async function writeVerified(root,path,data,{signal,assertCurrent}){
 signal?.throwIfAborted();assertCurrent?.();const parts=path.split('/'),name=parts.pop(),directory=await find(root,parts,{create:true,signal,assertCurrent});if(!nameOK(name))throw new Error('Unsafe output filename.');const handle=await directory.getFileHandle(name,{create:true});signal?.throwIfAborted();assertCurrent?.();const stream=await handle.createWritable();
 try{signal?.throwIfAborted();assertCurrent?.();await stream.write(data);signal?.throwIfAborted();assertCurrent?.();await stream.close();}catch(error){await stream.abort().catch(()=>{});throw error;}
 const file=await handle.getFile(),actual=new Uint8Array(await file.arrayBuffer()),sha256=await nativeOutputHash(data);if(actual.length!==data.length||await nativeOutputHash(actual)!==sha256)throw new Error('Native output readback failed: '+path);signal?.throwIfAborted();assertCurrent?.();return {path,byte_length:data.length,sha256};
}
export async function installNativeStateOutput(execute,review,{signal,assertCurrent,onProgress=()=>{}}={}){
 if(review?.schema_version!=='bisect-native-state-install-review-v1'||await nativeOutputHash(encode(review.output))!==review.package_sha256)throw new Error('Output package changed after target review. Review it again.');
 const recovery=review.recovery===true,authority=recoveryReviews.get(review);if(recovery&&(!authority||review.force!==false||authority.root!==review.root||authority.package_sha256!==review.package_sha256||authority.snapshot_sha256!==review.snapshot?.sha256))throw new Error('Review this interrupted installation in the currently selected folder first.');if(recovery)recoveryReviews.delete(review);
 const output=review.output,files=await nativeStateOutputEntries(output),snapshot=await observe(review.root,output.target_namespace,{signal});if(snapshot.sha256!==review.snapshot.sha256)throw new Error('Local target changed after review. No files written; review it again.');
 if(recovery)await recoveryGuard(execute,output,snapshot,review.package_sha256);else await guard(execute,output,snapshot,review.force);signal?.throwIfAborted();assertCurrent?.();const target=await find(review.root,output.target_namespace.split('/'),{create:true,signal,assertCurrent}),created=await observe(review.root,output.target_namespace,{signal});
 if(snapshot.exists?created.sha256!==snapshot.sha256:created.entries.length!==0)throw new Error('Local target changed during creation. No files written.');
 const options={signal,assertCurrent},marker={schema_version:'bisect-browser-native-install-incomplete-v1',target_namespace:output.target_namespace,package_sha256:review.package_sha256};
 await writeVerified(target,'manifest.tmp',encode(marker),options);
 if(review.force&&!recovery){for(const name of new Set(snapshot.entries.map(e=>e.path.split('/')[0]))){if(name==='manifest.tmp')continue;signal?.throwIfAborted();assertCurrent?.();if(!nameOK(name))throw new Error('Unexpected overwrite path.');await target.removeEntry(name,{recursive:true});}}
 const written=[];for(const [path,data]of files){if(path==='manifest.json')continue;onProgress({stage:'writing',path});written.push(await writeVerified(target,path,data,options));}
 const manifest=files.find(([path])=>path==='manifest.json')[1],receipt={schema_version:'bisect-browser-native-install-v1',target_namespace:output.target_namespace,package_sha256:review.package_sha256,previous_target_sha256:snapshot.sha256,force:review.force,files:[...written,{path:'manifest.json',byte_length:manifest.length,sha256:await nativeOutputHash(manifest)}],scope:'Explicit native-layout installation. Every generated file is closed and read back before the final manifest; manifest.tmp remains on failure and is removed last. Sequential browser file operations are not an atomic directory promotion or a lock against other programs. WASM/source records do not establish native executable provenance or legal certification.'};
 written.push(await writeVerified(target,'browser/install-receipt.json',encode(recovery?Object.assign(receipt,{recovery:{mode:'explicit-reviewed-same-package',interrupted_target_sha256:snapshot.sha256,interruption_marker_sha256:snapshot.entries.find(e=>e.path==='manifest.tmp').sha256}}):receipt),options));
 const finalSnapshot=await observe(review.root,output.target_namespace,{signal}),expected=new Map(written.map(e=>[e.path,e.sha256]));expected.set('manifest.tmp',await nativeOutputHash(encode(marker)));if(recovery){const previousManifest=snapshot.entries.find(e=>e.path==='manifest.json');if(previousManifest)expected.set('manifest.json',previousManifest.sha256);}const observedFiles=finalSnapshot.entries.filter(e=>e.kind==='file');if(observedFiles.length!==expected.size||observedFiles.some(e=>expected.get(e.path)!==e.sha256))throw new Error('Target files changed before final manifest publication. Incomplete marker retained.');
 onProgress({stage:'manifest',path:'manifest.json'});written.push(await writeVerified(target,'manifest.json',manifest,options));signal?.throwIfAborted();assertCurrent?.();await target.removeEntry('manifest.tmp');
 return {...receipt,status:'completed',install_receipt_sha256:written.find(e=>e.path==='browser/install-receipt.json').sha256};
}

// Batch checkpoints use the same bounded, verified writer after explicit review.
export async function writeNativeOutputCheckpoint(root,path,data,options={}){
 if(typeof path!=='string'||!/^[-A-Za-z0-9_]+\/browser\/output-install-[a-f0-9-]+\.json$/.test(path))throw new Error('Unsafe batch output checkpoint path.');return writeVerified(root,path,data,options);
}
export async function nativeOutputCheckpointExists(root,path){const parts=path.split('/'),name=parts.pop(),directory=await find(root,parts);if(!directory)return false;try{await directory.getFileHandle(name);return true;}catch(e){if(e.name==='NotFoundError')return false;throw e;}}
