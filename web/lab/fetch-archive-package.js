import {validateFetchArchiveSource,validateFetchAcquisition,fetchZipKind} from './fetch-acquisition.js';
import {validatePackagePath,encodeBytes,isBase64Encoding} from './package-files.js';
import {decodeFetchSourceArchive} from './fetch-source-archive.js';
import {storedZipBytes} from './reproducibility-files.js';
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export function validateFetchArchivePackageSettings(v){
 if(!v||Object.keys(v).sort().join()!=='force,item_index,selection'||typeof v.force!=='boolean'||!Number.isSafeInteger(v.item_index)||v.item_index<0||!v.selection||Object.keys(v.selection).sort().join()!=='data_types,manifest_json,selected_paths,states,year')throw new Error('Choose a retained archive and native Fetch selection.');
 return v;
}
export function fetchArchivePackageRequest(files){return {operation:'prepare-fetch-archive-package',input:{settings:validateFetchArchivePackageSettings(files.fetchArchivePackageSettings),archive:validateFetchArchiveSource(files.fetchArchiveSource),receipt:validateFetchAcquisition(files.fetchAcquisitionReceipt)}};}
export async function executeFetchArchivePackage(engine,input,{signal}={}){
 if(!input||Object.keys(input).sort().join()!=='archive,receipt,settings')throw new Error('Invalid archive package request.');
 const settings=validateFetchArchivePackageSettings(input.settings);signal?.throwIfAborted();
 const plan=engine.execute({operation:'plan-fetch-sources',input:settings.selection}),item=plan.items?.[settings.item_index];
 if(!item||!fetchZipKind(item.kind))throw new Error('Select a ZIP item from the native Fetch plan.');
 const scope='Native Fetch destination layout and done marker, with bounded local ZIP decoding and exact retained raw identity. Extraction paths are data, never filesystem grants. Marker presence does not prove the planned target exists, valid data, a spatial join or source authenticity. Download is explicit; no files, cache or engine partition were installed or run.';
 if(item.done&&!settings.force)return {schema_version:'bisect-fetch-archive-package-v1',status:'skipped',item,files_base64:{},files_sha256:{},source_availability:'not-read',installation_performed:false,execution_performed:false,scope};
 const archive=validateFetchArchiveSource(input.archive),receipt=validateFetchAcquisition(input.receipt);
 if(receipt.availability!=='acquired'||['state','year','kind','url'].some(key=>archive[key]!==receipt[key])||archive.state!==item.state_code||archive.year!==item.year||archive.kind!==item.kind||archive.url!==item.url)throw new Error('Retained ZIP, acquisition receipt and native item scope differ.');
 const bytes=Uint8Array.from(atob(archive.source.base64),c=>c.charCodeAt(0));if(bytes.length!==receipt.byte_length)throw new Error('Retained ZIP byte count differs from acquisition.');
 const decoded=await decodeFetchSourceArchive(bytes,{signal});signal?.throwIfAborted();if(decoded.archive_sha256!==receipt.source_sha256)throw new Error('Retained ZIP bytes differ from acquisition identity.');
 const parent=item.local_path.slice(0,item.local_path.lastIndexOf('/')),files=Object.create(null),hashes=Object.create(null),folded=new Set();validatePackagePath(item.done_marker);
 for(const file of decoded.files){signal?.throwIfAborted();const path=parent+'/'+file.path;validatePackagePath(path);if(path.toLowerCase()===item.done_marker.toLowerCase()||folded.has(path.toLowerCase()))throw new Error('Archive member collides with the native marker or another target.');folded.add(path.toLowerCase());files[path]=encodeBytes(file.bytes);hashes[path]=await hash(file.bytes);}
 const target_present=Object.hasOwn(files,item.local_path),marker=new TextEncoder().encode('done');files[item.done_marker]=encodeBytes(marker);hashes[item.done_marker]=await hash(marker);signal?.throwIfAborted();
 return {schema_version:'bisect-fetch-archive-package-v1',status:'prepared',item,files_base64:files,files_sha256:hashes,source_sha256:decoded.archive_sha256,source_byte_length:bytes.length,source_availability:'supplied',target_present,marker_records_extraction_only:true,expanded_bytes:decoded.files.reduce((sum,file)=>sum+file.bytes.length,0),installation_performed:false,execution_performed:false,scope};
}
export async function fetchArchivePackageZip(result){
 if(result?.schema_version!=='bisect-fetch-archive-package-v1'||result.status!=='prepared'||result.installation_performed!==false||!result.item||!result.files_base64||!result.files_sha256||Object.keys(result.files_base64).sort().join()!==Object.keys(result.files_sha256).sort().join()||!Object.hasOwn(result.files_base64,result.item.done_marker))throw new Error('Prepare a fresh archive package first.');
 const entries=[];for(const [path,base64]of Object.entries(result.files_base64)){validatePackagePath(path);if(!isBase64Encoding(base64)||base64.length>Math.ceil(128*1024*1024/3)*4)throw new Error('Invalid archive package encoding.');const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));if(encodeBytes(bytes)!==base64||await hash(bytes)!==result.files_sha256[path])throw new Error('Archive package bytes changed.');if(path===result.item.done_marker&&new TextDecoder().decode(bytes)!=='done')throw new Error('Native Fetch marker changed.');entries.push([path,bytes]);}
 return storedZipBytes(entries,{maxSourceBytes:128*1024*1024,maxEntries:10001});
}
