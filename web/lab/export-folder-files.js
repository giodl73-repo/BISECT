import {validateSourceFile,encodeBytes,validatePackagePath} from './package-files.js';
import {selectedFolderEntries} from './run-target-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const EXPORT_FOLDER_DEFAULTS={label:'export-plan',files:{},formats:['csv','gerrychain','geojson']};
const kinds=['csv','gerrychain','geojson','reproducibility-package'];
export function validateExportFolderInputs(v){
 if(!v||Object.keys(v).sort().join()!=='files,formats,label'||typeof v.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(v.label)||!Array.isArray(v.formats)||!v.formats.length||v.formats.length>8||v.formats.some(f=>!kinds.includes(f))||!v.files||Array.isArray(v.files)||typeof v.files!=='object'||Object.keys(v.files).length>504)throw new Error('Select a portable export label and native output formats.');
 let total=0;
 for(const [path,source]of Object.entries(v.files)){
  const analysis=/^analysis\/[A-Za-z0-9_-][A-Za-z0-9_.-]*\.json$/.test(path)&&path.slice(9).length<=120;
  if(![v.label+'.rplan','final_assignments.json','data/final_assignments.json','manifest.json'].includes(path)&&!analysis)throw new Error('Unexpected native export source path.');
  if(source!=='')validateSourceFile({name:path,base64:source},{maxBytes:3*1024*1024});total+=source.length/4*3-(source.endsWith('==')?2:source.endsWith('=')?1:0);
 }
 if(total>25*1024*1024)throw new Error('Native export sources exceed 25 MiB.');return v;
}
export function exportFolderRequest(v){validateExportFolderInputs(v);return {operation:'export-native-plan-folder',input:structuredClone(v)};}
export async function readExportFolder(files,settings,{signal}={}){
 const next=validateExportFolderInputs({label:settings.label,formats:settings.formats,files:{}}),{entries}=selectedFolderEntries(files);
 const selected=entries.filter(({path})=>[next.label+'.rplan','final_assignments.json','data/final_assignments.json','manifest.json'].includes(path)||/^analysis\/[^/]+\.json$/.test(path));
 if(selected.length>504||selected.some(({file})=>!Number.isSafeInteger(file.size)||file.size<0||file.size>3*1024*1024)||selected.reduce((n,{file})=>n+file.size,0)>25*1024*1024)throw new Error('Selected export files exceed the source byte/count limits.');
 for(const {file,path}of selected){
  if(signal?.aborted)throw signal.reason||new Error('Export folder selection cancelled.');
  const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Export source size changed during selection.');
  if(signal?.aborted)throw signal.reason||new Error('Export folder selection cancelled.');next.files[path]=encodeBytes(bytes);
 }
 return validateExportFolderInputs(next);
}
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
export async function exportFolderZip(result){
 if(result?.schema_version!=='bisect-native-folder-export-v1'||!['completed','partial'].includes(result.status)||!result.files||Object.keys(result.files).length>1000||Object.keys(result.files).length!==Object.keys(result.file_sha256||{}).length)throw new Error('Run native folder export first.');
 const entries=[];let total=0;
 for(const [path,text]of Object.entries(result.files)){
  validatePackagePath(path);if(typeof text!=='string')throw new Error('Native export file is not text.');
  const bytes=new TextEncoder().encode(text);total+=bytes.length;if(total>64*1024*1024||await sha(bytes)!==result.file_sha256[path])throw new Error('Native export file bytes changed or exceed 64 MiB.');entries.push([path,bytes]);
 }
 if(!entries.length)throw new Error('No export format completed. Inspect the recorded error.');
 return storedZipBytes(entries,{maxSourceBytes:64*1024*1024});
}
