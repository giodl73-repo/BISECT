import {fletchHistoryFromEntry} from './fletch-history.js';
import {validatedFletchCacheObject,validateFletchStageInputs} from './fletch-stage-files.js';
import {encodeBytes} from './package-files.js';
import {mergeFletchCacheInWorker,executeFletchCacheToolInWorker} from './fletch-cache-merge-worker.js';
export const FLETCH_CACHE_DB='bisect-fletch-cache-v1';
const MAX_OBJECT=64*1024*1024;
const emptyManifest='{"schema_version":"fletch.cache-manifest.v1","generated_by":"fletch-core/0.1.0","cache_root":"cache","entries":[]}';
function key(value){if(typeof value!=='string'||!/^sha256:[0-9a-f]{64}$/.test(value))throw new Error('Invalid local cache key.');return value;}
function check(signal){signal?.throwIfAborted();}
function connect(idb){return new Promise((resolve,reject)=>{
 if(!idb){reject(new Error('Browser cache storage is unavailable. Download the portable ZIP.'));return;}
 let done=false;const request=idb.open(FLETCH_CACHE_DB,1),timer=setTimeout(()=>finish(new Error('Browser cache storage timed out.')),10000);
 function finish(error,db){if(done){db?.close();return;}done=true;clearTimeout(timer);error?reject(error):resolve(db);}
 request.onupgradeneeded=()=>{for(const name of ['state','entries','objects'])request.result.createObjectStore(name);};
 request.onerror=()=>finish(request.error||new Error('Browser cache storage failed.'));
 request.onblocked=()=>finish(new Error('Close other cache connections and retry.'));
 request.onsuccess=()=>{request.result.onversionchange=()=>request.result.close();finish(null,request.result);};
});}
async function transact(mode,action,{idb=globalThis.indexedDB,signal}={}){
 check(signal);const db=await connect(idb);try{check(signal);}catch(error){db.close();throw error;}
 return new Promise((resolve,reject)=>{
  let tx,output,error,settled=false;
  const abort=()=>{try{tx?.abort();error=signal.reason||new Error('Cache action cancelled.');}catch{};};
  function finish(failure){if(settled)return;settled=true;signal?.removeEventListener('abort',abort);db.close();failure?reject(failure):resolve(output);}
  try{tx=db.transaction(['state','entries','objects'],mode);tx.oncomplete=()=>finish(error);tx.onabort=()=>finish(error||tx.error||new Error('Cache transaction aborted.'));tx.onerror=event=>{error=event.target?.error||tx.error||new Error('Cache transaction failed.');};signal?.addEventListener('abort',abort,{once:true});check(signal);
   action(tx,value=>{output=value;},failure=>{error=failure;try{tx.abort();}catch{finish(error);}});
  }catch(failure){error=failure;try{tx?.abort();}catch{};if(!tx)finish(error);}
 });
}
function state(value){if(value===undefined)return {token:null,manifest_json:emptyManifest};if(!value||typeof value.token!=='string'||value.token.length!==36||typeof value.manifest_json!=='string'||new TextEncoder().encode(value.manifest_json).length>16*1024*1024)throw new Error('Stored cache manifest is invalid.');return value;}
export function validateFletchCacheMetadata(value){
 if(!value||value.schema_version!=='bisect-local-cache-entry-v1'||!Number.isSafeInteger(value.bytes)||value.bytes<0||value.bytes>MAX_OBJECT||typeof value.entry_json!=='string'||value.entry_json.length>128*1024||typeof value.source_name!=='string'||!value.source_name||new TextEncoder().encode(value.source_name).length>200||typeof value.sha256!=='string'||!/^[0-9a-f]{64}$/.test(value.sha256))throw new Error('Stored cache entry is invalid.');key(value.cache_key);
 if(value.object_modified_at_ms!==undefined&&(typeof value.object_modified_at_ms!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(value.object_modified_at_ms)||BigInt(value.object_modified_at_ms)>((1n<<64n)-1n)))throw new Error('Invalid stored object time.');
 const input={source:null,dataset_id:value.dataset_id,version:value.version,source_url:value.source_url,fetched_at_ms:value.fetched_at_ms,expected_source_sha256:value.sha256};validateFletchStageInputs(input);return value;
}
const metadata=validateFletchCacheMetadata;
export function readFletchCacheSnapshot(options={}){return transact('readonly',(tx,set,fail)=>{
 const values={};let remaining=4;
 for(const [name,request]of [['state',tx.objectStore('state').get('active')],['rows',tx.objectStore('entries').getAll(undefined,2001)],['keys',tx.objectStore('objects').getAllKeys(undefined,2001)],['blobs',tx.objectStore('objects').getAll(undefined,2001)]])request.onsuccess=()=>{values[name]=request.result;if(--remaining)return;try{
  if(values.rows.length>2000||values.rows.length!==values.keys.length||values.keys.length!==values.blobs.length)throw new Error('Cache snapshot has missing or excess objects.');
  const rows=values.rows.map((row,i)=>{metadata(row);if(row.cache_key!==values.keys[i]||!(values.blobs[i] instanceof Blob)||values.blobs[i].size!==row.bytes)throw new Error('Cache snapshot object identity or size mismatch.');return {row,blob:values.blobs[i]};});set({state:state(values.state),objects:rows});
 }catch(error){fail(error);}};
},options);}
export function readFletchCacheState(options={}){return transact('readonly',(tx,set,fail)=>{const r=tx.objectStore('state').get('active');r.onsuccess=()=>{try{set(state(r.result));}catch(e){fail(e);}};},options);}
export function listFletchCacheObjects(options={}){return transact('readonly',(tx,set,fail)=>{const r=tx.objectStore('entries').getAll(undefined,2001);r.onsuccess=()=>{try{if(r.result.length>2000)throw new Error('Local cache exceeds 2000 objects.');set(r.result.map(metadata));}catch(e){fail(e);}};},options);}
export async function saveFletchCacheObject(result,value,files={},options={}){
 const {merge=(base,entries)=>mergeFletchCacheInWorker(base,entries,{signal:options.signal}),signal,force=false}=options;if(typeof merge!=='function'||typeof force!=='boolean')throw new Error('A native cache merge and explicit replacement choice are required.');
 check(signal);const captured=structuredClone(result),{bytes}=await validatedFletchCacheObject(captured,value,files);check(signal);
 const row=metadata({schema_version:'bisect-local-cache-entry-v1',cache_key:captured.cache_key,sha256:captured.source_sha256,bytes:bytes.length,entry_json:captured.entries_json,source_name:captured.source_name,dataset_id:captured.dataset_id,version:captured.version,source_url:captured.source_url,fetched_at_ms:captured.fetched_at_ms_text});
 const blob=new Blob([bytes]);
 for(let attempt=0;attempt<8;attempt++){
  const prior=await readFletchCacheState(options);check(signal);const merged=await merge(prior.manifest_json,row.entry_json);check(signal);
  if(merged?.schema_version!=='bisect-fletch-cache-merge-v1'||typeof merged.manifest_json!=='string'||new TextEncoder().encode(merged.manifest_json).length>16*1024*1024||!Number.isSafeInteger(merged.entry_count)||merged.entry_count>2000)throw new Error('Invalid native merged cache manifest.');
  const committed=await transact('readwrite',(tx,set,fail)=>{
   const s=tx.objectStore('state'),e=tx.objectStore('entries'),o=tx.objectStore('objects'),r=s.get('active');
   r.onsuccess=()=>{try{check(signal);options.assertCurrent?.();if(state(r.result).token!==prior.token){set(false);return;}
    const old=e.get(row.cache_key);old.onsuccess=()=>{try{check(signal);options.assertCurrent?.();if(old.result&&metadata(old.result).sha256!==row.sha256&&!force)throw new Error('Cache key already holds different bytes. Select explicit replacement to overwrite.');
     e.put({...row,object_modified_at_ms:String(Date.now())},row.cache_key);o.put(blob,row.cache_key);s.put({token:crypto.randomUUID(),manifest_json:merged.manifest_json},'active');set(true);
    }catch(error){fail(error);}};
   }catch(error){fail(error);}};
  },options);
  if(committed)return {cache_key:row.cache_key,sha256:row.sha256,bytes:row.bytes,entry_count:merged.entry_count};
 }
 throw new Error('Cache changed repeatedly in another tab. Retry this save.');
}
export async function loadFletchCacheObject(cacheKey,options={}){
 key(cacheKey);const record=await transact('readonly',(tx,set)=>{let row,blob;const a=tx.objectStore('entries').get(cacheKey),b=tx.objectStore('objects').get(cacheKey);a.onsuccess=()=>{row=a.result;set({row,blob});};b.onsuccess=()=>{blob=b.result;set({row,blob});};},options);check(options.signal);
 if(!record.row||!record.blob)throw new Error('Selected cache object is missing.');const row=metadata(record.row),blob=record.blob;
 if(row.cache_key!==cacheKey||!(blob instanceof Blob)||blob.size!==row.bytes)throw new Error('Stored cache object has invalid identity or size.');
 const bytes=new Uint8Array(await blob.arrayBuffer());check(options.signal);const sha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');check(options.signal);
 if(sha!==row.sha256)throw new Error('Stored cache object failed its fresh SHA-256 check.');
 const input={fetch_history:fletchHistoryFromEntry(row.entry_json),source:{name:row.source_name,base64:encodeBytes(bytes)},dataset_id:row.dataset_id,version:row.version,source_url:row.source_url,fetched_at_ms:row.fetched_at_ms,expected_source_sha256:row.sha256};validateFletchStageInputs(input,{requireComplete:true});return {input,cache_key:cacheKey,entry_json:row.entry_json};
}

export async function commitFletchCacheBackup(backup,options={}){
 const {signal,force=false,merge=(base,entries)=>mergeFletchCacheInWorker(base,entries,{signal})}=options;
 if(backup?.schema_version!=='bisect-validated-cache-backup-v1'||!Array.isArray(backup.objects)||backup.objects.length>2000||typeof backup.entries_json!=='string'||new TextEncoder().encode(backup.entries_json).length>16*1024*1024||typeof force!=='boolean'||typeof merge!=='function')throw new Error('A validated cache backup and explicit replacement choice are required.');
 const keys=new Set();for(const {row,blob}of backup.objects){metadata(row);if(keys.has(row.cache_key)||!(blob instanceof Blob)||blob.size!==row.bytes)throw new Error('Invalid validated backup object.');keys.add(row.cache_key);}
 if(backup.entries_json!=='['+backup.objects.map(o=>o.row.entry_json.trim().slice(1,-1)).join(',')+']')throw new Error('Validated backup entries differ from objects.');
 for(let attempt=0;attempt<8;attempt++){
  check(signal);const prior=await readFletchCacheState(options),merged=await merge(prior.manifest_json,backup.entries_json);check(signal);
  if(merged?.schema_version!=='bisect-fletch-cache-merge-v1'||typeof merged.manifest_json!=='string'||new TextEncoder().encode(merged.manifest_json).length>16*1024*1024||!Number.isSafeInteger(merged.entry_count)||merged.entry_count>2000)throw new Error('Invalid native restore merge.');
  const committed=await transact('readwrite',(tx,set,fail)=>{
   const states=tx.objectStore('state'),entries=tx.objectStore('entries'),objects=tx.objectStore('objects'),read=states.get('active');
   read.onsuccess=()=>{try{check(signal);options.assertCurrent?.();if(state(read.result).token!==prior.token){set(false);return;}
    const existing=entries.getAll(undefined,2001);existing.onsuccess=()=>{try{check(signal);options.assertCurrent?.();if(existing.result.length>2000)throw new Error('Existing cache exceeds 2000 objects.');const old=new Map(existing.result.map(row=>[metadata(row).cache_key,row]));
     for(const {row}of backup.objects)if(old.has(row.cache_key)&&old.get(row.cache_key).sha256!==row.sha256&&!force)throw new Error('Cache key already holds different bytes. Select explicit replacement to overwrite.');
     for(const {row,blob}of backup.objects){entries.put({...row,object_modified_at_ms:String(Date.now())},row.cache_key);objects.put(blob,row.cache_key);}states.put({token:crypto.randomUUID(),manifest_json:merged.manifest_json},'active');set(true);
    }catch(error){fail(error);}};
   }catch(error){fail(error);}};
  },options);
  if(committed)return {restored_count:backup.objects.length,entry_count:merged.entry_count,snapshot_sha256:backup.snapshot_sha256};
 }
 throw new Error('Cache changed repeatedly in another tab. Retry restoration.');
}

export function readFletchCacheInventory(options={}){return transact('readonly',(tx,set,fail)=>{
 const values={};let remaining=3;for(const [name,request]of [['state',tx.objectStore('state').get('active')],['keys',tx.objectStore('objects').getAllKeys(undefined,10001)],['blobs',tx.objectStore('objects').getAll(undefined,10001)]])request.onsuccess=()=>{values[name]=request.result;if(--remaining)return;try{
  if(values.keys.length>10000||values.keys.length!==values.blobs.length)throw new Error('Cache prune inventory exceeds 10000 objects.');
  const objects=values.keys.map((cache_key,i)=>{key(cache_key);const blob=values.blobs[i];if(!(blob instanceof Blob)||!Number.isSafeInteger(blob.size)||blob.size>MAX_OBJECT)throw new Error('Invalid cache inventory object.');return {cache_key,bytes:blob.size};});set({state:state(values.state),objects});
 }catch(error){fail(error);}};
},options);}
export async function planBrowserFletchCachePrune(options={}){
 const snapshot=await readFletchCacheInventory(options);check(options.signal);const files=Object.fromEntries(snapshot.objects.map(o=>['objects/sha256/'+o.cache_key.slice(7),String(o.bytes)]));
 const result=await executeFletchCacheToolInWorker({operation:'plan-fletch-cache-prune',cache_manifest_json:snapshot.state.manifest_json,files},options);check(options.signal);options.assertCurrent?.();
 if(result?.schema_version!=='bisect-fletch-cache-prune-v1'||typeof result.plan_json!=='string'||!Array.isArray(result.candidates)||result.candidates.length!==result.prune_count)throw new Error('Invalid native cache prune preview.');
 return {schema_version:'bisect-browser-cache-prune-preview-v1',state:snapshot.state,result};
}
export async function applyBrowserFletchCachePrune(preview,options={}){
 if(preview?.schema_version!=='bisect-browser-cache-prune-preview-v1')throw new Error('Preview cache pruning first.');
 const fresh=await planBrowserFletchCachePrune(options);if(fresh.state.token!==preview.state.token||fresh.state.manifest_json!==preview.state.manifest_json||fresh.result.plan_json!==preview.result.plan_json)throw new Error('Cache changed since the prune preview. Preview again.');
 const candidates=fresh.result.candidates.map(row=>{if(!/^objects\/sha256\/[0-9a-f]{64}$/.test(row.relative_path))throw new Error('Invalid browser prune object path.');return {cache_key:'sha256:'+row.relative_path.slice(15),bytes:Number(row.byte_count_text)};});
 const removed=await transact('readwrite',(tx,set,fail)=>{
  const states=tx.objectStore('state'),objects=tx.objectStore('objects'),entries=tx.objectStore('entries'),read=states.get('active');read.onsuccess=()=>{try{
   check(options.signal);options.assertCurrent?.();const current=state(read.result);if(current.token!==fresh.state.token||current.manifest_json!==fresh.state.manifest_json)throw new Error('Cache changed since the prune preview. Preview again.');
   if(!candidates.length){set(0);return;}let remaining=candidates.length;
   for(const candidate of candidates){const request=objects.get(candidate.cache_key);request.onsuccess=()=>{try{check(options.signal);options.assertCurrent?.();if(!(request.result instanceof Blob)||request.result.size!==candidate.bytes)throw new Error('Cache object changed since prune preview. Preview again.');
    if(--remaining)return;for(const object of candidates){objects.delete(object.cache_key);entries.delete(object.cache_key);}states.put({token:crypto.randomUUID(),manifest_json:current.manifest_json},'active');set(candidates.length);
   }catch(error){fail(error);}};}
  }catch(error){fail(error);}};
 },options);
 return {removed_count:removed,removed_bytes_text:fresh.result.prune_bytes_text};
}

export function readFletchCacheRecord(cacheKey,options={}){key(cacheKey);return transact('readonly',(tx,set,fail)=>{
 let row,blob,remaining=2;for(const [name,request]of [['row',tx.objectStore('entries').get(cacheKey)],['blob',tx.objectStore('objects').get(cacheKey)]])request.onsuccess=()=>{if(name==='row')row=request.result;else blob=request.result;if(--remaining)return;try{if(row&&options.validateMetadata!==false){metadata(row);if(row.cache_key!==cacheKey)throw new Error('Cache row key mismatch.');}if(blob&&(!(blob instanceof Blob)||blob.size>MAX_OBJECT))throw new Error('Invalid cached source Blob.');set({row,blob});}catch(error){fail(error);}};
},options);}
