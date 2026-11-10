import {validateAdjacencyReleaseScope} from './adjacency-release.js';
import {prepareNativeFetchReleaseFiles} from './native-fetch-release-package.js';
const grants=new WeakMap(),MAX=128*1024*1024;
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),x=>x.toString(16).padStart(2,'0')).join('');
const encode=x=>new TextEncoder().encode(JSON.stringify(x));
function layout(scope){const s=validateAdjacencyReleaseScope(scope),stem=s.state.toLowerCase()+'_adjacency_'+s.year;return {scope:s,parts:['V3','data',s.year,'adjacency'],primary:stem+'.pkl',marker:stem+'.done'};}
async function permission(root,o){check(o);if(root?.kind!=='directory')throw new Error('Select the native outputs folder.');if(root.queryPermission&&await root.queryPermission({mode:'readwrite'})!=='granted')throw new Error('Folder write permission is no longer granted. Select it again.');check(o);}
async function child(root,name,create,o){check(o);let count=0;for await(const [actual]of root.entries()){check(o);if(++count>10000)throw new Error('Folder exceeds 10,000 entries.');if(actual.toLowerCase()===name.toLowerCase()&&actual!==name)throw new Error('Case-colliding target path: '+name);}try{const result=await root.getDirectoryHandle(name,{create});check(o);return result;}catch(e){if(!create&&e.name==='NotFoundError')return null;throw e;}}
async function directory(root,parts,create,o){let d=root;for(const name of parts){d=await child(d,name,create,o);if(!d)return null;}return d;}
async function selectedEntries(root,l,names,o,{read=true}={}){
 await permission(root,o);const d=await directory(root,l.parts,false,o),result={};for(const n of names)result[n]=null;
 if(!d)return result;const wanted=new Set(names.map(n=>n.toLowerCase()));let count=0,total=0;
 for await(const [name,handle]of d.entries()){check(o);if(++count>10000)throw new Error('Adjacency folder exceeds 10,000 entries.');if(!wanted.has(name.toLowerCase()))continue;if(!names.includes(name))throw new Error('Case-colliding target filename: '+name);if(result[name])throw new Error('Duplicate target filename.');
  if(!read||handle.kind==='directory'){result[name]={kind:handle.kind};continue;}if(handle.kind!=='file')throw new Error('Unsupported target entry.');const file=await handle.getFile();check(o);if(!Number.isSafeInteger(file.size)||file.size<0||(total+=file.size)>MAX)throw new Error('Target review exceeds 128 MiB.');const bytes=new Uint8Array(await file.arrayBuffer());check(o);if(bytes.length!==file.size)throw new Error('Target changed while reading.');result[name]={kind:'file',byte_length:bytes.length,sha256:await hash(bytes)};
 }check(o);return result;
}
export async function inspectNativeFetchCompletion(root,scope,options={}){const l=layout(scope),seen=await selectedEntries(root,l,[l.primary,l.marker],options,{read:false});return {schema_version:'bisect-native-fetch-presence-v1',...l.scope,primary_exists:!!seen[l.primary],marker_exists:!!seen[l.marker],native_is_done:!!seen[l.primary]&&!!seen[l.marker],validation_performed:false};}
export function retireNativeFetchReview(review){grants.delete(review);}
export async function reviewNativeFetchInstallation(root,acquired,{force=false,...options}={}){
 if(typeof force!=='boolean')throw new Error('Choose force explicitly.');const prepared=await prepareNativeFetchReleaseFiles(acquired,options),l=layout(Object.fromEntries(['repository','tag','state','year'].map(k=>[k,prepared.receipt[k]]))),presence=await inspectNativeFetchCompletion(root,l.scope,options);
 const entries=prepared.entries.filter(([p])=>p.startsWith(l.parts.join('/')+'/')).map(([p,b])=>[p.split('/').at(-1),b.slice()]),names=[...new Set([...entries.map(([n])=>n),l.primary,l.marker])],skip=presence.native_is_done&&!force;
 const observed=await selectedEntries(root,l,names,options,{read:!skip});if(!skip&&Object.values(observed).some(v=>v&&v.kind!=='file'))throw new Error('A planned file or marker is a directory; no changes made.');
 const rows=[];for(const [name,bytes]of entries){check(options);rows.push(Object.freeze({name,byte_length:bytes.length,sha256:await hash(bytes),existing:observed[name]?Object.freeze({...observed[name]}):null}));}const inventory=await hash(encode(rows.map(({name,byte_length,sha256})=>({name,byte_length,sha256}))));check(options);
 const review=Object.freeze({schema_version:'bisect-native-fetch-install-review-v1',root_name:root.name||'selected outputs',scope:Object.freeze({...l.scope}),directory:l.parts.join('/'),force,action:skip?'skip':'write',native_presence:Object.freeze(presence),invalidate_existing_marker:!skip&&!!observed[l.marker],source_inventory_sha256:inventory,files_to_write:Object.freeze(skip?[]:rows),authority:'Live private session only; serialized reviews grant no write authority.'});
 grants.set(review,{root,l,entries,names,observed,skip,force});return review;
}
export async function applyNativeFetchInstallation(review,{confirmed=false,...options}={}){
 const g=grants.get(review);if(!g)throw new Error('Review the folder again; imported or consumed reviews grant no authority.');if(confirmed!==true)throw new Error('Confirm the concrete reviewed folder changes.');grants.delete(review);
 const expected=structuredClone(g.observed),receipt={schema_version:'bisect-native-fetch-install-outcome-v1',scope:{...g.l.scope},root_name:review.root_name,source_inventory_sha256:review.source_inventory_sha256,directory:review.directory,force:g.force,status:'incomplete',attempted_path:null,completed_paths:[],effects_may_have_started:false,marker_invalidated:false,marker_committed:false,installation_performed:false,source_authenticity_verified:false,authority:'Historical outcome only; reselect and review to obtain new authority.'};
 const emit=()=>{try{options.onProgress?.(structuredClone(receipt));}catch{}};
 async function recheck(){const actual=await selectedEntries(g.root,g.l,g.names,options,{read:!g.skip});if(JSON.stringify(actual)!==JSON.stringify(expected))throw new Error('Target changed after review. Review it again.');}
 try{
  await recheck();if(g.skip){receipt.status='skipped-native-presence';emit();return receipt;}
  if(expected[g.l.marker]){check(options);receipt.attempted_path=review.directory+'/'+g.l.marker;receipt.effects_may_have_started=true;emit();const d=await directory(g.root,g.l.parts,false,options);await d.removeEntry(g.l.marker);expected[g.l.marker]=null;receipt.marker_invalidated=true;receipt.attempted_path=null;await recheck();}
  for(const [name,bytes]of g.entries){
   await recheck();check(options);receipt.attempted_path=review.directory+'/'+name;receipt.effects_may_have_started=true;emit();const d=await directory(g.root,g.l.parts,true,options);const handle=await d.getFileHandle(name,{create:true});check(options);const stream=await handle.createWritable();try{check(options);await stream.write(bytes);check(options);await stream.close();}catch(error){await stream.abort().catch(()=>{});throw error;}
   const after=await selectedEntries(g.root,g.l,[name],options),wanted={kind:'file',byte_length:bytes.length,sha256:await hash(bytes)};if(JSON.stringify(after[name])!==JSON.stringify(wanted))throw new Error('Written file failed read-back: '+name);expected[name]=wanted;receipt.completed_paths.push(review.directory+'/'+name);receipt.installation_performed=true;if(name===g.l.marker)receipt.marker_committed=true;receipt.attempted_path=null;emit();
  }
  await recheck();receipt.status='completed';emit();return receipt;
 }catch(error){receipt.status=options.signal?.aborted?'cancelled':'interrupted';receipt.error=String(error.message).slice(0,1000);emit();error.installationReceipt=structuredClone(receipt);throw error;}
}
