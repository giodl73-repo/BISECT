import {parseSafeJson} from './project.js';
import {labelBatchText} from './label-analysis-files.js';
import {encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
const cap=25*1024*1024;
const record=v=>v&&typeof v==='object'&&!Array.isArray(v);
export function validateLabelPackageMergeInputs(v){
 if(!record(v)||Object.keys(v).join()!=='source')throw new Error('Invalid repository snapshot source.');
 if(v.source!==null){const input=parseSafeJson(labelBatchText(v.source));if(!record(input)||Object.keys(input).sort().join()!=='current_files,expected_file_sha256,generated_file_sha256,generated_files,reconciliation'||!record(input.reconciliation))throw new Error('Select a stage package with current snapshots and artifact expectations.');
 for(const key of ['current_files','generated_files'])if(!record(input[key])||Object.keys(input[key]).length>2000||Object.values(input[key]).some(t=>typeof t!=='string')||Object.values(input[key]).reduce((n,t)=>n+new TextEncoder().encode(t).length,0)>cap)throw new Error('Each repository file snapshot is limited to 2,000 text files and 25 MiB.');
 for(const key of ['expected_file_sha256','generated_file_sha256'])if(!record(input[key])||Object.values(input[key]).some(t=>!(key==='expected_file_sha256'&&t===null)&&!(typeof t==='string'&&/^[a-f0-9]{64}$/.test(t))))throw new Error('Supply SHA-256 digests or explicit absence expectations.');
 }
 return v;
}
export function labelPackageMergeRequest(v){validateLabelPackageMergeInputs(v);if(!v.source)throw new Error('Select a repository stage package first.');return {operation:'reconcile-label-package',input:parseSafeJson(labelBatchText(v.source))};}
export async function readLabelPackageMergeSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>cap)throw new Error('Select a nonempty stage package up to 25 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Stage package size changed while reading.');
 const source={name:file.name,base64:encodeBytes(bytes)};validateLabelPackageMergeInputs({source});return source;
}
const sha=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),v=>v.toString(16).padStart(2,'0')).join('');
export async function labelPackageMergeZip(result){
 if(result?.schema_version!=='bisect-label-package-merge-v1'||result.status!=='prepared'||!record(result.files)||!record(result.file_sha256)||Object.keys(result.files).sort().join()!==Object.keys(result.file_sha256).sort().join()||result.files['.bisect']!==result.reconciliation?.registry_json||result.file_sha256['.bisect']!==result.reconciliation?.registry_sha256)throw new Error('Run repository package preparation first.');
 const entries=[];let total=0;
 for(const [path,text]of Object.entries(result.files)){if(typeof text!=='string')throw new Error('Invalid repository file text.');const data=new TextEncoder().encode(text);total+=data.length;if(total>cap)throw new Error('Repository snapshot exceeds 25 MiB.');if(await sha(data)!==result.file_sha256[path])throw new Error('Repository artifact hash changed: '+path);entries.push([path,data]);}
 return storedZipBytes(entries,{maxPathLength:512});
}
