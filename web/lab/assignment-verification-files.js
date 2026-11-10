import {inspectJsonText,parseSafeJson} from './project.js';
import {encodeBytes,isBase64Encoding} from './package-files.js';
const limit=8*1024*1024,record=v=>v&&typeof v==='object'&&!Array.isArray(v);
function text(source){
 if(!record(source)||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||source.name.length>240||typeof source.base64!=='string'||source.base64.length>Math.ceil(limit/3)*4||!isBase64Encoding(source.base64))throw new Error('Invalid bounded assignment source.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(!bytes.length||bytes.length>limit||encodeBytes(bytes)!==source.base64)throw new Error('Select assignment JSON up to 8 MiB.');
 const value=new TextDecoder('utf-8',{fatal:true}).decode(bytes);inspectJsonText(value);if(!record(JSON.parse(value)))throw new Error('Assignment JSON must be a unit-to-district object.');return value;
}
export {text as assignmentSourceText};
export function validateAssignmentVerificationInputs(v){
 if(!record(v)||Object.keys(v).sort().join()!=='minSimilarity,original,reference'||!Number.isFinite(v.minSimilarity))throw new Error('Select assignment sources and a finite native similarity threshold.');
 for(const key of ['original','reference'])if(v[key]!==null)text(v[key]);return v;
}
export function assignmentVerificationRequest(v){validateAssignmentVerificationInputs(v);if(!v.original||!v.reference)throw new Error('Select original and reference assignment JSON files.');return {operation:'compare-verify-assignments',input:{original_json:text(v.original),reference_json:text(v.reference),min_similarity:v.minSimilarity}};}
export async function readAssignmentVerificationSource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>limit)throw new Error('Select a nonempty assignment JSON file up to 8 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Assignment source size changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};text(source);return source;
}
export function assignmentVerificationReceiptBytes(result){
 if(result?.schema_version!=='bisect-assignment-verification-v1'||!['match','different','unavailable'].includes(result.status)||result.native_exit_code!==(result.status==='match'?0:1))throw new Error('Run assignment comparison first.');
 const value=JSON.stringify(result,null,2)+'\n';parseSafeJson(value);return new TextEncoder().encode(value);
}
