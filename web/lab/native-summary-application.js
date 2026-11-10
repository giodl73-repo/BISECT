import {executeNativeSummaryBatch,nativeSummaryOutputFiles} from './native-label-summary.js';
import {validatePackagePath} from './package-files.js';
const grants=new WeakMap(),MAX=25*1024*1024,MIXED=50*1024*1024;
const bytes=text=>new TextEncoder().encode(text);
const sha=async value=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',value)),x=>x.toString(16).padStart(2,'0')).join('');
const identity=async value=>sha(bytes(JSON.stringify(value)));
const same=(a,b)=>a===null?b===null:b!==null&&a.sha256===b.sha256&&a.byte_length===b.byte_length;
const check=o=>{o.signal?.throwIfAborted();o.assertCurrent?.();};
function parts(path){validatePackagePath(path);const result=path.split('/');if(result.some(p=>p.endsWith('.')||p.endsWith(' ')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)))throw Error('Unsafe summary application path.');return result;}
async function permission(root,o){check(o);if(root?.kind!=='directory')throw Error('Select a live summary repository.');if(root.queryPermission&&await root.queryPermission({mode:'readwrite'})!=='granted')throw Error('Summary repository permission is unavailable.');check(o);}
async function child(dir,name,kind,create,o){check(o);let n=0;for await(const [actual,h]of dir.entries()){check(o);if(++n>20000)throw Error('Summary directory exceeds 20,000 entries.');if(actual.toLowerCase()===name.toLowerCase()){if(actual!==name)throw Error('Case-colliding summary path.');if(h.kind!==kind)throw Error('Summary path has the wrong file/directory kind.');}}
 try{return kind==='directory'?await dir.getDirectoryHandle(name,{create}):await dir.getFileHandle(name,{create});}catch(e){if(!create&&e.name==='NotFoundError')return null;throw e;}}
async function resolve(root,path,kind,create,o){const names=parts(path);let dir=root;for(const name of names.slice(0,-1)){dir=await child(dir,name,'directory',create,o);if(!dir)return null;}return child(dir,names.at(-1),kind,create,o);}
async function observation(root,path,o){const handle=await resolve(root,path,'file',false,o);if(!handle)return null;const file=await handle.getFile();check(o);if(!Number.isSafeInteger(file.size)||file.size<0||file.size>MAX)throw Error('Summary file exceeds 25 MiB.');const data=new Uint8Array(await file.arrayBuffer());check(o);if(data.length!==file.size)throw Error('Summary file changed while reading.');return {sha256:await sha(data),byte_length:data.length};}
async function observe(root,paths,dirs,o,limit=MIXED){await permission(root,o);const files={},directories={};let total=0;for(const path of paths){const value=await observation(root,path,o);if((total+=value?.byte_length||0)>limit)throw Error('Summary snapshot exceeds byte limits.');files[path]=value;}for(const path of dirs)directories[path]=!!await resolve(root,path,'directory',false,o);check(o);return {files,directories};}
function equalState(a,b){return Object.keys(a.files).every(p=>same(a.files[p],b.files[p]))&&Object.keys(a.directories).every(p=>a.directories[p]===b.directories[p]);}
export function validateNativeSummaryBaseline(value){
 if(!value||Object.keys(value).sort().join()!=='before,label,prepared_sha256,request_sha256,schema_version'||value.schema_version!=='bisect-native-summary-baseline-v1'||typeof value.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(value.label)||!['request_sha256','prepared_sha256'].every(k=>typeof value[k]==='string'&&/^[a-f0-9]{64}$/.test(value[k]))||!value.before||Object.keys(value.before).sort().join()!=='directories,files')throw Error('Invalid summary continuation baseline.');
 const {files,directories}=value.before;if(!files||Array.isArray(files)||!directories||Array.isArray(directories)||Object.keys(files).length>301||Object.keys(directories).length>100||!Object.hasOwn(files,'.bisect'))throw Error('Invalid summary baseline roster.');let total=0;
 for(const [path,v]of Object.entries(files)){parts(path);if(v!==null&&(!v||Object.keys(v).sort().join()!=='byte_length,sha256'||typeof v.sha256!=='string'||!/^[a-f0-9]{64}$/.test(v.sha256)||!Number.isSafeInteger(v.byte_length)||v.byte_length<0||(total+=v.byte_length)>MAX))throw Error('Invalid summary baseline file identity.');}
 for(const [path,present]of Object.entries(directories)){parts(path);if(typeof present!=='boolean'||!path.startsWith('analysis/'+value.label+'/')||!/^\d{4}$/.test(path.split('/').at(-1))||path.split('/').length!==3)throw Error('Invalid summary baseline source directory.');}return value;
}
async function prepare(engine,input,result,o){
 const request=structuredClone(input),writes=[],blobs=new Map();let traceBytes=0;
 const fresh=await executeNativeSummaryBatch(engine,request,{signal:o.signal,onProgress:async snapshot=>{
  check(o);const row=snapshot.years.at(-1);if(row.status!=='completed')return;
  const paths=[...row.files.filter(p=>!p.endsWith('/index.json')&&p!=='index.json'),...row.files.filter(p=>p.endsWith('/index.json')||p==='index.json'),'.bisect'];
  for(const path of paths){parts(path);const after=await fileIdentity(snapshot.files[path]);let data=blobs.get(after.sha256);if(!data){data=bytes(snapshot.files[path]);if((traceBytes+=data.length)>MIXED)throw Error('Summary transition bytes exceed 50 MiB.');blobs.set(after.sha256,data);}writes.push({kind:'file',path,year:row.year,ordinal:row.ordinal,data,after});}
 }});check(o);if(await identity(fresh)!==await identity(result))throw Error('Summary differs from fresh native recomputation.');await nativeSummaryOutputFiles(fresh);check(o);
 const source=request.source,sourceFiles={'.bisect':await fileIdentity(source.registry_json)},sourceDirs={};
 for(const year of new Set(fresh.years.map(row=>row.year))){const path='analysis/'+source.label+'/'+year,s=source.sources[year]||{analysis_directory_present:false,analysis_index_json:null};sourceDirs[path]=s.analysis_directory_present;sourceFiles[path+'/index.json']=s.analysis_index_json===null?null:await fileIdentity(s.analysis_index_json);}
 if(!writes.length)throw Error('No completed native summary prefix is available to apply.');
 return {fresh,request,sourceFiles,sourceDirs,writes,paths:[...new Set([...Object.keys(sourceFiles),...writes.map(w=>w.path)])].sort(),dirs:Object.keys(sourceDirs).sort(),requestSha:await identity(request),preparedSha:await identity(fresh)};
}
async function fileIdentity(text){const data=bytes(text);return {sha256:await sha(data),byte_length:data.length};}
function transitions(prepared,before){const dirs={...before.directories},steps=[];for(const write of prepared.writes){for(const path of prepared.dirs)if(!dirs[path]&&write.path.startsWith(path+'/')){steps.push({kind:'directory',path,year:write.year,ordinal:write.ordinal});dirs[path]=true;}steps.push(write);}return steps;}
function advance(state,step){if(step.kind==='directory')state.directories[step.path]=true;else state.files[step.path]=step.after;}
export async function reviewNativeSummaryApplication(engine,input,result,root,{continuation=false,baseline=null,...o}={}){
 if(typeof continuation!=='boolean')throw Error('Choose fresh summary review or continuation.');const p=await prepare(engine,input,result,o),current=await observe(root,p.paths,p.dirs,o,continuation?MIXED:MAX);let before;
 if(continuation){validateNativeSummaryBaseline(baseline);if(baseline.label!==p.request.source.label||baseline.request_sha256!==p.requestSha||baseline.prepared_sha256!==p.preparedSha||JSON.stringify(Object.keys(baseline.before.files).sort())!==JSON.stringify(p.paths)||JSON.stringify(Object.keys(baseline.before.directories).sort())!==JSON.stringify(p.dirs))throw Error('Summary baseline does not match original request/output.');before=structuredClone(baseline.before);}
 else before=structuredClone(current);
 for(const [path,original]of Object.entries(p.sourceFiles))if(!same(before.files[path],original))throw Error('Live original summary source differs: '+path);for(const [path,present]of Object.entries(p.sourceDirs))if(before.directories[path]!==present)throw Error('Live original summary source directory differs: '+path);
 const steps=transitions(p,before);let cursor=0,state=structuredClone(before),matched=equalState(state,current)?0:-1;
 if(continuation){for(const step of steps){advance(state,step);cursor++;if(equalState(state,current))matched=cursor;}if(matched<0)throw Error('Summary targets have unknown partial or concurrent bytes; select a fresh conflict review.');}else matched=0;
 const changes=steps.map((step,index)=>Object.freeze({index,kind:step.kind,path:step.path,year:step.year,ordinal:step.ordinal,after_sha256:step.after?.sha256??null,byte_length:step.after?.byte_length??0,retained:index<matched}));
 const baselineData={schema_version:'bisect-native-summary-baseline-v1',label:p.request.source.label,request_sha256:p.requestSha,prepared_sha256:p.preparedSha,before};validateNativeSummaryBaseline(baselineData);
 const review=Object.freeze({schema_version:'bisect-native-summary-review-v1',root_name:root.name,label:p.request.source.label,mode:continuation?'continuation':'fresh',command_status:p.fresh.status,command_error:p.fresh.error,cursor:matched,total_steps:steps.length,changes:Object.freeze(changes),request_sha256:p.requestSha,prepared_sha256:p.preparedSha,authority:'Private single-use current root review. Baselines are unauthenticated history, never permission.'});
 grants.set(review,{root,p,steps,current,baseline:baselineData,used:false});return {review,baseline:structuredClone(baselineData)};
}
export function retireNativeSummaryReview(review){if(review)grants.delete(review);}
export async function applyNativeSummaryApplication(review,{confirmed=false,...o}={}){
 const g=grants.get(review);if(!g||g.used)throw Error('Summary review has no current authority.');if(confirmed!==true)throw Error('Confirm the listed summary transitions.');g.used=true;
 const receipt={schema_version:'bisect-native-summary-application-v1',status:'incomplete',label:review.label,mode:review.mode,request_sha256:review.request_sha256,prepared_sha256:review.prepared_sha256,baseline:structuredClone(g.baseline),cursor:review.cursor,total_steps:g.steps.length,completed_steps:[],attempted_step:null,effects_may_have_started:false,command_status:review.command_status,command_error:review.command_error,authority:'Historical data only. Open never restores a root or write permission.'},expected=structuredClone(g.current);
 const emit=()=>{try{o.onProgress?.(structuredClone(receipt));}catch{}};
 async function recheck(){const actual=await observe(g.root,g.p.paths,g.p.dirs,o);if(!equalState(actual,expected))throw Error('Summary repository changed after review.');}
 try{await recheck();for(let i=review.cursor;i<g.steps.length;i++){const step=g.steps[i];await recheck();check(o);receipt.attempted_step=i;emit();check(o);receipt.effects_may_have_started=true;
  if(step.kind==='directory'){await resolve(g.root,step.path,'directory',true,o);check(o);}else{const handle=await resolve(g.root,step.path,'file',true,o);check(o);const stream=await handle.createWritable();try{check(o);await stream.write(step.data);check(o);await stream.close();}catch(e){await stream.abort().catch(()=>{});throw e;}if(!same(await observation(g.root,step.path,o),step.after))throw Error('Summary write read-back mismatch: '+step.path);}
  advance(expected,step);await recheck();receipt.completed_steps.push({index:i,path:step.path,kind:step.kind,year:step.year,ordinal:step.ordinal});receipt.cursor=i+1;receipt.attempted_step=null;emit();
 }await recheck();receipt.status=review.command_status==='generated'?'completed':'completed-prefix';emit();return receipt;
 }catch(e){receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=e.message;emit();e.summaryReceipt=structuredClone(receipt);throw e;}finally{grants.delete(review);}
}
