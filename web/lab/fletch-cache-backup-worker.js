import {instantiateEngine} from './wasm-engine.js';
import {validateFletchCacheMetadata} from './fletch-cache-store.js';
import {storedZipBytes} from './reproducibility-files.js';
self.onmessage=async({data})=>{try{
 if(!data||!/^[0-9a-f]{64}$/.test(data.snapshot_sha256)||!Number.isInteger(data.volume_index)||!Number.isInteger(data.volume_count)||data.volume_index<0||data.volume_index>=data.volume_count||data.volume_count>2000||!Array.isArray(data.objects)||data.objects.length>500)throw new Error('Invalid cache backup volume.');
 const response=await fetch(new URL('./bisect_wasm.wasm',import.meta.url));if(!response.ok)throw new Error('Cache backup engine is unavailable.');const engine=await instantiateEngine(await response.arrayBuffer()),observed=Object.create(null),entries=[],rows=[];let total=0;
 for(const {row,blob}of data.objects){validateFletchCacheMetadata(row);if(!(blob instanceof Blob)||blob.size!==row.bytes)throw new Error('Cache backup object size mismatch.');total+=blob.size;if(total>64*1024*1024)throw new Error('Cache backup volume exceeds 64 MiB of objects.');
  const path='objects/sha256/'+row.cache_key.slice(7);if(Object.hasOwn(observed,path))throw new Error('Duplicate backup object.');const digest=await engine.hashFile(blob);if(digest!==row.sha256)throw new Error('Cache backup object failed its fresh SHA-256 check.');observed[path]={sha256:'sha256:'+digest,bytes:blob.size};rows.push(row);entries.push([path,new Uint8Array(await blob.arrayBuffer())]);
 }
 const updates='['+rows.map(row=>row.entry_json.trim().slice(1,-1)).join(',')+']',merged=engine.execute({operation:'merge-fletch-cache-manifest',cache_manifest_json:'{"schema_version":"fletch.cache-manifest.v1","generated_by":"fletch-core/0.1.0","cache_root":"cache","entries":[]}',entries_json:updates});
 const verification=engine.execute({operation:'verify-fletch-cache-evidence',cache_manifest_json:merged.manifest_json,files:observed});if(!verification.all_selected_entries_verified||verification.entry_count!==rows.length)throw new Error('Native cache manifest does not verify against backup objects.');
 const metadata={schema_version:'bisect-cache-backup-volume-v1',snapshot_sha256:data.snapshot_sha256,volume_index:data.volume_index,volume_count:data.volume_count,rows};entries.unshift(['cache-manifest.json',new TextEncoder().encode(merged.manifest_json)],['cache-library.json',new TextEncoder().encode(JSON.stringify(metadata))]);
 const bytes=storedZipBytes(entries,{maxSourceBytes:128*1024*1024}),sha256=await engine.hashFile(new Blob([bytes]));self.postMessage({ok:true,bytes,sha256},[bytes.buffer]);
}catch(error){self.postMessage({ok:false,error:error.message});}};
