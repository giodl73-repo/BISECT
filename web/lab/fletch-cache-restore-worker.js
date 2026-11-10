import {parseCacheBackupZip} from './fletch-cache-backup-zip.js';
import {instantiateEngine} from './wasm-engine.js';
import {validateFletchCacheMetadata} from './fletch-cache-store.js';
import {parseSafeJson,parseBuildArchiveJson} from './project.js';
const empty='{"schema_version":"fletch.cache-manifest.v1","generated_by":"fletch-core/0.1.0","cache_root":"cache","entries":[]}';
const decoder=new TextDecoder('utf-8',{fatal:true}),sha=/^[0-9a-f]{64}$/,key=/^sha256:[0-9a-f]{64}$/;
export async function validateCacheBackup(indexFile,volumeFiles,engine,onProgress=()=>{}){
 if(!(indexFile instanceof Blob)||indexFile.size>40*1024*1024||!Array.isArray(volumeFiles)||!volumeFiles.length||volumeFiles.length>2000)throw new Error('Invalid cache backup selection.');
 const index=parseBuildArchiveJson(decoder.decode(await indexFile.arrayBuffer()));
 if(index.schema_version!=='bisect-cache-backup-index-v1'||!sha.test(index.snapshot_sha256)||typeof index.manifest_json!=='string'||new TextEncoder().encode(index.manifest_json).length>16*1024*1024||!Number.isInteger(index.object_count)||index.object_count<0||index.object_count>2000||!Array.isArray(index.volumes)||index.volumes.length!==volumeFiles.length)throw new Error('Invalid cache backup index or incomplete volume selection.');
 if(await engine.hashFile(new Blob([index.manifest_json]))!==index.snapshot_sha256)throw new Error('Cache snapshot manifest hash mismatch.');
 const selected=new Map(),objects=[],keys=new Set();for(const file of volumeFiles){if(!(file instanceof Blob)||typeof file.name!=='string'||file.size>128*1024*1024||selected.has(file.name))throw new Error('Invalid or duplicate backup volume file.');selected.set(file.name,file);}
 const names=new Set();
 for(const [ordinal,descriptor]of index.volumes.entries()){
  if(!descriptor||typeof descriptor.name!=='string'||descriptor.name!==`cache-${index.snapshot_sha256.slice(0,16)}-${String(ordinal+1).padStart(3,'0')}.zip`||names.has(descriptor.name)||!sha.test(descriptor.sha256)||!Number.isSafeInteger(descriptor.byte_length)||descriptor.byte_length<22||descriptor.byte_length>128*1024*1024||!Array.isArray(descriptor.object_keys)||descriptor.object_keys.length>500||descriptor.object_keys.some(k=>!key.test(k)))throw new Error('Invalid backup volume descriptor.');names.add(descriptor.name);
  const file=selected.get(descriptor.name);if(!file||file.size!==descriptor.byte_length)throw new Error('Missing or size-mismatched cache backup volume.');
  const bytes=new Uint8Array(await file.arrayBuffer());if(await engine.hashFile(new Blob([bytes]))!==descriptor.sha256)throw new Error('Cache backup volume SHA-256 mismatch.');const members=parseCacheBackupZip(bytes);
  const libraryMember=members.get('cache-library.json'),manifestMember=members.get('cache-manifest.json');if(!libraryMember||!manifestMember||libraryMember.size>8*1024*1024||manifestMember.size>8*1024*1024)throw new Error('Missing or oversized backup metadata.');
  const library=parseSafeJson(decoder.decode(libraryMember.data)),manifestText=decoder.decode(manifestMember.data);
  if(library.schema_version!=='bisect-cache-backup-volume-v1'||library.snapshot_sha256!==index.snapshot_sha256||library.volume_index!==ordinal||library.volume_count!==index.volumes.length||!Array.isArray(library.rows)||library.rows.length!==descriptor.object_keys.length||members.size!==library.rows.length+2)throw new Error('Cache backup volume identity or members disagree.');
  const observed=Object.create(null);let total=0;
  for(const [i,row]of library.rows.entries()){
   validateFletchCacheMetadata(row);if(row.cache_key!==descriptor.object_keys[i]||keys.has(row.cache_key))throw new Error('Duplicate or unordered backup object.');keys.add(row.cache_key);
   const native=JSON.parse(row.entry_json);if(!Array.isArray(native)||native.length!==1)throw new Error('Invalid backup native entry.');const entry=native[0];for(const field of ['dataset_id','version','source_url','cache_key','bytes'])if(entry[field]!==row[field])throw new Error('Backup row differs from native entry.');
   const path='objects/sha256/'+row.cache_key.slice(7);if(entry.sha256!=='sha256:'+row.sha256||entry.relative_path!==path||row.entry_json.match(/"fetched_at_ms"\s*:\s*(0|[1-9][0-9]*)(?=\s*[,}])/)?.[1]!==row.fetched_at_ms)throw new Error('Backup native identity, hash or exact timestamp differs.');
   const member=members.get(path);if(!member||member.size!==row.bytes)throw new Error('Missing backup object or byte count mismatch.');total+=member.size;if(total>64*1024*1024)throw new Error('Backup object volume exceeds 64 MiB.');
   // Copy each validated body into an immutable Blob, independent of later
   // source-file changes. Blob handles cross the worker boundary without base64.
   const blob=new Blob([member.data]);if(await engine.hashFile(blob)!==row.sha256)throw new Error('Cache backup object SHA-256 mismatch.');observed[path]={sha256:'sha256:'+row.sha256,bytes:row.bytes};objects.push({row,blob});onProgress({volume:ordinal+1,volumes:index.volumes.length,objects:objects.length});
  }
  const updates='['+library.rows.map(row=>row.entry_json.trim().slice(1,-1)).join(',')+']',nativeVolume=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:empty,entries_json:updates});if(nativeVolume.manifest_json!==manifestText)throw new Error('Backup native volume manifest differs from rows.');
  const verified=engine.execute({operation:'verify-fletch-cache-evidence',cache_manifest_json:manifestText,files:observed});if(!verified.all_selected_entries_verified||verified.entry_count!==library.rows.length)throw new Error('Native backup verification failed.');onProgress({volume:ordinal+1,volumes:index.volumes.length,objects:objects.length});
 }
 if(objects.length!==index.object_count)throw new Error('Backup total object count mismatch.');
 const entries_json='['+objects.map(o=>o.row.entry_json.trim().slice(1,-1)).join(',')+']';
 const restored=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:empty,entries_json}),original=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:index.manifest_json,entries_json:'[]'});
 if(JSON.parse(index.manifest_json).cache_root!=='cache'||restored.manifest_json!==original.manifest_json||original.entry_count!==objects.length)throw new Error('Full backup manifest differs from ordered volume entries.');
 return {schema_version:'bisect-validated-cache-backup-v1',snapshot_sha256:index.snapshot_sha256,objects,entries_json};
}
self.onmessage=async({data})=>{try{const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));if(!response.ok)throw new Error('Cache restore engine is unavailable.');const engine=await instantiateEngine(await response.arrayBuffer()),result=await validateCacheBackup(data.indexFile,data.volumeFiles,engine,progress=>self.postMessage({type:'progress',...progress}));self.postMessage({ok:true,result});}catch(error){self.postMessage({ok:false,error:error.message||'Invalid cache backup.'});}};
