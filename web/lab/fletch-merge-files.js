import {fletchCacheManifestText,readFletchCacheManifest,validateFletchCacheManifestSource} from './fletch-cache-files.js';
import {encodeBytes} from './package-files.js';
export const readFletchCacheUpdates=readFletchCacheManifest;
export const validateFletchCacheUpdates=validateFletchCacheManifestSource;
export function fletchMergeRequest(manifest,updates){
 if(!manifest||!updates)throw new Error('Select a base cache manifest and a JSON array of cache entries.');
 return {operation:'merge-fletch-cache-manifest',cache_manifest_json:fletchCacheManifestText(manifest),entries_json:fletchCacheManifestText(updates)};
}
export function fletchMergedManifestBytes(result){
 if(result?.schema_version!=='bisect-fletch-cache-merge-v1'||typeof result.manifest_json!=='string')throw new Error('Run a fresh cache manifest merge first.');
 const bytes=new TextEncoder().encode(result.manifest_json);
 if(bytes.length>16*1024*1024)throw new Error('Merged manifest exceeds 16 MiB.');
 return bytes;
}
export function fletchMergedManifestSource(result){
 const source={name:'cache-manifest.json',base64:encodeBytes(fletchMergedManifestBytes(result))};
 validateFletchCacheManifestSource(source);return source;
}
