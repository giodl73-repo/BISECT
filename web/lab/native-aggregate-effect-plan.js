import {executeNativeAggregateEffects} from './native-aggregate-effects.js';

import {aggregateWorkspaceVersion} from './native-aggregate-workspace.js';

import {validateNativeAggregateInputs} from './native-aggregate-files.js';

import {workspacePathParts} from './selected-workspace-io.js';

const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');

const stable=value=>Array.isArray(value)?value.map(stable):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(k=>[k,stable(value[k])])):value;

const identity=async value=>hash(new TextEncoder().encode(JSON.stringify(stable(value))));

const base64=bytes=>{let text='';for(let i=0;i<bytes.length;i+=32768)text+=String.fromCharCode(...bytes.subarray(i,i+32768));return btoa(text);};

const freeze=value=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};

const problem=(message,code)=>Object.assign(Error(message),{code});

// Simulation only: the supplied capability has no mutation methods. Scratch and

// directory effects occur in a private overlay, including predicted failure.

// Its JSON output is a review artifact and can never act as a live write grant.

export async function prepareNativeAggregateEffectPlan(execute,io,options={}){return prepare(execute,io,options);}
async function prepare(execute,io,{settings,version='v1',force=false,signal,assertCurrent,...unknown}={},historicalTargets=null){

 if(Object.keys(unknown).length)throw Error('Unsupported native effect plan option.');

 if(typeof execute!=='function'||!io||['stat','readdir','read'].some(k=>typeof io[k]!=='function'))throw Error('Read-only native effect snapshot capability is required.');

 aggregateWorkspaceVersion(version);if(typeof force!=='boolean'||!settings||!['csv','json','none','parquet'].includes(settings.format))throw Error('Invalid native effect plan settings.');validateNativeAggregateInputs({settings:{...settings,format:settings.format==='parquet'?'none':settings.format},reports:{}});settings=structuredClone(settings);

 const check=()=>{signal?.throwIfAborted();assertCurrent?.();},original=new Map(),overlay=new Map(),targets=new Map(),reads=new Map(),rosters=new Map(),actions=[],blobs=Object.create(null);let sourceBytes=0,targetBytes=0,blobBytes=0,snapshotFailure=null;

 const supplied=io;io=Object.fromEntries(['stat','readdir','read'].map(name=>[name,async(...args)=>{try{return await supplied[name](...args);}catch(e){snapshotFailure=e;throw e;}}]));

 const invalidSnapshot=message=>(snapshotFailure=Error(message));

 async function physicalStat(path){workspacePathParts(path);check();if(!original.has(path)){const found=await io.stat(path);check();if(found!==null&&(!found||Object.keys(found).join()!=='kind'||!['file','directory'].includes(found.kind)))throw invalidSnapshot('Invalid native effect stat reply.');original.set(path,found?{kind:found.kind}:null);}return original.get(path);}

 async function stat(path){return overlay.has(path)?overlay.get(path):await physicalStat(path);}

 async function identify(path){const value=await stat(path);if(!value)return null;if(value.kind==='directory')return {kind:'directory'};if(value.sha256)return {kind:'file',sha256:value.sha256,byte_length:value.byte_length};

  if(!targets.has(path)&&historicalTargets!==null){const value=historicalTargets[path];if(!value||Object.keys(value).sort().join()!=='byte_length,kind,sha256'||value.kind!=='file'||typeof value.sha256!=='string'||!/^[a-f0-9]{64}$/.test(value.sha256)||!Number.isSafeInteger(value.byte_length)||value.byte_length<0||value.byte_length>25*1024*1024||(targetBytes+=value.byte_length)>50*1024*1024)throw invalidSnapshot('Invalid historical native target identity.');targets.set(path,{...value});}
  if(!targets.has(path)){const bytes=await io.read(path,Math.min(25*1024*1024,50*1024*1024-targetBytes));check();if(!(bytes instanceof Uint8Array)||bytes.length>25*1024*1024||(targetBytes+=bytes.length)>50*1024*1024)throw problem('Native effect target snapshot exceeds its bounds.','ResourceLimit');targets.set(path,{kind:'file',sha256:await hash(bytes),byte_length:bytes.length});check();}return targets.get(path);

 }

 async function beforeAction(kind,path,to){check();if(actions.length>=128)throw problem('Native effect plan exceeds 128 actions.','ResourceLimit');const action={ordinal:actions.length,kind,path,...(to?{to}:{}),before:{},after:{},outcome:'incomplete'};actions.push(action);return action;}

 async function attempt(action,task){try{await task();action.outcome='completed';}catch(e){action.outcome='failed';action.failure={code:e.code||null,message:e.message};if(e.code==='ResourceLimit')snapshotFailure=e;throw e;}}

 const virtual={stat,readdir:async path=>{const value=await stat(path);if(value?.kind!=='directory')throw problem('Native source roster is not a directory.','NotDirectory');check();const entries=await io.readdir(path);check();if(!Array.isArray(entries)||entries.length>20000)throw problem('Native effect roster exceeds bounds.','ResourceLimit');const names=new Set();for(const e of entries){if(!e||Object.keys(e).sort().join()!=='kind,name'||typeof e.name!=='string'||!['file','directory'].includes(e.kind)||names.has(e.name.toLowerCase()))throw invalidSnapshot('Invalid or case-colliding native effect roster.');if(e.name.includes('/'))throw invalidSnapshot('Native effect roster entry is not a component.');try{workspacePathParts(e.name);}catch(e){snapshotFailure=e;throw e;}names.add(e.name.toLowerCase());}const sorted=entries.map(e=>({...e})).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);rosters.set(path,sorted);return sorted;},

  read:async(path,maxBytes)=>{const value=await stat(path);if(value?.kind!=='file')throw problem('Native source is not a file.','NotFile');if(!reads.has(path)){try{const parts=path.split('/'),overhead=parts.at(-3).length+parts.at(-1).slice(0,-5).length,remaining=8*1024*1024-sourceBytes-overhead;if(remaining<0)throw problem('Native effect source snapshot exceeds 8 MiB.','ResourceLimit');const bytes=await io.read(path,Math.min(maxBytes,remaining));check();if(!(bytes instanceof Uint8Array)||bytes.length>Math.min(maxBytes,remaining))throw problem('Native effect source snapshot exceeds its byte allowance.','ResourceLimit');const sha256=await hash(bytes);check();sourceBytes+=overhead+bytes.length;reads.set(path,{base64:base64(bytes),sha256,byte_length:bytes.length});}catch(e){snapshotFailure=e;throw e;}}return Uint8Array.from(atob(reads.get(path).base64),c=>c.charCodeAt(0));},

  mkdir:async path=>{const action=await beforeAction('mkdir',path);await attempt(action,async()=>{let current='';for(const part of path.split('/')){current=current?current+'/'+part:part;const value=await stat(current),before=value?{kind:value.kind}:null;action.before[current]=before;if(before?.kind==='file')throw problem('Native directory preparation collides with a file.','NotDirectory');if(!before)overlay.set(current,{kind:'directory'});action.after[current]={kind:'directory'};}});},

  write:async(path,bytes)=>{const action=await beforeAction('write',path);await attempt(action,async()=>{const parent=path.slice(0,path.lastIndexOf('/'));if((await stat(parent))?.kind!=='directory')throw problem('Native output parent is not a directory.','NotDirectory');const before=await identify(path);action.before[path]=before;if(before?.kind==='directory')throw problem('Native output write collides with a directory.','IsDirectory');const sha256=await hash(bytes);check();if(!Object.hasOwn(blobs,sha256)){if((blobBytes+=bytes.length)>50*1024*1024)throw problem('Native effect blobs exceed 50 MiB.','ResourceLimit');blobs[sha256]={base64:base64(bytes),byte_length:bytes.length};}action.sha256=sha256;action.byte_length=bytes.length;const after={kind:'file',sha256,byte_length:bytes.length};overlay.set(path,after);action.after[path]=after;});},

  rename:async(from,to)=>{const action=await beforeAction('rename',from,to);await attempt(action,async()=>{if(from.slice(0,from.lastIndexOf('/'))!==to.slice(0,to.lastIndexOf('/')))throw problem('Native effect plan requires a same-parent rename.','UnsupportedRename');const source=await identify(from),target=await identify(to);action.before[from]=source;action.before[to]=target;if(source?.kind!=='file'||!source.sha256)throw problem('Native temporary source is not a prepared file.','NotFile');if(target?.kind==='directory')throw problem('Native rename target is a directory.','IsDirectory');overlay.set(from,null);overlay.set(to,{...source});action.after[from]=null;action.after[to]={...source};});}

 };

 check();const result=await executeNativeAggregateEffects(execute,virtual,{settings,version,force,signal,assertCurrent});check();if(snapshotFailure)throw snapshotFailure;if(result.error?.code==='ResourceLimit')throw problem(result.error.message,'ResourceLimit');if(result.status==='failed'&&(!result.failed_operation||!['mkdir','discover-states','readdir','write','rename'].includes(result.failed_operation.operation)))throw Error('Cannot prepare a deterministic native effect review: '+result.error?.message);

 // Bind a stable physical snapshot after all simulation. Any drift invalidates

 // review preparation; no physical effect has been attempted by this module.

 for(const [path,expected]of original){check();const current=await io.stat(path);check();if(JSON.stringify(stable(current))!==JSON.stringify(stable(expected)))throw Error('Native effect workspace changed while preparing review.');}

 for(const [path,expected]of rosters){check();const current=(await io.readdir(path)).map(e=>({...e})).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);check();if(JSON.stringify(stable(current))!==JSON.stringify(stable(expected)))throw Error('Native effect state roster changed while preparing review.');}

 for(const [path,expected]of [...reads,...targets]){if(historicalTargets!==null&&targets.has(path)){if(!sameHistoryIdentity(expected,historicalTargets[path]))throw Error('Historical native target identity changed.');continue;}const bytes=await io.read(path,expected.byte_length);check();if(!(bytes instanceof Uint8Array)||bytes.length!==expected.byte_length||await hash(bytes)!==expected.sha256)throw Error('Native effect file changed while preparing review.');check();}

 const observation={stats:Object.fromEntries(original),rosters:Object.fromEntries(rosters),sources:Object.fromEntries(reads),targets:Object.fromEntries(targets)},plan={schema_version:'bisect-native-aggregate-effect-plan-prototype-v1',settings,version,force,observation,observation_sha256:await identity(observation),actions,blobs,projected:Object.fromEntries(overlay),predicted:{status:result.status,failed_operation:result.failed_operation||null,frames:result.frames,warnings:result.warnings,...(result.error?{error:result.error}:{})},required_capabilities:actions.some(a=>a.kind==='rename')?['same-parent-file-move']:[],scope:'Read-only bounded native-effect simulation. Native directory, scratch replacement and failure prefixes are reviewable history. No handles, write grant, permission, live effects, journal or recovery authority.'};check();plan.plan_sha256=await identity(plan);check();return freeze(plan);

}


const sameHistoryIdentity=(a,b)=>JSON.stringify(stable(a))===JSON.stringify(stable(b));
// Historical target bytes need not be retained to recompute native output.
// These identities are unauthenticated before-images, never current-root proof.
// The caller must separately compare a live prefix and obtain a fresh grant.
export async function recomputeNativeAggregateEffectPlan(execute,saved,{signal,assertCurrent,...unknown}={}){
 if(Object.keys(unknown).length)throw Error('Unsupported native history recomputation option.');
 if(!saved||saved.schema_version!=='bisect-native-aggregate-effect-plan-prototype-v1'||!saved.observation||!saved.observation.stats||!saved.observation.rosters||!saved.observation.sources||!saved.observation.targets)throw Error('Invalid saved native effect plan.');
 const plain={...saved};delete plain.plan_sha256;if(await identity(plain)!==saved.plan_sha256)throw Error('Saved native effect plan identity differs.');
 const observation=structuredClone(saved.observation),requireEntry=(record,path)=>{if(!Object.hasOwn(record,path))throw Error('Native history omits a required observation.');return record[path];},io={
  async stat(path){return structuredClone(requireEntry(observation.stats,path));},
  async readdir(path){return structuredClone(requireEntry(observation.rosters,path));},
  async read(path,max){const source=requireEntry(observation.sources,path);if(!source||typeof source.base64!=='string')throw Error('Native history source bytes missing.');if(source.base64.length>4*Math.ceil(max/3))throw problem('Historical source encoding exceeds its bound.','ResourceLimit');const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(bytes.length>max)throw problem('Historical source exceeds its bound.','ResourceLimit');return bytes;}
 };
 const recomputed=await prepare(execute,io,{settings:saved.settings,version:saved.version,force:saved.force,signal,assertCurrent},observation.targets);
 if(recomputed.plan_sha256!==saved.plan_sha256)throw Error('Saved native effect plan differs from actual runtime recomputation.');return recomputed;
}
