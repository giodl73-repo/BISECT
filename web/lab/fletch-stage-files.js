import {validateFletchHistory,fletchHistoryFromEntry} from './fletch-history.js';
import {encodeBytes,validateSourceFile,validatePackagePath} from './package-files.js';
import {validateFetchAcquisition,validateFetchArchiveSource,fetchZipKind} from './fetch-acquisition.js';
import {storedZipBytes} from './reproducibility-files.js';
const MAX=64*1024*1024;
function validateRawSource(source){
 // Native FLETCH accepts a successful zero-byte body; retain that case without
 // relaxing the existing nonempty election/source validators elsewhere.
 validateSourceFile(source?.base64===''?{...source,base64:'AA=='}:source,{maxBytes:MAX});
 if(!source.name||new TextEncoder().encode(source.name).length>200)throw new Error('Select a raw cache source name up to 200 UTF-8 bytes.');return source;
}
export function validateFletchStageInputs(v,{requireComplete=false}={}){
 if(!v||!['dataset_id,expected_source_sha256,fetched_at_ms,source,source_url,version','dataset_id,expected_source_sha256,fetched_at_ms,source,source_ref,source_url,version'].includes(Object.keys(v).filter(k=>k!=='fetch_history').sort().join())||Object.hasOwn(v,'source_ref')&&(v.source_ref!=='fetchArchiveSource'||v.source!==null)||typeof v.dataset_id!=='string'||new TextEncoder().encode(v.dataset_id).length>4096||typeof v.source_url!=='string'||new TextEncoder().encode(v.source_url).length>16384||v.version!==null&&(typeof v.version!=='string'||new TextEncoder().encode(v.version).length>1024)||typeof v.fetched_at_ms!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(v.fetched_at_ms)||BigInt(v.fetched_at_ms)>((1n<<64n)-1n)||v.expected_source_sha256!==null&&(typeof v.expected_source_sha256!=='string'||!/^[0-9a-f]{64}$/.test(v.expected_source_sha256)))throw new Error('Select bounded native cache metadata and an exact decimal u64 timestamp.');
 if(Object.hasOwn(v,'fetch_history'))validateFletchHistory(v.fetch_history);
 if(v.source!==null)validateRawSource(v.source);
 if(requireComplete&&(!v.source&&!v.source_ref||!v.dataset_id.trim()||!v.source_url.trim()))throw new Error('Select raw source bytes and enter their dataset and URL claims.');return v;
}
export async function readFletchStageSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<0||file.size>MAX)throw new Error('Select raw source bytes up to 64 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Source changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};validateRawSource(source);return source;
}
export function resolveFletchStageSource(value,files={}){
 const v=validateFletchStageInputs(value);
 if(!v.source_ref)return v.source;
 const archive=validateFetchArchiveSource(files.fetchArchiveSource),receipt=validateFetchAcquisition(files.fetchAcquisitionReceipt);
 if(receipt.availability!=='acquired'||['state','year','kind','url'].some(k=>archive[k]!==receipt[k])||v.expected_source_sha256!==receipt.source_sha256)throw new Error('Retained archive reference no longer matches acquisition scope or identity.');
 const b=archive.source.base64,n=b.length/4*3-(b.endsWith('==')?2:b.endsWith('=')?1:0);if(n!==receipt.byte_length)throw new Error('Retained archive byte count no longer matches acquisition.');return archive.source;
}
export function fletchStageRequest(value,files={}){const v=validateFletchStageInputs(value,{requireComplete:true}),source=resolveFletchStageSource(v,files);return {operation:'stage-fletch-cache-source',input:{source_name:source.name,source_base64:source.base64,dataset_id:v.dataset_id,version:v.version,source_url:v.source_url,fetched_at_ms:v.fetched_at_ms,expected_source_sha256:v.expected_source_sha256,...(v.fetch_history?{fetch_history:v.fetch_history}:{})}};}
export function fletchStageFromAcquisition(files,timestamp='0'){
 const receipt=validateFetchAcquisition(files.fetchAcquisitionReceipt);
 if(receipt.availability!=='acquired')throw new Error('Select retained acquired raw bytes.');
 const history=receipt.attempt_status?{fetch_history:validateFletchHistory({attempts:receipt.attempt_status.attempts,retries:receipt.attempt_status.retries,last_retryable_error:receipt.attempt_status.last_retryable_error})}:receipt.cache_entry_json?{fetch_history:fletchHistoryFromEntry(receipt.cache_entry_json)}:{};
 if(fetchZipKind(receipt.kind)){
  const v={...history,source:null,source_ref:'fetchArchiveSource',dataset_id:`bisect.${receipt.year}.${receipt.state.toLowerCase()}.${receipt.kind}`,version:receipt.year,source_url:receipt.url,fetched_at_ms:timestamp,expected_source_sha256:receipt.source_sha256};resolveFletchStageSource(v,files);return v;
 }
 const housing=receipt.kind==='acs-housing',inputs=files[housing?'acsHousingInputs':'lodesInputs'];
 if(!inputs?.source||inputs.settings?.state!==receipt.state||inputs.settings?.year!==receipt.year||!housing&&inputs.settings?.kind!==(receipt.kind==='lodes-wac'?'wac':'od'))throw new Error('Retained source scope no longer matches acquisition.');
 const v={...history,source:inputs.source,dataset_id:`bisect.${receipt.year}.${receipt.state.toLowerCase()}.${receipt.kind}`,version:receipt.year,source_url:receipt.url,fetched_at_ms:timestamp,expected_source_sha256:receipt.source_sha256};
 validateFletchStageInputs(v);if(Uint8Array.from(atob(v.source.base64),c=>c.charCodeAt(0)).length!==receipt.byte_length)throw new Error('Retained byte count no longer matches acquisition.');return v;
}
export function fletchStagedUpdatesSource(result){
 if(result?.schema_version!=='bisect-fletch-cache-staging-v1'||typeof result.entries_json!=='string')throw new Error('Stage a fresh cache source first.');
 const source={name:'cache-entry-updates.json',base64:encodeBytes(new TextEncoder().encode(result.entries_json))};validateSourceFile(source,{maxBytes:8*1024*1024});return source;
}
export async function validatedFletchCacheObject(result,value,files={}){
 const v=validateFletchStageInputs(value,{requireComplete:true}),source=resolveFletchStageSource(v,files);if(result?.schema_version!=='bisect-fletch-cache-staging-v1'||result.source_name!==source.name||typeof result.manifest_json!=='string'||result.dataset_id!==v.dataset_id||result.version!==v.version||result.source_url!==v.source_url||result.fetched_at_ms_text!==v.fetched_at_ms)throw new Error('Stage a fresh cache source before export.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)),sha=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
 if(sha!==result.source_sha256||String(bytes.length)!==result.byte_count_text)throw new Error('Raw cache source bytes changed after staging.');
 const manifestBytes=new TextEncoder().encode(result.manifest_json),manifestHash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',manifestBytes)),b=>b.toString(16).padStart(2,'0')).join('');if(manifestHash!==result.manifest_sha256)throw new Error('Staged cache manifest bytes changed.');
 validatePackagePath(result.relative_path);if(!/^objects\/sha256\/[0-9a-f]{64}$/.test(result.relative_path)||result.cache_key!=='sha256:'+result.relative_path.slice(15))throw new Error('Invalid native cache object path.');
 const history=v.fetch_history||{attempts:1,retries:0,last_retryable_error:null};
 const entries=JSON.parse(result.entries_json);if(!Array.isArray(entries)||entries.length!==1)throw new Error('Invalid staged native entry.');const e=entries[0],timestamps=[...result.entries_json.matchAll(/"fetched_at_ms"\s*:\s*(0|[1-9][0-9]*)(?=\s*[,}])/g)];
 if(e.dataset_id!==v.dataset_id||e.version!==v.version||e.source_url!==v.source_url||e.cache_key!==result.cache_key||e.relative_path!==result.relative_path||e.sha256!=='sha256:'+sha||e.bytes!==bytes.length||e.verified!==true||e.fetch_attempts!==history.attempts||e.retry_count!==history.retries||e.last_retryable_error!==history.last_retryable_error||timestamps.length!==1||timestamps[0][1]!==v.fetched_at_ms)throw new Error('Staged native entry no longer matches selected bytes and metadata.');
 return {bytes,manifestBytes,source,entry:e};
}
export async function fletchStagedCacheZip(result,value,files={}){
 const {bytes,manifestBytes}=await validatedFletchCacheObject(result,value,files);
 return storedZipBytes([['cache-manifest.json',manifestBytes],[result.relative_path,bytes]],{maxSourceBytes:128*1024*1024});
}
