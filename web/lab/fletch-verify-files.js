import {fletchCacheManifestText} from './fletch-cache-files.js';
export function fletchObjectPath(path){
 if(typeof path!=='string'||!path||new TextEncoder().encode(path).length>1024||path.split(/[\\/]/).some(p=>!p||p==='.'||p==='..'||/[\0:]/.test(p)))throw new Error('Select portable relative FLETCH object paths.');
 return path.replaceAll('\\','/');
}
export function selectFletchObjects(selected,{folder=false}={}){
 const files=Array.from(selected);if(files.length>10000)throw new Error('Select at most 10,000 cache files. Only manifest-referenced files are hashed.');
 const seen=new Set();return files.map(file=>{const raw=folder?file.webkitRelativePath:file.name,path=fletchObjectPath(folder?raw.slice(raw.indexOf('/')+1):raw);if(seen.has(path))throw new Error('Duplicate selected cache object path.');seen.add(path);if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid selected cache file size.');return {path,file};});
}
export function fletchVerifyRequest(source,files=[]){return {operation:'verify-fletch-cache-files',cache_manifest_json:fletchCacheManifestText(source),files};}
export async function verifyFletchCacheFiles(engine,request,{onProgress=()=>{}}={}){
 if(!request||Object.keys(request).sort().join()!=='cache_manifest_json,files,operation'||request.operation!=='verify-fletch-cache-files'||!Array.isArray(request.files)||request.files.length>10000)throw new Error('Invalid cache verification worker request.');
 const plan=engine.execute({operation:'plan-fletch-cache-verification',cache_manifest_json:request.cache_manifest_json}),wanted=new Set(plan.paths),seen=new Set(),selected=[];
 for(const entry of request.files){if(!entry||Object.keys(entry).sort().join()!=='file,path')throw new Error('Invalid selected cache object.');const path=fletchObjectPath(entry.path);if(path!==entry.path||seen.has(path))throw new Error('Duplicate or noncanonical selected cache path.');seen.add(path);if(wanted.has(path))selected.push(entry);}
 const observations=Object.create(null);let completed=0;
 for(const entry of selected){const file=entry.file;if(!Number.isSafeInteger(file?.size)||file.size<0)throw new Error('Invalid selected cache file size.');const sha256=await engine.hashFile(file);observations[entry.path]={sha256:'sha256:'+sha256,bytes:file.size};onProgress({kind:'fletch-cache-hash',completed:++completed,total:selected.length,path:entry.path});}
 const result=engine.execute({operation:'verify-fletch-cache-evidence',cache_manifest_json:request.cache_manifest_json,files:observations});
 result.digest_source='local-files-streamed-in-worker';result.selected_object_count=selected.length;result.ignored_selection_count=request.files.length-selected.length;return result;
}
export function fletchVerifyReportBytes(result){if(result?.schema_version!=='bisect-fletch-cache-verification-v1'||typeof result.report_json!=='string'||result.report_json.length>16*1024*1024)throw new Error('Run a fresh cache file verification first.');return new TextEncoder().encode(result.report_json+'\n');}
