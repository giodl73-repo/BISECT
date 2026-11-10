import {encodeBytes,validateSourceFile} from './package-files.js';
import {fetchPlanRequest} from './fetch-plan-files.js';
function manifestText(source){
 validateSourceFile(source,{maxBytes:8*1024*1024});
 // Preserve a BOM in the native parser input rather than silently accepting a
 // source that the desktop JSON reader refuses.
 return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));
}
export function validateFletchCacheInputs(value){
 if(!value||!['manifest,selection','manifest,selection,settings'].includes(Object.keys(value).sort().join()))throw new Error('Invalid FLETCH cache sources.');
 validateFletchCacheSettings(value.settings||{year:'selection'});
 fetchPlanRequest(value.selection);
 if(value.manifest!==null)manifestText(value.manifest);
 return value;
}
export function fletchCacheRequest(value){
 validateFletchCacheInputs(value);
 if(value.manifest===null)throw new Error('Select a FLETCH cache manifest.');
 const selection=fetchPlanRequest(value.selection).input,year=value.settings?.year||'selection';if(year!=='selection')selection.year=year;
 return {operation:'inspect-fletch-cache',input:{selection,cache_manifest_json:manifestText(value.manifest)}};
}
export async function readFletchCacheManifest(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select a FLETCH cache manifest up to 8 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());
 if(bytes.length!==file.size)throw new Error('FLETCH cache source changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};
 manifestText(source);return source;
}

export function validateFletchCacheManifestSource(source){manifestText(source);return source;}
export function fletchCacheReportBytes(result){
 if(result?.schema_version!=='bisect-fletch-cache-inspection-v1'||typeof result.report_json!=='string'||result.report_json.length>16*1024*1024)throw new Error('Run cache inspection before downloading the native report.');
 // The actual CLI's write_fletch_cache_index uses to_writer_pretty without a
 // trailing newline. Keep the downloaded artifact byte-identical.
 return new TextEncoder().encode(result.report_json);
}

export function validateFletchCacheSettings(value){
 if(!value||Object.keys(value).join()!=='year'||!['selection','2020','2010','2000','all'].includes(value.year))throw new Error('Select a native FLETCH Census vintage.');return value;
}

export function fletchCacheManifestText(source){return manifestText(source);}
