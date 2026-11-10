import {acsHousingRequest} from './acs-housing-files.js';
import {lodesRequest} from './lodes-files.js?v=lodes-gzip-v1';
import {storedZipBytes} from './reproducibility-files.js';
import {validatePackagePath,encodeBytes} from './package-files.js';
export function validateFetchDataSettings(v){if(!v||Object.keys(v).sort().join()!=='force,item_index,selection,source_kind'||typeof v.force!=='boolean'||!Number.isSafeInteger(v.item_index)||v.item_index<0||!['acs-housing','lodes','unavailable404'].includes(v.source_kind)||!v.selection||typeof v.selection!=='object')throw new Error('Invalid Fetch data package selection.');return v;}
export function fetchDataSettingsFromPrepared(result,files){
 const housing=result?.schema_version==='bisect-acs-housing-preparation-v1';if(!housing&&result?.schema_version!=='bisect-lodes-preparation-v1'||result.status!=='prepared')throw new Error('Prepare fresh ACS or LODES source data first.');
 const request=housing?acsHousingRequest(files.acsHousingInputs):lodesRequest(files.lodesInputs);if(request.input.state!==result.state||request.input.year!==result.year)throw new Error('Prepared source scope changed.');
 const type=housing?'acs-housing':request.input.kind==='wac'?'lodes':'lodes-od';
 return {selection:{manifest_json:null,states:[result.state],year:result.year,data_types:[type],selected_paths:[]},item_index:0,force:false,source_kind:housing?'acs-housing':'lodes'};
}
export function fetchDataRequest(files){const settings=validateFetchDataSettings(files.fetchDataPackageSettings);return {operation:'prepare-fetch-data-package',input:{selection:settings.selection,item_index:settings.item_index,force:settings.force,source:settings.source_kind==='unavailable404'?{kind:'unavailable404'}:{kind:settings.source_kind,input:settings.source_kind==='acs-housing'?acsHousingRequest(files.acsHousingInputs).input:lodesRequest(files.lodesInputs).input}}};}
export async function fetchDataZip(result){
 if(result?.schema_version!=='bisect-fetch-data-package-v1'||result.status!=='prepared'||result.installation_performed!==false||!result.item||Object.keys(result.files_base64||{}).sort().join()!==[result.item.local_path,result.item.done_marker].sort().join())throw new Error('Prepare a fresh Fetch data package first.');
 const entries=[];for(const [path,base64]of Object.entries(result.files_base64)){validatePackagePath(path);if(typeof base64!=='string'||base64.length>Math.ceil(128*1024*1024/3)*4||base64.length%4||!/^[A-Za-z0-9+/]*={0,2}$/.test(base64))throw new Error('Invalid Fetch artifact encoding.');const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0)),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');if(encodeBytes(bytes)!==base64)throw new Error('Noncanonical Fetch artifact encoding.');if(hash!==(path===result.item.local_path?result.target_sha256:result.marker_sha256))throw new Error('Fetch artifact hash changed.');if(path===result.item.done_marker&&new TextDecoder().decode(bytes)!=='done')throw new Error('Invalid native completion marker.');entries.push([path,bytes]);}return storedZipBytes(entries,{maxSourceBytes:128*1024*1024});
}
