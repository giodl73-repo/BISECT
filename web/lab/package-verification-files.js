import {inspectJsonText,parseSafeJson} from './project.js';
import {encodeBytes,isBase64Encoding} from './package-files.js';
const MiB=1024*1024,record=v=>v&&typeof v==='object'&&!Array.isArray(v);
const safe=p=>typeof p==='string'&&p.length>0&&p.length<=240&&/^[A-Za-z0-9_./+-]+$/.test(p)&&!p.split('/').some(s=>!s||['.','..','__proto__','constructor','prototype'].includes(s));
function decode(source,limit){
 if(!record(source)||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||source.name.length>240||typeof source.base64!=='string'||source.base64.length>Math.ceil(limit/3)*4||!isBase64Encoding(source.base64))throw new Error('Invalid bounded package source encoding.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(bytes.length>limit||encodeBytes(bytes)!==source.base64)throw new Error('Package source exceeds its byte limit or has noncanonical encoding.');return bytes;
}
function manifestText(source){const text=new TextDecoder('utf-8',{fatal:true}).decode(decode(source,MiB));if(!text.length)throw new Error('Select a nonempty package manifest.');inspectJsonText(text);JSON.parse(text);return text;}
export function validatePackageVerificationInputs(v){
 if(!record(v)||Object.keys(v).sort().join()!=='files,manifest'||!record(v.files)||Object.keys(v.files).length>100)throw new Error('Select one manifest and at most 100 package files.');
 if(v.manifest!==null)manifestText(v.manifest);
 let total=0;const seen=new Set();for(const [path,source]of Object.entries(v.files)){if(!safe(path)||seen.has(path.toLowerCase()))throw new Error('Unsafe or case-colliding package source path.');seen.add(path.toLowerCase());total+=decode(source,16*MiB).length;}if(total>16*MiB)throw new Error('Selected package files exceed 16 MiB.');return v;
}
export function packageVerificationRequest(v,operation='verify-rplan-package'){if(!['verify-rplan-package','verify-manifest-certificate','verify-manifest-prechecks','inspect-native-replay-recipe'].includes(operation))throw new Error('Select a supported verification operation.');validatePackageVerificationInputs(v);if(!v.manifest)throw new Error('Select a verification manifest.');if(['verify-manifest-prechecks','inspect-native-replay-recipe'].includes(operation)&&!safe(v.manifest.name))throw new Error('Select a manifest with a portable package-relative name.');return {operation,input:{manifest_json:manifestText(v.manifest),...(['verify-manifest-prechecks','inspect-native-replay-recipe'].includes(operation)?{manifest_path:v.manifest.name}:{}),files_base64:Object.fromEntries(Object.entries(v.files).map(([p,s])=>[p,s.base64]))}};}
function entries(files,folder){
 const list=Array.from(files);if(!list.length||list.length>(folder?101:100))throw new Error('Select a bounded package source set.');
 const prefix=folder?list[0].webkitRelativePath?.split('/')[0]+'/':'';
 if(folder&&(!list[0].webkitRelativePath||list.some(f=>!f.webkitRelativePath?.startsWith(prefix))))throw new Error('Select one package folder.');
 const seen=new Set();return list.map(file=>{const path=folder?file.webkitRelativePath.slice(prefix.length):file.name;if(!safe(path)||seen.has(path.toLowerCase()))throw new Error('Unsafe, duplicate or case-colliding package source path.');seen.add(path.toLowerCase());return {file,path};});
}
async function readFiles(list){
 let total=0;for(const {file}of list){if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid package source size.');total+=file.size;}if(total>16*MiB)throw new Error('Selected package files exceed 16 MiB.');
 const out=Object.create(null);for(const {path,file}of list){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Package source size changed while reading.');out[path]={name:file.name,base64:encodeBytes(bytes)};}return out;
}
export async function readPackageVerificationFiles(files){return readFiles(entries(files,false));}
export async function readPackageVerificationManifest(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MiB)throw new Error('Select a nonempty package manifest up to 1 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Manifest size changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};manifestText(source);return source;
}
export async function readPackageVerificationFolder(files){
 const list=entries(files,true),manifests=list.filter(e=>e.path==='manifest.json');if(manifests.length!==1)throw new Error('Select one package folder with manifest.json at its root.');
 const manifest=manifests[0].file,selected=list.filter(e=>e.path!=='manifest.json');
 if(!Number.isSafeInteger(manifest.size)||manifest.size<1||manifest.size>MiB||selected.some(e=>!Number.isSafeInteger(e.file.size)||e.file.size<0)||selected.reduce((n,e)=>n+e.file.size,0)>16*MiB)throw new Error('Package folder exceeds manifest or file byte limits.');
 return validatePackageVerificationInputs({manifest:await readPackageVerificationManifest(manifest),files:await readFiles(selected)});
}
export function packageVerificationReceiptBytes(result){
 const manifest=['bisect-manifest-certificate-verification-v1','bisect-manifest-prechecks-v1'].includes(result?.schema_version);if(!manifest&&result?.schema_version!=='bisect-rplan-package-verification-v1'||!(manifest?['verified','failed','not-recorded']:['verified','failed']).includes(result.status)||result.native_exit_code!==(result.status==='failed'?1:0)||typeof result.manifest_sha256!=='string'||!/^[a-f0-9]{64}$/.test(result.manifest_sha256))throw new Error('Run native package or manifest verification first.');
 const text=JSON.stringify(result,null,2)+'\n';parseSafeJson(text);return new TextEncoder().encode(text);
}
