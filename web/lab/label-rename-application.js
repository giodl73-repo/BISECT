import {labelRenameRequest} from './label-rename-files.js';
import {encodeBytes} from './package-files.js';
const grants=new WeakMap(),STAGES=['runs','analysis','reports'],MAX=16*1024*1024;
const bytes=s=>new TextEncoder().encode(s);
const digest=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
const id=async v=>digest(bytes(JSON.stringify(v)));
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
function parts(path){
 if(typeof path!=='string'||path.length>240)throw Error('Unsafe rename path.');const a=path.split('/');
 if(a.some(p=>!p||p.length>200||!/^[-A-Za-z0-9_. ]+$/.test(p)||['.','..','__proto__','constructor','prototype'].includes(p)||/[. ]$/.test(p)||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)))throw Error('Unsafe rename path.');return a;
}
async function permission(root,o,write=false){check(o);if(root?.kind!=='directory')throw Error('Select a live repository directory.');if(root.queryPermission&&await root.queryPermission({mode:write?'readwrite':'read'})!=='granted')throw Error('Repository permission is no longer granted. Select it again.');check(o);}
async function find(parent,name,kind,o){check(o);let count=0,found=null;for await(const [actual,h] of parent.entries()){check(o);if(++count>20000)throw Error('Directory exceeds 20,000 entries.');if(actual.toLowerCase()===name.toLowerCase()){if(actual!==name)throw Error('Case-colliding rename path.');if(h.kind!==kind)throw Error('Rename path has the wrong file/directory kind.');found=h;}}return found;}
async function resolve(root,path,kind,o,create=false){let h=root;const a=parts(path);for(let i=0;i<a.length;i++){const k=i===a.length-1?kind:'directory';let next=await find(h,a[i],k,o);if(!next&&create){check(o);next=k==='directory'?await h.getDirectoryHandle(a[i],{create:true}):await h.getFileHandle(a[i],{create:true});}if(!next)return null;h=next;}return h;}
async function read(handle,o,limit=MAX){check(o);const file=await handle.getFile();if(!Number.isSafeInteger(file.size)||file.size<0||file.size>limit)throw Error('Rename source exceeds byte limit.');const b=new Uint8Array(await file.arrayBuffer());check(o);if(b.length!==file.size)throw Error('Rename source changed while reading.');return b;}
const roots=settings=>STAGES.flatMap(stage=>[stage+'/'+settings.source_label,stage+'/'+settings.destination_label]);
async function snapshot(root,settings,o,retain=false){
 await permission(root,o);const files={},directories=[],raw={};let total=0;
 async function visit(h,path){directories.push(path+'/');if(directories.length>(retain?512:1024))throw Error('Rename supports 512 directories.');for await(const [name,child]of h.entries()){check(o);const p=path+'/'+name;parts(p);if(child.kind==='directory')await visit(child,p);else if(child.kind==='file'){if(Object.keys(files).length>=(retain?512:1024))throw Error('Rename supports 512 files.');const b=await read(child,o);total+=b.length;if(total>(retain?MAX:2*MAX))throw Error('Rename supports 16 MiB file bytes.');files[p]=await digest(b);if(retain)raw[p]=encodeBytes(b);}else throw Error('Unsupported rename entry kind.');}}
 const registry=await resolve(root,'.bisect','file',o);if(!registry)throw Error('Select a repository with its original .bisect registry.');const rb=await read(registry,o,1024*1024);
 for(const path of roots(settings)){const h=await resolve(root,path,'directory',o);if(h)await visit(h,path);}
 const folded=new Set();for(const path of [...Object.keys(files),...directories.map(p=>p.slice(0,-1))]){const k=path.toLowerCase();if(folded.has(k))throw Error('Case-colliding rename tree.');folded.add(k);}
 return {files,directories:directories.sort(),registry:await digest(rb),...(retain?{raw,registry_bytes:rb}:{})};
}
function footprint(fileHashes,dirs,settings,registry){const files={},directories=new Set();const selected=roots(settings);for(const path of [...Object.keys(fileHashes),...dirs.map(p=>p.replace(/\/$/,''))]){parts(path);if(!selected.some(r=>path===r||path.startsWith(r+'/')))continue;const a=path.split('/');for(let i=2;i<a.length;i++)directories.add(a.slice(0,i).join('/')+'/');if(Object.hasOwn(fileHashes,path))files[path]=fileHashes[path];else directories.add(path+'/');}return {files,directories:[...directories].sort(),registry};}
function subtree(s,path){return {files:Object.fromEntries(Object.entries(s.files).filter(([p])=>p.startsWith(path+'/'))),directories:s.directories.filter(p=>p.startsWith(path+'/'))};}
const equal=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
function treeEqual(a,b){return equal(Object.entries(a.files).sort(),Object.entries(b.files).sort())&&equal(a.directories,b.directories);}
function replaceTree(state,path,tree){for(const p of Object.keys(state.files))if(p.startsWith(path+'/'))delete state.files[p];Object.assign(state.files,tree.files);state.directories=state.directories.filter(p=>!p.startsWith(path+'/')).concat(tree.directories).sort();}
const exists=t=>t.directories.length>0||Object.keys(t.files).length>0;
function moved(tree,from,to){return {files:Object.fromEntries(Object.entries(tree.files).map(([p,h])=>[to+p.slice(from.length),h])),directories:tree.directories.map(p=>to+p.slice(from.length)).sort()};}
function prefix(actual,raw,final){return actual.directories.every(p=>raw.directories.includes(p))&&Object.entries(actual.files).every(([p,h])=>h===raw.files[p]||h===final.files[p]);}
export async function readLabelRenameRoot(root,settings,options={}){
 // Reuse portable request validation before following any user-supplied label.
 labelRenameRequest({registry:{name:'.bisect',base64:encodeBytes(bytes('{}'))},files:{},directories:[],settings});
 const observed=await snapshot(root,settings,options,true);return {registry:{name:'.bisect',base64:encodeBytes(observed.registry_bytes)},files:observed.raw,directories:observed.directories,settings:{...settings}};
}
export async function reviewLabelRenameApplication(engine,result,inputs,root,{continuation=false,...o}={}){
 if(typeof continuation!=='boolean')throw Error('Choose fresh rename or explicit continuation.');const request=labelRenameRequest(inputs);await permission(root,o,true);
 const fresh=await engine.execute(request);check(o);if(await id(fresh)!==await id(result))throw Error('Rename differs from fresh native recomputation.');
 if(fresh?.schema_version!=='bisect-label-rename-v1'||fresh.status!=='packaged'||!Array.isArray(fresh.files)||!Array.isArray(fresh.directories))throw Error('Prepare a fresh native rename first.');
 const hashes={};for(const [p,b]of Object.entries(inputs.files))hashes[p]=await digest(Uint8Array.from(atob(b),c=>c.charCodeAt(0)));
 const original=footprint(hashes,inputs.directories,inputs.settings,await digest(Uint8Array.from(atob(inputs.registry.base64),c=>c.charCodeAt(0))));
 const output={},outputHashes={};for(const row of fresh.files){parts(row.path);const text=row.base64??inputs.files[row.source_path];if(typeof text!=='string'||Object.hasOwn(output,row.path))throw Error('Invalid rename file roster.');const b=Uint8Array.from(atob(text),c=>c.charCodeAt(0));output[row.path]=b;outputHashes[row.path]=await digest(b);const declared=fresh.output_hashes.find(f=>f.path===row.path);if(!declared||declared.sha256!==outputHashes[row.path]||declared.bytes!==b.length)throw Error('Rename output hash mismatch.');}
 if(!output['.bisect']||new TextDecoder().decode(output['.bisect'])!==fresh.registry_json)throw Error('Missing renamed registry.');
 const final=footprint(outputHashes,fresh.directories,inputs.settings,outputHashes['.bisect']);
 const observed=await snapshot(root,inputs.settings,o);check(o);
 if(!continuation&&(!treeEqual(observed,original)||observed.registry!==original.registry))throw Error('Repository differs from original rename inputs. Read and prepare it again.');
 const actions=[];for(const stage of STAGES){const from=stage+'/'+inputs.settings.source_label,to=stage+'/'+inputs.settings.destination_label,source=subtree(original,from),before=subtree(original,to),liveSource=subtree(observed,from),liveTarget=subtree(observed,to),target=subtree(final,to),raw=moved(source,from,to);if(!exists(source)){if(!treeEqual(liveSource,source)||!treeEqual(liveTarget,before))throw Error('Untouched rename stage changed.');continue;}
  if(!treeEqual(liveSource,source)&&exists(liveSource))throw Error('Rename source changed or has unknown partial bytes.');
  const removeTarget=exists(liveSource)&&exists(before)&&treeEqual(liveTarget,before);
  if(removeTarget&&!inputs.settings.force)throw Error('Destination replacement requires force.');
  if(!removeTarget&&!prefix(liveTarget,raw,target))throw Error('Rename destination has unknown partial bytes.');
  if(!exists(liveSource)&&!treeEqual(liveTarget,raw)&&!treeEqual(liveTarget,target))throw Error('Missing source has no complete intended destination.');
  actions.push({stage,from,to,source,before,raw,target,removeTarget,sourcePresent:exists(liveSource)});
 }
 if(observed.registry!==original.registry&&(!continuation||observed.registry!==final.registry||!treeEqual(observed,final)))throw Error('Rename registry changed or records incomplete stages.');
 const review=Object.freeze({schema_version:'bisect-label-rename-review-v1',root_name:root.name,source_label:inputs.settings.source_label,destination_label:inputs.settings.destination_label,force:inputs.settings.force,mode:continuation?'continuation':'fresh',stages:Object.freeze(actions.map(a=>Object.freeze({stage:a.stage,source:a.from,destination:a.to,remove_destination:a.removeTarget,source_present:a.sourcePresent,files:Object.freeze(Object.keys(a.target.files).sort())}))),registry_last:true,request_sha256:await id(request),prepared_sha256:await id(fresh),authority:'Private current session only; serialized copies grant no authority.'});
 grants.set(review,{root,settings:{...inputs.settings},original,expected:observed,final,output,actions,used:false});return review;
}
export function retireLabelRenameReview(review){if(review)grants.delete(review);}
export async function applyLabelRenameApplication(review,{confirmed=false,...o}={}){
 const g=grants.get(review);if(!g||g.used)throw Error('Review rename again; imported or consumed reviews grant no authority.');if(confirmed!==true)throw Error('Confirm the concrete reviewed rename effects.');g.used=true;
 const receipt={schema_version:'bisect-label-rename-application-v1',status:'incomplete',root_name:review.root_name,source_label:review.source_label,destination_label:review.destination_label,mode:review.mode,request_sha256:review.request_sha256,prepared_sha256:review.prepared_sha256,completed_events:[],attempted_event:null,effects_may_have_started:false,registry_write_started:false,registry_readback_verified:false,authority:'Historical record only; Open grants no root, execution or write authority.'};
 const emit=()=>{try{o.onProgress?.(structuredClone(receipt));}catch{}};
 async function recheck(){await permission(g.root,o,true);const now=await snapshot(g.root,g.settings,o);if(now.registry!==g.expected.registry||!treeEqual(now,g.expected))throw Error('Repository changed after rename review.');}
 function begin(kind,path){check(o);receipt.attempted_event={kind,path};emit();check(o);receipt.effects_may_have_started=true;}
 function done(kind,path){receipt.completed_events.push({kind,path});receipt.attempted_event=null;emit();}
 async function write(path,data){await recheck();begin('write',path);const h=await resolve(g.root,path,'file',o,true);check(o);const stream=await h.createWritable();try{check(o);await stream.write(data);check(o);await stream.close();}catch(e){await stream.abort().catch(()=>{});throw e;}const actual=await read(h,o,path==='.bisect'?1024*1024:MAX);if(await digest(actual)!==await digest(data))throw Error('Rename write read-back mismatch: '+path);if(path==='.bisect')g.expected.registry=await digest(data);else g.expected.files[path]=await digest(data);done('write',path);}
 async function remove(path){await recheck();begin('remove',path);const a=parts(path),parent=await resolve(g.root,a.slice(0,-1).join('/'),'directory',o);check(o);await parent.removeEntry(a.at(-1),{recursive:true});replaceTree(g.expected,path,{files:{},directories:[]});await recheck();done('remove',path);}
 try{
  await recheck();for(const a of g.actions){check(o);if(a.removeTarget)await remove(a.to);
   if(a.sourcePresent){
    for(const dir of a.raw.directories.slice().sort((x,y)=>x.split('/').length-y.split('/').length||x.localeCompare(y))){if(g.expected.directories.includes(dir))continue;await recheck();const path=dir.slice(0,-1);begin('mkdir',path);await resolve(g.root,path,'directory',o,true);g.expected.directories.push(dir);g.expected.directories.sort();await recheck();done('mkdir',path);}
    for(const [path,sha]of Object.entries(a.raw.files).sort()){if(g.expected.files[path]===sha||g.expected.files[path]===a.target.files[path])continue;const sourcePath=a.from+path.slice(a.to.length),h=await resolve(g.root,sourcePath,'file',o),b=await read(h,o);if(await digest(b)!==a.source.files[sourcePath])throw Error('Rename source changed before copying.');await write(path,b);}
    await recheck();if(!prefix(subtree(g.expected,a.to),a.raw,a.target)||Object.keys(subtree(g.expected,a.to).files).length!==Object.keys(a.raw.files).length||!equal(subtree(g.expected,a.to).directories,a.raw.directories))throw Error('Rename destination copy is incomplete.');
    await remove(a.from);
   }
   const index=a.to+'/index.json';if(Object.hasOwn(a.target.files,index)&&g.expected.files[index]!==a.target.files[index])await write(index,g.output[index]);
   await recheck();if(!treeEqual(subtree(g.expected,a.to),a.target))throw Error('Renamed stage differs from intended bytes.');done('stage',a.stage);
  }
  await recheck();if(!treeEqual(g.expected,g.final))throw Error('Rename stages are not complete.');
  if(g.expected.registry!==g.final.registry){receipt.registry_write_started=true;await write('.bisect',g.output['.bisect']);}
  await recheck();receipt.registry_readback_verified=true;receipt.status='completed';emit();return receipt;
 }catch(error){receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=error.message;emit();error.renameReceipt=structuredClone(receipt);throw error;}finally{grants.delete(review);}
}
