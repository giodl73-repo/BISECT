import {validatePackageVerificationInputs,packageVerificationRequest,readPackageVerificationManifest} from './package-verification-files.js';
import {encodeBytes} from './package-files.js';
import {validateEngineReplayInputs} from './engine-replay-files.js';
export {readPackageVerificationManifest as readDiscoveryManifest};
const safe=p=>typeof p==='string'&&p.length>0&&p.length<=240&&/^[A-Za-z0-9_./+-]+$/.test(p)&&!p.split('/').some(s=>!s||['.','..','__proto__','constructor','prototype'].includes(s));
export function validateAssignmentDiscoveryInputs(v){
 if(!v||Object.keys(v).sort().join()!=='explicitLabel,files,manifest'||typeof v.explicitLabel!=='string'||v.explicitLabel!==''&&(!safe(v.explicitLabel)||v.explicitLabel.includes('/')))throw new Error('Select a safe optional plan label.');
 validatePackageVerificationInputs({manifest:v.manifest,files:v.files});return v;
}
export function assignmentDiscoveryRequest(v){validateAssignmentDiscoveryInputs(v);const input=packageVerificationRequest({manifest:v.manifest,files:v.files}).input;return {operation:'discover-original-assignments',input:{...input,explicit_label:v.explicitLabel||null}};}
export async function readAssignmentDiscoveryFolder(files){
 const list=Array.from(files);if(!list.length||list.length>100)throw new Error('Select an output tree with at most 100 files.');
 const prefix=list[0].webkitRelativePath?.split('/')[0]+'/';let total=0;const seen=new Set();
 const entries=list.map(file=>{if(!file.webkitRelativePath?.startsWith(prefix))throw new Error('Select one output root folder.');const path=file.webkitRelativePath.slice(prefix.length);if(!safe(path)||seen.has(path.toLowerCase()))throw new Error('Unsafe or case-colliding output path.');seen.add(path.toLowerCase());if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid output file size.');total+=file.size;return {path,file};});
 if(total>16*1024*1024)throw new Error('Selected output files exceed 16 MiB.');
 const out=Object.create(null);for(const {path,file}of entries){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Output file changed while reading.');out[path]={name:file.name,base64:encodeBytes(bytes)};}
 return out;
}
export function discoveredReplayInputs(result,selected,current={run:null,original:null,minSimilarity:.99}){
 validateAssignmentDiscoveryInputs(selected);
 if(result?.schema_version!=='bisect-original-assignment-discovery-v1'||result.status!=='found'||!safe(result.source_path)||typeof result.source_json!=='string')throw new Error('Run discovery and find a nonempty original assignment first.');
 const source=selected.files[result.source_path];if(!source||new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)))!==result.source_json)throw new Error('Discovered source differs from the selected output tree.');
 const next={...current,original:{name:result.source_path,base64:source.base64}};validateEngineReplayInputs(next);return next;
}
