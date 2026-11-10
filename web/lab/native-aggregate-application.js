import {readNativeAggregateWorkspace} from './native-aggregate-workspace.js';
import {prepareNativeAggregateApplicationPlan} from './native-aggregate-application-plan.js';
import {resolvedNativeAggregateTypes,NATIVE_AGGREGATE_TYPES} from './native-aggregate-files.js';
import {workspaceCheck,workspacePermission,workspaceLookup,workspaceEntries,workspaceReadFile} from './selected-workspace-io.js';
import {aggregateJournalHash,aggregateJournalId,aggregateJournalDirectory,createAggregateJournal,readAggregateJournal,checkAggregateJournal,appendAggregateJournal,validateAggregateJournalTrace} from './native-aggregate-application-journal.js';
const grants=new WeakMap(),epochs=new WeakMap(),encode=s=>new TextEncoder().encode(s);
// A fixed origin-wide lock deliberately covers all Aggregate destinations: picker
// handles have no portable stable path identity. Fail fast; never queue stale
// confirmations. This coordinates participating application writers, not OS tools.
let applying=false;
async function exclusiveApplication(action,o){
 workspaceCheck(o);if(applying)throw Error('Another aggregate application is active. Review again after it finishes.');applying=true;
 try{
  if(globalThis.navigator?.locks?.request)return await navigator.locks.request('bisect-native-aggregate-application-v1',{mode:'exclusive',ifAvailable:true},async lock=>{workspaceCheck(o);if(!lock)throw Error('Another aggregate application is active in this browser origin. Review again after it finishes.');return await action();});
  if(typeof window!=='undefined')throw Error('Selected-folder application requires browser write coordination. Use portable input and output ZIP in this browser.');
  // Non-browser callers have one module-local execution context. Browser folder
  // application fails closed without Web Locks; it never falls back to this.
  return await action();
 }finally{applying=false;}
}
// Record field order has no semantic meaning; array order and repetition do.
const same=(a,b)=>{if(a===b)return true;if(!a||!b||typeof a!=='object'||typeof b!=='object')return false;if(Array.isArray(a)||Array.isArray(b))return Array.isArray(a)&&Array.isArray(b)&&a.length===b.length&&a.every((v,i)=>same(v,b[i]));const ak=Object.keys(a).sort(),bk=Object.keys(b).sort();return ak.length===bk.length&&ak.every((k,i)=>k===bk[i]&&same(a[k],b[k]));};
const activeTypes=input=>[...new Set(resolvedNativeAggregateTypes(input.settings.types))].filter(t=>input.completion?.force||!input.completion?.existing_outputs.includes('us_'+t+'.json'));
const completion=input=>({year:input.settings.year,force:input.completion?.force??false,existing_outputs:[...(input.completion?.existing_outputs||[])].sort()});
function options({signal,assertCurrent,onProgress,...unknown}={}){
 if(Object.keys(unknown).length)throw Error('Unsupported aggregate application option.');
 return {signal,assertCurrent,onProgress};
}
async function sources(root,p,o){
 const prefix=p.version+'/'+p.year,states=await workspaceLookup(root,prefix+'/states',{...o,kind:'directory',create:false});
 if(!states)throw Error('Aggregate state directory disappeared.');
 const roster=(await workspaceEntries(states,o)).filter(([,h])=>h.kind==='directory').map(([n])=>n).sort();
 if(!roster.length||roster.length>60)throw Error('Aggregate source roster changed or exceeds bounds.');
 const active=activeTypes(p.input),files={};let total=0;
 for(const state of roster)for(const type of active){const path=prefix+'/states/'+state+'/analysis/'+type+'.json',handle=await workspaceLookup(root,path,{...o,kind:'file',create:false});
  if(!handle){files[path]=null;continue;}const remaining=8*1024*1024-total-state.length-type.length;if(remaining<0)throw Error('Aggregate source snapshot exceeds 8 MiB.');const file=await workspaceReadFile(handle,{...o,maxBytes:Math.min(3*1024*1024,remaining)});if((total+=state.length+type.length+file.byte_length)>8*1024*1024)throw Error('Aggregate source snapshot exceeds 8 MiB.');files[path]={sha256:file.sha256,byte_length:file.byte_length};
 }
 return {roster,files};
}
async function targets(root,p,o){
 const directory=await workspaceLookup(root,p.target_namespace,{...o,kind:'directory',create:false}),files={};
 const written=new Set(p.operations.map(w=>w.path)),paths=[...new Set([...written,...NATIVE_AGGREGATE_TYPES.map(t=>p.target_namespace+'/us_'+t+'.json')])].sort();let total=0;
 for(const path of paths){const handle=await workspaceLookup(root,path,{...o,create:false});if(!handle){files[path]=null;continue;}
  if(!written.has(path)){files[path]={kind:handle.kind};continue;}
  if(handle.kind!=='file')throw Error('Aggregate output collides with a directory: '+path);
  const f=await workspaceReadFile(handle,{...o,maxBytes:Math.min(25*1024*1024,50*1024*1024-total)});if((total+=f.byte_length)>50*1024*1024)throw Error('Aggregate target snapshot exceeds 50 MiB.');files[path]={kind:'file',sha256:f.sha256,byte_length:f.byte_length};
 }
 return {directory:!!directory,files};
}
async function recheck(g,expected,o){
 workspaceCheck(o);if(epochs.get(g.root)!==g.epoch)throw Error('Aggregate review was superseded.');
 await workspacePermission(g.root,o,'readwrite');
 if(!same(await sources(g.root,g.plan,o),g.source))throw Error('Aggregate sources changed after review.');
 if(!same(await targets(g.root,g.plan,o),expected))throw Error('Aggregate targets changed after review.');workspaceCheck(o);
 if(g.journal)await checkAggregateJournal(g.root,g.journal,o);workspaceCheck(o);
}
// This live review is process-local and single-use. Its serialized data is history,
// never permission. The append-only journal is checked before each native effect.
export async function reviewNativeAggregateApplication(execute,input,root,{version='v1',...rest}={}){
 const o=options(rest);await workspacePermission(root,o,'readwrite');const epoch=(epochs.get(root)||0)+1;epochs.set(root,epoch);
 const plan=await prepareNativeAggregateApplicationPlan(execute,input,{version,signal:o.signal,assertCurrent:o.assertCurrent});
 const selected=await readNativeAggregateWorkspace(root,plan.input.settings,{version,force:plan.input.completion?.force??false,signal:o.signal,assertCurrent:o.assertCurrent});
 if(!same(completion(selected.input),completion(plan.input)))throw Error('Live aggregate output observation differs from the retained input. Read the workspace again.');
 const source=await sources(root,plan,o),before=await targets(root,plan,o);
 // Compare the effective native sources, not transport object order, advisory
 // filenames or retained bodies that native completion skips. Arrays carrying
 // analyzer order remain intact in the actual request and prepared write trace.
 const declared=await declaredSources(plan,o),observedFiles=Object.fromEntries(selected.observation.sources.map(({path,sha256,byte_length})=>[path,{sha256,byte_length}])),observed={roster:selected.observation.states,files:Object.fromEntries(Object.keys(declared.files).map(path=>[path,observedFiles[path]??null]))};
 if(Object.keys(observedFiles).some(path=>!Object.hasOwn(declared.files,path))||!same(source,declared)||!same(observed,declared))throw Error('Aggregate sources changed or differ from the retained input.');
 const changes=Object.freeze(plan.operations.map(w=>Object.freeze({ordinal:w.ordinal,frame:w.frame,path:w.path,sha256:w.sha256,byte_length:w.byte_length,replaces:before.files[w.path]!==null})));
 const review=Object.freeze({schema_version:'bisect-native-aggregate-live-review-v1',root_name:root.name,version,year:plan.year,input_sha256:plan.input_sha256,result_sha256:plan.result_sha256,directory:plan.target_namespace,creates_directory:!before.directory,changes,mode:'fresh',scope:'Current selected root, single-use ordered final-file application with read-back-verified journal. No portable grant or atomic multi-file transaction.'});
 const g={root,epoch,plan,source,before};await recheck(g,before,o);grants.set(review,g);return review;
}
export function retireNativeAggregateApplication(review){grants.delete(review);}
async function declaredSources(p,o){
 const prefix=p.version+'/'+p.year,roster=Object.keys(p.input.reports).sort(),files={},active=activeTypes(p.input);
 for(const state of roster)for(const type of active){const path=prefix+'/states/'+state+'/analysis/'+type+'.json',raw=p.input.reports[state][type];if(!raw){files[path]=null;continue;}const bytes=Uint8Array.from(atob(raw.base64),c=>c.charCodeAt(0));files[path]={sha256:await aggregateJournalHash(bytes),byte_length:bytes.length};workspaceCheck(o);}return {roster,files};
}
function validateBefore(p,before){
 const writes=new Set(p.operations.map(w=>w.path)),paths=[...new Set([...writes,...NATIVE_AGGREGATE_TYPES.map(t=>p.target_namespace+'/us_'+t+'.json')])].sort();
 if(!before||Object.keys(before).sort().join()!=='directory,files'||typeof before.directory!=='boolean'||!before.files||Array.isArray(before.files)||!same(Object.keys(before.files).sort(),paths))throw Error('Invalid aggregate journal target baseline.');let total=0;
 for(const [path,f]of Object.entries(before.files)){if(f===null)continue;if(!before.directory||!f||typeof f!=='object'||Array.isArray(f))throw Error('Invalid aggregate journal target identity.');
  if(writes.has(path)){if(Object.keys(f).sort().join()!=='byte_length,kind,sha256'||f.kind!=='file'||typeof f.sha256!=='string'||!/^[a-f0-9]{64}$/.test(f.sha256)||!Number.isSafeInteger(f.byte_length)||f.byte_length<0||f.byte_length>25*1024*1024||(total+=f.byte_length)>50*1024*1024)throw Error('Invalid aggregate journal file identity.');}
  else if(Object.keys(f).join()!=='kind'||!['file','directory'].includes(f.kind))throw Error('Invalid aggregate journal completion metadata.');
 }
}
function prefixState(p,before,cursor,directory=true){const state=structuredClone(before);state.directory=directory;for(const w of p.operations.slice(0,cursor))state.files[w.path]={kind:'file',sha256:w.sha256,byte_length:w.byte_length};return state;}
export async function reviewNativeAggregateRecovery(execute,root,id,rest={}){
 const o=options(rest);aggregateJournalId(id);await workspacePermission(root,o,'readwrite');const epoch=(epochs.get(root)||0)+1;epochs.set(root,epoch);
 const journal=await readAggregateJournal(root,id,o),b=journal.baseline;
 if(!b||Object.keys(b).sort().join()!=='before,id,input,input_sha256,operations_sha256,result_sha256,schema_version,source,version'||b.schema_version!=='bisect-native-aggregate-journal-baseline-v1'||b.id!==id)throw Error('Invalid aggregate journal baseline.');
 const plan=await prepareNativeAggregateApplicationPlan(execute,b.input,{version:b.version,signal:o.signal,assertCurrent:o.assertCurrent});
 if(b.input_sha256!==plan.input_sha256||b.result_sha256!==plan.result_sha256||b.operations_sha256!==await aggregateJournalHash(encode(JSON.stringify(plan.operations))))throw Error('Aggregate recovery recomputation differs from journal.');
 validateBefore(plan,b.before);const source=await declaredSources(plan,o);if(!same(b.source,source)||!same(await sources(root,plan,o),source))throw Error('Aggregate recovery sources differ from original input.');
 const trace=validateAggregateJournalTrace(journal,plan.operations.length),current=await targets(root,plan,o);let cursor=trace.cursor,pendingAck=false,disposition=trace.phase;
 if(trace.phase==='completed'){if(!same(current,prefixState(plan,b.before,cursor)))throw Error('Completed aggregate output has changed.');throw Error('Aggregate journal is already completed.');}
 let expected=prefixState(plan,b.before,cursor,trace.phase==='prepared'?b.before.directory:true);
 if(trace.phase==='directory-intent'){const original=prefixState(plan,b.before,0,b.before.directory);if(same(current,original))expected=original;}
 if(trace.phase==='file-intent'){
  const after=prefixState(plan,b.before,cursor+1),preMatches=same(current,expected),postMatches=same(current,after);
  if(preMatches&&postMatches)throw Error('Aggregate intent is ambiguous: identical before/after bytes cannot prove whether the repeated write completed.');
  if(postMatches){expected=after;cursor++;pendingAck=true;disposition='observed-unacknowledged-close';}
  else if(!preMatches){const w=plan.operations[cursor],empty=structuredClone(expected);empty.files[w.path]={kind:'file',sha256:await aggregateJournalHash(new Uint8Array()),byte_length:0};if(expected.files[w.path]===null&&same(current,empty)){expected=empty;disposition='created-empty-file';}else throw Error('Aggregate intent has unknown partial or concurrent target bytes.');}
 }
 if(!same(current,expected))throw Error('Aggregate recovery target differs from recorded prefix.');
 const directoryDone=['directory-done','file-intent','file-done'].includes(trace.phase);
 const needed=(plan.operations.length-cursor)*2+(pendingAck?1:0)+(directoryDone?0:2)+1;if(journal.record_slots+needed>128)throw Error('Aggregate recovery would exceed its journal record limit.');
 const changes=Object.freeze(plan.operations.map(w=>Object.freeze({ordinal:w.ordinal,frame:w.frame,path:w.path,sha256:w.sha256,byte_length:w.byte_length,retained:w.ordinal<cursor,replaces:current.files[w.path]!==null})));
 const review=Object.freeze({schema_version:'bisect-native-aggregate-live-review-v1',root_name:root.name,version:plan.version,year:plan.year,input_sha256:plan.input_sha256,result_sha256:plan.result_sha256,directory:plan.target_namespace,creates_directory:!current.directory,changes,mode:'recovery',journal_id:id,cursor,disposition,uncommitted_journal_slots:Object.freeze([...journal.uncommitted_slots]),scope:'Fresh selected-root recovery review. Journal is unauthenticated history; empty uncommitted entries are retained and acknowledge nothing. Explicit confirmation authorizes the listed remaining writes. Unknown or ambiguous effects refuse.'});
 const g={root,epoch,plan,source,before:current,journal,cursor,pendingAck,directoryDone};await recheck(g,current,o);grants.set(review,g);return review;
}
export async function applyNativeAggregateApplication(review,{confirmed=false,...rest}={}){
 const o=options(rest),g=grants.get(review);if(!g)throw Error('Aggregate review has no current authority.');
 if(confirmed!==true)throw Error('Confirm the reviewed aggregate changes.');grants.delete(review);
 const expected=structuredClone(g.before),receipt={schema_version:'bisect-native-aggregate-application-v1',status:'incomplete',mode:review.mode,input_sha256:review.input_sha256,result_sha256:review.result_sha256,journal_id:g.journal?.id??null,journal_effects_may_have_started:false,recovered_operations:g.cursor??0,completed_operations:[],attempted_operation:null,effects_may_have_started:false,scope:'Historical ordered final-file application record. Journal requires fresh selected-root review and explicit confirmation for recovery; no portable permission, automatic retry or atomic multi-file transaction.'};
 const emit=()=>{try{o.onProgress?.(structuredClone(receipt));}catch{}};
 try{return await exclusiveApplication(async()=>{
  await recheck(g,expected,o);workspaceCheck(o);
  if(!g.journal){const id=crypto.randomUUID();receipt.journal_id=id;receipt.journal_effects_may_have_started=true;const baseline={schema_version:'bisect-native-aggregate-journal-baseline-v1',id,version:g.plan.version,input:g.plan.input,input_sha256:g.plan.input_sha256,result_sha256:g.plan.result_sha256,operations_sha256:await aggregateJournalHash(encode(JSON.stringify(g.plan.operations))),source:g.source,before:g.before};workspaceCheck(o);g.journal=await createAggregateJournal(g.root,baseline,o);await appendAggregateJournal(g.root,g.journal,'prepared',0,o);}
  await recheck(g,expected,o);
  receipt.journal_effects_may_have_started=true;
  if(g.pendingAck){await appendAggregateJournal(g.root,g.journal,'file-done',g.cursor,o);await recheck(g,expected,o);}
  if(!g.directoryDone){await appendAggregateJournal(g.root,g.journal,'directory-intent',0,o);await recheck(g,expected,o);receipt.effects_may_have_started=true;emit();workspaceCheck(o);await workspaceLookup(g.root,g.plan.target_namespace,{...o,kind:'directory',create:true});expected.directory=true;await recheck(g,expected,o);await appendAggregateJournal(g.root,g.journal,'directory-done',0,o);}
  for(const w of g.plan.operations.slice(g.cursor??0)){
   await recheck(g,expected,o);receipt.attempted_operation=w.ordinal;emit();workspaceCheck(o);
   await appendAggregateJournal(g.root,g.journal,'file-intent',w.ordinal,o);await recheck(g,expected,o);receipt.effects_may_have_started=true;workspaceCheck(o);
   const handle=await workspaceLookup(g.root,w.path,{...o,kind:'file',create:true});workspaceCheck(o);
   const stream=await handle.createWritable();try{workspaceCheck(o);await stream.write(encode(g.plan.blobs[w.sha256].text));workspaceCheck(o);await stream.close();}catch(e){await stream.abort().catch(()=>{});throw e;}
   const actual=await workspaceReadFile(handle,{...o,maxBytes:25*1024*1024});if(actual.sha256!==w.sha256||actual.byte_length!==w.byte_length)throw Error('Aggregate output read-back mismatch: '+w.path);
   expected.files[w.path]={kind:'file',sha256:w.sha256,byte_length:w.byte_length};await recheck(g,expected,o);await appendAggregateJournal(g.root,g.journal,'file-done',w.ordinal+1,o);await recheck(g,expected,o);
   receipt.completed_operations.push({ordinal:w.ordinal,path:w.path,sha256:w.sha256,byte_length:w.byte_length});receipt.attempted_operation=null;emit();workspaceCheck(o);
  }
  await recheck(g,expected,o);await appendAggregateJournal(g.root,g.journal,'completed',g.plan.operations.length,o);await recheck(g,expected,o);receipt.status='completed';emit();return receipt;
 },o);}catch(e){receipt.status=o.signal?.aborted?'cancelled':'interrupted';receipt.error=e.message;emit();e.aggregateReceipt=structuredClone(receipt);throw e;}
}
