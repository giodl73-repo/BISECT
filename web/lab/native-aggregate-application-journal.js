import {workspaceCheck,workspacePermission,workspaceLookup,workspaceEntries,workspaceReadFile} from './selected-workspace-io.js';
export const aggregateJournalDirectory='.bisect-aggregate-journals';
const encode=s=>new TextEncoder().encode(s),decode=b=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(b);
export const aggregateJournalHash=async data=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',data)),b=>b.toString(16).padStart(2,'0')).join('');
const isJournalId=id=>typeof id==='string'&&/^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$/.test(id);
export function aggregateJournalId(id){if(!isJournalId(id))throw Error('Invalid aggregate journal ID.');return id;}
// Only this internal metadata parent may follow an existing case variant.
// Native source and output paths retain their exact-case lookup policy.
async function journalNamespace(root,o){
 await workspacePermission(root,o);const match=(await workspaceEntries(root,o)).find(([name])=>name.toLowerCase()===aggregateJournalDirectory);
 if(match&&match[1].kind!=='directory')throw Error('Aggregate journal parent is not a directory.');return match?match[0]:aggregateJournalDirectory;
}
function key(namespace,id,name){if(typeof namespace!=='string'||namespace.toLowerCase()!==aggregateJournalDirectory)throw Error('Invalid aggregate journal namespace.');return namespace+'/'+aggregateJournalId(id)+'/'+name;}
// Listing history is metadata-only. It grants neither a root nor permission.
export async function listNativeAggregateJournals(root,o={}){
 const namespace=await journalNamespace(root,o),directory=await workspaceLookup(root,namespace,{...o,kind:'directory',create:false});if(!directory)return [];
 // Native-valid versions may share this parent with year folders or notes.
 // Never read unrelated bodies or interpret them as journal records.
 const entries=(await workspaceEntries(directory,o)).filter(([id])=>isJournalId(id));if(entries.length>512)throw Error('Aggregate journal listing exceeds 512 entries.');
 for(const [,handle]of entries)if(handle.kind!=='directory')throw Error('Aggregate journal ID is not a directory.');workspaceCheck(o);return entries.map(([id])=>id).sort();
}
async function writeNew(root,path,value,limit,o){
 workspaceCheck(o);const bytes=encode(JSON.stringify(value));if(bytes.length>limit)throw Error('Aggregate journal exceeds its byte limit.');
 if(await workspaceLookup(root,path,{...o,create:false}))throw Error('Aggregate journal entry already exists.');
 const handle=await workspaceLookup(root,path,{...o,kind:'file',create:true});workspaceCheck(o);const stream=await handle.createWritable();
 try{workspaceCheck(o);await stream.write(bytes);workspaceCheck(o);await stream.close();}catch(e){await stream.abort().catch(()=>{});throw e;}
 const result=await workspaceReadFile(handle,{...o,maxBytes:limit});const sha256=await aggregateJournalHash(bytes);workspaceCheck(o);if(result.sha256!==sha256||result.byte_length!==bytes.length)throw Error('Aggregate journal read-back mismatch.');return {sha256,byte_length:bytes.length};
}
export async function createAggregateJournal(root,baseline,o={}){
 await workspacePermission(root,o,'readwrite');const namespace=await journalNamespace(root,o),id=aggregateJournalId(baseline.id),path=namespace+'/'+id;
 if(await workspaceLookup(root,path,{...o,create:false}))throw Error('Aggregate journal ID already exists.');
 const entry=await writeNew(root,key(namespace,id,'baseline.json'),baseline,16*1024*1024,o);
 return {id,namespace,entries:{'baseline.json':entry},previous:entry.sha256,records:[],record_slots:0,uncommitted_slots:[]};
}
export async function checkAggregateJournal(root,journal,o={}){
 const observed=await readAggregateJournal(root,journal.id,o);
 if(observed.namespace!==journal.namespace||JSON.stringify(observed.entries)!==JSON.stringify(journal.entries))throw Error('Aggregate journal changed after review.');return observed;
}
export async function appendAggregateJournal(root,journal,phase,cursor,o={}){
 await checkAggregateJournal(root,journal,o);const sequence=journal.record_slots;if(sequence>=128)throw Error('Aggregate journal record limit reached.');
 const record={schema_version:'bisect-native-aggregate-journal-record-v1',sequence,previous_sha256:journal.previous,phase,cursor},name=String(sequence).padStart(5,'0')+'.json';
 const entry=await writeNew(root,key(journal.namespace,journal.id,name),record,4096,o);journal.entries[name]=entry;journal.previous=entry.sha256;journal.records.push(record);journal.record_slots++;await checkAggregateJournal(root,journal,o);return record;
}
export async function readAggregateJournal(root,id,o={}){
 aggregateJournalId(id);const namespace=await journalNamespace(root,o),directory=await workspaceLookup(root,namespace+'/'+id,{...o,kind:'directory',create:false});if(!directory)throw Error('Aggregate journal is missing.');
 const entries=await workspaceEntries(directory,o);if(entries.length<1||entries.length>129||entries.some(([name,h])=>h.kind!=='file'||(name!=='baseline.json'&&!/^\d{5}\.json$/.test(name))))throw Error('Invalid aggregate journal roster.');
 const names=entries.map(([name])=>name).sort(),base=entries.find(([name])=>name==='baseline.json');if(!base)throw Error('Aggregate journal baseline is missing.');const raw=await workspaceReadFile(base[1],{...o,maxBytes:16*1024*1024});let baseline;
 try{baseline=JSON.parse(decode(raw.data));}catch{throw Error('Aggregate journal baseline is malformed.');}
 const identities={'baseline.json':{sha256:raw.sha256,byte_length:raw.byte_length}},records=[],uncommitted_slots=[];let previous=raw.sha256,record_slots=0;
 for(const name of names.filter(n=>n!=='baseline.json')){
  const sequence=record_slots++;if(name!==String(sequence).padStart(5,'0')+'.json')throw Error('Aggregate journal has a missing record.');const handle=entries.find(([n])=>n===name)[1],file=await workspaceReadFile(handle,{...o,maxBytes:4096});
  // A failed create/write can leave an empty file. Retain it as uncommitted
  // history; it cannot acknowledge an effect or advance the semantic cursor.
  if(file.byte_length===0){identities[name]={sha256:file.sha256,byte_length:0};uncommitted_slots.push(sequence);continue;}
  let r;try{r=JSON.parse(decode(file.data));}catch{throw Error('Aggregate journal record is malformed.');}
  if(!r||Object.keys(r).sort().join()!=='cursor,phase,previous_sha256,schema_version,sequence'||r.schema_version!=='bisect-native-aggregate-journal-record-v1'||r.sequence!==sequence||r.previous_sha256!==previous||!Number.isSafeInteger(r.cursor)||r.cursor<0||!['prepared','directory-intent','directory-done','file-intent','file-done','completed'].includes(r.phase))throw Error('Invalid aggregate journal record chain.');
  identities[name]={sha256:file.sha256,byte_length:file.byte_length};previous=file.sha256;records.push(r);
 }
 const finalNames=(await workspaceEntries(directory,o)).map(([n,h])=>[n,h.kind]).sort();if(JSON.stringify(finalNames)!==JSON.stringify(entries.map(([n,h])=>[n,h.kind]).sort()))throw Error('Aggregate journal roster changed while reading.');workspaceCheck(o);return {id,namespace,baseline,entries:identities,previous,records,record_slots,uncommitted_slots};
}
export function validateAggregateJournalTrace(journal,operations){
 let phase=null,cursor=0;
 for(const r of journal.records){let valid=false;
  if(phase===null)valid=r.phase==='prepared'&&r.cursor===0;
  else if(phase==='prepared')valid=r.phase==='directory-intent'&&r.cursor===0;
  else if(phase==='directory-intent')valid=['directory-intent','directory-done'].includes(r.phase)&&r.cursor===0;
  else if(phase==='directory-done'||phase==='file-done')valid=(r.phase==='file-intent'&&r.cursor===cursor&&cursor<operations)||(r.phase==='completed'&&cursor===operations&&r.cursor===cursor);
  else if(phase==='file-intent')valid=(r.phase==='file-intent'&&r.cursor===cursor)||(r.phase==='file-done'&&r.cursor===cursor+1);
  if(!valid)throw Error('Aggregate journal transition is invalid.');phase=r.phase;cursor=r.cursor;
 }
 if(phase===null)throw Error('Aggregate journal has no prepared checkpoint.');return {phase,cursor};
}
