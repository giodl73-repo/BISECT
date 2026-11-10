import {readFletchCacheSnapshot} from './fletch-cache-store.js';
const MAX_VOLUME=64*1024*1024,MAX_METADATA=4*1024*1024;
export async function prepareFletchCacheBackup({targetBytes=MAX_VOLUME,...options}={}){
 if(!Number.isSafeInteger(targetBytes)||targetBytes<1||targetBytes>MAX_VOLUME)throw new Error('Invalid cache backup volume target.');
 const snapshot=await readFletchCacheSnapshot(options);options.signal?.throwIfAborted();
 const text=snapshot.state.manifest_json,manifest=JSON.parse(text),entries=manifest.entries;
 if(manifest.schema_version!=='fletch.cache-manifest.v1'||manifest.cache_root!=='cache'||!Array.isArray(entries)||entries.length!==snapshot.objects.length)throw new Error('Cache manifest does not match snapshot objects.');
 const objects=new Map(snapshot.objects.map(o=>[o.row.cache_key,o])),seen=new Set(),ordered=[];
 const timestamps=[...text.matchAll(/"fetched_at_ms"\s*:\s*(0|[1-9][0-9]*)(?=\s*[,}])/g)];if(timestamps.length!==entries.length)throw new Error('Invalid exact cache manifest timestamps.');
 for(const [i,entry]of entries.entries()){
  const object=objects.get(entry.cache_key);if(!object||seen.has(entry.cache_key))throw new Error('Duplicate or missing cache manifest object.');seen.add(entry.cache_key);
  const row=object.row,record=JSON.parse(row.entry_json);if(!Array.isArray(record)||record.length!==1)throw new Error('Invalid cached native entry.');
  for(const k of ['dataset_id','version','source_url','cache_key','relative_path','sha256','bytes','verified','fetch_attempts','retry_count','last_retryable_error'])if(entry[k]!==record[0][k])throw new Error('Cached native entry differs from manifest.');
  for(const k of ['dataset_id','version','source_url','cache_key','bytes'])if(entry[k]!==row[k])throw new Error('Cached backup metadata differs from native entry.');
  if(entry.sha256!=='sha256:'+row.sha256||entry.relative_path!=='objects/sha256/'+row.cache_key.slice(7))throw new Error('Cached backup object path or hash differs from native entry.');
  if(timestamps[i][1]!==row.fetched_at_ms)throw new Error('Cached exact timestamp differs from manifest.');ordered.push(object);
 }
 const sha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');options.signal?.throwIfAborted();
 const groups=[];let group=[],bytes=0,metadata=0;
 for(const object of ordered){const size=new TextEncoder().encode(JSON.stringify(object.row)).length;if(group.length&&(bytes+object.row.bytes>targetBytes||metadata+size>MAX_METADATA||group.length>=500)){groups.push(group);group=[];bytes=0;metadata=0;}group.push(object);bytes+=object.row.bytes;metadata+=size;}
 if(group.length||!groups.length)groups.push(group);
 return {schema_version:'bisect-cache-backup-session-v1',snapshot_sha256:sha,manifest_json:text,object_count:ordered.length,groups,volumes:new Map()};
}
export function buildFletchCacheBackupVolume(session,index,{signal}={}){
 if(session?.schema_version!=='bisect-cache-backup-session-v1'||!Number.isInteger(index)||index<0||index>=session.groups.length)return Promise.reject(new Error('Select a cache backup volume.'));
 return new Promise((resolve,reject)=>{
  if(signal?.aborted){reject(signal.reason);return;}
  const worker=new Worker(new URL('./fletch-cache-backup-worker.js',import.meta.url),{type:'module'});let done=false;
  const finish=(error,value)=>{if(done)return;done=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(value);};
  const abort=()=>finish(signal.reason||new Error('Cache backup cancelled.')),timer=setTimeout(()=>finish(new Error('Cache backup volume timed out.')),60000);signal?.addEventListener('abort',abort,{once:true});worker.onerror=worker.onmessageerror=()=>finish(new Error('Cache backup worker failed.'));
  worker.onmessage=({data})=>{if(data.ok!==true){finish(new Error(data.error||'Cache backup failed.'));return;}if(!(data.bytes instanceof Uint8Array)||data.bytes.length>128*1024*1024||typeof data.sha256!=='string'||!/^[0-9a-f]{64}$/.test(data.sha256)){finish(new Error('Invalid cache backup output.'));return;}
   const volume={name:`cache-${session.snapshot_sha256.slice(0,16)}-${String(index+1).padStart(3,'0')}.zip`,sha256:data.sha256,byte_length:data.bytes.length,object_keys:session.groups[index].map(o=>o.row.cache_key)};session.volumes.set(index,volume);finish(null,{volume,bytes:data.bytes});};
  worker.postMessage({snapshot_sha256:session.snapshot_sha256,volume_index:index,volume_count:session.groups.length,objects:session.groups[index]});
 });
}
export function fletchCacheBackupIndexBytes(session){
 if(session?.schema_version!=='bisect-cache-backup-session-v1'||session.volumes.size!==session.groups.length)throw new Error('Generate and download every cache volume before the backup index.');
 return new TextEncoder().encode(JSON.stringify({schema_version:'bisect-cache-backup-index-v1',snapshot_sha256:session.snapshot_sha256,manifest_json:session.manifest_json,object_count:session.object_count,volumes:Array.from({length:session.groups.length},(_,i)=>session.volumes.get(i)),scope:'Complete snapshot of browser cache metadata and freshly checked original objects. Save all listed volumes with this index. Integrity hashes are not provider authentication; source URLs are inert claims.'},null,2));
}
