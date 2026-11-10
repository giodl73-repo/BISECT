import {storedZipBytes} from './reproducibility-files.js';
import {validateAdjacencyReleaseScope} from './adjacency-release.js';
export async function prepareNativeFetchReleaseFiles(acquired,{signal,assertCurrent=()=>{}}={}){
 const ensure=()=>{signal?.throwIfAborted();assertCurrent();};ensure();
 const r=acquired?.release_receipt,files=acquired?.opaque_files;
 if(r?.schema_version!=='bisect-adjacency-release-acquisition-v1'||r.purpose!=='native-fetch-bundle'||!Array.isArray(files)||!files.length||files.length>256||!Array.isArray(r.files)||r.files.length!==files.length)throw new Error('Acquire a complete native Fetch bundle first.');
 const scope=validateAdjacencyReleaseScope(Object.fromEntries(['repository','tag','state','year'].map(k=>[k,r[k]]))),stem=scope.state.toLowerCase()+'_adjacency_'+scope.year,directory='V3/data/'+scope.year+'/adjacency/',seen=new Set(),entries=[],records=[];let total=0;
 for(const file of files){
  ensure();
  const name=file?.name,key=typeof name==='string'?name.toLowerCase():'';
  if(typeof name!=='string'||!name.startsWith(stem)||!/^[A-Za-z0-9_.-]{1,180}$/.test(name)||name.endsWith('.')||key===stem+'.done'||seen.has(key)||!(file.bytes instanceof Uint8Array)||(total+=file.bytes.length)>64*1024*1024)throw new Error('Unsafe, duplicate or oversized native Fetch bundle.');
  seen.add(key);const expected=r.files.find(x=>x.name===name),sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',file.bytes)),v=>v.toString(16).padStart(2,'0')).join('');
  ensure();if(!expected||expected.byte_length!==file.bytes.length||expected.sha256!==sha256)throw new Error('Native Fetch bundle bytes differ from the acquisition receipt.');
  entries.push([directory+name,file.bytes]);records.push({name,byte_length:file.bytes.length,sha256,published_digest:expected.published_digest,published_digest_verified:expected.published_digest_verified});
 }
 ensure();entries.sort(([a],[b])=>a<b?-1:a>b?1:0);
 const receipt={schema_version:'bisect-native-fetch-release-package-v1',...scope,release_id:r.release_id,files:records,transport:r.transport,native_layout:{root:'manifest.local_outputs_dir',directory,primary_asset:stem+'.pkl',primary_asset_present:files.some(f=>f.name===stem+'.pkl'),done_marker:stem+'.done',done_marker_text:'done'},opaque_files_executed:false,installed:false,source_authenticity_verified:false,scope:'Exact bounded native wildcard assets and native marker bytes in a portable ZIP. Opaque pickle/request/display files are retained only. Presence of a marker is not validation, installation evidence, source authenticity or execution authority. Review the actual destination separately; no existing files are inspected or overwritten here.'};
 entries.push([directory+stem+'.done',new TextEncoder().encode('done')],['native-fetch-release-receipt.json',new TextEncoder().encode(JSON.stringify(receipt,null,2)+'\n')]);
 return {entries,receipt,name:stem+'-native-fetch.zip'};
}
export async function nativeFetchReleasePackage(acquired,options={}){const {entries,...artifact}=await prepareNativeFetchReleaseFiles(acquired,options);options.signal?.throwIfAborted();options.assertCurrent?.();return {...artifact,bytes:storedZipBytes(entries,{maxSourceBytes:66*1024*1024,maxEntries:258})};}
